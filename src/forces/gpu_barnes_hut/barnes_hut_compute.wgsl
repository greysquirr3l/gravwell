// Barnes-Hut Compute Shader for GPU Force Calculation
// Implements tree traversal with multipole expansion for O(N log N) scaling

struct OctreeNode {
    // Bounding box
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    
    // Center of mass and total mass
    center_of_mass: vec3<f32>,
    total_mass: f32,
    
    // Tree structure
    child_mask: u32,        // Bitmask for which children exist
    first_child_index: u32, // Index of first child (if internal)
    particle_start: u32,    // Start index in particle array (if leaf)
    particle_count: u32,    // Number of particles (if leaf)
}

struct TraversalParams {
    theta: f32,
    particle_count: u32,
    max_stack_depth: u32,
    padding: u32,
}

// Input/Output buffers
@group(0) @binding(0) var<storage, read> positions: array<vec3<f32>>;
@group(0) @binding(1) var<storage, read> masses: array<f32>;
@group(0) @binding(2) var<storage, read_write> forces: array<vec3<f32>>;
@group(0) @binding(3) var<uniform> params: TraversalParams;
@group(0) @binding(4) var<storage, read> tree_nodes: array<OctreeNode>;

// Gravitational constant (in simulation units)
const G: f32 = 6.67430e-11;

// Softening parameter to avoid singularities
const SOFTENING: f32 = 1e-9;

// Stack for tree traversal (avoiding recursion)
var<workgroup> traversal_stack: array<u32, 64>;

@compute @workgroup_size(64, 1, 1)
fn barnes_hut_forces(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_idx = global_id.x;
    
    if (particle_idx >= params.particle_count) {
        return;
    }
    
    let particle_pos = positions[particle_idx];
    let particle_mass = masses[particle_idx];
    
    var total_force = vec3<f32>(0.0, 0.0, 0.0);
    
    // Initialize stack with root node
    var stack_ptr = 0u;
    traversal_stack[stack_ptr] = 0u; // Root node index
    stack_ptr += 1u;
    
    // Tree traversal loop
    while (stack_ptr > 0u && stack_ptr < params.max_stack_depth) {
        stack_ptr -= 1u;
        let node_idx = traversal_stack[stack_ptr];
        let node = tree_nodes[node_idx];
        
        // Calculate distance from particle to node center of mass
        let r_vec = node.center_of_mass - particle_pos;
        let r_squared = dot(r_vec, r_vec) + SOFTENING;
        let r = sqrt(r_squared);
        
        // Calculate node size (diagonal of bounding box)
        let node_size = distance(
            vec3<f32>(node.min_x, node.min_y, node.min_z),
            vec3<f32>(node.max_x, node.max_y, node.max_z)
        );
        
        // Barnes-Hut approximation criterion
        let s_over_d = node_size / r;
        
        if (s_over_d < params.theta || node.particle_count == 1u) {
            // Use this node for force calculation (far enough or leaf)
            if (node.total_mass > 0.0 && r > SOFTENING) {
                let force_magnitude = G * particle_mass * node.total_mass / r_squared;
                total_force += force_magnitude * r_vec / r;
            }
        } else {
            // Node is too close, traverse children
            if (node.child_mask != 0u) {
                // Add children to stack (in reverse order for proper traversal)
                for (var child_offset = 7i; child_offset >= 0; child_offset -= 1) {
                    if ((node.child_mask & (1u << u32(child_offset))) != 0u) {
                        if (stack_ptr < params.max_stack_depth) {
                            traversal_stack[stack_ptr] = node.first_child_index + u32(child_offset);
                            stack_ptr += 1u;
                        }
                    }
                }
            } else {
                // Leaf node - calculate direct forces to all particles in this node
                for (var i = 0u; i < node.particle_count; i += 1u) {
                    let other_idx = node.particle_start + i;
                    
                    if (other_idx != particle_idx) {
                        let other_pos = positions[other_idx];
                        let other_mass = masses[other_idx];
                        
                        let r_vec_direct = other_pos - particle_pos;
                        let r_squared_direct = dot(r_vec_direct, r_vec_direct) + SOFTENING;
                        let r_direct = sqrt(r_squared_direct);
                        
                        if (r_direct > SOFTENING) {
                            let force_magnitude_direct = G * particle_mass * other_mass / r_squared_direct;
                            total_force += force_magnitude_direct * r_vec_direct / r_direct;
                        }
                    }
                }
            }
        }
    }
    
    forces[particle_idx] = total_force;
}

// Helper function for multipole expansion (future enhancement)
fn calculate_multipole_force(
    particle_pos: vec3<f32>,
    particle_mass: f32,
    node: OctreeNode,
    order: u32
) -> vec3<f32> {
    // This would implement higher-order multipole expansion
    // for improved accuracy in the far-field
    
    let r_vec = node.center_of_mass - particle_pos;
    let r_squared = dot(r_vec, r_vec) + SOFTENING;
    let r = sqrt(r_squared);
    
    if (r > SOFTENING) {
        let force_magnitude = G * particle_mass * node.total_mass / r_squared;
        return force_magnitude * r_vec / r;
    }
    
    return vec3<f32>(0.0, 0.0, 0.0);
}

// Optimized distance calculation for bounding box diagonal
fn bbox_diagonal(min_pos: vec3<f32>, max_pos: vec3<f32>) -> f32 {
    let size = max_pos - min_pos;
    return length(size);
}

// Check if a point is inside a bounding box
fn point_in_bbox(point: vec3<f32>, min_pos: vec3<f32>, max_pos: vec3<f32>) -> bool {
    return point.x >= min_pos.x && point.x <= max_pos.x &&
           point.y >= min_pos.y && point.y <= max_pos.y &&
           point.z >= min_pos.z && point.z <= max_pos.z;
}