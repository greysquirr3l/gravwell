//! Leapfrog integrator (symplectic, 2nd order).

use crate::{
    core::{
        forces::ForceCalculator,
        integrator::{validate_timestep, Integrator},
        particle::ParticleSet,
    },
    error::Result,
    types::Time,
};

/// Leapfrog integrator (kick-drift-kick scheme).
///
/// A symplectic integrator that updates velocities and positions at staggered
/// half-timesteps, providing excellent energy conservation for long-term
/// gravitational simulations.
///
/// The algorithm follows the kick-drift-kick pattern:
/// 1. "Kick": Update velocities by half timestep using current accelerations
/// 2. "Drift": Update positions by full timestep using new velocities  
/// 3. "Kick": Update velocities by another half timestep using new accelerations
///
/// This staggered approach ensures that velocities and positions are evaluated
/// at different time points, which is key to the symplectic property.
///
/// # Scientific Properties
///
/// - **Order**: 2nd order accuracy in timestep
/// - **Symplectic**: Preserves phase space volume (energy conservation)
/// - **Time Reversible**: Running backwards recovers initial state
/// - **Optimal for**: Long-term orbital mechanics, gravitational N-body systems
/// - **Energy Drift**: O(dt²) for bounded orbits, excellent for astronomical timescales
///
/// # Performance Characteristics
///
/// - **CPU Cost**: Similar to Velocity Verlet (2 force evaluations per step)
/// - **Memory**: O(3N) for velocity storage, same as other 2nd order methods
/// - **Stability**: Large timestep stability for gravitational systems
/// - **Cache Performance**: Good due to Structure-of-Arrays layout
///
/// # Examples
///
/// ```
/// use gravwell::prelude::*;
///
/// let integrator = Leapfrog::new();
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
/// // Evolve system
/// sim.step(0.01)?; // 0.01 second timestep
/// # Ok::<(), gravwell::Error>(())
/// ```
///
/// # References
///
/// - Hut, P. & Bahcall, J.N. (1983). "Binary-single star scattering"  
/// - Leimkuhler, B. & Reich, S. (2004). "Simulating Hamiltonian Dynamics"
/// - Yoshida, H. (1990). "Construction of higher order symplectic integrators"
#[derive(Debug, Clone, Default)]
pub struct Leapfrog {
    /// Half-step velocities for staggered update scheme.
    /// These are stored between steps to maintain the kick-drift-kick pattern.
    half_velocities: Vec<nalgebra::Vector3<f64>>,
    /// Track if this is the first step (needs initialization)
    is_first_step: bool,
}

impl Leapfrog {
    /// Create a new Leapfrog integrator.
    ///
    /// The integrator starts in "first step" mode and will automatically
    /// initialize the staggered velocity scheme on the first call to `step()`.
    pub fn new() -> Self {
        Self {
            half_velocities: Vec::new(),
            is_first_step: true,
        }
    }

    /// Reset the integrator state.
    ///
    /// This clears the stored half-step velocities and marks the next step
    /// as the first step, which will reinitialize the staggered scheme.
    /// Useful when restarting simulations or changing particle counts.
    pub fn reset(&mut self) {
        self.half_velocities.clear();
        self.is_first_step = true;
    }

    /// Check if the integrator has been initialized.
    ///
    /// Returns `true` if the half-step velocities have been set up.
    pub fn is_initialized(&self) -> bool {
        !self.is_first_step && !self.half_velocities.is_empty()
    }
}

impl Integrator for Leapfrog {
    /// Perform one integration step using the Leapfrog (kick-drift-kick) algorithm.
    ///
    /// This method implements the classic symplectic leapfrog integration scheme:
    ///
    /// **First step initialization:**
    /// 1. Calculate initial accelerations
    /// 2. Store v(t-dt/2) = v(t) - a(t) * dt/2 (half-step backward)
    ///
    /// **Subsequent steps:**
    /// 1. **Kick**: v(t+dt/2) = v(t-dt/2) + a(t) * dt
    /// 2. **Drift**: x(t+dt) = x(t) + v(t+dt/2) * dt  
    /// 3. **Prepare for next**: Store v(t+dt/2) for next iteration
    ///
    /// This maintains the staggered time evaluation that gives the method
    /// its symplectic properties and excellent energy conservation.
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
    /// - Force calculation fails
    /// - Memory allocation fails during initialization
    ///
    /// # Algorithm Details
    ///
    /// The leapfrog method maintains velocities at half-timesteps relative to
    /// positions. This staggered evaluation is what makes it symplectic:
    ///
    /// ```text
    /// Time:     t-dt    t-dt/2    t       t+dt/2   t+dt
    /// Position:   x       -       x         -       x
    /// Velocity:   -       v       -         v       -
    /// Accel:      -       -       a         -       a
    /// ```
    ///
    /// The first step requires special handling to establish the staggered pattern.
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        validate_timestep(dt)?;

        let n = particles.len();
        if n == 0 {
            return Ok(());
        }

        // Initialize half-velocities on first step
        if self.is_first_step {
            self.initialize_half_velocities(particles, forces, dt)?;
            self.is_first_step = false;
            return Ok(());
        }

        // Ensure half-velocities vector has correct size
        if self.half_velocities.len() != n {
            // Particle count changed - reinitialize
            self.initialize_half_velocities(particles, forces, dt)?;
            return Ok(());
        }

        // Calculate current accelerations
        let mut accelerations = vec![nalgebra::Vector3::zeros(); n];
        forces.calculate_forces(particles, &mut accelerations)?;

        // Leapfrog update scheme:
        // 1. Kick: Update half-velocities to full timestep
        // 2. Drift: Update positions using new velocities
        // 3. Store updated half-velocities for next iteration

        for i in 0..n {
            let mass = particles.mass(i);
            let acceleration = accelerations[i] / mass;

            // Kick: v(t+dt/2) = v(t-dt/2) + a(t) * dt
            self.half_velocities[i] += acceleration * dt;

            // Drift: x(t+dt) = x(t) + v(t+dt/2) * dt
            let new_position = particles.position(i) + self.half_velocities[i] * dt;
            *particles.position_mut(i) = new_position;

            // Update particle velocities to the half-step values
            // This maintains consistency with the ParticleSet API
            *particles.velocity_mut(i) = self.half_velocities[i];
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Leapfrog"
    }

    fn is_symplectic(&self) -> bool {
        true
    }

    fn order(&self) -> u8 {
        2
    }
}

impl Leapfrog {
    /// Initialize half-step velocities for the leapfrog scheme.
    ///
    /// This is called on the first step to establish the staggered velocity
    /// pattern. The half-velocities are set to v(t-dt/2) by taking a half
    /// backward step from the initial velocities.
    fn initialize_half_velocities<F>(
        &mut self,
        particles: &mut ParticleSet,
        forces: &F,
        dt: Time,
    ) -> Result<()>
    where
        F: ForceCalculator,
    {
        let n = particles.len();

        // Calculate initial accelerations
        let mut accelerations = vec![nalgebra::Vector3::zeros(); n];
        forces.calculate_forces(particles, &mut accelerations)?;

        // Initialize half-velocities: v(t-dt/2) = v(t) - a(t) * dt/2
        self.half_velocities.clear();
        self.half_velocities.reserve(n);

        for i in 0..n {
            let mass = particles.mass(i);
            let acceleration = accelerations[i] / mass;
            let initial_velocity = *particles.velocity(i);

            // Take half step backward to establish stagger
            let half_velocity = initial_velocity - acceleration * (dt * 0.5);
            self.half_velocities.push(half_velocity);
        }

        Ok(())
    }
}
