// WGSL compute shader for gravitational force calculation
// Optimized for WebGPU with 64-thread workgroups

struct SimParams {
    gravity_constant: f32,
    particle_count: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> masses: array<f32>;
@group(0) @binding(2) var<storage, read_write> forces: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: SimParams;

const WORKGROUP_SIZE: u32 = 64u;
const SOFTENING_EPSILON: f32 = 1e-6;

@compute @workgroup_size(64, 1, 1)
fn calculate_forces(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    let particle_count = u32(params.particle_count);
    
    if i >= particle_count {
        return;
    }
    
    // Initialize force accumulator
    var total_force = vec3<f32>(0.0);
    
    // Current particle data
    let pos_i = positions[i].xyz;
    let mass_i = masses[i];
    
    // Calculate forces from all other particles
    for (var j = 0u; j < particle_count; j++) {
        if i == j {
            continue;
        }
        
        let pos_j = positions[j].xyz;
        let mass_j = masses[j];
        
        // Calculate displacement vector
        let r_vec = pos_j - pos_i;
        let r_squared = dot(r_vec, r_vec) + SOFTENING_EPSILON;
        let r = sqrt(r_squared);
        
        // Calculate force magnitude
        let force_magnitude = params.gravity_constant * mass_i * mass_j / r_squared;
        
        // Add force contribution (F = GMm/r² * r̂)
        total_force += force_magnitude * (r_vec / r);
    }
    
    // Store result (with zero w component)
    forces[i] = vec4<f32>(total_force, 0.0);
}