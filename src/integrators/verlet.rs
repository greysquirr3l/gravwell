//! Velocity Verlet integrator (symplectic, 2nd order).

use crate::{
    core::{
        forces::ForceCalculator,
        integrator::{validate_accelerations, validate_timestep, Integrator},
        particle::ParticleSet,
    },
    error::Result,
    types::{Acceleration, Time},
};

/// Velocity Verlet integrator.
///
/// This is a symplectic integrator that provides good energy conservation
/// for long-term simulations. It's particularly well-suited for gravitational
/// systems where energy conservation is important.
#[derive(Debug, Clone, Default)]
pub struct VelocityVerlet;

impl VelocityVerlet {
    /// Create a new Velocity Verlet integrator.
    pub fn new() -> Self {
        Self
    }
}

impl Integrator for VelocityVerlet {
    fn step<F>(&mut self, particles: &mut ParticleSet, forces: &F, dt: Time) -> Result<()>
    where
        F: ForceCalculator,
    {
        validate_timestep(dt)?;

        let n = particles.len();
        let mut accelerations = vec![Acceleration::zeros(); n];

        // Calculate initial accelerations
        forces.calculate_accelerations(particles, &mut accelerations)?;
        validate_accelerations(&accelerations)?;

        // Update positions: x += v*dt + 0.5*a*dt²
        for i in 0..n {
            let velocity = *particles.velocity(i);
            *particles.position_mut(i) += velocity * dt + 0.5 * accelerations[i] * dt * dt;
        }

        // Calculate new accelerations at new positions
        let mut new_accelerations = vec![Acceleration::zeros(); n];
        forces.calculate_accelerations(particles, &mut new_accelerations)?;
        validate_accelerations(&new_accelerations)?;

        // Update velocities: v += 0.5*(a_old + a_new)*dt
        for i in 0..n {
            *particles.velocity_mut(i) += 0.5 * (accelerations[i] + new_accelerations[i]) * dt;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Velocity Verlet"
    }

    fn is_symplectic(&self) -> bool {
        true
    }

    fn order(&self) -> u8 {
        2
    }
}
