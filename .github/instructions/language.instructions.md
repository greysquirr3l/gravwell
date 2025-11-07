---
applyTo: "**/src/**/*.{rs}"
description: "Rust library development patterns, performance optimizations, and Gravwell gravity simulation best practices"
references:
  - rust_book: "https://doc.rust-lang.org/book/"
  - nalgebra_docs: "https://docs.rs/nalgebra/"
  - criterion_docs: "https://docs.rs/criterion/"
  - rust_library_guide: "https://doc.rust-lang.org/cargo/guide/"
---

# Gravwell - Rust Library Development Standards

## Language-Agnostic Core Principles

### SOLID Principles Application
```
S - Single Responsibility: Each class/function has one reason to change
O - Open/Closed: Open for extension, closed for modification
L - Liskov Substitution: Subtypes must be substitutable for base types
I - Interface Segregation: Many specific interfaces over one general interface
D - Dependency Inversion: Depend on abstractions, not concretions
```

### Code Quality Standards
```
- Clear, descriptive naming that explains intent
- Functions/methods should do one thing well
- Minimize cognitive complexity and nesting levels
- Use consistent formatting and style conventions
- Write self-documenting code with strategic comments
- Handle errors explicitly and gracefully
- Follow language-specific idioms and best practices
```

## 🚫 Anti-Patterns & Code to Avoid

### Common Bad Practices That Confuse AI Suggestions

**❌ Poor Error Handling**
```rust
// DON'T: Using unwrap() everywhere (panic on error)
let result = risky_operation().unwrap();

// DON'T: Generic error messages
return Err("something went wrong".to_string());

// ✅ DO: Proper error propagation with context
let result = risky_operation()
    .map_err(|e| SimulationError::OperationFailed { 
        user_id: user_id.clone(), 
        source: e 
    })?;
```

**❌ Poor Naming Conventions**
```rust
// DON'T: Cryptic names that provide no context
fn calc(x: f64, y: f64) -> f64 { x + y }
let d: Duration;
fn proc(data: &[u8]) -> Result<(), Error> { /* ... */ }

// ✅ DO: Descriptive names that explain intent
fn calculate_gravitational_force(mass1: Mass, mass2: Mass, distance: f64) -> Force;
let integration_timestep: Duration;
fn process_simulation_state(particle_data: &[u8]) -> Result<(), SimulationError>;
```

**❌ Performance Anti-Patterns**
```rust
// DON'T: Unnecessary clones in hot paths
for particle in particles.iter() {
    let force = calculate_force(particle.clone()); // Expensive clone
}

// DON'T: Not using SIMD for physics calculations
for i in 0..positions.len() {
    forces[i] = calculate_scalar_force(positions[i]); // Scalar operations
}

// ✅ DO: Efficient memory usage and vectorization
for (particle, force) in particles.iter().zip(forces.iter_mut()) {
    *force = calculate_force(particle); // No clone needed
}

// Use SIMD for parallel force calculation
use std::simd::f64x4;
let simd_forces = calculate_vectorized_forces(&positions_simd);
```

**❌ Physics-Specific Anti-Patterns**
```rust
// DON'T: Using f32 for physics calculations (precision loss)
let position: Vector3<f32> = Vector3::new(1e12, 0.0, 0.0); // Precision issues

// DON'T: Ignoring numerical stability
let distance = (pos1 - pos2).magnitude(); // Could be zero!
let force = G * mass1 * mass2 / (distance * distance); // Division by zero

// ✅ DO: Use f64 and softening for stability
let position: Vector3<f64> = Vector3::new(1e12, 0.0, 0.0);
let distance_sq = (pos1 - pos2).magnitude_squared() + softening_sq;
let force = G * mass1 * mass2 / distance_sq;
```

### Language-Specific Anti-Patterns

**Rust Anti-Patterns:**
- ❌ Using `unwrap()` everywhere: `result.unwrap()`
- ❌ Not using `?` operator for error propagation
- ❌ Creating unnecessary clones: `data.clone().process()`
- ❌ Using `panic!()` for recoverable errors
- ❌ Not leveraging the type system for safety
- ❌ Ignoring compiler warnings

## Error Handling Patterns

### Rust Error Handling Patterns

```rust
// Custom error types using thiserror for clean error handling
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Validation error for field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Processing error: {message}")]
    Processing { message: String },

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Network error")]
    Network(#[from] reqwest::Error),
}

// Result-based error handling (idiomatic Rust)
pub async fn create_user(request: CreateUserRequest) -> Result<User, DomainError> {
    // Validate input
    if request.email.is_empty() {
        return Err(DomainError::Validation {
            field: "email".to_string(),
            message: "Email cannot be empty".to_string(),
        });
    }

    // Process with proper error propagation
    match user_repository.create(request).await {
        Ok(user) => {
            tracing::info!(
                user_id = %user.id,
                email = %user.email,
                "User created successfully"
            );
            Ok(user)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                request = ?request,
                "Failed to create user"
            );
            Err(DomainError::Processing {
                message: "User creation failed".to_string(),
            })
        }
    }
}
```

## Logging and Observability Patterns

### Structured Logging Templates

#### Universal Logging Fields

```text
- timestamp: ISO 8601 format timestamp
- level: log level (DEBUG, INFO, WARN, ERROR, FATAL)
- logger: component/class name
- message: human-readable message
- correlation_id: request/operation tracking
- user_id: authenticated user identifier
- operation: specific operation being performed
- duration: operation execution time
- error: error details if applicable
```

#### Language-Specific Logging

##### Rust (tracing)

```rust
use tracing::{info, error, warn, instrument, Span};
use std::time::Instant;

// Structured logging with tracing
#[derive(Debug)]
pub struct SimulationService {
    integrator: Box<dyn Integrator>,
}

impl SimulationService {
    #[instrument(
        name = "create_simulation", 
        skip(self, request),
        fields(
            particle_count = request.particles.len(),
            integrator_type = %request.integrator_type,
            timestep = request.timestep
        )
    )]
    pub async fn create_simulation(
        &self, 
        request: CreateSimulationRequest
    ) -> Result<Simulation, SimulationError> {
        let start = Instant::now();
        
        info!(
            particle_count = request.particles.len(),
            timestep = request.timestep,
            "Starting simulation creation"
        );

        match self.validate_request(&request) {
            Ok(_) => {
                info!("Simulation request validated successfully");
            }
            Err(e) => {
                error!(
                    error = %e,
                    particle_count = request.particles.len(),
                    "Simulation validation failed"
                );
                return Err(e);
            }
        }

        let simulation = match self.build_simulation(request).await {
            Ok(sim) => {
                info!(
                    simulation_id = %sim.id(),
                    duration = ?start.elapsed(),
                    "Simulation created successfully"
                );
                sim
            }
            Err(e) => {
                error!(
                    error = %e,
                    duration = ?start.elapsed(),
                    "Failed to build simulation"
                );
                return Err(SimulationError::BuildFailed { source: e });
            }
        };

        Ok(simulation)
    }

    #[instrument(skip(self))]
    async fn build_simulation(
        &self, 
        request: CreateSimulationRequest
    ) -> Result<Simulation, BuildError> {
        // Implementation with automatic span tracking
        todo!()
    }
}
```

## Performance Optimization Patterns

### Memory Management

```text
- Use object pooling for frequently allocated objects
- Implement proper resource disposal (using/with/defer patterns)
- Avoid memory leaks with proper cleanup of event listeners/subscriptions
- Use streaming for large data processing
- Implement pagination for large datasets
- Cache expensive computations with appropriate TTL
```

### Database Optimization

```text
- Use connection pooling with appropriate pool sizes
- Implement query batching for bulk operations
- Use proper indexing strategies
- Implement read replicas for read-heavy workloads
- Use database-specific optimization techniques
- Monitor query performance and optimize slow queries
```

### Caching Strategies

```text
- Application-level caching (in-memory, Redis)
- Database query result caching
- HTTP response caching with appropriate headers
- CDN usage for static assets
- Cache invalidation strategies
- Cache warming for critical data
```

## Concurrency and Async Patterns

### Language-Specific Concurrency

#### Rust Concurrency Patterns

```rust
use rayon::prelude::*;
use tokio::task;
use std::sync::Arc;
use std::time::Duration;

// Parallel processing with Rayon for CPU-intensive tasks
pub fn process_particles_parallel(
    particles: &mut [Particle],
    force_calculator: &dyn ForceCalculator,
) -> Result<(), SimulationError> {
    // Split particles into chunks for parallel processing
    let chunk_size = particles.len().div_ceil(rayon::current_num_threads());
    
    particles
        .par_chunks_mut(chunk_size)
        .enumerate()
        .try_for_each(|(chunk_idx, particle_chunk)| {
            // Process each chunk in parallel
            for (i, particle) in particle_chunk.iter_mut().enumerate() {
                let global_idx = chunk_idx * chunk_size + i;
                let force = force_calculator.calculate_force_for_particle(global_idx, particles)?;
                particle.apply_force(force);
            }
            Ok::<(), SimulationError>(())
        })?;
    
    Ok(())
}

// Async task spawning for I/O operations
pub async fn save_simulation_states(
    simulations: Vec<Simulation>,
    storage: Arc<dyn StorageService>,
) -> Result<Vec<String>, SimulationError> {
    let tasks: Vec<_> = simulations
        .into_iter()
        .map(|sim| {
            let storage = Arc::clone(&storage);
            task::spawn(async move {
                storage.save_simulation(sim).await
            })
        })
        .collect();
    
    // Wait for all tasks to complete
    let mut results = Vec::new();
    for task in tasks {
        let result = task.await??; // Handle both JoinError and SimulationError
        results.push(result);
    }
    
    Ok(results)
}

// Channel-based communication for streaming results
use tokio::sync::mpsc;

pub async fn stream_simulation_results(
    mut simulation: Simulation,
    steps: usize,
) -> mpsc::Receiver<SimulationState> {
    let (tx, rx) = mpsc::channel(100); // Buffer up to 100 states
    
    task::spawn(async move {
        for step in 0..steps {
            match simulation.step() {
                Ok(_) => {
                    let state = simulation.get_current_state();
                    if tx.send(state).await.is_err() {
                        // Receiver dropped, stop simulation
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Simulation step {} failed: {}", step, e);
                    break;
                }
            }
        }
    });
    
    rx
}

// SIMD-parallel force calculation
use std::simd::{f64x4, SimdFloat};

pub fn calculate_forces_simd(
    positions: &[Vector3<f64>],
    masses: &[f64],
    forces: &mut [Vector3<f64>],
) {
    forces.par_iter_mut().enumerate().for_each(|(i, force_i)| {
        let pos_i = positions[i];
        let mass_i = masses[i];
        
        // Process 4 particles at a time with SIMD
        for chunk in (0..positions.len()).collect::<Vec<_>>().chunks(4) {
            if chunk.contains(&i) { continue; } // Skip self-interaction
            
            // Load 4 positions into SIMD vectors
            let mut x_vals = [0.0; 4];
            let mut y_vals = [0.0; 4];
            let mut z_vals = [0.0; 4];
            let mut mass_vals = [0.0; 4];
            
            for (idx, &j) in chunk.iter().enumerate() {
                if j < positions.len() {
                    x_vals[idx] = positions[j].x;
                    y_vals[idx] = positions[j].y;
                    z_vals[idx] = positions[j].z;
                    mass_vals[idx] = masses[j];
                }
            }
            
            let x_simd = f64x4::from_array(x_vals);
            let y_simd = f64x4::from_array(y_vals);
            let z_simd = f64x4::from_array(z_vals);
            let mass_simd = f64x4::from_array(mass_vals);
            
            // Compute distance vectors
            let dx = x_simd - f64x4::splat(pos_i.x);
            let dy = y_simd - f64x4::splat(pos_i.y);
            let dz = z_simd - f64x4::splat(pos_i.z);
            
            // Distance squared and force calculation
            let r_sq = dx * dx + dy * dy + dz * dz;
            let r = r_sq.sqrt();
            let force_mag = f64x4::splat(G * mass_i) * mass_simd / r_sq;
            
            // Accumulate forces
            let fx = dx * force_mag / r;
            let fy = dy * force_mag / r;
            let fz = dz * force_mag / r;
            
            // Sum SIMD results
            let fx_array: [f64; 4] = fx.into();
            let fy_array: [f64; 4] = fy.into();
            let fz_array: [f64; 4] = fz.into();
            
            for k in 0..chunk.len() {
                if chunk[k] < positions.len() && !fx_array[k].is_nan() {
                    force_i.x += fx_array[k];
                    force_i.y += fy_array[k];
                    force_i.z += fz_array[k];
                }
            }
        }
    });
}
```

## Security Best Practices

### Input Validation and Sanitization

```text
- Validate all external inputs at boundaries
- Use allowlists over blocklists for validation
- Sanitize data before processing or storage
- Implement rate limiting for public endpoints
- Use parameterized queries to prevent SQL injection
- Validate file uploads (type, size, content)
- Implement proper authentication and authorization
```

### Secure Configuration Management

```text
- Store secrets in environment variables or secret managers
- Use different configurations for different environments
- Implement proper secret rotation strategies
- Avoid hardcoding sensitive information
- Use secure default configurations
- Implement configuration validation
```

### Audit Logging and Monitoring

```text
- Log all authentication and authorization events
- Log all data modification operations
- Implement proper log rotation and retention
- Monitor for suspicious activity patterns
- Use structured logging for better analysis
- Implement real-time alerting for security events
```

---

## Language-Specific Customization

### Quick Reference Commands

```bash
# Rust Physics Library Commands
# Core Development
cargo fmt                   # Format code with rustfmt
cargo clippy                # Run Rust linter
cargo check                 # Quick syntax/type check
cargo build --release       # Build optimized library
cargo test --release        # Run tests in release mode
cargo bench                 # Run criterion benchmarks
cargo doc --open            # Generate and open documentation

# Physics-Specific Validation
cargo test energy_conservation -- --nocapture  # Energy conservation tests
cargo test kepler_orbit -- --nocapture        # Orbital accuracy tests
cargo bench force_calculation                  # Benchmark force algorithms
cargo run --example solar_system              # Run solar system example


```

### Framework Integration

```text
Library Development Framework Patterns (Gravwell):
- nalgebra: Linear algebra operations, vector/matrix math
- rayon: Parallel computing for force calculations
- criterion: Performance benchmarking for optimization
- thiserror: Ergonomic error handling with custom error types
- tracing: Structured logging for debugging and performance analysis
- serde: Serialization for save/load simulation state
- wgpu: Optional GPU acceleration for large-scale simulations
```

---

## 🔬 Gravwell Specific Rust Patterns

### Core Library Architecture

#### Trait-Based Simulation Design
```rust
/// Core trait for numerical integrators
pub trait Integrator: Send + Sync {
    /// Advance the system by one timestep
    fn step(
        &mut self,
        positions: &mut [Vector3<f64>],
        velocities: &mut [Vector3<f64>],
        masses: &[Mass],
        dt: f64,
        force_calculator: &dyn ForceCalculator,
    ) -> Result<(), SimulationError>;

    /// Get integrator order (for adaptive stepping)
    fn order(&self) -> u32;

    /// Check if integrator is symplectic (energy-conserving)
    fn is_symplectic(&self) -> bool;
}

/// Velocity Verlet integrator - symplectic, 2nd order
#[derive(Debug, Clone)]
pub struct VelocityVerlet {
    accelerations: Vec<Vector3<f64>>,
}

impl Integrator for VelocityVerlet {
    fn step(
        &mut self,
        positions: &mut [Vector3<f64>],
        velocities: &mut [Vector3<f64>],
        masses: &[Mass],
        dt: f64,
        force_calculator: &dyn ForceCalculator,
    ) -> Result<(), SimulationError> {
        // Velocity Verlet algorithm implementation
        Ok(())
    }

    fn order(&self) -> u32 { 2 }
    fn is_symplectic(&self) -> bool { true }
}
```

#### Physics Simulation Patterns
```rust
use nalgebra::Vector3;
use rayon::prelude::*;
use std::sync::Arc;

/// Gravitational body representation
#[derive(Debug, Clone)]
pub struct Body {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub mass: Mass,
    pub radius: Option<f64>,
    pub name: Option<String>,
}

/// Mass with type safety and physical constants
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(pub f64);

impl Mass {
    pub fn new(value: f64) -> Result<Self, SimulationError> {
        if value <= 0.0 {
            return Err(SimulationError::InvalidMass(value));
        }
        Ok(Self(value))
    }

    pub const SOLAR_MASS: Self = Self(1.98847e30);
    pub const EARTH_MASS: Self = Self(5.97219e24);
    pub const JUPITER_MASS: Self = Self(1.89813e27);

    pub fn value(&self) -> f64 { self.0 }
}

/// Structure-of-Arrays for SIMD-friendly data layout
#[derive(Debug, Clone)]
pub struct ParticleSet {
    pub positions_x: Vec<f64>,
    pub positions_y: Vec<f64>,
    pub positions_z: Vec<f64>,
    pub velocities_x: Vec<f64>,
    pub velocities_y: Vec<f64>,
    pub velocities_z: Vec<f64>,
    pub masses: Vec<f64>,
    pub count: usize,
}

impl ParticleSet {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            positions_x: Vec::with_capacity(capacity),
            positions_y: Vec::with_capacity(capacity),
            positions_z: Vec::with_capacity(capacity),
            velocities_x: Vec::with_capacity(capacity),
            velocities_y: Vec::with_capacity(capacity),
            velocities_z: Vec::with_capacity(capacity),
            masses: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    pub fn add_body(&mut self, body: Body) {
        self.positions_x.push(body.position.x);
        self.positions_y.push(body.position.y);
        self.positions_z.push(body.position.z);
        self.velocities_x.push(body.velocity.x);
        self.velocities_y.push(body.velocity.y);
        self.velocities_z.push(body.velocity.z);
        self.masses.push(body.mass.value());
        self.count += 1;
    }

    pub fn get_body(&self, index: usize) -> Option<Body> {
        if index >= self.count {
            return None;
        }
        
        Some(Body {
            position: Vector3::new(
                self.positions_x[index],
                self.positions_y[index],
                self.positions_z[index],
            ),
            velocity: Vector3::new(
                self.velocities_x[index],
                self.velocities_y[index],
                self.velocities_z[index],
            ),
            mass: Mass(self.masses[index]),
            radius: None,
            name: None,
        })
    }
}
```

#### Force Calculation System (Library-Specific)
```rust
use nalgebra::Vector3;
use rayon::prelude::*;

/// Gravitational constant (m³ kg⁻¹ s⁻²)
pub const G: f64 = 6.67430e-11;

/// Trait for force calculation algorithms
pub trait ForceCalculator: Send + Sync {
    /// Calculate gravitational forces between all particles
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Vector3<f64>],
    ) -> Result<(), SimulationError>;

    /// Get computational complexity order
    fn complexity(&self) -> Complexity;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Complexity {
    QuadraticDirect,      // O(N²)
    TreeBased,           // O(N log N)
    FastMultipole,       // O(N)
}

/// Direct N-body force calculation - O(N²) but very accurate
#[derive(Debug, Clone)]
pub struct DirectGravity {
    softening_length: f64,
    use_simd: bool,
}

impl DirectGravity {
    pub fn new() -> Self {
        Self {
            softening_length: 0.0,
            use_simd: true,
        }
    }

    pub fn with_softening(mut self, epsilon: f64) -> Self {
        self.softening_length = epsilon;
        self
    }

    pub fn without_simd(mut self) -> Self {
        self.use_simd = false;
        self
    }
}

impl ForceCalculator for DirectGravity {
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Vector3<f64>],
    ) -> Result<(), SimulationError> {
        if forces.len() != particles.count {
            return Err(SimulationError::MismatchedArraySizes);
        }

        // Clear forces
        forces.iter_mut().for_each(|f| *f = Vector3::zeros());

        // Parallel force calculation
        forces.par_iter_mut().enumerate().for_each(|(i, force_i)| {
            for j in 0..particles.count {
                if i == j {
                    continue;
                }

                // Calculate distance vector
                let dx = particles.positions_x[j] - particles.positions_x[i];
                let dy = particles.positions_y[j] - particles.positions_y[i];
                let dz = particles.positions_z[j] - particles.positions_z[i];

                // Distance squared with softening
                let r_sq = dx * dx + dy * dy + dz * dz + 
                          self.softening_length * self.softening_length;
                let r = r_sq.sqrt();

                // Gravitational force magnitude
                let force_mag = G * particles.masses[i] * particles.masses[j] / r_sq;

                // Force components
                let force_over_r = force_mag / r;
                *force_i += Vector3::new(
                    dx * force_over_r,
                    dy * force_over_r,
                    dz * force_over_r,
                );
            }
        });

        Ok(())
    }

    fn complexity(&self) -> Complexity {
        Complexity::QuadraticDirect
    }
}

/// Barnes-Hut tree algorithm - O(N log N) approximation
#[derive(Debug, Clone)]
pub struct BarnesHut {
    theta: f64,          // Opening angle criterion
    tree: Option<OctreeNode>,
}

#[derive(Debug, Clone)]
struct OctreeNode {
    center_of_mass: Vector3<f64>,
    total_mass: f64,
    center: Vector3<f64>,
    size: f64,
    children: Option<Box<[OctreeNode; 8]>>,
    particle_index: Option<usize>,
}

impl BarnesHut {
    pub fn new() -> Self {
        Self {
            theta: 0.5,  // Standard theta value
            tree: None,
        }
    }

    pub fn theta(mut self, theta: f64) -> Self {
        self.theta = theta;
        self
    }

    fn build_tree(&mut self, particles: &ParticleSet) {
        // Tree construction implementation
        // (Complex implementation omitted for brevity)
    }

    fn calculate_force_recursive(
        &self,
        node: &OctreeNode,
        particle_idx: usize,
        particles: &ParticleSet,
    ) -> Vector3<f64> {
        // Recursive force calculation
        // (Implementation details omitted)
        Vector3::zeros()
    }
}

impl ForceCalculator for BarnesHut {
    fn calculate_forces(
        &self,
        particles: &ParticleSet,
        forces: &mut [Vector3<f64>],
    ) -> Result<(), SimulationError> {
        // Build octree and calculate forces
        // (Implementation details omitted for brevity)
        Ok(())
    }

    fn complexity(&self) -> Complexity {
        Complexity::TreeBased
    }
}
```

#### Simulation Builder Pattern (Library-Specific)
```rust
use std::marker::PhantomData;

/// Builder for configuring gravity simulations
#[derive(Debug)]
pub struct SimulationBuilder<I = DefaultIntegrator, F = DefaultForce> {
    integrator: Option<I>,
    force_calculator: Option<F>,
    timestep: Option<f64>,
    particles: ParticleSet,
    _phantom: PhantomData<(I, F)>,
}

/// Default integrator for simulations
type DefaultIntegrator = VelocityVerlet;
type DefaultForce = DirectGravity;

impl SimulationBuilder {
    /// Create a new simulation builder with default settings
    pub fn new() -> Self {
        Self {
            integrator: None,
            force_calculator: None,
            timestep: None,
            particles: ParticleSet::with_capacity(64),
            _phantom: PhantomData,
        }
    }
}

impl<I, F> SimulationBuilder<I, F> {
    /// Set the numerical integrator
    pub fn integrator<NewI: Integrator>(self, integrator: NewI) -> SimulationBuilder<NewI, F> {
        SimulationBuilder {
            integrator: Some(integrator),
            force_calculator: self.force_calculator,
            timestep: self.timestep,
            particles: self.particles,
            _phantom: PhantomData,
        }
    }

    /// Set the force calculation method
    pub fn gravity<NewF: ForceCalculator>(self, force_calc: NewF) -> SimulationBuilder<I, NewF> {
        SimulationBuilder {
            integrator: self.integrator,
            force_calculator: Some(force_calc),
            timestep: self.timestep,
            particles: self.particles,
            _phantom: PhantomData,
        }
    }

    /// Set simulation timestep
    pub fn timestep(mut self, dt: f64) -> Self {
        self.timestep = Some(dt);
        self
    }

    /// Add a gravitational body
    pub fn add_body(mut self, body: Body) -> Self {
        self.particles.add_body(body);
        self
    }

    /// Add multiple bodies from iterator
    pub fn add_bodies<Iter: IntoIterator<Item = Body>>(mut self, bodies: Iter) -> Self {
        for body in bodies {
            self.particles.add_body(body);
        }
        self
    }

    /// Build the simulation
    pub fn build(self) -> Result<Simulation<I, F>, BuildError>
    where
        I: Integrator + Default,
        F: ForceCalculator + Default,
    {
        let integrator = self.integrator.unwrap_or_default();
        let force_calculator = self.force_calculator.unwrap_or_default();
        let timestep = self.timestep.unwrap_or(0.01);

        if self.particles.count == 0 {
            return Err(BuildError::NoBodies);
        }

        if timestep <= 0.0 {
            return Err(BuildError::InvalidTimestep(timestep));
        }

        Ok(Simulation {
            integrator,
            force_calculator,
            particles: self.particles,
            timestep,
            time: 0.0,
            forces: vec![Vector3::zeros(); self.particles.count],
        })
    }
}

/// Main simulation structure
#[derive(Debug)]
pub struct Simulation<I: Integrator, F: ForceCalculator> {
    integrator: I,
    force_calculator: F,
    particles: ParticleSet,
    timestep: f64,
    time: f64,
    forces: Vec<Vector3<f64>>,
}

impl<I: Integrator, F: ForceCalculator> Simulation<I, F> {
    /// Create a new simulation builder
    pub fn builder() -> SimulationBuilder {
        SimulationBuilder::new()
    }

    /// Advance simulation by one timestep
    pub fn step(&mut self) -> Result<(), SimulationError> {
        // Calculate gravitational forces
        self.force_calculator.calculate_forces(&self.particles, &mut self.forces)?;

        // Integrate equations of motion
        let mut positions: Vec<Vector3<f64>> = (0..self.particles.count)
            .map(|i| Vector3::new(
                self.particles.positions_x[i],
                self.particles.positions_y[i],
                self.particles.positions_z[i],
            ))
            .collect();

        let mut velocities: Vec<Vector3<f64>> = (0..self.particles.count)
            .map(|i| Vector3::new(
                self.particles.velocities_x[i],
                self.particles.velocities_y[i],
                self.particles.velocities_z[i],
            ))
            .collect();

        let masses: Vec<Mass> = self.particles.masses
            .iter()
            .map(|&m| Mass(m))
            .collect();

        self.integrator.step(
            &mut positions,
            &mut velocities,
            &masses,
            self.timestep,
            &self.force_calculator,
        )?;

        // Update particle set
        for (i, (pos, vel)) in positions.iter().zip(velocities.iter()).enumerate() {
            self.particles.positions_x[i] = pos.x;
            self.particles.positions_y[i] = pos.y;
            self.particles.positions_z[i] = pos.z;
            self.particles.velocities_x[i] = vel.x;
            self.particles.velocities_y[i] = vel.y;
            self.particles.velocities_z[i] = vel.z;
        }

        self.time += self.timestep;
        Ok(())
    }

    /// Get current simulation time
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Get particle count
    pub fn particle_count(&self) -> usize {
        self.particles.count
    }

    /// Calculate total energy (kinetic + potential)
    pub fn total_energy(&self) -> f64 {
        let kinetic = self.kinetic_energy();
        let potential = self.potential_energy();
        kinetic + potential
    }

    fn kinetic_energy(&self) -> f64 {
        (0..self.particles.count)
            .map(|i| {
                let v_sq = self.particles.velocities_x[i].powi(2)
                         + self.particles.velocities_y[i].powi(2)
                         + self.particles.velocities_z[i].powi(2);
                0.5 * self.particles.masses[i] * v_sq
            })
            .sum()
    }

    fn potential_energy(&self) -> f64 {
        let mut potential = 0.0;
        for i in 0..self.particles.count {
            for j in (i + 1)..self.particles.count {
                let dx = self.particles.positions_x[j] - self.particles.positions_x[i];
                let dy = self.particles.positions_y[j] - self.particles.positions_y[i];
                let dz = self.particles.positions_z[j] - self.particles.positions_z[i];
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                
                if r > 0.0 {
                    potential -= G * self.particles.masses[i] * self.particles.masses[j] / r;
                }
            }
        }
        potential
    }
}
```

#### Performance Optimization Patterns
```rust
use rayon::prelude::*;
use std::simd::f64x4;

/// SIMD-optimized force calculation for modern CPUs
#[derive(Debug, Clone)]
pub struct SIMDGravity {
    softening: f64,
    chunk_size: usize,
}

impl SIMDGravity {
    pub fn new() -> Self {
        Self {
            softening: 0.0,
            chunk_size: 4, // Process 4 particles at once
        }
    }

    /// SIMD force calculation using portable_simd
    fn calculate_forces_simd(
        &self,
        particles: &ParticleSet,
        forces: &mut [Vector3<f64>],
    ) -> Result<(), SimulationError> {
        // Clear forces
        forces.par_iter_mut().for_each(|f| *f = Vector3::zeros());

        // Process in SIMD chunks
        let chunk_size = 4; // AVX2 can process 4 f64s at once
        
        for i_chunk in (0..particles.count).step_by(chunk_size) {
            let i_end = (i_chunk + chunk_size).min(particles.count);
            
            for j in 0..particles.count {
                if i_chunk <= j && j < i_end {
                    continue; // Skip self-interaction
                }

                // Load positions into SIMD vectors
                let mut i_x = [0.0f64; 4];
                let mut i_y = [0.0f64; 4];
                let mut i_z = [0.0f64; 4];
                let mut i_m = [0.0f64; 4];
                
                for (k, idx) in (i_chunk..i_end).enumerate() {
                    i_x[k] = particles.positions_x[idx];
                    i_y[k] = particles.positions_y[idx];
                    i_z[k] = particles.positions_z[idx];
                    i_m[k] = particles.masses[idx];
                }

                let i_x_simd = f64x4::from_array(i_x);
                let i_y_simd = f64x4::from_array(i_y);
                let i_z_simd = f64x4::from_array(i_z);
                let i_m_simd = f64x4::from_array(i_m);

                // Broadcast j particle data
                let j_x_simd = f64x4::splat(particles.positions_x[j]);
                let j_y_simd = f64x4::splat(particles.positions_y[j]);
                let j_z_simd = f64x4::splat(particles.positions_z[j]);
                let j_m_simd = f64x4::splat(particles.masses[j]);

                // Calculate distance vectors
                let dx = j_x_simd - i_x_simd;
                let dy = j_y_simd - i_y_simd;
                let dz = j_z_simd - i_z_simd;

                // Distance squared with softening
                let r_sq = dx * dx + dy * dy + dz * dz + f64x4::splat(self.softening * self.softening);
                let r = r_sq.sqrt();

                // Force magnitude
                let g_simd = f64x4::splat(G);
                let force_mag = g_simd * i_m_simd * j_m_simd / r_sq;

                // Force components
                let fx = dx * force_mag / r;
                let fy = dy * force_mag / r;
                let fz = dz * force_mag / r;

                // Store results back to forces array
                let fx_array: [f64; 4] = fx.into();
                let fy_array: [f64; 4] = fy.into();
                let fz_array: [f64; 4] = fz.into();

                for (k, idx) in (i_chunk..i_end).enumerate() {
                    forces[idx].x += fx_array[k];
                    forces[idx].y += fy_array[k];
                    forces[idx].z += fz_array[k];
                }
            }
        }

        Ok(())
    }
}

/// Adaptive timestep controller for maintaining accuracy
#[derive(Debug, Clone)]
pub struct AdaptiveTimestep {
    min_dt: f64,
    max_dt: f64,
    tolerance: f64,
    safety_factor: f64,
}

impl AdaptiveTimestep {
    pub fn new(min_dt: f64, max_dt: f64, tolerance: f64) -> Self {
        Self {
            min_dt,
            max_dt,
            tolerance,
            safety_factor: 0.9,
        }
    }

    /// Calculate optimal timestep based on system dynamics
    pub fn calculate_timestep(&self, particles: &ParticleSet) -> f64 {
        let mut min_dynamical_time = f64::INFINITY;

        // Find minimum orbital/dynamical timescale
        for i in 0..particles.count {
            for j in (i + 1)..particles.count {
                let dx = particles.positions_x[j] - particles.positions_x[i];
                let dy = particles.positions_y[j] - particles.positions_y[i];
                let dz = particles.positions_z[j] - particles.positions_z[i];
                let r = (dx * dx + dy * dy + dz * dz).sqrt();

                if r > 0.0 {
                    let total_mass = particles.masses[i] + particles.masses[j];
                    let orbital_period = 2.0 * std::f64::consts::PI * (r * r * r / (G * total_mass)).sqrt();
                    let dynamical_time = orbital_period / 100.0; // 100 steps per orbit
                    
                    min_dynamical_time = min_dynamical_time.min(dynamical_time);
                }
            }
        }

        // Clamp to safe range
        (min_dynamical_time * self.safety_factor)
            .max(self.min_dt)
            .min(self.max_dt)
    }
}

/// Memory pool for frequent allocations
#[derive(Debug)]
pub struct MemoryPool<T> {
    pool: std::sync::Mutex<Vec<Vec<T>>>,
    default_capacity: usize,
}

impl<T: Default + Clone> MemoryPool<T> {
    pub fn new(default_capacity: usize) -> Self {
        Self {
            pool: std::sync::Mutex::new(Vec::new()),
            default_capacity,
        }
    }

    pub fn get(&self) -> Vec<T> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop().unwrap_or_else(|| Vec::with_capacity(self.default_capacity))
    }

    pub fn return_vec(&self, mut vec: Vec<T>) {
        vec.clear();
        if vec.capacity() <= self.default_capacity * 2 {
            let mut pool = self.pool.lock().unwrap();
            pool.push(vec);
        }
        // If capacity is too large, just drop it to prevent memory bloat
    }
}

/// High-performance spatial hash grid for collision detection
#[derive(Debug, Clone)]
pub struct SpatialHashGrid {
    grid: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
    cell_size: f64,
}

impl SpatialHashGrid {
    pub fn new(cell_size: f64) -> Self {
        Self {
            grid: std::collections::HashMap::new(),
            cell_size,
        }
    }

    pub fn insert(&mut self, particle_idx: usize, position: Vector3<f64>) {
        let cell = self.get_cell_coords(position);
        self.grid.entry(cell).or_default().push(particle_idx);
    }

    pub fn query_neighbors(&self, position: Vector3<f64>, radius: f64) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let cells_to_check = ((radius / self.cell_size).ceil() as i32).max(1);
        
        let center_cell = self.get_cell_coords(position);
        
        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                for dz in -cells_to_check..=cells_to_check {
                    let cell = (
                        center_cell.0 + dx,
                        center_cell.1 + dy,
                        center_cell.2 + dz,
                    );
                    
                    if let Some(particles) = self.grid.get(&cell) {
                        neighbors.extend(particles.iter().copied());
                    }
                }
            }
        }
        
        neighbors
    }

    fn get_cell_coords(&self, position: Vector3<f64>) -> (i32, i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }
}
```

#### Performance Benchmarking Patterns
```rust
use criterion::{black_box, Criterion};
use std::time::{Duration, Instant};

/// Performance monitor for tracking simulation metrics
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    frame_times: std::collections::VecDeque<Duration>,
    max_samples: usize,
    last_frame: Instant,
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: std::collections::VecDeque::with_capacity(max_samples),
            max_samples,
            last_frame: Instant::now(),
        }
    }

    pub fn start_frame(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn end_frame(&mut self) {
        let frame_time = self.last_frame.elapsed();
        
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time);
    }

    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    pub fn fps(&self) -> f64 {
        let avg_frame_time = self.average_frame_time();
        if avg_frame_time.is_zero() {
            return 0.0;
        }
        1.0 / avg_frame_time.as_secs_f64()
    }

    pub fn percentile(&self, p: f64) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        
        let mut sorted: Vec<Duration> = self.frame_times.iter().copied().collect();
        sorted.sort();
        
        let index = ((sorted.len() - 1) as f64 * p / 100.0) as usize;
        sorted[index]
    }
}

/// Criterion benchmark setup for gravity simulation
pub fn benchmark_gravity_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_simulation");
    
    // Benchmark different particle counts
    for &n_particles in &[10, 50, 100, 500, 1000] {
        group.bench_function(
            format!("direct_n{}", n_particles),
            |b| {
                let mut sim = create_test_simulation_direct(n_particles);
                b.iter(|| {
                    black_box(sim.step()).unwrap();
                });
            },
        );
        
        group.bench_function(
            format!("barnes_hut_n{}", n_particles),
            |b| {
                let mut sim = create_test_simulation_barnes_hut(n_particles);
                b.iter(|| {
                    black_box(sim.step()).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

/// Create test simulation with direct force calculation
fn create_test_simulation_direct(n: usize) -> Simulation<VelocityVerlet, DirectGravity> {
    let mut builder = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(DirectGravity::new())
        .timestep(0.01);
    
    // Add test bodies in a rough circular distribution
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let radius = 1e11 * (1.0 + 0.1 * (i as f64 / 10.0));
        
        let position = Vector3::new(
            radius * angle.cos(),
            radius * angle.sin(),
            0.0,
        );
        
        let velocity = Vector3::new(
            -3e4 * angle.sin(),
            3e4 * angle.cos(),
            0.0,
        );
        
        builder = builder.add_body(Body {
            position,
            velocity,
            mass: Mass::EARTH_MASS,
            radius: Some(6.371e6),
            name: Some(format!("Body{}", i)),
        });
    }
    
    builder.build().unwrap()
}

/// Create test simulation with Barnes-Hut
fn create_test_simulation_barnes_hut(n: usize) -> Simulation<VelocityVerlet, BarnesHut> {
    let mut builder = Simulation::builder()
        .integrator(VelocityVerlet::new())
        .gravity(BarnesHut::new().theta(0.5))
        .timestep(0.01);
    
    // Same body setup as direct method
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let radius = 1e11 * (1.0 + 0.1 * (i as f64 / 10.0));
        
        let position = Vector3::new(
            radius * angle.cos(),
            radius * angle.sin(),
            0.0,
        );
        
        let velocity = Vector3::new(
            -3e4 * angle.sin(),
            3e4 * angle.cos(),
            0.0,
        );
        
        builder = builder.add_body(Body {
            position,
            velocity,
            mass: Mass::EARTH_MASS,
            radius: Some(6.371e6),
            name: Some(format!("Body{}", i)),
        });
    }
    
    builder.build().unwrap()
}

/// Real-time performance target checker
pub struct TargetPerformance {
    target_fps: f64,
    target_frame_time: Duration,
    violations: usize,
    total_frames: usize,
}

impl TargetPerformance {
    pub fn new(target_fps: f64) -> Self {
        Self {
            target_fps,
            target_frame_time: Duration::from_secs_f64(1.0 / target_fps),
            violations: 0,
            total_frames: 0,
        }
    }

    pub fn check_frame(&mut self, frame_time: Duration) {
        self.total_frames += 1;
        if frame_time > self.target_frame_time {
            self.violations += 1;
        }
    }

    pub fn violation_rate(&self) -> f64 {
        if self.total_frames == 0 {
            return 0.0;
        }
        self.violations as f64 / self.total_frames as f64
    }

    pub fn meets_target(&self, max_violation_rate: f64) -> bool {
        self.violation_rate() <= max_violation_rate
    }
}
```

This provides Gravwell-specific Rust patterns focusing on physics simulation, library architecture, performance optimization, SIMD acceleration, and comprehensive benchmarking for scientific computing applications.
