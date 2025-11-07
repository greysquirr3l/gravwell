//! Runge-Kutta 4th order integrator (non-symplectic, 4th order).

use crate::{
    core::{
        forces::ForceCalculator,
        integrator::{validate_timestep, Integrator},
        particle::{Body, ParticleSet},
    },
    error::Result,
    types::Time,
};

/// Runge-Kutta 4th order integrator (RK4).
///
/// A highly accurate, non-symplectic integrator that uses four intermediate
/// evaluations (k1, k2, k3, k4) to achieve 4th-order accuracy in the timestep.
/// Excellent for scientific applications requiring high precision over shorter
/// timescales.
///
/// The RK4 method evaluates the derivative (acceleration) at four points:
/// 1. **k1**: At the beginning of the interval (t, y)
/// 2. **k2**: At the midpoint using k1 slope (t+dt/2, y+k1*dt/2)  
/// 3. **k3**: At the midpoint using k2 slope (t+dt/2, y+k2*dt/2)
/// 4. **k4**: At the end using k3 slope (t+dt, y+k3*dt)
///
/// The final update combines all four with weights: y' = y + (k1 + 2*k2 + 2*k3 + k4)*dt/6
///
/// # Scientific Properties
///
/// - **Order**: 4th order accuracy in timestep (error ~ O(dt⁵))
/// - **Symplectic**: No (does not preserve phase space volume)
/// - **Energy Conservation**: Good for short timescales, may drift for long simulations
/// - **Stability**: Excellent stability properties for smooth force fields
/// - **Optimal for**: High-precision short-term calculations, smooth dynamics
///
/// # Performance Characteristics  
///
/// - **CPU Cost**: 4 force evaluations per timestep (2x more expensive than 2nd order methods)
/// - **Memory**: O(12N) temporary storage for k-vectors (positions + velocities)
/// - **Accuracy vs Cost**: Excellent - can often use larger timesteps than 2nd order methods
/// - **Cache Performance**: Good with Structure-of-Arrays, but higher memory bandwidth
///
/// # When to Use RK4
///
/// - **High-precision scientific calculations** where accuracy is paramount
/// - **Smooth force fields** without discontinuities or near-singularities  
/// - **Short to medium-term** simulations (years to decades, not millennia)
/// - **Comparison/validation** against analytical solutions
/// - **Systems with complex dynamics** requiring high-order accuracy
///
/// # When NOT to Use RK4
///
/// - **Long-term orbital mechanics** (use symplectic methods like Leapfrog)
/// - **Real-time applications** where 4 force evaluations are too expensive
/// - **Energy conservation** is critical over very long timescales
/// - **Near-singular forces** (close encounters in N-body systems)
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let integrator = RungeKutta4::new();
///
/// let mut sim = Simulation::builder()
///     .with_integrator(integrator)
///     .with_force_calculator(DirectGravity::new())
///     .add_body(Body::new()
///         .with_mass(Mass::SOLAR_MASS)
///         .with_position([0.0, 0.0, 0.0])
///         .with_velocity([0.0, 0.0, 0.0])
///     )?
///     .build()?;
///
/// // Use smaller timestep to take advantage of high accuracy
/// sim.step(0.001)?; // 1 millisecond timestep
/// # Ok::<(), gravwell::Error>(())
/// ```
///
/// # Algorithm Details
///
/// For a system dx/dt = f(t,x), the RK4 update is:
///
/// ```text
/// k1 = f(t_n,         x_n)
/// k2 = f(t_n + dt/2,  x_n + k1*dt/2)  
/// k3 = f(t_n + dt/2,  x_n + k2*dt/2)
/// k4 = f(t_n + dt,    x_n + k3*dt)
/// x_{n+1} = x_n + (k1 + 2*k2 + 2*k3 + k4) * dt/6
/// ```
///
/// In our case, x = [positions, velocities] and f computes [velocities, accelerations].
///
/// # References
///
/// - Runge, C. (1895). "Über die numerische Auflösung von Differentialgleichungen"
/// - Kutta, W. (1901). "Beitrag zur näherungsweisen Integration totaler Differentialgleichungen"  
/// - Butcher, J.C. (2003). "Numerical Methods for Ordinary Differential Equations"
/// - Press, W.H. et al. (2007). "Numerical Recipes: The Art of Scientific Computing"
#[derive(Debug, Clone, Default)]
pub struct RungeKutta4 {
    /// Temporary storage for k1 position derivatives (velocities)
    k1_positions: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k1 velocity derivatives (accelerations)
    k1_velocities: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k2 position derivatives
    k2_positions: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k2 velocity derivatives  
    k2_velocities: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k3 position derivatives
    k3_positions: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k3 velocity derivatives
    k3_velocities: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k4 position derivatives
    k4_positions: Vec<nalgebra::Vector3<f64>>,
    /// Temporary storage for k4 velocity derivatives
    k4_velocities: Vec<nalgebra::Vector3<f64>>,
    /// Temporary particle state for intermediate evaluations
    temp_particles: ParticleSet,
}

impl RungeKutta4 {
    /// Create a new RK4 integrator.
    ///
    /// The integrator initializes with empty temporary storage vectors
    /// that will be allocated on the first call to `step()`.
    pub fn new() -> Self {
        Self {
            k1_positions: Vec::new(),
            k1_velocities: Vec::new(),
            k2_positions: Vec::new(),
            k2_velocities: Vec::new(),
            k3_positions: Vec::new(),
            k3_velocities: Vec::new(),
            k4_positions: Vec::new(),
            k4_velocities: Vec::new(),
            temp_particles: ParticleSet::new(),
        }
    }

    /// Reset the integrator state and clear temporary storage.
    ///
    /// Useful when changing particle counts or restarting simulations.
    pub fn reset(&mut self) {
        self.k1_positions.clear();
        self.k1_velocities.clear();
        self.k2_positions.clear();
        self.k2_velocities.clear();
        self.k3_positions.clear();
        self.k3_velocities.clear();
        self.k4_positions.clear();
        self.k4_velocities.clear();
        self.temp_particles = ParticleSet::new();
    }

    /// Check if temporary storage is allocated for the current particle count.
    pub fn is_allocated(&self, n_particles: usize) -> bool {
        self.k1_positions.len() == n_particles
            && self.k1_velocities.len() == n_particles
            && self.k2_positions.len() == n_particles
            && self.k2_velocities.len() == n_particles
            && self.k3_positions.len() == n_particles
            && self.k3_velocities.len() == n_particles
            && self.k4_positions.len() == n_particles
            && self.k4_velocities.len() == n_particles
    }
}

impl Integrator for RungeKutta4 {
    /// Perform one RK4 integration step.
    ///
    /// Implements the classic 4th-order Runge-Kutta method with four stages:
    ///
    /// 1. **Stage 1 (k1)**: Evaluate derivatives at current state
    /// 2. **Stage 2 (k2)**: Evaluate derivatives at midpoint using k1 slope
    /// 3. **Stage 3 (k3)**: Evaluate derivatives at midpoint using k2 slope  
    /// 4. **Stage 4 (k4)**: Evaluate derivatives at endpoint using k3 slope
    /// 5. **Update**: Combine all stages with RK4 weights
    ///
    /// Each stage requires a force evaluation, making RK4 computationally
    /// expensive but very accurate.
    ///
    /// # Arguments
    ///
    /// * `particles` - Mutable reference to particle positions, velocities, masses
    /// * `forces` - Force calculator implementing the ForceCalculator trait
    /// * `dt` - Integration timestep in seconds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The timestep is not positive and finite
    /// - Force calculation fails at any stage
    /// - Memory allocation fails
    /// - Temporary particle state becomes invalid
    ///
    /// # Performance Notes
    ///
    /// - **4 force evaluations** per timestep (expensive)
    /// - **12N memory** for temporary k-vectors
    /// - **Copy operations** for intermediate particle states
    /// - Consider larger timesteps to amortize cost
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        validate_timestep(dt)?;

        let n = particles.len();
        if n == 0 {
            return Ok(());
        }

        // Allocate temporary storage if needed
        if !self.is_allocated(n) {
            self.allocate_storage(n);
        }

        // Store initial state for intermediate calculations
        self.copy_particle_state(particles)?;

        // Stage 1: k1 = f(t_n, y_n)
        // k1 represents derivatives at the current state
        self.evaluate_stage_1(particles, forces)?;

        // Stage 2: k2 = f(t_n + dt/2, y_n + k1*dt/2)
        // k2 represents derivatives at midpoint using k1 slope
        self.evaluate_stage_2(particles, forces, dt)?;

        // Stage 3: k3 = f(t_n + dt/2, y_n + k2*dt/2)
        // k3 represents derivatives at midpoint using k2 slope
        self.evaluate_stage_3(particles, forces, dt)?;

        // Stage 4: k4 = f(t_n + dt, y_n + k3*dt)
        // k4 represents derivatives at endpoint using k3 slope
        self.evaluate_stage_4(particles, forces, dt)?;

        // Final RK4 update: y_{n+1} = y_n + (k1 + 2*k2 + 2*k3 + k4) * dt/6
        self.apply_rk4_update(particles, dt)?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Runge-Kutta 4"
    }

    fn is_symplectic(&self) -> bool {
        false
    }

    fn order(&self) -> u8 {
        4
    }
}

impl RungeKutta4 {
    /// Allocate temporary storage vectors for the given number of particles.
    fn allocate_storage(&mut self, n: usize) {
        self.k1_positions.clear();
        self.k1_positions.resize(n, nalgebra::Vector3::zeros());

        self.k1_velocities.clear();
        self.k1_velocities.resize(n, nalgebra::Vector3::zeros());

        self.k2_positions.clear();
        self.k2_positions.resize(n, nalgebra::Vector3::zeros());

        self.k2_velocities.clear();
        self.k2_velocities.resize(n, nalgebra::Vector3::zeros());

        self.k3_positions.clear();
        self.k3_positions.resize(n, nalgebra::Vector3::zeros());

        self.k3_velocities.clear();
        self.k3_velocities.resize(n, nalgebra::Vector3::zeros());

        self.k4_positions.clear();
        self.k4_positions.resize(n, nalgebra::Vector3::zeros());

        self.k4_velocities.clear();
        self.k4_velocities.resize(n, nalgebra::Vector3::zeros());
    }

    /// Copy the current particle state to temporary storage for intermediate calculations.
    fn copy_particle_state(&mut self, particles: &ParticleSet) -> Result<()> {
        // Reset temporary particle set
        self.temp_particles = ParticleSet::new();

        // Copy all particles to temporary storage
        for i in 0..particles.len() {
            let body = Body::new()
                .with_mass(particles.mass(i))
                .with_position([
                    particles.position(i).x,
                    particles.position(i).y,
                    particles.position(i).z,
                ])
                .with_velocity([
                    particles.velocity(i).x,
                    particles.velocity(i).y,
                    particles.velocity(i).z,
                ]);
            self.temp_particles.add_body(body)?;
        }

        Ok(())
    }

    /// Stage 1: Evaluate derivatives at current state.
    /// k1 = f(t_n, y_n)
    fn evaluate_stage_1<F>(&mut self, particles: &ParticleSet, forces: &F) -> Result<()>
    where
        F: ForceCalculator,
    {
        let n = particles.len();

        // k1 positions = current velocities
        for i in 0..n {
            self.k1_positions[i] = *particles.velocity(i);
        }

        // k1 velocities = current accelerations
        forces.calculate_forces(particles, &mut self.k1_velocities)?;
        for i in 0..n {
            let mass = particles.mass(i);
            self.k1_velocities[i] /= mass;
        }

        Ok(())
    }

    /// Stage 2: Evaluate derivatives at midpoint using k1 slope.
    /// k2 = f(t_n + dt/2, y_n + k1*dt/2)
    fn evaluate_stage_2<F>(&mut self, particles: &ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        let n = particles.len();
        let half_dt = dt * 0.5;

        // Update temp_particles to midpoint state using k1
        for i in 0..n {
            let new_position = particles.position(i) + self.k1_positions[i] * half_dt;
            let new_velocity = particles.velocity(i) + self.k1_velocities[i] * half_dt;

            *self.temp_particles.position_mut(i) = new_position;
            *self.temp_particles.velocity_mut(i) = new_velocity;
        }

        // k2 positions = velocities at midpoint
        for i in 0..n {
            self.k2_positions[i] = *self.temp_particles.velocity(i);
        }

        // k2 velocities = accelerations at midpoint
        forces.calculate_forces(&self.temp_particles, &mut self.k2_velocities)?;
        for i in 0..n {
            let mass = self.temp_particles.mass(i);
            self.k2_velocities[i] /= mass;
        }

        Ok(())
    }

    /// Stage 3: Evaluate derivatives at midpoint using k2 slope.
    /// k3 = f(t_n + dt/2, y_n + k2*dt/2)
    fn evaluate_stage_3<F>(&mut self, particles: &ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        let n = particles.len();
        let half_dt = dt * 0.5;

        // Update temp_particles to midpoint state using k2
        for i in 0..n {
            let new_position = particles.position(i) + self.k2_positions[i] * half_dt;
            let new_velocity = particles.velocity(i) + self.k2_velocities[i] * half_dt;

            *self.temp_particles.position_mut(i) = new_position;
            *self.temp_particles.velocity_mut(i) = new_velocity;
        }

        // k3 positions = velocities at midpoint
        for i in 0..n {
            self.k3_positions[i] = *self.temp_particles.velocity(i);
        }

        // k3 velocities = accelerations at midpoint
        forces.calculate_forces(&self.temp_particles, &mut self.k3_velocities)?;
        for i in 0..n {
            let mass = self.temp_particles.mass(i);
            self.k3_velocities[i] /= mass;
        }

        Ok(())
    }

    /// Stage 4: Evaluate derivatives at endpoint using k3 slope.
    /// k4 = f(t_n + dt, y_n + k3*dt)
    fn evaluate_stage_4<F>(&mut self, particles: &ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        let n = particles.len();

        // Update temp_particles to endpoint state using k3
        for i in 0..n {
            let new_position = particles.position(i) + self.k3_positions[i] * dt;
            let new_velocity = particles.velocity(i) + self.k3_velocities[i] * dt;

            *self.temp_particles.position_mut(i) = new_position;
            *self.temp_particles.velocity_mut(i) = new_velocity;
        }

        // k4 positions = velocities at endpoint
        for i in 0..n {
            self.k4_positions[i] = *self.temp_particles.velocity(i);
        }

        // k4 velocities = accelerations at endpoint
        forces.calculate_forces(&self.temp_particles, &mut self.k4_velocities)?;
        for i in 0..n {
            let mass = self.temp_particles.mass(i);
            self.k4_velocities[i] /= mass;
        }

        Ok(())
    }

    /// Apply the final RK4 update using weighted combination of all stages.
    /// y_{n+1} = y_n + (k1 + 2*k2 + 2*k3 + k4) * dt/6
    fn apply_rk4_update(&mut self, particles: &mut ParticleSet, dt: Time) -> Result<()> {
        let n = particles.len();
        let dt_over_6 = dt / 6.0;

        for i in 0..n {
            // Update positions using weighted combination of position derivatives
            let position_update = (self.k1_positions[i]
                + 2.0 * self.k2_positions[i]
                + 2.0 * self.k3_positions[i]
                + self.k4_positions[i])
                * dt_over_6;

            *particles.position_mut(i) += position_update;

            // Update velocities using weighted combination of velocity derivatives (accelerations)
            let velocity_update = (self.k1_velocities[i]
                + 2.0 * self.k2_velocities[i]
                + 2.0 * self.k3_velocities[i]
                + self.k4_velocities[i])
                * dt_over_6;

            *particles.velocity_mut(i) += velocity_update;
        }

        Ok(())
    }
}
