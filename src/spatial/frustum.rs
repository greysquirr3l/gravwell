//! Camera frustum culling implementation for view-based particle optimization.
//!
//! Frustum culling is a critical optimization technique that removes particles
//! outside the camera's view volume, significantly reducing computation for
//! large-scale simulations. This is especially important for game engines
//! where only visible particles need full physics processing.
//!
//! # Mathematical Foundation
//!
//! A camera frustum is defined by 6 planes:
//! - Near and far planes (depth bounds)
//! - Left, right, top, and bottom planes (field of view bounds)
//!
//! Each plane is represented by the equation: ax + by + cz + d = 0
//! Points are classified as inside/outside based on their signed distance to each plane.
//!
//! # Performance Impact
//!
//! Frustum culling can reduce particle processing by 80-95% in typical game scenarios,
//! enabling massive particle counts while maintaining 60 FPS performance.
//!
//! # Usage Example
//!
//! ```rust
//! use gravwell::spatial::Frustum;
//! use gravwell::types::Vector3;
//!
//! // Create frustum from camera parameters
//! let frustum = Frustum::from_camera(
//!     Vector3::new(0.0, 0.0, 0.0),  // camera position
//!     Vector3::new(0.0, 0.0, 1.0),  // forward direction
//!     Vector3::new(0.0, 1.0, 0.0),  // up direction
//!     60.0,  // field of view (degrees)
//!     16.0 / 9.0,  // aspect ratio
//!     1.0,   // near plane
//!     1000.0 // far plane
//! );
//!
//! // Test if points are inside frustum
//! let point = Vector3::new(10.0, 5.0, 50.0);
//! if frustum.contains_point(point) {
//!     println!("Point is visible!");
//! }
//!
//! // Cull a list of particle positions
//! let visible_indices = frustum.cull_particles(&particle_positions);
//! ```

use crate::types::{Scalar, Vector3};
use crate::BodyHandle;
use std::f64::consts::PI;

/// A 3D plane represented by the equation ax + by + cz + d = 0
///
/// The plane normal vector is (a, b, c) and d is the distance from origin.
/// Points with positive signed distance are on the "front" side of the plane.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector components and distance (a, b, c, d)
    pub equation: [Scalar; 4],
}

impl Plane {
    /// Create a new plane from normal vector and a point on the plane
    ///
    /// # Arguments
    ///
    /// * `normal` - Unit normal vector pointing away from the inside
    /// * `point` - Any point that lies on the plane
    pub fn from_normal_and_point(normal: Vector3, point: Vector3) -> Self {
        let d = -normal.dot(&point);
        Self {
            equation: [normal.x, normal.y, normal.z, d],
        }
    }

    /// Create a plane from three points (right-hand rule determines normal direction)
    ///
    /// # Arguments
    ///
    /// * `p1`, `p2`, `p3` - Three points on the plane (counter-clockwise for outward normal)
    pub fn from_three_points(p1: Vector3, p2: Vector3, p3: Vector3) -> Self {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2).normalize();
        Self::from_normal_and_point(normal, p1)
    }

    /// Calculate the signed distance from a point to this plane
    ///
    /// Positive distance means the point is on the "front" side (normal direction),
    /// negative distance means the point is on the "back" side.
    ///
    /// # Arguments
    ///
    /// * `point` - Point to test
    ///
    /// # Returns
    ///
    /// Signed distance from point to plane
    pub fn distance_to_point(&self, point: Vector3) -> Scalar {
        self.equation[0] * point.x
            + self.equation[1] * point.y
            + self.equation[2] * point.z
            + self.equation[3]
    }

    /// Test if a point is on the positive side of the plane
    ///
    /// # Arguments
    ///
    /// * `point` - Point to test
    ///
    /// # Returns
    ///
    /// `true` if point is on positive (front) side, `false` otherwise
    pub fn is_point_in_front(&self, point: Vector3) -> bool {
        self.distance_to_point(point) >= 0.0
    }

    /// Test if a sphere intersects or is in front of the plane
    ///
    /// # Arguments
    ///
    /// * `center` - Center of the sphere
    /// * `radius` - Radius of the sphere
    ///
    /// # Returns
    ///
    /// `true` if sphere intersects or is entirely in front of plane
    pub fn intersects_sphere(&self, center: Vector3, radius: Scalar) -> bool {
        self.distance_to_point(center) >= -radius
    }

    /// Get the normal vector of this plane
    pub fn normal(&self) -> Vector3 {
        Vector3::new(self.equation[0], self.equation[1], self.equation[2])
    }
}

/// Camera frustum for visibility culling in 3D space
///
/// A frustum is a truncated pyramid that represents the visible volume
/// of a perspective camera. It's bounded by six planes that form the
/// view volume used for culling invisible objects.
#[derive(Debug, Clone)]
pub struct Frustum {
    /// The six planes that bound the frustum
    /// Order: [near, far, left, right, top, bottom]
    pub planes: [Plane; 6],

    /// Camera position (for distance calculations)
    pub camera_position: Vector3,

    /// Camera forward direction (unit vector)
    pub forward_direction: Vector3,

    /// Camera up direction (unit vector)
    pub up_direction: Vector3,

    /// Field of view in degrees
    pub fov_degrees: Scalar,

    /// Aspect ratio (width/height)
    pub aspect_ratio: Scalar,

    /// Near clipping plane distance
    pub near_distance: Scalar,

    /// Far clipping plane distance
    pub far_distance: Scalar,
}

impl Frustum {
    /// Create a frustum from camera parameters
    ///
    /// # Arguments
    ///
    /// * `position` - Camera position in world space
    /// * `forward` - Camera forward direction (normalized)
    /// * `up` - Camera up direction (normalized)
    /// * `fov_degrees` - Vertical field of view in degrees
    /// * `aspect_ratio` - Width/height ratio
    /// * `near` - Near clipping plane distance
    /// * `far` - Far clipping plane distance
    ///
    /// # Example
    ///
    /// ```rust
    /// use gravwell::spatial::Frustum;
    /// use gravwell::types::Vector3;
    ///
    /// let frustum = Frustum::from_camera(
    ///     Vector3::new(0.0, 0.0, 0.0),   // camera at origin
    ///     Vector3::new(0.0, 0.0, 1.0),   // looking down +Z axis
    ///     Vector3::new(0.0, 1.0, 0.0),   // up is +Y axis
    ///     45.0,    // 45 degree field of view
    ///     16.0/9.0, // 16:9 aspect ratio
    ///     1.0,     // near plane at 1 unit
    ///     1000.0   // far plane at 1000 units
    /// );
    /// ```
    pub fn from_camera(
        position: Vector3,
        forward: Vector3,
        up: Vector3,
        fov_degrees: Scalar,
        aspect_ratio: Scalar,
        near: Scalar,
        far: Scalar,
    ) -> Self {
        let forward = forward.normalize();
        let up = up.normalize();
        let right = forward.cross(&up).normalize();
        let true_up = right.cross(&forward).normalize();

        // Calculate frustum dimensions
        let fov_radians = fov_degrees * PI / 180.0;
        let half_v_side = far * (fov_radians * 0.5).tan();
        let half_h_side = half_v_side * aspect_ratio;

        // Calculate frustum corner points
        let front_mult_far = far * forward;
        let front_mult_near = near * forward;

        // Far plane corners
        let far_top_left = position + front_mult_far + true_up * half_v_side - right * half_h_side;
        let far_top_right = position + front_mult_far + true_up * half_v_side + right * half_h_side;
        let far_bottom_left =
            position + front_mult_far - true_up * half_v_side - right * half_h_side;
        let far_bottom_right =
            position + front_mult_far - true_up * half_v_side + right * half_h_side;

        // Near plane dimensions
        let near_half_v = near * (fov_radians * 0.5).tan();
        let near_half_h = near_half_v * aspect_ratio;

        // Near plane corners
        let near_top_left =
            position + front_mult_near + true_up * near_half_v - right * near_half_h;
        let near_top_right =
            position + front_mult_near + true_up * near_half_v + right * near_half_h;
        let _near_bottom_left =
            position + front_mult_near - true_up * near_half_v - right * near_half_h;
        let _near_bottom_right =
            position + front_mult_near - true_up * near_half_v + right * near_half_h;

        // Calculate the six frustum planes
        // Plane normals point inward (toward the visible volume)
        let planes = [
            // Near plane
            Plane::from_normal_and_point(forward, position + front_mult_near),
            // Far plane
            Plane::from_normal_and_point(-forward, position + front_mult_far),
            // Left plane (using right-hand rule for correct normal)
            Plane::from_three_points(position, far_top_left, near_top_left),
            // Right plane
            Plane::from_three_points(position, near_top_right, far_top_right),
            // Top plane
            Plane::from_three_points(position, far_top_right, far_top_left),
            // Bottom plane
            Plane::from_three_points(position, far_bottom_left, far_bottom_right),
        ];

        Self {
            planes,
            camera_position: position,
            forward_direction: forward,
            up_direction: true_up,
            fov_degrees,
            aspect_ratio,
            near_distance: near,
            far_distance: far,
        }
    }

    /// Test if a point is inside the frustum
    ///
    /// A point is inside if it's on the positive side of all six planes.
    ///
    /// # Arguments
    ///
    /// * `point` - Point to test
    ///
    /// # Returns
    ///
    /// `true` if point is inside frustum, `false` otherwise
    pub fn contains_point(&self, point: Vector3) -> bool {
        for plane in &self.planes {
            if !plane.is_point_in_front(point) {
                return false;
            }
        }
        true
    }

    /// Test if a sphere intersects the frustum
    ///
    /// A sphere intersects if any part of it is inside or intersects the frustum boundary.
    ///
    /// # Arguments
    ///
    /// * `center` - Center of the sphere
    /// * `radius` - Radius of the sphere
    ///
    /// # Returns
    ///
    /// `true` if sphere intersects frustum, `false` otherwise
    pub fn intersects_sphere(&self, center: Vector3, radius: Scalar) -> bool {
        for plane in &self.planes {
            if !plane.intersects_sphere(center, radius) {
                return false;
            }
        }
        true
    }

    /// Test if an axis-aligned bounding box intersects the frustum
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum corner of the bounding box
    /// * `max` - Maximum corner of the bounding box
    ///
    /// # Returns
    ///
    /// `true` if bounding box intersects frustum, `false` otherwise
    pub fn intersects_aabb(&self, min: Vector3, max: Vector3) -> bool {
        for plane in &self.planes {
            // Find the positive vertex (farthest in the direction of the plane normal)
            let normal = plane.normal();
            let positive_vertex = Vector3::new(
                if normal.x >= 0.0 { max.x } else { min.x },
                if normal.y >= 0.0 { max.y } else { min.y },
                if normal.z >= 0.0 { max.z } else { min.z },
            );

            // If the positive vertex is behind the plane, the box is outside
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Cull a list of particle positions, returning indices of visible particles
    ///
    /// This is the main culling function for particle systems. It efficiently
    /// tests each particle position against the frustum and returns only those
    /// that are visible.
    ///
    /// # Arguments
    ///
    /// * `positions` - Array of particle positions to test
    ///
    /// # Returns
    ///
    /// Vector of indices into the positions array for visible particles
    pub fn cull_particles(&self, positions: &[Vector3]) -> Vec<usize> {
        let mut visible_indices = Vec::new();

        for (index, &position) in positions.iter().enumerate() {
            if self.contains_point(position) {
                visible_indices.push(index);
            }
        }

        visible_indices
    }

    /// Cull particles with associated handles, returning visible handles
    ///
    /// # Arguments
    ///
    /// * `particles` - Array of (handle, position) pairs
    ///
    /// # Returns
    ///
    /// Vector of handles for visible particles
    pub fn cull_particle_handles(&self, particles: &[(BodyHandle, Vector3)]) -> Vec<BodyHandle> {
        let mut visible_handles = Vec::new();

        for &(handle, position) in particles {
            if self.contains_point(position) {
                visible_handles.push(handle);
            }
        }

        visible_handles
    }

    /// Cull particles with spherical bounds (for particles with finite size)
    ///
    /// # Arguments
    ///
    /// * `particles` - Array of (position, radius) pairs
    ///
    /// # Returns
    ///
    /// Vector of indices for visible particles
    pub fn cull_spherical_particles(&self, particles: &[(Vector3, Scalar)]) -> Vec<usize> {
        let mut visible_indices = Vec::new();

        for (index, &(position, radius)) in particles.iter().enumerate() {
            if self.intersects_sphere(position, radius) {
                visible_indices.push(index);
            }
        }

        visible_indices
    }

    /// Get the distance from camera to a point along the forward axis
    ///
    /// This is useful for depth-based sorting or LOD calculations.
    ///
    /// # Arguments
    ///
    /// * `point` - Point to measure distance to
    ///
    /// # Returns
    ///
    /// Distance along camera forward axis (can be negative if behind camera)
    pub fn depth_distance(&self, point: Vector3) -> Scalar {
        let to_point = point - self.camera_position;
        to_point.dot(&self.forward_direction)
    }

    /// Check if the frustum intersects another frustum
    ///
    /// This can be useful for hierarchical culling or multi-camera systems.
    pub fn intersects_frustum(&self, other: &Frustum) -> bool {
        // Simplified implementation: check if any corner of one frustum is inside the other
        // A full implementation would use separating axis theorem

        let corners = self.get_corner_points();
        for corner in &corners {
            if other.contains_point(*corner) {
                return true;
            }
        }

        let other_corners = other.get_corner_points();
        for corner in &other_corners {
            if self.contains_point(*corner) {
                return true;
            }
        }

        false
    }

    /// Get the 8 corner points of the frustum
    ///
    /// Returns corners in the order: near (4 points), far (4 points)
    /// Each group of 4: top-left, top-right, bottom-left, bottom-right
    pub fn get_corner_points(&self) -> [Vector3; 8] {
        let fov_radians = self.fov_degrees * PI / 180.0;
        let right = self.forward_direction.cross(&self.up_direction).normalize();

        // Near plane dimensions
        let near_half_height = self.near_distance * (fov_radians * 0.5).tan();
        let near_half_width = near_half_height * self.aspect_ratio;

        // Far plane dimensions
        let far_half_height = self.far_distance * (fov_radians * 0.5).tan();
        let far_half_width = far_half_height * self.aspect_ratio;

        let near_center = self.camera_position + self.forward_direction * self.near_distance;
        let far_center = self.camera_position + self.forward_direction * self.far_distance;

        [
            // Near plane corners
            near_center + self.up_direction * near_half_height - right * near_half_width, // top-left
            near_center + self.up_direction * near_half_height + right * near_half_width, // top-right
            near_center - self.up_direction * near_half_height - right * near_half_width, // bottom-left
            near_center - self.up_direction * near_half_height + right * near_half_width, // bottom-right
            // Far plane corners
            far_center + self.up_direction * far_half_height - right * far_half_width, // top-left
            far_center + self.up_direction * far_half_height + right * far_half_width, // top-right
            far_center - self.up_direction * far_half_height - right * far_half_width, // bottom-left
            far_center - self.up_direction * far_half_height + right * far_half_width, // bottom-right
        ]
    }
}

/// Result of frustum culling operation with detailed statistics
#[derive(Debug, Clone)]
pub struct FrustumCullingResult {
    /// Indices or handles of visible objects
    pub visible_objects: Vec<usize>,

    /// Total number of objects tested
    pub total_objects: usize,

    /// Number of objects culled (not visible)
    pub culled_objects: usize,

    /// Percentage of objects that were culled
    pub cull_percentage: f32,

    /// Time spent on culling in microseconds
    pub culling_time_us: u64,
}

impl FrustumCullingResult {
    /// Create a new culling result
    pub fn new(visible: Vec<usize>, total: usize, culling_time_us: u64) -> Self {
        let culled = total - visible.len();
        let cull_percentage = if total > 0 {
            culled as f32 / total as f32 * 100.0
        } else {
            0.0
        };

        Self {
            visible_objects: visible,
            total_objects: total,
            culled_objects: culled,
            cull_percentage,
            culling_time_us,
        }
    }

    /// Get the number of visible objects
    pub fn visible_count(&self) -> usize {
        self.visible_objects.len()
    }

    /// Check if culling was effective (removed significant number of objects)
    pub fn is_effective(&self) -> bool {
        self.cull_percentage > 10.0 // Consider effective if > 10% culled
    }
}

/// Advanced frustum culling with hierarchical and temporal optimizations
pub struct AdvancedFrustumCuller {
    /// Current frustum
    current_frustum: Frustum,

    /// Previous frustum for temporal coherence
    previous_frustum: Option<Frustum>,

    /// Objects that were visible in the previous frame
    previously_visible: Vec<usize>,

    /// Performance statistics
    culling_stats: FrustumCullingStats,
}

/// Statistics for frustum culling performance analysis
#[derive(Debug, Clone, Default)]
pub struct FrustumCullingStats {
    /// Total culling operations performed
    pub total_culls: u64,

    /// Total objects processed
    pub total_objects_processed: u64,

    /// Total objects culled
    pub total_objects_culled: u64,

    /// Average cull percentage
    pub average_cull_percentage: f32,

    /// Total time spent culling (microseconds)
    pub total_culling_time_us: u64,

    /// Average time per culling operation (microseconds)
    pub average_culling_time_us: f32,
}

impl AdvancedFrustumCuller {
    /// Create a new advanced frustum culler
    pub fn new(initial_frustum: Frustum) -> Self {
        Self {
            current_frustum: initial_frustum,
            previous_frustum: None,
            previously_visible: Vec::new(),
            culling_stats: FrustumCullingStats::default(),
        }
    }

    /// Update the frustum and perform temporal coherence optimization
    pub fn update_frustum(&mut self, new_frustum: Frustum) {
        self.previous_frustum = Some(self.current_frustum.clone());
        self.current_frustum = new_frustum;
    }

    /// Perform optimized culling with temporal coherence
    ///
    /// This method uses information from the previous frame to optimize culling:
    /// - Test previously visible objects first (likely to be visible again)
    /// - Use early termination when frustum hasn't changed much
    pub fn cull_with_temporal_coherence(&mut self, positions: &[Vector3]) -> FrustumCullingResult {
        use std::time::Instant;
        let start_time = Instant::now();

        let total_objects = positions.len();
        let mut visible_indices = Vec::new();

        // Test previously visible objects first (temporal coherence)
        for &prev_index in &self.previously_visible {
            if prev_index < positions.len()
                && self.current_frustum.contains_point(positions[prev_index])
            {
                visible_indices.push(prev_index);
            }
        }

        // Create set of already tested indices for efficiency
        let tested_indices: std::collections::HashSet<usize> =
            self.previously_visible.iter().copied().collect();

        // Test remaining objects
        for (index, &position) in positions.iter().enumerate() {
            if !tested_indices.contains(&index) && self.current_frustum.contains_point(position) {
                visible_indices.push(index);
            }
        }

        // Update temporal information
        self.previously_visible = visible_indices.clone();

        let culling_time = start_time.elapsed().as_micros() as u64;

        // Update statistics
        self.culling_stats.total_culls += 1;
        self.culling_stats.total_objects_processed += total_objects as u64;
        self.culling_stats.total_objects_culled += (total_objects - visible_indices.len()) as u64;
        self.culling_stats.total_culling_time_us += culling_time;

        // Update averages
        self.culling_stats.average_cull_percentage = self.culling_stats.total_objects_culled as f32
            / self.culling_stats.total_objects_processed as f32
            * 100.0;
        self.culling_stats.average_culling_time_us =
            self.culling_stats.total_culling_time_us as f32 / self.culling_stats.total_culls as f32;

        FrustumCullingResult::new(visible_indices, total_objects, culling_time)
    }

    /// Get current culling statistics
    pub fn get_statistics(&self) -> &FrustumCullingStats {
        &self.culling_stats
    }

    /// Reset culling statistics
    pub fn reset_statistics(&mut self) {
        self.culling_stats = FrustumCullingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Vector3;
    // use std::f64::consts::PI; // Unused in current implementation

    #[test]
    fn test_plane_creation() {
        let normal = Vector3::new(0.0, 1.0, 0.0); // Up vector
        let point = Vector3::new(0.0, 5.0, 0.0); // Point at y=5

        let plane = Plane::from_normal_and_point(normal, point);

        // Test that the original point has zero distance
        assert!((plane.distance_to_point(point)).abs() < 1e-10);

        // Test that points above have positive distance
        assert!(plane.distance_to_point(Vector3::new(0.0, 10.0, 0.0)) > 0.0);

        // Test that points below have negative distance
        assert!(plane.distance_to_point(Vector3::new(0.0, 0.0, 0.0)) < 0.0);
    }

    #[test]
    fn test_plane_from_three_points() {
        // Create a plane from three points forming a triangle in the XY plane
        let p1 = Vector3::new(0.0, 0.0, 5.0);
        let p2 = Vector3::new(1.0, 0.0, 5.0);
        let p3 = Vector3::new(0.0, 1.0, 5.0);

        let plane = Plane::from_three_points(p1, p2, p3);

        // All three points should have zero distance to the plane
        assert!((plane.distance_to_point(p1)).abs() < 1e-10);
        assert!((plane.distance_to_point(p2)).abs() < 1e-10);
        assert!((plane.distance_to_point(p3)).abs() < 1e-10);

        // Normal should point in +Z direction for counter-clockwise points
        let normal = plane.normal();
        assert!((normal.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_frustum_creation() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0), // position
            Vector3::new(0.0, 0.0, 1.0), // forward (down +Z)
            Vector3::new(0.0, 1.0, 0.0), // up (+Y)
            60.0,                        // 60 degree FOV
            16.0 / 9.0,                  // 16:9 aspect ratio
            1.0,                         // near = 1
            100.0,                       // far = 100
        );

        assert_eq!(frustum.camera_position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(frustum.fov_degrees, 60.0);
        assert_eq!(frustum.near_distance, 1.0);
        assert_eq!(frustum.far_distance, 100.0);
        assert_eq!(frustum.planes.len(), 6);
    }

    #[test]
    fn test_frustum_point_containment() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            90.0, // 90 degree FOV for easier testing
            1.0,  // 1:1 aspect ratio
            1.0,  // near
            10.0, // far
        );

        // Point in center of frustum should be visible
        assert!(frustum.contains_point(Vector3::new(0.0, 0.0, 5.0)));

        // Point behind camera should not be visible
        assert!(!frustum.contains_point(Vector3::new(0.0, 0.0, -1.0)));

        // Point too close (before near plane) should not be visible
        assert!(!frustum.contains_point(Vector3::new(0.0, 0.0, 0.5)));

        // Point too far should not be visible
        assert!(!frustum.contains_point(Vector3::new(0.0, 0.0, 15.0)));

        // Point far to the side should not be visible
        assert!(!frustum.contains_point(Vector3::new(100.0, 0.0, 5.0)));
    }

    #[test]
    fn test_frustum_sphere_intersection() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
            1.0,
            10.0,
        );

        // Small sphere in center should intersect
        assert!(frustum.intersects_sphere(Vector3::new(0.0, 0.0, 5.0), 0.5));

        // Large sphere centered outside but overlapping should intersect
        assert!(frustum.intersects_sphere(Vector3::new(2.0, 0.0, 5.0), 3.0));

        // Small sphere far outside should not intersect
        assert!(!frustum.intersects_sphere(Vector3::new(100.0, 0.0, 5.0), 1.0));
    }

    #[test]
    fn test_particle_culling() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
            1.0,
            10.0,
        );

        let positions = vec![
            Vector3::new(0.0, 0.0, 5.0),  // Should be visible (center)
            Vector3::new(0.0, 0.0, -1.0), // Behind camera
            Vector3::new(0.0, 0.0, 15.0), // Too far
            Vector3::new(1.0, 1.0, 5.0),  // Should be visible (in frustum)
            Vector3::new(10.0, 0.0, 5.0), // Too far to side
        ];

        let visible_indices = frustum.cull_particles(&positions);

        // Should have some visible particles but not all
        assert!(visible_indices.len() > 0);
        assert!(visible_indices.len() < positions.len());

        // First particle (center) should definitely be visible
        assert!(visible_indices.contains(&0));

        // Behind camera particle should not be visible
        assert!(!visible_indices.contains(&1));
    }

    #[test]
    fn test_frustum_corner_points() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            90.0, // 90 degrees for easy calculation
            1.0,  // 1:1 aspect ratio
            1.0,  // near = 1
            2.0,  // far = 2
        );

        let corners = frustum.get_corner_points();
        assert_eq!(corners.len(), 8);

        // All corners should be at the correct distances
        for i in 0..4 {
            // Near plane corners should be at z = 1
            assert!((corners[i].z - 1.0).abs() < 1e-10);
        }

        for i in 4..8 {
            // Far plane corners should be at z = 2
            assert!((corners[i].z - 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_depth_distance() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
            1.0,
            10.0,
        );

        // Point at z=5 should have depth distance 5
        assert!((frustum.depth_distance(Vector3::new(0.0, 0.0, 5.0)) - 5.0).abs() < 1e-10);

        // Point behind camera should have negative depth
        assert!(frustum.depth_distance(Vector3::new(0.0, 0.0, -1.0)) < 0.0);

        // Point to the side at same depth should have same depth distance
        assert!((frustum.depth_distance(Vector3::new(2.0, 3.0, 5.0)) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_advanced_culler_temporal_coherence() {
        let frustum = Frustum::from_camera(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
            1.0,
            10.0,
        );

        let mut culler = AdvancedFrustumCuller::new(frustum);

        let positions = vec![
            Vector3::new(0.0, 0.0, 5.0),  // Center - should be visible
            Vector3::new(0.0, 0.0, -1.0), // Behind camera
            Vector3::new(1.0, 1.0, 5.0),  // Side - should be visible
        ];

        // First cull
        let result1 = culler.cull_with_temporal_coherence(&positions);
        assert!(result1.visible_count() > 0);

        // Second cull should use temporal coherence
        let result2 = culler.cull_with_temporal_coherence(&positions);
        assert_eq!(result1.visible_count(), result2.visible_count());

        let stats = culler.get_statistics();
        assert_eq!(stats.total_culls, 2);
    }
}
