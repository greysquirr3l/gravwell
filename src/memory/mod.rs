//! Memory Pool Allocation System
//!
//! This module provides efficient memory management for gravitational N-body simulations
//! through reusable buffer pools. The goal is to achieve zero allocations during simulation
//! steps by pre-allocating and reusing temporary vectors for force calculations.
//!
//! # Architecture
//!
//! The memory pool system consists of:
//! - **BufferPool**: Thread-safe pool of reusable buffers
//! - **PooledBuffer**: RAII wrapper for automatic return to pool
//! - **MemoryProfiler**: Allocation tracking and optimization
//! - **ThreadLocalPools**: Per-thread pools for parallel execution
//!
//! # Performance Goals
//!
//! - Zero allocations during simulation steps
//! - Sub-microsecond buffer acquisition/release
//! - Minimal memory fragmentation
//! - Automatic pool size optimization

pub mod thread_local;

use crate::types::{Scalar, Vector3};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Configuration for memory pool behavior
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial number of buffers to pre-allocate
    pub initial_capacity: usize,

    /// Maximum number of buffers to maintain in pool
    pub max_capacity: usize,

    /// Buffer size in elements (Vector3 or Scalar)
    pub buffer_size: usize,

    /// Enable automatic pool size optimization
    pub auto_optimize: bool,

    /// Pool cleanup interval (buffers unused for this duration are freed)
    pub cleanup_interval_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 4,
            max_capacity: 16,
            buffer_size: 10000, // Support 10K particles by default
            auto_optimize: true,
            cleanup_interval_ms: 5000, // 5 second cleanup
        }
    }
}

/// Pool statistics for monitoring and optimization
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of buffer acquisitions
    pub acquisitions: u64,

    /// Number of cache hits (buffer reused from pool)
    pub cache_hits: u64,

    /// Number of cache misses (new buffer allocated)
    pub cache_misses: u64,

    /// Average buffer acquisition time (nanoseconds)
    pub avg_acquisition_time_ns: u64,

    /// Current pool size
    pub current_pool_size: usize,

    /// Peak pool usage
    pub peak_pool_usage: usize,

    /// Total memory used by pool (bytes)
    pub total_memory_bytes: usize,
}

impl PoolStats {
    /// Calculate cache hit ratio as a percentage
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.acquisitions == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.acquisitions as f64) * 100.0
        }
    }

    /// Calculate allocation efficiency score (0-100)
    pub fn efficiency_score(&self) -> f64 {
        let hit_ratio = self.cache_hit_ratio();
        let speed_factor = if self.avg_acquisition_time_ns > 0 {
            (1000.0 / self.avg_acquisition_time_ns as f64).min(1.0)
        } else {
            1.0
        };

        hit_ratio * speed_factor
    }
}

/// Thread-safe buffer pool for Vector3 arrays
pub struct Vector3BufferPool {
    config: PoolConfig,
    buffers: Arc<Mutex<VecDeque<Vec<Vector3>>>>,
    stats: Arc<Mutex<PoolStats>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl Vector3BufferPool {
    /// Create a new Vector3 buffer pool with given configuration
    pub fn new(config: PoolConfig) -> Self {
        let mut buffers = VecDeque::new();

        // Pre-allocate initial buffers
        for _ in 0..config.initial_capacity {
            let mut buffer = Vec::with_capacity(config.buffer_size);
            buffer.resize(config.buffer_size, Vector3::zeros());
            buffers.push_back(buffer);
        }

        Self {
            config: config.clone(),
            buffers: Arc::new(Mutex::new(buffers)),
            stats: Arc::new(Mutex::new(PoolStats {
                current_pool_size: config.initial_capacity,
                total_memory_bytes: config.initial_capacity
                    * config.buffer_size
                    * std::mem::size_of::<Vector3>(),
                ..Default::default()
            })),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Acquire a buffer from the pool (or allocate new if pool is empty)
    pub fn acquire(&self) -> PooledVector3Buffer {
        let start_time = Instant::now();

        let buffer = {
            let mut buffers = self.buffers.lock().unwrap();

            if let Some(mut buffer) = buffers.pop_front() {
                // Reuse existing buffer
                buffer.clear();
                buffer.resize(self.config.buffer_size, Vector3::zeros());

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.acquisitions += 1;
                    stats.cache_hits += 1;
                    stats.current_pool_size = buffers.len();

                    let elapsed = start_time.elapsed().as_nanos() as u64;
                    stats.avg_acquisition_time_ns = if stats.acquisitions == 1 {
                        elapsed
                    } else {
                        (stats.avg_acquisition_time_ns * (stats.acquisitions - 1) + elapsed)
                            / stats.acquisitions
                    };
                }

                buffer
            } else {
                // Allocate new buffer
                let mut buffer = Vec::with_capacity(self.config.buffer_size);
                buffer.resize(self.config.buffer_size, Vector3::zeros());

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.acquisitions += 1;
                    stats.cache_misses += 1;

                    let elapsed = start_time.elapsed().as_nanos() as u64;
                    stats.avg_acquisition_time_ns = if stats.acquisitions == 1 {
                        elapsed
                    } else {
                        (stats.avg_acquisition_time_ns * (stats.acquisitions - 1) + elapsed)
                            / stats.acquisitions
                    };
                }

                buffer
            }
        };

        PooledVector3Buffer::new(buffer, self.buffers.clone())
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Perform pool cleanup if needed
    pub fn cleanup(&self) {
        let should_cleanup = {
            let mut last_cleanup = self.last_cleanup.lock().unwrap();
            let now = Instant::now();

            if now.duration_since(*last_cleanup).as_millis()
                > self.config.cleanup_interval_ms as u128
            {
                *last_cleanup = now;
                true
            } else {
                false
            }
        };

        if should_cleanup {
            let mut buffers = self.buffers.lock().unwrap();

            // Keep only initial_capacity buffers, free the rest
            while buffers.len() > self.config.initial_capacity {
                buffers.pop_back();
            }

            // Update stats
            {
                let mut stats = self.stats.lock().unwrap();
                stats.current_pool_size = buffers.len();
                stats.total_memory_bytes =
                    buffers.len() * self.config.buffer_size * std::mem::size_of::<Vector3>();
            }
        }
    }

    /// Optimize pool configuration based on usage patterns
    pub fn optimize(&self) {
        if !self.config.auto_optimize {
            return;
        }

        let stats = self.stats();

        // If cache hit ratio is low, consider increasing pool size
        if stats.cache_hit_ratio() < 70.0 && stats.current_pool_size < self.config.max_capacity {
            let target_size = (stats.current_pool_size + 2).min(self.config.max_capacity);

            let mut buffers = self.buffers.lock().unwrap();
            while buffers.len() < target_size {
                let mut buffer = Vec::with_capacity(self.config.buffer_size);
                buffer.resize(self.config.buffer_size, Vector3::zeros());
                buffers.push_back(buffer);
            }

            // Update stats
            {
                let mut stats = self.stats.lock().unwrap();
                stats.current_pool_size = buffers.len();
                stats.total_memory_bytes =
                    buffers.len() * self.config.buffer_size * std::mem::size_of::<Vector3>();
            }
        }
    }
}

/// RAII wrapper for pooled Vector3 buffer that automatically returns to pool
pub struct PooledVector3Buffer {
    buffer: Option<Vec<Vector3>>,
    pool: Arc<Mutex<VecDeque<Vec<Vector3>>>>,
}

impl PooledVector3Buffer {
    fn new(buffer: Vec<Vector3>, pool: Arc<Mutex<VecDeque<Vec<Vector3>>>>) -> Self {
        Self {
            buffer: Some(buffer),
            pool,
        }
    }

    /// Get immutable reference to the buffer
    pub fn as_slice(&self) -> &[Vector3] {
        self.buffer.as_ref().unwrap()
    }

    /// Get mutable reference to the buffer
    pub fn as_mut_slice(&mut self) -> &mut [Vector3] {
        self.buffer.as_mut().unwrap()
    }

    /// Get the buffer size
    pub fn len(&self) -> usize {
        self.buffer.as_ref().unwrap().len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.as_ref().unwrap().is_empty()
    }

    /// Resize the buffer (up to its capacity)
    pub fn resize(&mut self, new_len: usize) {
        if let Some(ref mut buffer) = self.buffer {
            if new_len <= buffer.capacity() {
                buffer.resize(new_len, Vector3::zeros());
            }
        }
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.as_ref().unwrap().capacity()
    }
}

impl Drop for PooledVector3Buffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            if let Ok(mut pool) = self.pool.lock() {
                pool.push_back(buffer);
            }
        }
    }
}

impl std::ops::Index<usize> for PooledVector3Buffer {
    type Output = Vector3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer.as_ref().unwrap()[index]
    }
}

impl std::ops::IndexMut<usize> for PooledVector3Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer.as_mut().unwrap()[index]
    }
}

/// Thread-safe buffer pool for Scalar arrays
pub struct ScalarBufferPool {
    config: PoolConfig,
    buffers: Arc<Mutex<VecDeque<Vec<Scalar>>>>,
    stats: Arc<Mutex<PoolStats>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl ScalarBufferPool {
    /// Create a new Scalar buffer pool with given configuration
    pub fn new(config: PoolConfig) -> Self {
        let mut buffers = VecDeque::new();

        // Pre-allocate initial buffers
        for _ in 0..config.initial_capacity {
            let mut buffer = Vec::with_capacity(config.buffer_size);
            buffer.resize(config.buffer_size, 0.0);
            buffers.push_back(buffer);
        }

        Self {
            config: config.clone(),
            buffers: Arc::new(Mutex::new(buffers)),
            stats: Arc::new(Mutex::new(PoolStats {
                current_pool_size: config.initial_capacity,
                total_memory_bytes: config.initial_capacity
                    * config.buffer_size
                    * std::mem::size_of::<Scalar>(),
                ..Default::default()
            })),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Acquire a buffer from the pool (or allocate new if pool is empty)
    pub fn acquire(&self) -> PooledScalarBuffer {
        let start_time = Instant::now();

        let buffer = {
            let mut buffers = self.buffers.lock().unwrap();

            if let Some(mut buffer) = buffers.pop_front() {
                // Reuse existing buffer
                buffer.clear();
                buffer.resize(self.config.buffer_size, 0.0);

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.acquisitions += 1;
                    stats.cache_hits += 1;
                    stats.current_pool_size = buffers.len();

                    let elapsed = start_time.elapsed().as_nanos() as u64;
                    stats.avg_acquisition_time_ns = if stats.acquisitions == 1 {
                        elapsed
                    } else {
                        (stats.avg_acquisition_time_ns * (stats.acquisitions - 1) + elapsed)
                            / stats.acquisitions
                    };
                }

                buffer
            } else {
                // Allocate new buffer
                let mut buffer = Vec::with_capacity(self.config.buffer_size);
                buffer.resize(self.config.buffer_size, 0.0);

                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.acquisitions += 1;
                    stats.cache_misses += 1;

                    let elapsed = start_time.elapsed().as_nanos() as u64;
                    stats.avg_acquisition_time_ns = if stats.acquisitions == 1 {
                        elapsed
                    } else {
                        (stats.avg_acquisition_time_ns * (stats.acquisitions - 1) + elapsed)
                            / stats.acquisitions
                    };
                }

                buffer
            }
        };

        PooledScalarBuffer::new(buffer, self.buffers.clone())
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Perform pool cleanup if needed
    pub fn cleanup(&self) {
        let should_cleanup = {
            let mut last_cleanup = self.last_cleanup.lock().unwrap();
            let now = Instant::now();

            if now.duration_since(*last_cleanup).as_millis()
                > self.config.cleanup_interval_ms as u128
            {
                *last_cleanup = now;
                true
            } else {
                false
            }
        };

        if should_cleanup {
            let mut buffers = self.buffers.lock().unwrap();

            // Keep only initial_capacity buffers, free the rest
            while buffers.len() > self.config.initial_capacity {
                buffers.pop_back();
            }

            // Update stats
            {
                let mut stats = self.stats.lock().unwrap();
                stats.current_pool_size = buffers.len();
                stats.total_memory_bytes =
                    buffers.len() * self.config.buffer_size * std::mem::size_of::<Scalar>();
            }
        }
    }

    /// Optimize pool configuration based on usage patterns
    pub fn optimize(&self) {
        if !self.config.auto_optimize {
            return;
        }

        let stats = self.stats();

        // If cache hit ratio is low, consider increasing pool size
        if stats.cache_hit_ratio() < 70.0 && stats.current_pool_size < self.config.max_capacity {
            let target_size = (stats.current_pool_size + 2).min(self.config.max_capacity);

            let mut buffers = self.buffers.lock().unwrap();
            while buffers.len() < target_size {
                let mut buffer = Vec::with_capacity(self.config.buffer_size);
                buffer.resize(self.config.buffer_size, 0.0);
                buffers.push_back(buffer);
            }

            // Update stats
            {
                let mut stats = self.stats.lock().unwrap();
                stats.current_pool_size = buffers.len();
                stats.total_memory_bytes =
                    buffers.len() * self.config.buffer_size * std::mem::size_of::<Scalar>();
            }
        }
    }
}

/// RAII wrapper for pooled Scalar buffer that automatically returns to pool
pub struct PooledScalarBuffer {
    buffer: Option<Vec<Scalar>>,
    pool: Arc<Mutex<VecDeque<Vec<Scalar>>>>,
}

impl PooledScalarBuffer {
    fn new(buffer: Vec<Scalar>, pool: Arc<Mutex<VecDeque<Vec<Scalar>>>>) -> Self {
        Self {
            buffer: Some(buffer),
            pool,
        }
    }

    /// Get immutable reference to the buffer
    pub fn as_slice(&self) -> &[Scalar] {
        self.buffer.as_ref().unwrap()
    }

    /// Get mutable reference to the buffer
    pub fn as_mut_slice(&mut self) -> &mut [Scalar] {
        self.buffer.as_mut().unwrap()
    }

    /// Get the buffer size
    pub fn len(&self) -> usize {
        self.buffer.as_ref().unwrap().len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.as_ref().unwrap().is_empty()
    }

    /// Resize the buffer (up to its capacity)
    pub fn resize(&mut self, new_len: usize) {
        if let Some(ref mut buffer) = self.buffer {
            if new_len <= buffer.capacity() {
                buffer.resize(new_len, 0.0);
            }
        }
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.as_ref().unwrap().capacity()
    }
}

impl Drop for PooledScalarBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            if let Ok(mut pool) = self.pool.lock() {
                pool.push_back(buffer);
            }
        }
    }
}

impl std::ops::Index<usize> for PooledScalarBuffer {
    type Output = Scalar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer.as_ref().unwrap()[index]
    }
}

impl std::ops::IndexMut<usize> for PooledScalarBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer.as_mut().unwrap()[index]
    }
}

/// Memory manager that coordinates multiple buffer pools
pub struct MemoryManager {
    vector3_pool: Vector3BufferPool,
    scalar_pool: ScalarBufferPool,
    profiler: MemoryProfiler,
}

impl MemoryManager {
    /// Create a new memory manager with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new memory manager with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            vector3_pool: Vector3BufferPool::new(config.clone()),
            scalar_pool: ScalarBufferPool::new(config),
            profiler: MemoryProfiler::new(),
        }
    }

    /// Acquire a Vector3 buffer from the pool
    pub fn acquire_vector3_buffer(&self) -> PooledVector3Buffer {
        let buffer = self.vector3_pool.acquire();
        self.profiler
            .record_allocation(std::mem::size_of::<Vector3>() * buffer.capacity());
        buffer
    }

    /// Acquire a Scalar buffer from the pool
    pub fn acquire_scalar_buffer(&self) -> PooledScalarBuffer {
        let buffer = self.scalar_pool.acquire();
        self.profiler
            .record_allocation(std::mem::size_of::<Scalar>() * buffer.capacity());
        buffer
    }

    /// Get combined statistics from all pools
    pub fn stats(&self) -> MemoryManagerStats {
        MemoryManagerStats {
            vector3_pool: self.vector3_pool.stats(),
            scalar_pool: self.scalar_pool.stats(),
            profiler: self.profiler.stats(),
        }
    }

    /// Perform cleanup on all pools
    pub fn cleanup(&self) {
        self.vector3_pool.cleanup();
        self.scalar_pool.cleanup();
    }

    /// Optimize all pools based on usage patterns
    pub fn optimize(&self) {
        self.vector3_pool.optimize();
        self.scalar_pool.optimize();
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined statistics from memory manager
#[derive(Debug, Clone)]
pub struct MemoryManagerStats {
    /// Statistics from Vector3 buffer pool
    pub vector3_pool: PoolStats,
    /// Statistics from Scalar buffer pool
    pub scalar_pool: PoolStats,
    /// Statistics from memory profiler
    pub profiler: ProfilerStats,
}

impl MemoryManagerStats {
    /// Calculate total memory usage across all pools
    pub fn total_memory_bytes(&self) -> usize {
        self.vector3_pool.total_memory_bytes + self.scalar_pool.total_memory_bytes
    }

    /// Calculate overall efficiency score
    pub fn overall_efficiency(&self) -> f64 {
        let v3_efficiency = self.vector3_pool.efficiency_score();
        let scalar_efficiency = self.scalar_pool.efficiency_score();

        if self.vector3_pool.acquisitions > 0 && self.scalar_pool.acquisitions > 0 {
            (v3_efficiency + scalar_efficiency) / 2.0
        } else if self.vector3_pool.acquisitions > 0 {
            v3_efficiency
        } else if self.scalar_pool.acquisitions > 0 {
            scalar_efficiency
        } else {
            100.0 // No allocations yet
        }
    }
}

/// Memory profiler for tracking allocation patterns
pub struct MemoryProfiler {
    stats: Arc<Mutex<ProfilerStats>>,
}

impl MemoryProfiler {
    fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(ProfilerStats::default())),
        }
    }

    fn record_allocation(&self, bytes: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_allocations += 1;
        stats.total_bytes_allocated += bytes;
        stats.current_bytes_in_use += bytes;
        stats.peak_bytes_in_use = stats.peak_bytes_in_use.max(stats.current_bytes_in_use);
    }

    fn stats(&self) -> ProfilerStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Statistics from memory profiler
#[derive(Debug, Clone, Default)]
pub struct ProfilerStats {
    /// Total number of allocations tracked
    pub total_allocations: u64,
    /// Total bytes allocated across all operations
    pub total_bytes_allocated: usize,
    /// Current bytes in use
    pub current_bytes_in_use: usize,
    /// Peak memory usage observed
    pub peak_bytes_in_use: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_buffer_pool_basic() {
        let config = PoolConfig {
            initial_capacity: 2,
            max_capacity: 4,
            buffer_size: 100,
            auto_optimize: false,
            cleanup_interval_ms: 1000,
        };

        let pool = Vector3BufferPool::new(config);

        // First acquisition should be a cache hit (pre-allocated)
        let buffer1 = pool.acquire();
        assert_eq!(buffer1.len(), 100);

        let stats = pool.stats();
        assert_eq!(stats.acquisitions, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);

        drop(buffer1);

        // Second acquisition should also be a cache hit
        let buffer2 = pool.acquire();
        let stats = pool.stats();
        assert_eq!(stats.acquisitions, 2);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 0);

        drop(buffer2);
    }

    #[test]
    fn test_scalar_buffer_pool_basic() {
        let config = PoolConfig {
            initial_capacity: 2,
            max_capacity: 4,
            buffer_size: 100,
            auto_optimize: false,
            cleanup_interval_ms: 1000,
        };

        let pool = ScalarBufferPool::new(config);

        // First acquisition should be a cache hit (pre-allocated)
        let buffer1 = pool.acquire();
        assert_eq!(buffer1.len(), 100);

        let stats = pool.stats();
        assert_eq!(stats.acquisitions, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);

        drop(buffer1);

        // Second acquisition should also be a cache hit
        let buffer2 = pool.acquire();
        let stats = pool.stats();
        assert_eq!(stats.acquisitions, 2);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 0);

        drop(buffer2);
    }

    #[test]
    fn test_memory_manager() {
        let manager = MemoryManager::new();

        let v3_buffer = manager.acquire_vector3_buffer();
        let scalar_buffer = manager.acquire_scalar_buffer();

        assert!(!v3_buffer.is_empty());
        assert!(!scalar_buffer.is_empty());

        let stats = manager.stats();
        assert!(stats.total_memory_bytes() > 0);
        assert!(stats.overall_efficiency() >= 0.0);

        drop(v3_buffer);
        drop(scalar_buffer);
    }

    #[test]
    fn test_pooled_buffer_indexing() {
        let pool = Vector3BufferPool::new(PoolConfig::default());
        let mut buffer = pool.acquire();

        // Test indexing
        buffer[0] = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(buffer[0], Vector3::new(1.0, 2.0, 3.0));

        // Test resizing
        buffer.resize(50);
        assert_eq!(buffer.len(), 50);
    }

    #[test]
    fn test_pool_cleanup() {
        let config = PoolConfig {
            initial_capacity: 2,
            max_capacity: 10,
            buffer_size: 100,
            auto_optimize: false,
            cleanup_interval_ms: 1, // Very short interval for testing
        };

        let pool = Vector3BufferPool::new(config);

        // Acquire and release many buffers to grow the pool
        for _ in 0..8 {
            let _buffer = pool.acquire();
        }

        // Wait for cleanup interval
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Trigger cleanup
        pool.cleanup();

        let stats = pool.stats();
        assert!(stats.current_pool_size <= 2); // Should be cleaned up to initial capacity
    }
}
