//! Simulation builder for configuring Gravwell simulations.

use crate::{
    core::{
        forces::ForceCalculator,
        integrator::Integrator,
        particle::{Body, ParticleSet},
    },
    error::{GravwellError, Result},
    types::{Time, Vector3},
};

/// Handle for referencing bodies in the simulation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BodyHandle {
    index: usize,
}

impl BodyHandle {
    /// Create a new body handle with the given index.
    pub fn new(index: usize, _generation: u32) -> Self {
        Self { index }
    }

    /// Get the index of this handle.
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Builder pattern for creating configured simulations.
///
/// This provides a fluent API for setting up complex simulations with
/// various integrators, force calculators, and initial conditions.
pub struct SimulationBuilder<I = (), F = ()> {
    integrator: Option<I>,
    force_calculator: Option<F>,
    particles: ParticleSet,
}

impl SimulationBuilder<(), ()> {
    /// Create a new simulation builder.
    pub fn new() -> Self {
        Self {
            integrator: None,
            force_calculator: None,
            particles: ParticleSet::new(),
        }
    }
}

impl<I, F> SimulationBuilder<I, F> {
    /// Set the numerical integrator.
    pub fn with_integrator<NewI>(self, integrator: NewI) -> SimulationBuilder<NewI, F>
    where
        NewI: Integrator,
    {
        SimulationBuilder {
            integrator: Some(integrator),
            force_calculator: self.force_calculator,
            particles: self.particles,
        }
    }

    /// Set the force calculator.
    pub fn with_force_calculator<NewF>(self, force_calculator: NewF) -> SimulationBuilder<I, NewF>
    where
        NewF: ForceCalculator,
    {
        SimulationBuilder {
            integrator: self.integrator,
            force_calculator: Some(force_calculator),
            particles: self.particles,
        }
    }

    /// Add a body to the initial conditions.
    pub fn add_body(mut self, body: Body) -> Result<Self> {
        self.particles.add_body(body)?;
        Ok(self)
    }

    /// Build the simulation.
    pub fn build(self) -> Result<Simulation<I, F>>
    where
        I: Integrator,
        F: ForceCalculator,
    {
        let integrator = self
            .integrator
            .ok_or_else(|| GravwellError::configuration("No integrator specified"))?;

        let force_calculator = self
            .force_calculator
            .ok_or_else(|| GravwellError::configuration("No force calculator specified"))?;

        if self.particles.is_empty() {
            return Err(GravwellError::configuration(
                "No particles added to simulation",
            ));
        }

        self.particles.validate()?;
        force_calculator.validate(&self.particles)?;

        Ok(Simulation {
            integrator,
            force_calculator,
            particles: self.particles,
        })
    }
}

impl Default for SimulationBuilder<(), ()> {
    fn default() -> Self {
        Self::new()
    }
}

/// A configured gravity simulation ready to run.
pub struct Simulation<I, F> {
    integrator: I,
    force_calculator: F,
    particles: ParticleSet,
}

impl<I, F> Simulation<I, F>
where
    I: Integrator,
    F: ForceCalculator,
{
    /// Step the simulation forward by one timestep.
    pub fn step(&mut self, dt: Time) -> Result<()> {
        self.integrator
            .step(&mut self.particles, &self.force_calculator, dt)
    }

    /// Get read-only access to the particle set.
    pub fn particles(&self) -> &ParticleSet {
        &self.particles
    }

    /// Add a body to the simulation.
    pub fn add_body(&mut self, body: Body) -> Result<BodyHandle> {
        let index = self.particles.len();
        self.particles.add_body(body)?;
        Ok(BodyHandle { index })
    }

    /// Get the name of the integrator being used.
    pub fn integrator_name(&self) -> &'static str {
        self.integrator.name()
    }

    /// Get the name of the force calculator being used.
    pub fn force_calculator_name(&self) -> &'static str {
        self.force_calculator.name()
    }

    /// Reset the integrator state (if stateful).
    pub fn reset_integrator(&mut self) {
        self.integrator.reset();
    }

    /// Calculate the total energy of the system.
    pub fn total_energy(&self) -> f64 {
        // For now, just return kinetic energy as placeholder
        // Total energy would need potential energy calculation which requires force calculator
        self.particles.kinetic_energy()
    }

    /// Get the position of a body by handle.
    pub fn position(&self, handle: BodyHandle) -> Vector3 {
        *self.particles.position(handle.index)
    }

    /// Get the velocity of a body by handle.
    pub fn velocity(&self, handle: BodyHandle) -> Vector3 {
        *self.particles.velocity(handle.index)
    }

    /// Get a body handle by index (for testing purposes).
    pub fn get_body_handle(&self, index: usize) -> BodyHandle {
        BodyHandle { index }
    }
}
