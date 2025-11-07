# Advanced Integration Examples

This document provides comprehensive examples showing how to integrate Gravwell's advanced optimization systems: spatial culling, LOD (Level of Detail), and memory pools.

## Example 1: Complete High-Performance Setup

This example demonstrates setting up Gravwell for maximum performance with all optimization systems enabled.

```rust
use gravwell::prelude::*;
use gravwell::{
    spatial::{SpatialCuller, SpatialCullerConfig},
    lod::{LODSystem, LODConfig, DetailLevel},
    memory::{MemoryPool, PoolConfig},
};
use std::sync::Arc;

/// High-performance simulation configuration for 100,000+ particles
pub struct HighPerformanceSimulation {
    simulation: Simulation<VelocityVerlet, BarnesHut>,
    spatial_culler: SpatialCuller,
    lod_system: LODSystem,
    memory_pool: Arc<MemoryPool>,
    camera_position: Vector3,
    performance_budget: usize,
}

impl HighPerformanceSimulation {
    /// Create a new high-performance simulation
    pub fn new() -> Result<Self> {
        // Configure simulation with performance-optimized settings
        let simulation = SimulationBuilder::new()
            .with_integrator(VelocityVerlet::new())
            .with_force_calculator(
                BarnesHut::new()
                    .theta(0.7)  // Faster approximation for massive simulations
                    .parallel(true)
                    .simd(true)
            )
            .build()?;

        // Configure spatial culling system
        let spatial_culler = SpatialCuller::new(SpatialCullerConfig {
            hash_grid_enabled: true,
            frustum_culling_enabled: true,
            activation_enabled: true,
            max_active_particles: 10000,  // Performance budget
            cell_size: 50.0,              // Optimized for typical distributions
            distance_threshold: 1500.0,   // Aggressive distance culling
            importance_weight: 1.2,       // Slight importance bias
        });

        // Configure LOD system
        let lod_system = LODSystem::new(LODConfig {
            distance_thresholds: vec![500.0, 1500.0, 5000.0, 15000.0],
            detail_levels: vec![
                DetailLevel::Ultra,     // < 500 units: highest detail
                DetailLevel::High,      // < 1500 units: high detail
                DetailLevel::Medium,    // < 5000 units: medium detail
                DetailLevel::Low,       // < 15000 units: low detail
                DetailLevel::Minimal,   // > 15000 units: minimal processing
            ],
            update_frequency: 60,       // Update LOD every 60 frames
            hysteresis_factor: 0.15,    // Prevent LOD flickering
        });

        // Configure memory pool for zero-allocation performance
        let memory_pool = Arc::new(MemoryPool::new(PoolConfig {
            initial_vector3_capacity: 50000,
            initial_scalar_capacity: 50000,
            max_pool_size: 1024 * 1024 * 100, // 100MB pool limit
            thread_local_enabled: true,
            growth_factor: 1.5,
        }));

        Ok(Self {
            simulation,
            spatial_culler,
            lod_system,
            memory_pool,
            camera_position: Vector3::new(0.0, 0.0, 1000.0),
            performance_budget: 10000,
        })
    }

    /// Add a large number of particles efficiently
    pub fn populate_galaxy(&mut self, particle_count: usize) -> Result<()> {
        println!("Populating galaxy with {} particles...", particle_count);

        // Use memory pool for temporary generation buffers
        let mut positions_buffer = self.memory_pool.get_vector3_buffer(particle_count)?;
        let mut velocities_buffer = self.memory_pool.get_vector3_buffer(particle_count)?;
        let mut masses_buffer = self.memory_pool.get_scalar_buffer(particle_count)?;

        // Generate galaxy distribution in parallel
        use rayon::prelude::*;
        
        (0..particle_count).into_par_iter().enumerate().for_each(|(i, idx)| {
            let angle = 2.0 * std::f64::consts::PI * idx as f64 / particle_count as f64;
            let radius = generate_spiral_galaxy_radius() * (2000.0 + 1000.0 * (i % 5) as f64);
            let height = generate_galaxy_height() * 200.0;

            positions_buffer[i] = Vector3::new(
                radius * angle.cos(),
                height,
                radius * angle.sin(),
            );

            // Orbital velocity for stable rotation
            let orbital_speed = (100.0 / radius.sqrt()).max(10.0);
            velocities_buffer[i] = Vector3::new(
                -orbital_speed * angle.sin(),
                0.0,
                orbital_speed * angle.cos(),
            );

            masses_buffer[i] = generate_stellar_mass() * 1e21;
        });

        // Add particles to simulation
        for i in 0..particle_count {
            let body = Body {
                mass: masses_buffer[i],
                position: positions_buffer[i],
                velocity: velocities_buffer[i],
            };
            self.simulation.add_body(body)?;
        }

        // Buffers automatically return to pool when dropped
        println!("✅ Galaxy populated with {} particles", particle_count);
        Ok(())
    }

    /// Update camera position for frustum culling
    pub fn update_camera(&mut self, position: Vector3, target: Vector3) -> Result<()> {
        self.camera_position = position;
        
        // Create frustum from camera parameters
        let frustum = Frustum::from_camera(
            position,
            target,
            Vector3::new(0.0, 1.0, 0.0), // up vector
            60.0_f64.to_radians(),        // field of view
            16.0 / 9.0,                   // aspect ratio
            1.0,                          // near plane
            50000.0,                      // far plane
        )?;

        // Update spatial culler with new camera frustum
        self.spatial_culler.update_camera_frustum(frustum);
        
        Ok(())
    }

    /// Perform one high-performance simulation step
    pub fn step(&mut self, dt: f64) -> Result<PerformanceStats> {
        let step_start = std::time::Instant::now();
        
        // 1. Update spatial partitioning
        let spatial_start = std::time::Instant::now();
        let all_particles = self.simulation.get_all_particles();
        let all_positions = self.simulation.get_all_positions();
        let all_velocities = self.simulation.get_all_velocities();
        let all_masses = self.simulation.get_all_masses();

        self.spatial_culler.update_particles(
            &all_particles,
            &all_positions,
            Some(&all_velocities),
            Some(&all_masses),
        );
        let spatial_time = spatial_start.elapsed();

        // 2. Perform spatial culling with camera frustum
        let culling_start = std::time::Instant::now();
        let active_particles = self.spatial_culler.cull_particles(
            &all_particles,
            self.camera_position,
            None, // Frustum already updated in spatial culler
            self.performance_budget,
        );
        let culling_time = culling_start.elapsed();

        // 3. Update LOD system based on spatial data
        let lod_start = std::time::Instant::now();
        self.spatial_culler.update_lod_system(&mut self.lod_system, self.camera_position);
        let lod_time = lod_start.elapsed();

        // 4. Apply LOD detail levels to active particles
        for &particle_handle in &active_particles {
            let distance = self.spatial_culler.get_distance_to_camera(particle_handle);
            let detail_level = self.lod_system.calculate_detail_level(distance);
            self.lod_system.set_detail_level(particle_handle, detail_level);
        }

        // 5. Set active particles in simulation
        self.simulation.set_active_particles(active_particles.clone());

        // 6. Perform physics step with memory pool
        let physics_start = std::time::Instant::now();
        
        // Get force calculation buffers from memory pool
        let mut forces_buffer = self.memory_pool.get_vector3_buffer(active_particles.len())?;
        let mut accelerations_buffer = self.memory_pool.get_vector3_buffer(active_particles.len())?;

        // Physics calculation using active particles only
        self.simulation.step_with_buffers(
            dt,
            forces_buffer.as_mut_slice(),
            accelerations_buffer.as_mut_slice(),
        )?;

        let physics_time = physics_start.elapsed();
        let total_time = step_start.elapsed();

        // Collect performance statistics
        let spatial_stats = self.spatial_culler.get_statistics();
        let lod_stats = self.lod_system.get_statistics();
        let memory_stats = self.memory_pool.get_statistics();

        Ok(PerformanceStats {
            total_time,
            spatial_time,
            culling_time,
            lod_time,
            physics_time,
            total_particles: all_particles.len(),
            active_particles: active_particles.len(),
            culling_efficiency: 1.0 - (active_particles.len() as f64 / all_particles.len() as f64),
            spatial_stats,
            lod_stats,
            memory_stats,
        })
    }

    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            spatial_stats: self.spatial_culler.get_detailed_statistics(),
            lod_stats: self.lod_system.get_detailed_statistics(),
            memory_stats: self.memory_pool.get_detailed_statistics(),
        }
    }
}

#[derive(Debug)]
pub struct PerformanceStats {
    pub total_time: std::time::Duration,
    pub spatial_time: std::time::Duration,
    pub culling_time: std::time::Duration,
    pub lod_time: std::time::Duration,
    pub physics_time: std::time::Duration,
    pub total_particles: usize,
    pub active_particles: usize,
    pub culling_efficiency: f64,
    pub spatial_stats: SpatialStatistics,
    pub lod_stats: LODStatistics,
    pub memory_stats: MemoryStatistics,
}

impl PerformanceStats {
    pub fn print_summary(&self) {
        println!("🚀 Performance Summary");
        println!("====================");
        println!("Total Time:        {:.2}ms", self.total_time.as_secs_f64() * 1000.0);
        println!("  Spatial Update:  {:.2}ms", self.spatial_time.as_secs_f64() * 1000.0);
        println!("  Culling:         {:.2}ms", self.culling_time.as_secs_f64() * 1000.0);
        println!("  LOD Update:      {:.2}ms", self.lod_time.as_secs_f64() * 1000.0);
        println!("  Physics:         {:.2}ms", self.physics_time.as_secs_f64() * 1000.0);
        println!();
        println!("Particle Counts:");
        println!("  Total:           {}", self.total_particles);
        println!("  Active:          {}", self.active_particles);
        println!("  Culling Eff:     {:.1}%", self.culling_efficiency * 100.0);
        println!();
        println!("Estimated FPS:     {:.1}", 1.0 / self.total_time.as_secs_f64());
    }
}

// Helper functions for galaxy generation
fn generate_spiral_galaxy_radius() -> f64 {
    let uniform: f64 = rand::random();
    (-uniform.ln() * 0.4 + 0.2).min(3.0) // Exponential distribution with cutoff
}

fn generate_galaxy_height() -> f64 {
    let uniform: f64 = rand::random();
    (uniform - 0.5) * 2.0 // Uniform distribution [-1, 1]
}

fn generate_stellar_mass() -> f64 {
    let uniform: f64 = rand::random();
    // Log-normal distribution approximating stellar mass function
    (uniform * 4.0 + 0.1).exp()
}
```

## Example 2: Adaptive Performance Management

This example shows how to dynamically adjust performance parameters based on runtime conditions.

```rust
use gravwell::prelude::*;
use std::collections::VecDeque;

pub struct AdaptiveSimulation {
    simulation: HighPerformanceSimulation,
    fps_history: VecDeque<f64>,
    target_fps: f64,
    adaptation_enabled: bool,
    last_adaptation: std::time::Instant,
}

impl AdaptiveSimulation {
    pub fn new(target_fps: f64) -> Result<Self> {
        Ok(Self {
            simulation: HighPerformanceSimulation::new()?,
            fps_history: VecDeque::with_capacity(60), // 1 second of history at 60 FPS
            target_fps,
            adaptation_enabled: true,
            last_adaptation: std::time::Instant::now(),
        })
    }

    pub fn step_with_adaptation(&mut self, dt: f64) -> Result<()> {
        let frame_start = std::time::Instant::now();
        
        // Perform simulation step
        let stats = self.simulation.step(dt)?;
        
        // Calculate current FPS
        let frame_time = frame_start.elapsed();
        let current_fps = 1.0 / frame_time.as_secs_f64();
        
        // Update FPS history
        self.fps_history.push_back(current_fps);
        if self.fps_history.len() > 60 {
            self.fps_history.pop_front();
        }

        // Perform adaptive optimization if needed
        if self.adaptation_enabled && self.should_adapt() {
            self.adapt_performance_parameters()?;
        }

        Ok(())
    }

    fn should_adapt(&self) -> bool {
        // Only adapt every 2 seconds to avoid oscillation
        if self.last_adaptation.elapsed() < std::time::Duration::from_secs(2) {
            return false;
        }

        // Need at least 30 frames of history
        if self.fps_history.len() < 30 {
            return false;
        }

        // Calculate average FPS over recent history
        let avg_fps = self.fps_history.iter().sum::<f64>() / self.fps_history.len() as f64;
        
        // Adapt if significantly below or above target
        (avg_fps < self.target_fps * 0.9) || (avg_fps > self.target_fps * 1.2)
    }

    fn adapt_performance_parameters(&mut self) -> Result<()> {
        let avg_fps = self.fps_history.iter().sum::<f64>() / self.fps_history.len() as f64;
        
        println!("🔧 Adapting performance parameters:");
        println!("   Current FPS: {:.1}, Target: {:.1}", avg_fps, self.target_fps);

        if avg_fps < self.target_fps * 0.9 {
            // Performance is below target - reduce quality for better FPS
            self.reduce_quality_for_performance()?;
        } else if avg_fps > self.target_fps * 1.2 {
            // Performance is above target - increase quality
            self.increase_quality_with_headroom()?;
        }

        self.last_adaptation = std::time::Instant::now();
        Ok(())
    }

    fn reduce_quality_for_performance(&mut self) -> Result<()> {
        let current_budget = self.simulation.performance_budget;
        
        // Reduce active particle budget by 15%
        let new_budget = (current_budget as f64 * 0.85) as usize;
        self.simulation.performance_budget = new_budget.max(1000); // Minimum 1000 particles

        // Increase distance culling threshold
        let mut config = self.simulation.spatial_culler.get_config();
        config.distance_threshold *= 0.9; // More aggressive culling
        config.importance_weight *= 1.1;  // Stronger importance bias
        self.simulation.spatial_culler.update_config(config);

        // Reduce LOD update frequency
        let mut lod_config = self.simulation.lod_system.get_config();
        lod_config.update_frequency = (lod_config.update_frequency * 2).min(120);
        self.simulation.lod_system.update_config(lod_config);

        println!("   ↓ Reduced active budget: {} → {}", current_budget, new_budget);
        println!("   ↓ Increased distance culling");
        println!("   ↓ Reduced LOD update frequency");

        Ok(())
    }

    fn increase_quality_with_headroom(&mut self) -> Result<()> {
        let current_budget = self.simulation.performance_budget;
        
        // Increase active particle budget by 10%
        let new_budget = (current_budget as f64 * 1.1) as usize;
        self.simulation.performance_budget = new_budget.min(25000); // Maximum 25K particles

        // Reduce distance culling threshold (less aggressive)
        let mut config = self.simulation.spatial_culler.get_config();
        config.distance_threshold *= 1.1; // Less aggressive culling
        config.importance_weight *= 0.95; // Weaker importance bias
        self.simulation.spatial_culler.update_config(config);

        // Increase LOD update frequency
        let mut lod_config = self.simulation.lod_system.get_config();
        lod_config.update_frequency = (lod_config.update_frequency / 2).max(30);
        self.simulation.lod_system.update_config(lod_config);

        println!("   ↑ Increased active budget: {} → {}", current_budget, new_budget);
        println!("   ↑ Reduced distance culling");
        println!("   ↑ Increased LOD update frequency");

        Ok(())
    }

    pub fn get_adaptive_stats(&self) -> AdaptiveStats {
        let avg_fps = if !self.fps_history.is_empty() {
            self.fps_history.iter().sum::<f64>() / self.fps_history.len() as f64
        } else {
            0.0
        };

        let fps_variance = if self.fps_history.len() > 1 {
            let mean = avg_fps;
            let variance = self.fps_history.iter()
                .map(|fps| (fps - mean).powi(2))
                .sum::<f64>() / (self.fps_history.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        AdaptiveStats {
            target_fps: self.target_fps,
            average_fps: avg_fps,
            fps_variance,
            frames_tracked: self.fps_history.len(),
            adaptation_enabled: self.adaptation_enabled,
            time_since_last_adaptation: self.last_adaptation.elapsed(),
        }
    }
}

#[derive(Debug)]
pub struct AdaptiveStats {
    pub target_fps: f64,
    pub average_fps: f64,
    pub fps_variance: f64,
    pub frames_tracked: usize,
    pub adaptation_enabled: bool,
    pub time_since_last_adaptation: std::time::Duration,
}

impl AdaptiveStats {
    pub fn print_report(&self) {
        println!("📊 Adaptive Performance Report");
        println!("=============================");
        println!("Target FPS:        {:.1}", self.target_fps);
        println!("Average FPS:       {:.1} ± {:.1}", self.average_fps, self.fps_variance);
        println!("Frames Tracked:    {}", self.frames_tracked);
        println!("Adaptation:        {}", if self.adaptation_enabled { "Enabled" } else { "Disabled" });
        println!("Last Adapted:      {:.1}s ago", self.time_since_last_adaptation.as_secs_f64());
        
        let performance_status = if self.average_fps >= self.target_fps * 0.95 {
            "✅ On Target"
        } else if self.average_fps >= self.target_fps * 0.8 {
            "⚠️  Below Target"
        } else {
            "❌ Significantly Below Target"
        };
        println!("Status:            {}", performance_status);
    }
}
```

## Example 3: Game Engine Integration

This example demonstrates integrating Gravwell with a game engine rendering loop.

```rust
use gravwell::prelude::*;

pub struct GamePhysicsEngine {
    simulation: AdaptiveSimulation,
    render_interpolation: bool,
    physics_timestep: f64,
    last_physics_update: std::time::Instant,
    interpolation_factor: f64,
}

impl GamePhysicsEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            simulation: AdaptiveSimulation::new(60.0)?, // Target 60 FPS
            render_interpolation: true,
            physics_timestep: 1.0 / 60.0, // 60 Hz physics
            last_physics_update: std::time::Instant::now(),
            interpolation_factor: 0.0,
        })
    }

    /// Update physics at fixed timestep, separate from rendering
    pub fn update_physics(&mut self) -> Result<bool> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_physics_update);
        
        if elapsed.as_secs_f64() >= self.physics_timestep {
            // Perform physics step
            self.simulation.step_with_adaptation(self.physics_timestep)?;
            self.last_physics_update = now;
            
            // Reset interpolation
            self.interpolation_factor = 0.0;
            
            return Ok(true); // Physics step performed
        }
        
        // Update interpolation factor for smooth rendering
        if self.render_interpolation {
            self.interpolation_factor = elapsed.as_secs_f64() / self.physics_timestep;
        }
        
        Ok(false) // No physics step
    }

    /// Get particle positions for rendering (with interpolation)
    pub fn get_render_positions(&self) -> Vec<Vector3> {
        let current_positions = self.simulation.simulation.get_active_positions();
        
        if !self.render_interpolation || self.interpolation_factor <= 0.0 {
            return current_positions;
        }

        // Interpolate positions based on velocities
        let velocities = self.simulation.simulation.get_active_velocities();
        let dt = self.physics_timestep * self.interpolation_factor;
        
        current_positions.iter().zip(velocities.iter())
            .map(|(pos, vel)| pos + vel * dt)
            .collect()
    }

    /// Get LOD information for rendering optimization
    pub fn get_render_lod_info(&self) -> Vec<(usize, DetailLevel)> {
        self.simulation.simulation.lod_system.get_all_detail_levels()
    }

    /// Update camera for spatial culling
    pub fn update_camera(&mut self, position: Vector3, target: Vector3) -> Result<()> {
        self.simulation.simulation.update_camera(position, target)
    }

    /// Get performance statistics for debugging
    pub fn get_debug_info(&self) -> GameEngineDebugInfo {
        let adaptive_stats = self.simulation.get_adaptive_stats();
        let performance_report = self.simulation.simulation.get_performance_report();
        
        GameEngineDebugInfo {
            physics_fps: adaptive_stats.average_fps,
            physics_variance: adaptive_stats.fps_variance,
            interpolation_factor: self.interpolation_factor,
            active_particles: performance_report.spatial_stats.active_particles,
            total_particles: performance_report.spatial_stats.total_particles,
            culling_efficiency: performance_report.spatial_stats.culling_efficiency,
            memory_usage_mb: performance_report.memory_stats.total_allocated_mb,
        }
    }
}

#[derive(Debug)]
pub struct GameEngineDebugInfo {
    pub physics_fps: f64,
    pub physics_variance: f64,
    pub interpolation_factor: f64,
    pub active_particles: usize,
    pub total_particles: usize,
    pub culling_efficiency: f64,
    pub memory_usage_mb: f64,
}

/// Example game loop integration
pub fn example_game_loop() -> Result<()> {
    let mut physics_engine = GamePhysicsEngine::new()?;
    
    // Populate with 50,000 particles
    physics_engine.simulation.simulation.populate_galaxy(50000)?;
    
    let mut frame_count = 0;
    let start_time = std::time::Instant::now();
    
    loop {
        frame_count += 1;
        
        // Update camera (example: circular motion)
        let time = start_time.elapsed().as_secs_f64();
        let camera_pos = Vector3::new(
            2000.0 * (time * 0.1).cos(),
            1000.0,
            2000.0 * (time * 0.1).sin(),
        );
        physics_engine.update_camera(camera_pos, Vector3::zeros())?;
        
        // Update physics
        let physics_updated = physics_engine.update_physics()?;
        
        // Get render data
        let render_positions = physics_engine.get_render_positions();
        let lod_info = physics_engine.get_render_lod_info();
        
        // Render particles based on LOD
        render_particles_with_lod(&render_positions, &lod_info);
        
        // Print debug info every 60 frames
        if frame_count % 60 == 0 {
            let debug_info = physics_engine.get_debug_info();
            println!("Frame {}: Physics FPS: {:.1}, Active: {}/{}, Culling: {:.1}%",
                frame_count,
                debug_info.physics_fps,
                debug_info.active_particles,
                debug_info.total_particles,
                debug_info.culling_efficiency * 100.0
            );
        }
        
        // Target 60 FPS rendering
        std::thread::sleep(std::time::Duration::from_millis(16));
        
        // Stop after 10 seconds for this example
        if start_time.elapsed() > std::time::Duration::from_secs(10) {
            break;
        }
    }
    
    println!("Example completed successfully!");
    Ok(())
}

fn render_particles_with_lod(positions: &[Vector3], lod_info: &[(usize, DetailLevel)]) {
    // Example rendering function - would integrate with actual graphics API
    for (particle_idx, detail_level) in lod_info {
        let position = positions[*particle_idx];
        
        match detail_level {
            DetailLevel::Ultra => render_particle_ultra_detail(position),
            DetailLevel::High => render_particle_high_detail(position),
            DetailLevel::Medium => render_particle_medium_detail(position),
            DetailLevel::Low => render_particle_low_detail(position),
            DetailLevel::Minimal => render_particle_minimal_detail(position),
        }
    }
}

// Mock rendering functions
fn render_particle_ultra_detail(_position: Vector3) { /* High-quality rendering */ }
fn render_particle_high_detail(_position: Vector3) { /* High-quality rendering */ }
fn render_particle_medium_detail(_position: Vector3) { /* Medium-quality rendering */ }
fn render_particle_low_detail(_position: Vector3) { /* Low-quality rendering */ }
fn render_particle_minimal_detail(_position: Vector3) { /* Minimal rendering */ }
```

## Summary

These examples demonstrate the power of combining Gravwell's optimization systems:

1. **High-Performance Setup**: Shows complete integration of spatial culling, LOD, and memory pools for maximum performance
2. **Adaptive Management**: Demonstrates dynamic parameter adjustment based on runtime performance
3. **Game Engine Integration**: Shows proper separation of physics and rendering with interpolation

Key benefits achieved:
- **100,000+ particle capability** with real-time performance
- **Automatic quality adaptation** to maintain target frame rates
- **Memory efficiency** through pool allocation
- **Smooth rendering** through physics/render decoupling
- **Comprehensive monitoring** for performance optimization

This integration approach enables massive-scale simulations while maintaining the responsiveness required for interactive applications.
