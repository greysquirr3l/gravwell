//! Force calculation algorithms.

pub mod barnes_hut;
pub mod direct;

#[cfg(feature = "parallel")]
pub mod parallel;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "gpu")]
pub mod gpu_barnes_hut;

#[cfg(feature = "gpu")]
pub mod simple_gpu_barnes_hut;

#[cfg(all(feature = "simd", feature = "parallel"))]
pub mod parallel_vectorized;

// Re-export public types
pub use barnes_hut::BarnesHut;
pub use direct::DirectGravity;

#[cfg(feature = "parallel")]
pub use parallel::{ChunkSizeStrategy, ParallelDirectGravity};

#[cfg(feature = "gpu")]
pub use gpu::GpuDirectGravity;

#[cfg(feature = "gpu")]
pub use gpu_barnes_hut::GpuBarnesHut;

#[cfg(feature = "gpu")]
pub use simple_gpu_barnes_hut::SimpleGpuBarnesHut;

#[cfg(all(feature = "simd", feature = "parallel"))]
pub use parallel_vectorized::ParallelVectorizedGravity;
