//! Particle and body definitions for the simulation.

use crate::{
    core::math::Math,
    error::{GravwellError, Result},
    types::{Mass, Position, Scalar, Velocity},
};

/// A single gravitational body in the simulation.
#[derive(Debug, Clone)]
pub struct Body {
    /// Mass of the body in kilograms.
    pub mass: Mass,
    /// Position in 3D space (meters).
    pub position: Position,
    /// Velocity vector (m/s).
    pub velocity: Velocity,
}

impl Body {
    /// Create a new body with default values (mass=1, origin position, zero velocity).
    pub fn new() -> Self {
        Self {
            mass: 1.0,
            position: Position::zeros(),
            velocity: Velocity::zeros(),
        }
    }

    /// Set the mass of this body.
    pub fn with_mass(mut self, mass: Mass) -> Self {
        self.mass = mass;
        self
    }

    /// Set the position of this body.
    pub fn with_position(mut self, position: [f64; 3]) -> Self {
        self.position = Position::new(position[0], position[1], position[2]);
        self
    }

    /// Set the velocity of this body.
    pub fn with_velocity(mut self, velocity: [f64; 3]) -> Self {
        self.velocity = Velocity::new(velocity[0], velocity[1], velocity[2]);
        self
    }

    /// Validate that this body has physically reasonable values.
    pub fn validate(&self) -> Result<()> {
        if self.mass <= 0.0 || !self.mass.is_finite() {
            return Err(GravwellError::invalid_particle(format!(
                "Invalid mass: {}",
                self.mass
            )));
        }

        if !Math::is_valid_vector(&self.position) {
            return Err(GravwellError::invalid_particle(format!(
                "Invalid position: [{}, {}, {}]",
                self.position.x, self.position.y, self.position.z
            )));
        }

        if !Math::is_valid_vector(&self.velocity) {
            return Err(GravwellError::invalid_particle(format!(
                "Invalid velocity: [{}, {}, {}]",
                self.velocity.x, self.velocity.y, self.velocity.z
            )));
        }

        Ok(())
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::new()
    }
}

// Note: From implementations for nalgebra types are handled by nalgebra itself

/// A collection of particles organized for efficient computation.
///
/// Uses Structure-of-Arrays (SoA) layout for better cache performance
/// and SIMD vectorization opportunities.
#[derive(Debug, Clone)]
pub struct ParticleSet {
    /// Masses of all particles.
    masses: Vec<Mass>,
    /// Positions of all particles.
    positions: Vec<Position>,
    /// Velocities of all particles.
    velocities: Vec<Velocity>,
}

impl ParticleSet {
    /// Create an empty particle set.
    pub fn new() -> Self {
        Self {
            masses: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    /// Create a particle set with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            masses: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
        }
    }

    /// Add a body to the particle set.
    pub fn add_body(&mut self, body: Body) -> Result<()> {
        body.validate()?;

        self.masses.push(body.mass);
        self.positions.push(body.position);
        self.velocities.push(body.velocity);

        Ok(())
    }

    /// Get the number of particles.
    pub fn len(&self) -> usize {
        self.masses.len()
    }

    /// Check if the particle set is empty.
    pub fn is_empty(&self) -> bool {
        self.masses.is_empty()
    }

    /// Get the mass of a particle by index.
    pub fn mass(&self, index: usize) -> Mass {
        self.masses[index]
    }

    /// Get the position of a particle by index.
    pub fn position(&self, index: usize) -> &Position {
        &self.positions[index]
    }

    /// Get a mutable reference to the position of a particle.
    pub fn position_mut(&mut self, index: usize) -> &mut Position {
        &mut self.positions[index]
    }

    /// Get the velocity of a particle by index.
    pub fn velocity(&self, index: usize) -> &Velocity {
        &self.velocities[index]
    }

    /// Get a mutable reference to the velocity of a particle.
    pub fn velocity_mut(&mut self, index: usize) -> &mut Velocity {
        &mut self.velocities[index]
    }

    /// Get slices of all masses.
    pub fn masses(&self) -> &[Mass] {
        &self.masses
    }

    /// Get slices of all positions.
    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    /// Get mutable slices of all positions.
    pub fn positions_mut(&mut self) -> &mut [Position] {
        &mut self.positions
    }

    /// Get slices of all velocities.
    pub fn velocities(&self) -> &[Velocity] {
        &self.velocities
    }

    /// Get mutable slices of all velocities.
    pub fn velocities_mut(&mut self) -> &mut [Velocity] {
        &mut self.velocities
    }

    /// Calculate total kinetic energy of the system.
    pub fn kinetic_energy(&self) -> Scalar {
        self.masses
            .iter()
            .zip(self.velocities.iter())
            .map(|(&mass, velocity)| 0.5 * mass * velocity.magnitude_squared())
            .sum()
    }

    /// Calculate center of mass of the system.
    pub fn center_of_mass(&self) -> Position {
        let total_mass: Mass = self.masses.iter().sum();
        if total_mass > 0.0 {
            let weighted_sum = self
                .masses
                .iter()
                .zip(self.positions.iter())
                .map(|(&mass, pos)| pos * mass)
                .fold(Position::zeros(), |acc, pos| acc + pos);
            weighted_sum / total_mass
        } else {
            Position::zeros()
        }
    }

    /// Validate all particles in the set.
    pub fn validate(&self) -> Result<()> {
        for i in 0..self.len() {
            let body = Body {
                mass: self.masses[i],
                position: self.positions[i],
                velocity: self.velocities[i],
            };
            body.validate()
                .map_err(|e| GravwellError::invalid_particle(format!("Particle {}: {}", i, e)))?;
        }
        Ok(())
    }
}

impl Default for ParticleSet {
    fn default() -> Self {
        Self::new()
    }
}
