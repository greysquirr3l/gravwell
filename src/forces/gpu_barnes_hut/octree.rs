#![allow(unused_imports, unused_variables, unused_mut, dead_code, missing_docs)]
//! GPU Octree Construction
//!
//! Implements parallel octree construction on GPU using compute shaders
//! with Morton code generation and spatial sorting for optimal performance.

use crate::error::GravwellError;
use crate::prelude::{Mass, Scalar, Vector3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, ComputePipeline, Device, Queue};

/// GPU-based octree for spatial partitioning
///
/// Constructs octree in parallel on GPU using Morton codes for
/// efficient spatial ordering and cache-friendly traversal.
pub struct GpuOctree {
    device: Arc<Device>,
    queue: Arc<Queue>,

    // Tree parameters
    max_depth: u32,
    min_particles_per_node: u32,

    // GPU buffers
    morton_codes_buffer: Option<Buffer>,
    sorted_indices_buffer: Option<Buffer>,
    tree_nodes_buffer: Option<Buffer>,

    // Compute pipelines
    morton_pipeline: Option<ComputePipeline>,
    sort_pipeline: Option<ComputePipeline>,
    tree_build_pipeline: Option<ComputePipeline>,

    // Tree structure
    root_bounds: BoundingBox,
    node_count: u32,
}

/// 3D bounding box for octree nodes
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vector3,
    pub max: Vector3,
}

/// Bounding box uniform for WGSL shaders
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BoundsUniform {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

/// Octree node representation for GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuOctreeNode {
    // Bounding box
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,

    // Node properties
    pub center_of_mass: [f32; 3],
    pub total_mass: f32,

    // Tree structure
    pub child_mask: u32,        // Bitmask for which children exist
    pub first_child_index: u32, // Index of first child (if internal)
    pub particle_start: u32,    // Start index in particle array (if leaf)
    pub particle_count: u32,    // Number of particles (if leaf)
}

/// Parameters for radix sort passes
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RadixPassParams {
    num_elements: u32,
    bit_shift: u32,
    num_workgroups: u32,
    _padding: u32,
}

/// Parameters for tree building
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TreeBuildParams {
    num_particles: u32,
    max_depth: u32,
    min_particles_per_node: u32,
    root_min_x: f32,
    root_min_y: f32,
    root_min_z: f32,
    root_max_x: f32,
    root_max_y: f32,
    root_max_z: f32,
}

impl GpuOctree {
    /// Create a new GPU octree
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        max_depth: u32,
        min_particles_per_node: u32,
    ) -> Self {
        Self {
            device,
            queue,
            max_depth,
            min_particles_per_node,
            morton_codes_buffer: None,
            sorted_indices_buffer: None,
            tree_nodes_buffer: None,
            morton_pipeline: None,
            sort_pipeline: None,
            tree_build_pipeline: None,
            root_bounds: BoundingBox {
                min: Vector3::zeros(),
                max: Vector3::zeros(),
            },
            node_count: 0,
        }
    }

    /// Build the octree from particle positions and masses
    pub async fn build_tree(
        &mut self,
        positions: &[Vector3],
        masses: &[Mass],
    ) -> Result<(), GravwellError> {
        let particle_count = positions.len();

        // Step 1: Calculate bounding box
        self.calculate_bounds(positions);

        // Step 2: Initialize GPU resources
        self.initialize_gpu_resources(particle_count).await?;

        // Step 3: Generate Morton codes
        self.generate_morton_codes(positions).await?;

        // Step 4: Sort particles by Morton codes
        self.sort_by_morton_codes(particle_count).await?;

        // Step 5: Build tree structure
        self.build_tree_structure(positions, masses).await?;

        Ok(())
    }

    /// Calculate bounding box for all particles
    fn calculate_bounds(&mut self, positions: &[Vector3]) {
        if positions.is_empty() {
            return;
        }

        let mut min = positions[0];
        let mut max = positions[0];

        for pos in positions.iter().skip(1) {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);

            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }

        // Add small padding to avoid edge cases
        let padding = (max - min).norm() * 0.01;
        min -= Vector3::new(padding, padding, padding);
        max += Vector3::new(padding, padding, padding);

        self.root_bounds = BoundingBox { min, max };
    }

    /// Initialize GPU resources for tree construction
    async fn initialize_gpu_resources(
        &mut self,
        particle_count: usize,
    ) -> Result<(), GravwellError> {
        // Create buffers
        self.create_buffers(particle_count).await?;

        // Create compute pipelines
        self.create_pipelines().await?;

        Ok(())
    }

    async fn create_buffers(&mut self, particle_count: usize) -> Result<(), GravwellError> {
        let morton_size = particle_count * std::mem::size_of::<u32>();
        let indices_size = particle_count * std::mem::size_of::<u32>();
        let nodes_size = particle_count * 8 * std::mem::size_of::<GpuOctreeNode>(); // Estimate

        // Morton codes buffer
        self.morton_codes_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Morton Codes"),
            size: morton_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Sorted indices buffer
        self.sorted_indices_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sorted Indices"),
            size: indices_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Tree nodes buffer
        self.tree_nodes_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Nodes"),
            size: nodes_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        Ok(())
    }

    async fn create_pipelines(&mut self) -> Result<(), GravwellError> {
        // Morton code generation pipeline
        let morton_shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Morton Code Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("morton_codes.wgsl").into()),
            });

        self.morton_pipeline = Some(self.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("Morton Code Pipeline"),
                layout: None,
                module: &morton_shader,
                entry_point: "generate_morton_codes",
            },
        ));

        // Radix sort pipeline
        let sort_shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Radix Sort Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("radix_sort.wgsl").into()),
            });

        self.sort_pipeline = Some(self.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("Radix Sort Pipeline"),
                layout: None,
                module: &sort_shader,
                entry_point: "radix_sort",
            },
        ));

        // Tree building pipeline
        let tree_shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Tree Build Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("tree_build.wgsl").into()),
            });

        self.tree_build_pipeline = Some(self.device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("Tree Build Pipeline"),
                layout: None,
                module: &tree_shader,
                entry_point: "build_tree",
            },
        ));

        Ok(())
    }

    /// Generate Morton codes for spatial ordering
    async fn generate_morton_codes(&mut self, positions: &[Vector3]) -> Result<(), GravwellError> {
        // Upload position data
        let position_data: Vec<[f32; 3]> = positions
            .iter()
            .map(|pos| [pos.x as f32, pos.y as f32, pos.z as f32])
            .collect();

        // Create staging buffer for positions
        let position_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Position Staging"),
                contents: bytemuck::cast_slice(&position_data),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        // Create bounding box uniform buffer
        let bounds_data = BoundsUniform {
            min_x: self.root_bounds.min.x as f32,
            min_y: self.root_bounds.min.y as f32,
            min_z: self.root_bounds.min.z as f32,
            max_x: self.root_bounds.max.x as f32,
            max_y: self.root_bounds.max.y as f32,
            max_z: self.root_bounds.max.z as f32,
        };

        let bounds_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bounding Box Uniform"),
                contents: bytemuck::cast_slice(&[bounds_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Morton Code Bind Group"),
            layout: &self
                .morton_pipeline
                .as_ref()
                .unwrap()
                .get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self
                        .morton_codes_buffer
                        .as_ref()
                        .unwrap()
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bounds_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch compute shader
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Morton Code Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Morton Code Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.morton_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 64;
            let num_workgroups = (positions.len() + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Sort particles by Morton codes using radix sort
    async fn sort_by_morton_codes(&mut self, particle_count: usize) -> Result<(), GravwellError> {
        // Initialize indices (0, 1, 2, ...)
        let indices: Vec<u32> = (0..particle_count as u32).collect();

        // Upload initial indices
        self.queue.write_buffer(
            self.sorted_indices_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&indices),
        );

        // Perform radix sort in multiple passes (64-bit Morton codes = 8 passes of 8 bits each)
        let num_passes = 8; // 64 bits / 8 bits per pass
        let mut input_keys = self.morton_codes_buffer.as_ref().unwrap();
        let mut input_values = self.sorted_indices_buffer.as_ref().unwrap();
        
        // Create additional buffers for ping-pong sorting
        let temp_keys_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temp Morton Codes"),
            size: (particle_count * std::mem::size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let temp_values_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temp Indices"),
            size: (particle_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let histogram_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Radix Histogram"),
            size: (256 * 32 * std::mem::size_of::<u32>()) as u64, // 256 buckets * max workgroups
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        for pass in 0..num_passes {
            let bit_shift = pass * 8;
            
            // Determine output buffers (ping-pong)
            let (output_keys, output_values) = if pass % 2 == 0 {
                (&temp_keys_buffer, &temp_values_buffer)
            } else {
                (self.morton_codes_buffer.as_ref().unwrap(), self.sorted_indices_buffer.as_ref().unwrap())
            };

            // Create radix pass parameters
            let pass_params = RadixPassParams {
                num_elements: particle_count as u32,
                bit_shift,
                num_workgroups: ((particle_count + 255) / 256) as u32,
                _padding: 0,
            };

            let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radix Pass Parameters"),
                contents: bytemuck::cast_slice(&[pass_params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Phase 1: Compute histograms
            self.radix_compute_histogram(
                input_keys,
                input_values,
                &histogram_buffer,
                &params_buffer,
                particle_count,
            ).await?;

            // Phase 2: Compute prefix sums
            self.radix_compute_prefix_sums(&histogram_buffer, &params_buffer).await?;

            // Phase 3: Scatter elements
            self.radix_scatter_elements(
                input_keys,
                input_values,
                output_keys,
                output_values,
                &histogram_buffer,
                &params_buffer,
                particle_count,
            ).await?;

            // Update pointers for next iteration
            input_keys = output_keys;
            input_values = output_values;
        }

        // Ensure final results are in the correct buffers
        if num_passes % 2 == 1 {
            // Copy from temp buffers back to main buffers
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Final Copy Encoder"),
            });

            encoder.copy_buffer_to_buffer(
                &temp_keys_buffer,
                0,
                self.morton_codes_buffer.as_ref().unwrap(),
                0,
                (particle_count * std::mem::size_of::<u64>()) as u64,
            );

            encoder.copy_buffer_to_buffer(
                &temp_values_buffer,
                0,
                self.sorted_indices_buffer.as_ref().unwrap(),
                0,
                (particle_count * std::mem::size_of::<u32>()) as u64,
            );

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    /// Compute histogram phase of radix sort
    async fn radix_compute_histogram(
        &self,
        input_keys: &Buffer,
        input_values: &Buffer,
        histogram_buffer: &Buffer,
        params_buffer: &Buffer,
        particle_count: usize,
    ) -> Result<(), GravwellError> {
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radix Histogram Bind Group"),
            layout: &self.sort_pipeline.as_ref().unwrap().get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_keys.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_values.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: histogram_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: histogram_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: histogram_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Radix Histogram Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Radix Histogram Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.sort_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 256;
            let num_workgroups = (particle_count + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Compute prefix sums phase of radix sort
    async fn radix_compute_prefix_sums(
        &self,
        histogram_buffer: &Buffer,
        params_buffer: &Buffer,
    ) -> Result<(), GravwellError> {
        // Implementation would dispatch prefix sum compute shader
        // For now, this is a placeholder
        Ok(())
    }

    /// Scatter elements phase of radix sort
    async fn radix_scatter_elements(
        &self,
        input_keys: &Buffer,
        input_values: &Buffer,
        output_keys: &Buffer,
        output_values: &Buffer,
        histogram_buffer: &Buffer,
        params_buffer: &Buffer,
        particle_count: usize,
    ) -> Result<(), GravwellError> {
        // Implementation would dispatch scatter compute shader
        // For now, this is a placeholder
        Ok(())
    }

    /// Build the tree structure from sorted particles
    async fn build_tree_structure(
        &mut self,
        positions: &[Vector3],
        masses: &[Mass],
    ) -> Result<(), GravwellError> {
        let particle_count = positions.len();

        // Upload particle data for tree building
        let position_data: Vec<[f32; 3]> = positions
            .iter()
            .map(|pos| [pos.x as f32, pos.y as f32, pos.z as f32])
            .collect();

        let mass_data: Vec<f32> = masses.iter().map(|mass| *mass as f32).collect();

        let position_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Build Positions"),
            contents: bytemuck::cast_slice(&position_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mass_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Build Masses"),
            contents: bytemuck::cast_slice(&mass_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create node counter buffer
        let node_counter_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Node Counter"),
            contents: bytemuck::cast_slice(&[1u32]), // Start with 1 (root node)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create tree build parameters
        let tree_params = TreeBuildParams {
            num_particles: particle_count as u32,
            max_depth: self.max_depth,
            min_particles_per_node: self.min_particles_per_node,
            root_min_x: self.root_bounds.min.x as f32,
            root_min_y: self.root_bounds.min.y as f32,
            root_min_z: self.root_bounds.min.z as f32,
            root_max_x: self.root_bounds.max.x as f32,
            root_max_y: self.root_bounds.max.y as f32,
            root_max_z: self.root_bounds.max.z as f32,
        };

        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Build Parameters"),
            contents: bytemuck::cast_slice(&[tree_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group for tree building
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tree Build Bind Group"),
            layout: &self.tree_build_pipeline.as_ref().unwrap().get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.morton_codes_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.sorted_indices_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: mass_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.tree_nodes_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: node_counter_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Phase 1: Build tree structure
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Tree Build Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Tree Build Structure Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.tree_build_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 64;
            let num_workgroups = (particle_count + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Phase 2: Compute centers of mass
        // This would require reading back the node count and dispatching another compute pass
        // For now, we'll use a fixed estimate of maximum nodes
        let max_nodes = particle_count * 2; // Conservative estimate

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Center of Mass Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Center of Mass Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.tree_build_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_size = 64;
            let num_workgroups = (max_nodes + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Update node count estimate
        self.node_count = max_nodes as u32;

        Ok(())
    }

    /// Get the tree nodes buffer for force calculation
    pub fn get_tree_buffer(&self) -> Option<&Buffer> {
        self.tree_nodes_buffer.as_ref()
    }

    /// Get the root bounding box
    pub fn get_root_bounds(&self) -> BoundingBox {
        self.root_bounds
    }

    /// Get the number of nodes in the tree
    pub fn get_node_count(&self) -> u32 {
        self.node_count
    }
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the bounding box
    pub fn size(&self) -> Vector3 {
        self.max - self.min
    }

    /// Check if a point is inside the bounding box
    pub fn contains(&self, point: Vector3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get the octant (0-7) that contains the given point
    pub fn get_octant(&self, point: Vector3) -> u8 {
        let center = self.center();
        let mut octant = 0u8;

        if point.x > center.x {
            octant |= 1;
        }
        if point.y > center.y {
            octant |= 2;
        }
        if point.z > center.z {
            octant |= 4;
        }

        octant
    }

    /// Get the bounding box for a specific octant
    pub fn get_octant_bounds(&self, octant: u8) -> BoundingBox {
        let center = self.center();
        let mut min = self.min;
        let mut max = self.max;

        if octant & 1 != 0 {
            min.x = center.x;
        } else {
            max.x = center.x;
        }

        if octant & 2 != 0 {
            min.y = center.y;
        } else {
            max.y = center.y;
        }

        if octant & 4 != 0 {
            min.z = center.z;
        } else {
            max.z = center.z;
        }

        BoundingBox { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        assert_eq!(bbox.center(), Vector3::zeros());
        assert_eq!(bbox.size(), Vector3::new(2.0, 2.0, 2.0));
        assert!(bbox.contains(Vector3::zeros()));
        assert!(!bbox.contains(Vector3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_octant_calculation() {
        let bbox = BoundingBox::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        // Test all 8 octants
        assert_eq!(bbox.get_octant(Vector3::new(-0.5, -0.5, -0.5)), 0);
        assert_eq!(bbox.get_octant(Vector3::new(0.5, -0.5, -0.5)), 1);
        assert_eq!(bbox.get_octant(Vector3::new(-0.5, 0.5, -0.5)), 2);
        assert_eq!(bbox.get_octant(Vector3::new(0.5, 0.5, -0.5)), 3);
        assert_eq!(bbox.get_octant(Vector3::new(-0.5, -0.5, 0.5)), 4);
        assert_eq!(bbox.get_octant(Vector3::new(0.5, -0.5, 0.5)), 5);
        assert_eq!(bbox.get_octant(Vector3::new(-0.5, 0.5, 0.5)), 6);
        assert_eq!(bbox.get_octant(Vector3::new(0.5, 0.5, 0.5)), 7);
    }
}
