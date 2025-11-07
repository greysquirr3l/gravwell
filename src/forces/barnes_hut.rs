//! Barnes-Hut tree algorithm for O(N log N) gravitational force calculation.
//!
//! The Barnes-Hut algorithm uses spatial partitioning to approximate forces from
//! distant particles, achieving O(N log N) complexity instead of O(N²) direct calculation.
//!
//! # Algorithm Overview
//!
//! 1. Build an octree by recursively subdividing space into 8 cubic regions
//! 2. Store particle masses and center of mass at each node
//! 3. For each particle, traverse the tree and use multipole expansion for distant nodes
//! 4. Use the theta parameter to control accuracy vs performance trade-off
//!
//! # Performance Characteristics
//!
//! - **Complexity**: O(N log N) for N particles
//! - **Memory**: O(N) for tree storage
//! - **Theta = 0.5**: Balanced accuracy/performance (recommended)
//! - **Theta = 0.3**: High accuracy, slower
//! - **Theta = 0.7**: Lower accuracy, faster

use crate::{
    core::{forces::ForceCalculator, particle::ParticleSet},
    error::{GravwellError, Result},
    types::{Force, Mass, Position, Scalar},
    utils::constants::G,
};
use nalgebra::Point3;

/// Barnes-Hut gravitational force calculator using an octree spatial data structure.
///
/// This implementation provides O(N log N) complexity for force calculation,
/// making it suitable for systems with 1,000 to 100,000+ particles.
#[derive(Debug, Clone)]
pub struct BarnesHut {
    /// Accuracy parameter - smaller values = higher accuracy, larger values = faster computation
    theta: Scalar,
    /// Softening parameter to avoid singularities when particles are very close
    softening: Scalar,
}

impl BarnesHut {
    /// Create a new Barnes-Hut force calculator with default parameters.
    ///
    /// # Default Values
    ///
    /// - **theta**: 0.5 (balanced accuracy/performance)
    /// - **softening**: 0.0 (no softening)
    pub fn new() -> Self {
        Self {
            theta: 0.5,
            softening: 0.0,
        }
    }

    /// Create a Barnes-Hut calculator with custom accuracy parameter.
    pub fn with_theta(theta: Scalar) -> Self {
        Self {
            theta,
            softening: 0.0,
        }
    }

    /// Set the theta parameter for accuracy vs performance trade-off.
    ///
    /// # Parameters
    ///
    /// - `theta`: Opening angle parameter (0.1 to 1.0)
    ///   - Lower values (0.3): Higher accuracy, slower computation
    ///   - Higher values (0.7): Lower accuracy, faster computation
    ///   - Recommended: 0.5 for balanced performance
    pub fn theta(mut self, theta: Scalar) -> Self {
        self.theta = theta;
        self
    }

    /// Set the softening parameter to avoid singularities.
    ///
    /// # Parameters
    ///
    /// - `epsilon`: Softening length scale
    ///   - Typical values: 1e-3 to 1e-6
    ///   - Set to 0.0 to disable softening
    pub fn softening(mut self, epsilon: Scalar) -> Self {
        self.softening = epsilon;
        self
    }

    /// Calculate force on a single particle using the Barnes-Hut tree walk.
    fn calculate_particle_force(&self, particle_index: usize, particles: &ParticleSet, octree: &Octree) -> Force {
        let particle_pos = particles.position(particle_index);
        let particle_mass = particles.mass(particle_index);

        self.tree_walk(octree, *particle_pos, particle_mass, particle_index, particles)
    }

    /// Recursive tree walk to calculate gravitational force.
    fn tree_walk(
        &self,
        node: &Octree,
        particle_pos: Position,
        particle_mass: Mass,
        _particle_index: usize,
        _particles: &ParticleSet,
    ) -> Force {
        // Distance from particle to node's center of mass
        let r_vec = node.center_of_mass - particle_pos;
        let distance = r_vec.norm();

        // Avoid self-interaction
        if distance < 1e-10 {
            return Force::zeros();
        }

        // Opening angle criterion: s/d < theta
        let opening_angle = node.size / distance;

        if node.is_leaf() || opening_angle < self.theta {
            // Use this node's multipole expansion
            self.calculate_force_from_node(node, particle_pos, particle_mass, distance, r_vec)
        } else {
            // Recursively traverse children
            let mut total_force = Force::zeros();
            
            for child in &node.children {
                if let Some(child_node) = child {
                    total_force += self.tree_walk(
                        child_node, 
                        particle_pos, 
                        particle_mass, 
                        _particle_index, 
                        _particles
                    );
                }
            }
            
            total_force
        }
    }

    /// Calculate gravitational force from a single octree node.
    fn calculate_force_from_node(
        &self,
        node: &Octree,
        _particle_pos: Position,
        _particle_mass: Mass,
        distance: Scalar,
        r_vec: Position,
    ) -> Force {
        // Apply softening parameter
        let softened_distance_sq = distance * distance + self.softening * self.softening;
        let softened_distance = softened_distance_sq.sqrt();

        // Newton's law of gravitation: F = G * m1 * m2 / r²
        let force_magnitude = G * node.total_mass / softened_distance_sq;
        
        // Force direction (normalized r_vec)
        force_magnitude * r_vec / softened_distance
    }
}

impl Default for BarnesHut {
    fn default() -> Self {
        Self::new()
    }
}

impl ForceCalculator for BarnesHut {
    fn calculate_forces(&self, particles: &ParticleSet, forces: &mut [Force]) -> Result<()> {
        // Build octree from current particle positions
        let octree = Octree::build_from_particles(particles)?;

        // Calculate forces for each particle
        for i in 0..particles.len() {
            forces[i] = self.calculate_particle_force(i, particles, &octree);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Barnes-Hut Tree"
    }

    fn complexity(&self) -> &'static str {
        "O(N log N)"
    }

    fn supports_parallel(&self) -> bool {
        true // Force calculations are independent per particle
    }
}

/// Octree data structure for spatial partitioning in 3D space.
///
/// Each node represents a cubic region of space and stores:
/// - Total mass of all particles in the subtree
/// - Center of mass of all particles in the subtree
/// - Up to 8 child nodes (octants)
/// - Direct particle storage for leaf nodes
#[derive(Debug, Clone)]
pub struct Octree {
    /// Center point of this cubic region
    center: Point3<Scalar>,
    /// Size (edge length) of this cubic region
    size: Scalar,
    /// Total mass of all particles in this subtree
    total_mass: Scalar,
    /// Center of mass of all particles in this subtree
    center_of_mass: Position,
    /// Child nodes (up to 8 for octree)
    children: [Option<Box<Octree>>; 8],
    /// Particles stored directly in this node (for leaf nodes)
    particles: Vec<usize>,
    /// Maximum particles per leaf node before subdivision
    max_particles_per_node: usize,
}

impl Octree {
    /// Build an octree from a particle set.
    pub fn build_from_particles(particles: &ParticleSet) -> Result<Self> {
        if particles.is_empty() {
            return Err(GravwellError::configuration(
                "Cannot build octree from empty particle set"
            ));
        }

        // Calculate bounding box of all particles
        let (min_bound, max_bound) = Self::calculate_bounding_box(particles);
        
        // Create root node with expanded bounding box
        let center = (min_bound + max_bound.coords) * 0.5;
        let diff = max_bound - min_bound;
        let size = diff.x.abs().max(diff.y.abs()).max(diff.z.abs()) * 1.1; // Add 10% margin

        let mut root = Self::new(center.into(), size);
        
        // Insert all particles into the octree
        for i in 0..particles.len() {
            root.insert_particle(i, particles)?;
        }

        Ok(root)
    }

    /// Create a new octree node.
    fn new(center: Point3<Scalar>, size: Scalar) -> Self {
        Self {
            center,
            size,
            total_mass: 0.0,
            center_of_mass: Position::zeros(),
            children: [None, None, None, None, None, None, None, None],
            particles: Vec::new(),
            max_particles_per_node: 1, // Subdivide after 1 particle for optimal tree
        }
    }

    /// Calculate axis-aligned bounding box of all particles.
    fn calculate_bounding_box(particles: &ParticleSet) -> (Point3<Scalar>, Point3<Scalar>) {
        let first_pos = particles.position(0);
        let mut min_bound = Point3::new(first_pos.x, first_pos.y, first_pos.z);
        let mut max_bound = Point3::new(first_pos.x, first_pos.y, first_pos.z);

        for i in 1..particles.len() {
            let pos = particles.position(i);
            let point = Point3::new(pos.x, pos.y, pos.z);
            
            min_bound = Point3::new(
                min_bound.x.min(point.x),
                min_bound.y.min(point.y),
                min_bound.z.min(point.z),
            );
            
            max_bound = Point3::new(
                max_bound.x.max(point.x),
                max_bound.y.max(point.y),
                max_bound.z.max(point.z),
            );
        }

        (min_bound, max_bound)
    }

    /// Insert a particle into the octree, subdividing as necessary.
    fn insert_particle(&mut self, particle_index: usize, particles: &ParticleSet) -> Result<()> {
        let particle_pos = particles.position(particle_index);
        let particle_mass = particles.mass(particle_index);

        // Update this node's total mass and center of mass
        self.update_mass_properties(particle_mass, *particle_pos);

        // Check if particle is within this node's bounds
        if !self.contains_point(*particle_pos) {
            return Err(GravwellError::configuration(
                "Particle outside octree bounds"
            ));
        }

        // If this is a leaf node with room, store the particle directly
        if self.is_leaf() && self.particles.len() < self.max_particles_per_node {
            self.particles.push(particle_index);
            return Ok(());
        }

        // Need to subdivide or insert into existing children
        if self.is_leaf() {
            // First time subdividing - create children and redistribute existing particles
            self.subdivide(particles)?;
        }

        // Insert particle into appropriate child
        let octant = self.get_octant(*particle_pos);
        if let Some(ref mut child) = self.children[octant] {
            child.insert_particle(particle_index, particles)?;
        }

        Ok(())
    }

    /// Update the mass properties (total mass and center of mass) for this node.
    fn update_mass_properties(&mut self, particle_mass: Mass, particle_pos: Position) {
        let old_total_mass = self.total_mass;
        let new_total_mass = old_total_mass + particle_mass;
        
        if new_total_mass > 0.0 {
            // Update center of mass using weighted average
            self.center_of_mass = (self.center_of_mass * old_total_mass + particle_pos * particle_mass) / new_total_mass;
        }
        
        self.total_mass = new_total_mass;
    }

    /// Check if a point is contained within this octree node.
    fn contains_point(&self, point: Position) -> bool {
        let half_size = self.size * 0.5;
        let min_bound = self.center.coords - Position::new(half_size, half_size, half_size);
        let max_bound = self.center.coords + Position::new(half_size, half_size, half_size);

        point.x >= min_bound.x && point.x < max_bound.x &&
        point.y >= min_bound.y && point.y < max_bound.y &&
        point.z >= min_bound.z && point.z < max_bound.z
    }

    /// Subdivide this node into 8 children and redistribute existing particles.
    fn subdivide(&mut self, particles: &ParticleSet) -> Result<()> {
        let quarter_size = self.size * 0.25;
        let half_size = self.size * 0.5;

        // Create 8 child nodes (octants)
        for i in 0..8 {
            let offset = Position::new(
                if i & 1 != 0 { quarter_size } else { -quarter_size },
                if i & 2 != 0 { quarter_size } else { -quarter_size },
                if i & 4 != 0 { quarter_size } else { -quarter_size },
            );

            let child_center = Point3::from(self.center.coords + offset);
            self.children[i] = Some(Box::new(Octree::new(child_center, half_size)));
        }

        // Redistribute existing particles to children
        let existing_particles = std::mem::take(&mut self.particles);
        for &particle_index in &existing_particles {
            let particle_pos = particles.position(particle_index);
            let octant = self.get_octant(*particle_pos);
            
            if let Some(ref mut child) = self.children[octant] {
                child.insert_particle(particle_index, particles)?;
            }
        }

        Ok(())
    }

    /// Determine which octant (0-7) a point belongs to relative to this node's center.
    fn get_octant(&self, point: Position) -> usize {
        let mut octant = 0;
        
        if point.x >= self.center.x { octant |= 1; }
        if point.y >= self.center.y { octant |= 2; }
        if point.z >= self.center.z { octant |= 4; }
        
        octant
    }

    /// Check if this is a leaf node (no children).
    fn is_leaf(&self) -> bool {
        self.children.iter().all(|child| child.is_none())
    }
}
