#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
//! GPU Tree Traversal
//!
//! Implements efficient tree traversal for force calculation using
//! stack-based approach optimized for GPU execution.

use crate::error::GravwellError;
use crate::prelude::{Mass, Scalar, Vector3};
use wgpu::util::DeviceExt;
use wgpu::{Buffer, ComputePipeline, Device, Queue};

/// GPU tree traversal for Barnes-Hut force calculation
pub struct TreeTraversal {
    // Traversal parameters
    max_stack_depth: u32,

    // GPU resources
    compute_pass_buffer: Option<Buffer>,
    staging_buffer: Option<Buffer>,
}

/// GPU tree node representation (compatible with WGSL shader)
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuTreeNode {
    // Bounding box
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,

    // Center of mass and total mass
    center_of_mass: [f32; 3],
    total_mass: f32,

    // Tree structure
    child_mask: u32,        // Bitmask for which children exist
    first_child_index: u32, // Index of first child (if internal)
    particle_start: u32,    // Start index in particle array (if leaf)
    particle_count: u32,    // Number of particles (if leaf)
}

impl Default for GpuTreeNode {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 0.0,
            max_y: 0.0,
            max_z: 0.0,
            center_of_mass: [0.0, 0.0, 0.0],
            total_mass: 0.0,
            child_mask: 0,
            first_child_index: 0xFFFFFFFF,
            particle_start: 0,
            particle_count: 0,
        }
    }
}

impl TreeTraversal {
    /// Create a new tree traversal handler
    pub fn new() -> Self {
        Self {
            max_stack_depth: 64, // Maximum recursion depth for stack
            compute_pass_buffer: None,
            staging_buffer: None,
        }
    }

    /// Calculate forces using GPU tree traversal
    pub async fn calculate_forces_gpu(
        &mut self,
        device: &Device,
        queue: &Queue,
        compute_pipeline: &ComputePipeline,
        positions: &[Vector3],
        masses: &[Mass],
        forces: &mut [Vector3],
        theta: Scalar,
    ) -> Result<(), GravwellError> {
        let particle_count = positions.len();

        // Prepare input data
        let position_data: Vec<[f32; 3]> = positions
            .iter()
            .map(|pos| [pos.x as f32, pos.y as f32, pos.z as f32])
            .collect();

        let mass_data: Vec<f32> = masses.iter().map(|mass| *mass as f32).collect();

        // Create input buffers
        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Position Input"),
            contents: bytemuck::cast_slice(&position_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mass_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mass Input"),
            contents: bytemuck::cast_slice(&mass_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create output buffer
        let force_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Force Output"),
            size: (particle_count * std::mem::size_of::<[f32; 3]>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for reading results
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Force Staging"),
            size: (particle_count * std::mem::size_of::<[f32; 3]>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create uniform buffer for parameters
        let params = TraversalParams {
            theta: theta as f32,
            particle_count: particle_count as u32,
            max_stack_depth: self.max_stack_depth,
            _padding: 0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Traversal Parameters"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create dummy tree buffer for now (would come from octree)
        let dummy_tree_data = vec![GpuTreeNode::default(); particle_count.max(1)];
        let tree_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Nodes"),
            contents: bytemuck::cast_slice(&dummy_tree_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tree Traversal Bind Group"),
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: mass_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: force_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: tree_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch compute shader
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Tree Traversal Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Tree Traversal Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 64;
            let num_workgroups = (particle_count + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        // Copy results to staging buffer
        encoder.copy_buffer_to_buffer(
            &force_buffer,
            0,
            &staging_buffer,
            0,
            (particle_count * std::mem::size_of::<[f32; 3]>()) as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read results back from GPU
        self.read_forces_from_gpu(device, queue, &staging_buffer, forces, particle_count)
            .await?;

        Ok(())
    }

    /// Read force results from GPU staging buffer
    async fn read_forces_from_gpu(
        &mut self,
        device: &Device,
        queue: &Queue,
        staging_buffer: &Buffer,
        forces: &mut [Vector3],
        particle_count: usize,
    ) -> Result<(), GravwellError> {
        // Map the staging buffer for reading
        let buffer_slice = staging_buffer.slice(..);

        // Create a simple channel for completion notification
        let (sender, receiver) = std::sync::mpsc::channel();

        // Request mapping
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        // Poll device until mapping is complete
        device.poll(wgpu::Maintain::Wait);

        // Wait for mapping completion
        match receiver.recv() {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let force_data: &[[f32; 3]] = bytemuck::cast_slice(&data);

                // Copy results back to output array
                for (i, force_gpu) in force_data.iter().enumerate() {
                    if i < particle_count && i < forces.len() {
                        forces[i] = Vector3::new(
                            force_gpu[0] as Scalar,
                            force_gpu[1] as Scalar,
                            force_gpu[2] as Scalar,
                        );
                    }
                }

                drop(data);
                staging_buffer.unmap();
                Ok(())
            }
            _ => {
                // Fallback: zero out forces if reading fails
                for force in forces.iter_mut() {
                    *force = Vector3::zeros();
                }

                Err(GravwellError::GpuError(
                    "Failed to map staging buffer for reading".into(),
                ))
            }
        }
    }
}

/// Parameters for tree traversal compute shader
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TraversalParams {
    theta: f32,
    particle_count: u32,
    max_stack_depth: u32,
    _padding: u32,
}

impl Default for TreeTraversal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mass;

    #[test]
    fn test_traversal_creation() {
        let traversal = TreeTraversal::new();
        assert_eq!(traversal.max_stack_depth, 64);
    }

    #[test]
    async fn test_traversal_params() {
        let params = TraversalParams {
            theta: 0.5,
            particle_count: 1000,
            max_stack_depth: 64,
            _padding: 0,
        };

        assert_eq!(params.theta, 0.5);
        assert_eq!(params.particle_count, 1000);
    }
}
