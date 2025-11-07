//! Thread-Local Memory Pools
//!
//! This module provides thread-local buffer pools for high-performance parallel
//! gravitational N-body simulations. Each thread maintains its own set of pools
//! to avoid contention and provide zero-allocation simulation steps.

use crate::memory::{
    PoolConfig, PooledScalarBuffer, PooledVector3Buffer, ScalarBufferPool, Vector3BufferPool,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread;

thread_local! {
    static VECTOR3_POOLS: RefCell<HashMap<String, Vector3BufferPool>> = RefCell::new(HashMap::new());
    static SCALAR_POOLS: RefCell<HashMap<String, ScalarBufferPool>> = RefCell::new(HashMap::new());
}

/// Thread-local pool manager for zero-allocation parallel execution
pub struct ThreadLocalPools;

impl ThreadLocalPools {
    /// Get or create a thread-local Vector3 buffer pool
    pub fn get_vector3_pool(pool_name: &str, config: Option<PoolConfig>) -> PooledVector3Buffer {
        VECTOR3_POOLS.with(|pools| {
            let mut pools = pools.borrow_mut();

            // Get or create pool
            if !pools.contains_key(pool_name) {
                let config = config.unwrap_or_default();
                pools.insert(pool_name.to_string(), Vector3BufferPool::new(config));
            }

            pools.get(pool_name).unwrap().acquire()
        })
    }

    /// Get or create a thread-local Scalar buffer pool
    pub fn get_scalar_pool(pool_name: &str, config: Option<PoolConfig>) -> PooledScalarBuffer {
        SCALAR_POOLS.with(|pools| {
            let mut pools = pools.borrow_mut();

            // Get or create pool
            if !pools.contains_key(pool_name) {
                let config = config.unwrap_or_default();
                pools.insert(pool_name.to_string(), ScalarBufferPool::new(config));
            }

            pools.get(pool_name).unwrap().acquire()
        })
    }

    /// Get force calculation buffers for current thread
    pub fn get_force_buffers(particle_count: usize) -> ForceBuffers {
        let config = PoolConfig {
            buffer_size: particle_count.max(10000), // Ensure sufficient size
            ..Default::default()
        };
        
        let mut forces = Self::get_vector3_pool("forces", Some(config.clone()));
        let mut temp_forces = Self::get_vector3_pool("temp_forces", Some(config.clone()));
        let mut distances = Self::get_scalar_pool("distances", Some(config));
        
        // Resize to exactly the needed size
        forces.resize(particle_count);
        temp_forces.resize(particle_count);
        distances.resize(particle_count);
        
        ForceBuffers {
            forces,
            temp_forces,
            distances,
        }
    }
    
    /// Get integration buffers for current thread
    pub fn get_integration_buffers(particle_count: usize) -> IntegrationBuffers {
        let config = PoolConfig {
            buffer_size: particle_count.max(10000), // Ensure sufficient size
            ..Default::default()
        };
        
        let mut accelerations = Self::get_vector3_pool("accelerations", Some(config.clone()));
        let mut temp_positions = Self::get_vector3_pool("temp_positions", Some(config.clone()));
        let mut temp_velocities = Self::get_vector3_pool("temp_velocities", Some(config));
        
        // Resize to exactly the needed size
        accelerations.resize(particle_count);
        temp_positions.resize(particle_count);
        temp_velocities.resize(particle_count);
        
        IntegrationBuffers {
            accelerations,
            temp_positions,
            temp_velocities,
        }
    }    /// Cleanup all thread-local pools
    pub fn cleanup_all() {
        VECTOR3_POOLS.with(|pools| {
            let pools = pools.borrow();
            for pool in pools.values() {
                pool.cleanup();
            }
        });

        SCALAR_POOLS.with(|pools| {
            let pools = pools.borrow();
            for pool in pools.values() {
                pool.cleanup();
            }
        });
    }

    /// Optimize all thread-local pools
    pub fn optimize_all() {
        VECTOR3_POOLS.with(|pools| {
            let pools = pools.borrow();
            for pool in pools.values() {
                pool.optimize();
            }
        });

        SCALAR_POOLS.with(|pools| {
            let pools = pools.borrow();
            for pool in pools.values() {
                pool.optimize();
            }
        });
    }

    /// Get statistics for all pools in current thread
    pub fn thread_stats() -> ThreadPoolStats {
        let thread_id = thread::current().id();

        let vector3_stats = VECTOR3_POOLS.with(|pools| {
            let pools = pools.borrow();
            pools
                .iter()
                .map(|(name, pool)| (name.clone(), pool.stats()))
                .collect()
        });

        let scalar_stats = SCALAR_POOLS.with(|pools| {
            let pools = pools.borrow();
            pools
                .iter()
                .map(|(name, pool)| (name.clone(), pool.stats()))
                .collect()
        });

        ThreadPoolStats {
            thread_id,
            vector3_pools: vector3_stats,
            scalar_pools: scalar_stats,
        }
    }
}

/// Collection of buffers needed for force calculations
pub struct ForceBuffers {
    /// Primary forces buffer
    pub forces: PooledVector3Buffer,
    /// Temporary forces buffer for intermediate calculations
    pub temp_forces: PooledVector3Buffer,
    /// Distance calculations buffer
    pub distances: PooledScalarBuffer,
}

impl ForceBuffers {
    /// Resize all buffers to match particle count
    pub fn resize(&mut self, particle_count: usize) {
        self.forces.resize(particle_count);
        self.temp_forces.resize(particle_count);
        self.distances.resize(particle_count);
    }

    /// Clear all buffers (zero out values)
    pub fn clear(&mut self) {
        for i in 0..self.forces.len() {
            self.forces[i] = crate::types::Vector3::zeros();
            self.temp_forces[i] = crate::types::Vector3::zeros();
        }

        for i in 0..self.distances.len() {
            self.distances[i] = 0.0;
        }
    }
}

/// Collection of buffers needed for numerical integration
pub struct IntegrationBuffers {
    /// Acceleration buffer
    pub accelerations: PooledVector3Buffer,
    /// Temporary positions buffer
    pub temp_positions: PooledVector3Buffer,
    /// Temporary velocities buffer
    pub temp_velocities: PooledVector3Buffer,
}

impl IntegrationBuffers {
    /// Resize all buffers to match particle count
    pub fn resize(&mut self, particle_count: usize) {
        self.accelerations.resize(particle_count);
        self.temp_positions.resize(particle_count);
        self.temp_velocities.resize(particle_count);
    }

    /// Clear all buffers (zero out values)
    pub fn clear(&mut self) {
        for i in 0..self.accelerations.len() {
            self.accelerations[i] = crate::types::Vector3::zeros();
            self.temp_positions[i] = crate::types::Vector3::zeros();
            self.temp_velocities[i] = crate::types::Vector3::zeros();
        }
    }
}

/// Statistics for all pools in a single thread
#[derive(Debug)]
pub struct ThreadPoolStats {
    /// Thread identifier
    pub thread_id: thread::ThreadId,
    /// Statistics for all Vector3 pools in this thread
    pub vector3_pools: HashMap<String, crate::memory::PoolStats>,
    /// Statistics for all Scalar pools in this thread
    pub scalar_pools: HashMap<String, crate::memory::PoolStats>,
}

impl ThreadPoolStats {
    /// Calculate total memory usage across all pools in this thread
    pub fn total_memory_bytes(&self) -> usize {
        let v3_total: usize = self
            .vector3_pools
            .values()
            .map(|stats| stats.total_memory_bytes)
            .sum();
        let scalar_total: usize = self
            .scalar_pools
            .values()
            .map(|stats| stats.total_memory_bytes)
            .sum();

        v3_total + scalar_total
    }

    /// Calculate overall efficiency across all pools in this thread
    pub fn overall_efficiency(&self) -> f64 {
        let all_efficiencies: Vec<f64> = self
            .vector3_pools
            .values()
            .chain(self.scalar_pools.values())
            .map(|stats| stats.efficiency_score())
            .collect();

        if all_efficiencies.is_empty() {
            100.0
        } else {
            all_efficiencies.iter().sum::<f64>() / all_efficiencies.len() as f64
        }
    }

    /// Get total number of acquisitions across all pools
    pub fn total_acquisitions(&self) -> u64 {
        let v3_acquisitions: u64 = self
            .vector3_pools
            .values()
            .map(|stats| stats.acquisitions)
            .sum();
        let scalar_acquisitions: u64 = self
            .scalar_pools
            .values()
            .map(|stats| stats.acquisitions)
            .sum();

        v3_acquisitions + scalar_acquisitions
    }
}

/// Macro for convenient buffer acquisition in force calculation functions
#[macro_export]
macro_rules! with_force_buffers {
    ($particle_count:expr, $buffers:ident, $body:block) => {{
        let mut $buffers =
            $crate::memory::thread_local::ThreadLocalPools::get_force_buffers($particle_count);
        $buffers.resize($particle_count);
        $buffers.clear();
        $body
    }};
}

/// Macro for convenient buffer acquisition in integration functions
#[macro_export]
macro_rules! with_integration_buffers {
    ($particle_count:expr, $buffers:ident, $body:block) => {{
        let mut $buffers = $crate::memory::thread_local::ThreadLocalPools::get_integration_buffers(
            $particle_count,
        );
        $buffers.resize($particle_count);
        $buffers.clear();
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_thread_local_pools_basic() {
        let buffer = ThreadLocalPools::get_vector3_pool("test", None);
        assert!(!buffer.is_empty());

        let stats = ThreadLocalPools::thread_stats();
        assert!(stats.vector3_pools.contains_key("test"));
    }

    #[test]
    fn test_force_buffers() {
        let mut buffers = ThreadLocalPools::get_force_buffers(100);
        assert_eq!(buffers.forces.len(), 100);
        assert_eq!(buffers.temp_forces.len(), 100);
        assert_eq!(buffers.distances.len(), 100);

        buffers.clear();
        assert_eq!(buffers.forces[0], crate::types::Vector3::zeros());
    }

    #[test]
    fn test_integration_buffers() {
        let mut buffers = ThreadLocalPools::get_integration_buffers(50);
        assert_eq!(buffers.accelerations.len(), 50);
        assert_eq!(buffers.temp_positions.len(), 50);
        assert_eq!(buffers.temp_velocities.len(), 50);

        buffers.clear();
        assert_eq!(buffers.accelerations[0], crate::types::Vector3::zeros());
    }

    #[test]
    fn test_parallel_thread_pools() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for i in 0..4 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let _buffer = ThreadLocalPools::get_vector3_pool(&format!("thread_{}", i), None);
                counter.fetch_add(1, Ordering::SeqCst);

                let stats = ThreadLocalPools::thread_stats();
                assert_eq!(stats.vector3_pools.len(), 1);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_buffer_macros() {
        let particle_count = 10;

        // Test force buffers macro
        with_force_buffers!(particle_count, buffers, {
            assert_eq!(buffers.forces.len(), particle_count);
            assert_eq!(buffers.forces[0], crate::types::Vector3::zeros());
        });

        // Test integration buffers macro
        with_integration_buffers!(particle_count, buffers, {
            assert_eq!(buffers.accelerations.len(), particle_count);
            assert_eq!(buffers.accelerations[0], crate::types::Vector3::zeros());
        });
    }

    #[test]
    fn test_pool_cleanup_and_optimization() {
        // Create some pools
        let _buffer1 = ThreadLocalPools::get_vector3_pool("cleanup_test1", None);
        let _buffer2 = ThreadLocalPools::get_scalar_pool("cleanup_test2", None);

        // Test cleanup
        ThreadLocalPools::cleanup_all();

        // Test optimization
        ThreadLocalPools::optimize_all();

        // Verify pools still work after cleanup/optimization
        let _buffer3 = ThreadLocalPools::get_vector3_pool("cleanup_test1", None);
    }
}
