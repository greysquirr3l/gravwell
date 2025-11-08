#![allow(unused_imports, dead_code)]
//! GPU Barnes-Hut Algorithm Implementation
//!
//! This module implements the Barnes-Hut algorithm on GPU using WebGPU compute shaders,
//! providing O(N log N) scaling for massive particle systems. The implementation includes:
//!
//! - Parallel octree construction on GPU
//! - Efficient tree traversal with stack-based approach
//! - Multipole expansion calculations in WGSL
//! - Configurable theta parameter for accuracy vs performance
//!
//! # Performance
//!
//! Target: 50,000+ particles @ 60 FPS
//! Scaling: O(N log N) vs O(N²) direct gravity
//! Memory: Efficient GPU memory usage with spatial locality

use crate::core::particle::ParticleSet;
use crate::error::GravwellError;
use crate::prelude::{ForceCalculator, Mass, Scalar, Vector3};
use crate::types::Force;
use std::sync::Arc;
use wgpu::{BindGroup, Buffer, ComputePipeline, Device, Queue};

pub mod octree;
pub mod traversal;

use octree::GpuOctree;
use traversal::TreeTraversal;

/// GPU Barnes-Hut force calculator with O(N log N) scaling
///
/// Uses WebGPU compute shaders for parallel octree construction and
/// tree traversal with configurable accuracy parameters.
///
/// # Example
///
/// ```rust
/// use gravwell::forces::GpuBarnesHut;
///
/// let barnes_hut = GpuBarnesHut::new()
///     .theta(0.5)           // Accuracy parameter
///     .max_depth(20)        // Tree depth limit
///     .min_particles(8);    // Leaf threshold
/// ```
pub struct GpuBarnesHut {
    device: Arc<Device>,
    queue: Arc<Queue>,

    // Algorithm parameters
    theta: Scalar,
    max_depth: u32,
    min_particles_per_node: u32,

    // GPU resources
    octree: Option<GpuOctree>,
    traversal: TreeTraversal,

    // Compute pipeline and buffers
    compute_pipeline: Option<ComputePipeline>,
    bind_group: Option<BindGroup>,

    // Buffer management
    position_buffer: Option<Buffer>,
    mass_buffer: Option<Buffer>,
    force_buffer: Option<Buffer>,
    tree_buffer: Option<Buffer>,

    // Performance tracking
    last_particle_count: usize,
}

impl GpuBarnesHut {
    /// Create a new GPU Barnes-Hut force calculator
    pub fn new() -> Self {
        // Create a simple device/queue pair without async complications for now
        let (device, queue) = pollster::block_on(Self::create_device_and_queue());

        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            theta: 0.5,
            max_depth: 20,
            min_particles_per_node: 8,
            octree: None,
            traversal: TreeTraversal::new(),
            compute_pipeline: None,
            bind_group: None,
            position_buffer: None,
            mass_buffer: None,
            force_buffer: None,
            tree_buffer: None,
            last_particle_count: 0,
        }
    }

    /// Set the theta parameter for accuracy vs performance trade-off
    ///
    /// - theta = 0.3: High accuracy, slower
    /// - theta = 0.5: Balanced (recommended)
    /// - theta = 0.7: Lower accuracy, faster
    pub fn theta(mut self, theta: Scalar) -> Self {
        self.theta = theta;
        self
    }

    /// Set maximum octree depth
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set minimum particles per octree node before subdivision
    pub fn min_particles(mut self, count: u32) -> Self {
        self.min_particles_per_node = count;
        self
    }

    /// Initialize GPU resources for the given particle count
    async fn initialize_gpu_resources(
        &mut self,
        particle_count: usize,
    ) -> Result<(), GravwellError> {
        if self.last_particle_count == particle_count && self.compute_pipeline.is_some() {
            return Ok(());
        }

        // Create compute pipeline
        self.compute_pipeline = Some(self.create_compute_pipeline().await?);

        // Create buffers
        self.create_buffers(particle_count).await?;

        // Initialize octree
        self.octree = Some(GpuOctree::new(
            self.device.clone(),
            self.queue.clone(),
            self.max_depth,
            self.min_particles_per_node,
        ));

        self.last_particle_count = particle_count;
        Ok(())
    }

    async fn create_device_and_queue() -> (Device, Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU Barnes-Hut Physics Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    // memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        (device, queue)
    }

    async fn create_compute_pipeline(&self) -> Result<ComputePipeline, GravwellError> {
        let shader_source = include_str!("barnes_hut_compute.wgsl");

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Barnes-Hut Compute Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        let compute_pipeline =
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Barnes-Hut Compute Pipeline"),
                    layout: None,
                    module: &shader,
                    entry_point: "barnes_hut_forces",
                });

        Ok(compute_pipeline)
    }

    async fn create_buffers(&mut self, particle_count: usize) -> Result<(), GravwellError> {
        let position_size = particle_count * std::mem::size_of::<[f32; 3]>();
        let mass_size = particle_count * std::mem::size_of::<f32>();
        let force_size = particle_count * std::mem::size_of::<[f32; 3]>();

        // Position buffer (read-only)
        self.position_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Position Buffer"),
            size: position_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Mass buffer (read-only)
        self.mass_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mass Buffer"),
            size: mass_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Force buffer (write-only)
        self.force_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Force Buffer"),
            size: force_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        // Tree buffer for octree data
        let tree_size = particle_count * 8 * std::mem::size_of::<[f32; 4]>(); // Estimate
        self.tree_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Octree Buffer"),
            size: tree_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        Ok(())
    }
}

impl ForceCalculator for GpuBarnesHut {
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Force],
    ) -> Result<(), GravwellError> {
        // Extract positions and masses from ParticleSet
        let positions: Vec<Vector3> = (0..particles.len())
            .map(|i| particles.position(i).clone())
            .collect();

        let masses: Vec<Mass> = (0..particles.len())
            .map(|i| particles.mass(i).clone())
            .collect();

        // Block on async GPU computation
        pollster::block_on(self.calculate_forces_async(&positions, &masses, forces))
    }

    fn name(&self) -> &'static str {
        "GPU Barnes-Hut"
    }

    fn complexity(&self) -> &'static str {
        "O(N log N)"
    }

    fn supports_parallel(&self) -> bool {
        true // GPU inherently parallel
    }
}

impl GpuBarnesHut {
    /// Async GPU force calculation implementation
    async fn calculate_forces_async(
        &self,
        positions: &[Vector3],
        masses: &[Mass],
        forces: &mut [Force],
    ) -> Result<(), GravwellError> {
        let particle_count = positions.len();

        // Small systems: fall back to direct calculation
        if particle_count < 1000 {
            return self.calculate_forces_direct(positions, masses, forces);
        }

        // Initialize GPU resources
        let mut gpu_barnes_hut = GpuBarnesHut::new();
        gpu_barnes_hut
            .initialize_gpu_resources(particle_count)
            .await?;

        // Build octree
        if let Some(ref mut octree) = gpu_barnes_hut.octree {
            octree.build_tree(positions, masses).await?;
        }

        // Calculate forces using GPU tree traversal
        let mut traversal = TreeTraversal::new();
        traversal
            .calculate_forces_gpu(
                &self.device,
                &self.queue,
                self.compute_pipeline.as_ref().unwrap(),
                positions,
                masses,
                forces,
                self.theta,
            )
            .await?;

        Ok(())
    }

    /// Direct force calculation for small systems
    fn calculate_forces_direct(
        &self,
        positions: &[Vector3],
        masses: &[Mass],
        forces: &mut [Force],
    ) -> Result<(), GravwellError> {
        // Convert forces to Vector3 for internal computation
        let mut force_vectors = vec![Vector3::zeros(); forces.len()];

        // Direct gravity calculation with softening
        for i in 0..positions.len() {
            let mut force = Vector3::zeros();

            for j in 0..positions.len() {
                if i == j {
                    continue;
                }

                let r_vec = positions[j] - positions[i];
                let r_squared = r_vec.norm_squared() + 1e-12; // Small softening to avoid singularities
                let r = r_squared.sqrt();

                if r > 1e-12 {
                    let force_magnitude =
                        crate::utils::constants::G * masses[i] * masses[j] / r_squared;
                    force += force_magnitude * r_vec / r;
                }
            }

            force_vectors[i] = force;
        }

        // Convert back to Force type
        for (i, force) in forces.iter_mut().enumerate() {
            *force = force_vectors[i];
        }

        Ok(())
    }
}

impl Default for GpuBarnesHut {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mass;

    #[test]
    fn test_gpu_barnes_hut_creation() {
        let barnes_hut = GpuBarnesHut::new()
            .theta(0.5)
            .max_depth(20)
            .min_particles(8);

        assert_eq!(barnes_hut.theta, 0.5);
        assert_eq!(barnes_hut.max_depth, 20);
        assert_eq!(barnes_hut.min_particles_per_node, 8);
    }

    #[test]
    async fn test_gpu_barnes_hut_small_system() {
        let barnes_hut = GpuBarnesHut::new();

        // Create a simple test particle set
        let mut particle_set = ParticleSet::new();
        let handle1 =
            particle_set.add_body(Vector3::new(0.0, 0.0, 0.0), Vector3::zeros(), 1.0, 0.1);
        let handle2 =
            particle_set.add_body(Vector3::new(1.0, 0.0, 0.0), Vector3::zeros(), 1.0, 0.1);
        let handle3 =
            particle_set.add_body(Vector3::new(0.0, 1.0, 0.0), Vector3::zeros(), 1.0, 0.1);

        let mut forces = vec![Vector3::zeros(); 3];

        // Should handle small systems gracefully
        let result = barnes_hut.calculate_forces(&particle_set, &mut forces);
        assert!(result.is_ok());
    }
}
