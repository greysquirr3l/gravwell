// GPU Tree Construction for Barnes-Hut Algorithm
// Builds hierarchical octree from sorted Morton codes

struct OctreeNode {
    // Child indices (8 children for octree, 0xFFFFFFFF = no child)
    children: array<u32, 8>,
    
    // Particle range for leaf nodes
    particle_start: u32,
    particle_count: u32,
    
    // Bounding box
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    
    // Center of mass and total mass
    com_x: f32,
    com_y: f32,
    com_z: f32,
    total_mass: f32,
    
    // Node type: 0 = internal, 1 = leaf
    node_type: u32,
    
    // Padding for alignment
    _padding: array<u32, 3>,
}

@group(0) @binding(0) var<storage, read> sorted_morton_codes: array<u64>;
@group(0) @binding(1) var<storage, read> sorted_particle_indices: array<u32>;
@group(0) @binding(2) var<storage, read> particle_positions: array<vec3<f32>>;
@group(0) @binding(3) var<storage, read> particle_masses: array<f32>;
@group(0) @binding(4) var<storage, read_write> octree_nodes: array<OctreeNode>;
@group(0) @binding(5) var<storage, read_write> node_counter: atomic<u32>;
@group(0) @binding(6) var<uniform> tree_params: TreeBuildParams;

struct TreeBuildParams {
    num_particles: u32,
    max_depth: u32,
    min_particles_per_node: u32,
    root_min_x: f32,
    root_min_y: f32,
    root_min_z: f32,
    root_max_x: f32,
    root_max_y: f32,
    root_max_z: f32,
}

// Phase 1: Build tree structure from Morton codes
@compute @workgroup_size(64, 1, 1)
fn build_tree_structure(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;
    
    if (thread_id >= tree_params.num_particles) {
        return;
    }
    
    // For first thread, initialize root node
    if (thread_id == 0u) {
        let root_idx = atomicAdd(&node_counter, 1u);
        init_root_node(root_idx);
    }
    
    workgroupBarrier();
    
    // Each thread processes one particle and finds its leaf node
    let particle_idx = sorted_particle_indices[thread_id];
    let morton_code = sorted_morton_codes[thread_id];
    
    insert_particle_recursive(0u, particle_idx, morton_code, 0u);
}

// Phase 2: Compute centers of mass bottom-up
@compute @workgroup_size(64, 1, 1)
fn compute_centers_of_mass(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let node_idx = global_id.x;
    let total_nodes = atomicLoad(&node_counter);
    
    if (node_idx >= total_nodes) {
        return;
    }
    
    // Process nodes in reverse order (bottom-up)
    let actual_node_idx = total_nodes - 1u - node_idx;
    compute_node_center_of_mass(actual_node_idx);
}

// Initialize root node covering entire space
fn init_root_node(node_idx: u32) {
    octree_nodes[node_idx].children = array<u32, 8>(
        0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu,
        0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu
    );
    
    octree_nodes[node_idx].particle_start = 0u;
    octree_nodes[node_idx].particle_count = tree_params.num_particles;
    
    octree_nodes[node_idx].min_x = tree_params.root_min_x;
    octree_nodes[node_idx].min_y = tree_params.root_min_y;
    octree_nodes[node_idx].min_z = tree_params.root_min_z;
    octree_nodes[node_idx].max_x = tree_params.root_max_x;
    octree_nodes[node_idx].max_y = tree_params.root_max_y;
    octree_nodes[node_idx].max_z = tree_params.root_max_z;
    
    octree_nodes[node_idx].com_x = 0.0;
    octree_nodes[node_idx].com_y = 0.0;
    octree_nodes[node_idx].com_z = 0.0;
    octree_nodes[node_idx].total_mass = 0.0;
    
    octree_nodes[node_idx].node_type = 0u; // Internal node initially
}

// Recursively insert particle into tree
fn insert_particle_recursive(node_idx: u32, particle_idx: u32, morton_code: u64, depth: u32) {
    if (depth >= tree_params.max_depth) {
        // Force leaf node at maximum depth
        octree_nodes[node_idx].node_type = 1u;
        return;
    }
    
    let node = &octree_nodes[node_idx];
    
    // Check if this should be a leaf node
    if (node.particle_count <= tree_params.min_particles_per_node) {
        octree_nodes[node_idx].node_type = 1u;
        return;
    }
    
    // Determine which octant the particle belongs to
    let octant = determine_octant(node_idx, particle_idx);
    
    // Create child node if it doesn't exist
    if (octree_nodes[node_idx].children[octant] == 0xFFFFFFFFu) {
        let child_idx = atomicAdd(&node_counter, 1u);
        octree_nodes[node_idx].children[octant] = child_idx;
        init_child_node(child_idx, node_idx, octant);
    }
    
    let child_idx = octree_nodes[node_idx].children[octant];
    insert_particle_recursive(child_idx, particle_idx, morton_code, depth + 1u);
}

// Determine which octant a particle belongs to
fn determine_octant(node_idx: u32, particle_idx: u32) -> u32 {
    let node = &octree_nodes[node_idx];
    let pos = particle_positions[particle_idx];
    
    let center_x = (node.min_x + node.max_x) * 0.5;
    let center_y = (node.min_y + node.max_y) * 0.5;
    let center_z = (node.min_z + node.max_z) * 0.5;
    
    var octant = 0u;
    
    if (pos.x >= center_x) { octant |= 1u; }
    if (pos.y >= center_y) { octant |= 2u; }
    if (pos.z >= center_z) { octant |= 4u; }
    
    return octant;
}

// Initialize child node with appropriate bounding box
fn init_child_node(child_idx: u32, parent_idx: u32, octant: u32) {
    let parent = &octree_nodes[parent_idx];
    
    let center_x = (parent.min_x + parent.max_x) * 0.5;
    let center_y = (parent.min_y + parent.max_y) * 0.5;
    let center_z = (parent.min_z + parent.max_z) * 0.5;
    
    // Initialize child structure
    octree_nodes[child_idx].children = array<u32, 8>(
        0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu,
        0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu, 0xFFFFFFFFu
    );
    
    octree_nodes[child_idx].particle_start = 0u;
    octree_nodes[child_idx].particle_count = 0u;
    
    // Set bounding box based on octant
    if ((octant & 1u) != 0u) {
        octree_nodes[child_idx].min_x = center_x;
        octree_nodes[child_idx].max_x = parent.max_x;
    } else {
        octree_nodes[child_idx].min_x = parent.min_x;
        octree_nodes[child_idx].max_x = center_x;
    }
    
    if ((octant & 2u) != 0u) {
        octree_nodes[child_idx].min_y = center_y;
        octree_nodes[child_idx].max_y = parent.max_y;
    } else {
        octree_nodes[child_idx].min_y = parent.min_y;
        octree_nodes[child_idx].max_y = center_y;
    }
    
    if ((octant & 4u) != 0u) {
        octree_nodes[child_idx].min_z = center_z;
        octree_nodes[child_idx].max_z = parent.max_z;
    } else {
        octree_nodes[child_idx].min_z = parent.min_z;
        octree_nodes[child_idx].max_z = center_z;
    }
    
    octree_nodes[child_idx].com_x = 0.0;
    octree_nodes[child_idx].com_y = 0.0;
    octree_nodes[child_idx].com_z = 0.0;
    octree_nodes[child_idx].total_mass = 0.0;
    
    octree_nodes[child_idx].node_type = 0u; // Internal node initially
}

// Compute center of mass for a node
fn compute_node_center_of_mass(node_idx: u32) {
    let node = &octree_nodes[node_idx];
    
    if (node.node_type == 1u) {
        // Leaf node: compute from particles
        compute_leaf_center_of_mass(node_idx);
    } else {
        // Internal node: compute from children
        compute_internal_center_of_mass(node_idx);
    }
}

// Compute center of mass for leaf node from particles
fn compute_leaf_center_of_mass(node_idx: u32) {
    let node = &octree_nodes[node_idx];
    
    var total_mass = 0.0;
    var com_x = 0.0;
    var com_y = 0.0;
    var com_z = 0.0;
    
    // Sum over all particles in this leaf
    for (var i = node.particle_start; i < node.particle_start + node.particle_count; i += 1u) {
        let particle_idx = sorted_particle_indices[i];
        let mass = particle_masses[particle_idx];
        let pos = particle_positions[particle_idx];
        
        total_mass += mass;
        com_x += mass * pos.x;
        com_y += mass * pos.y;
        com_z += mass * pos.z;
    }
    
    if (total_mass > 0.0) {
        octree_nodes[node_idx].com_x = com_x / total_mass;
        octree_nodes[node_idx].com_y = com_y / total_mass;
        octree_nodes[node_idx].com_z = com_z / total_mass;
    }
    
    octree_nodes[node_idx].total_mass = total_mass;
}

// Compute center of mass for internal node from children
fn compute_internal_center_of_mass(node_idx: u32) {
    let node = &octree_nodes[node_idx];
    
    var total_mass = 0.0;
    var com_x = 0.0;
    var com_y = 0.0;
    var com_z = 0.0;
    
    // Sum over all children
    for (var i = 0u; i < 8u; i += 1u) {
        let child_idx = node.children[i];
        
        if (child_idx != 0xFFFFFFFFu) {
            let child = &octree_nodes[child_idx];
            let child_mass = child.total_mass;
            
            if (child_mass > 0.0) {
                total_mass += child_mass;
                com_x += child_mass * child.com_x;
                com_y += child_mass * child.com_y;
                com_z += child_mass * child.com_z;
            }
        }
    }
    
    if (total_mass > 0.0) {
        octree_nodes[node_idx].com_x = com_x / total_mass;
        octree_nodes[node_idx].com_y = com_y / total_mass;
        octree_nodes[node_idx].com_z = com_z / total_mass;
    }
    
    octree_nodes[node_idx].total_mass = total_mass;
}

// Alternative: Parallel reduction for large leaf nodes
@compute @workgroup_size(64, 1, 1)
fn compute_leaf_com_parallel(@builtin(global_invocation_id) global_id: vec3<u32>,
                            @builtin(local_invocation_id) local_id: vec3<u32>) {
    let thread_id = local_id.x;
    let node_idx = global_id.y; // Node index passed as Y coordinate
    
    // Shared memory for reduction
    var<workgroup> local_mass: array<f32, 64>;
    var<workgroup> local_com_x: array<f32, 64>;
    var<workgroup> local_com_y: array<f32, 64>;
    var<workgroup> local_com_z: array<f32, 64>;
    
    let node = &octree_nodes[node_idx];
    let particle_idx = node.particle_start + thread_id;
    
    // Load particle data
    if (thread_id < node.particle_count) {
        let actual_particle_idx = sorted_particle_indices[particle_idx];
        let mass = particle_masses[actual_particle_idx];
        let pos = particle_positions[actual_particle_idx];
        
        local_mass[thread_id] = mass;
        local_com_x[thread_id] = mass * pos.x;
        local_com_y[thread_id] = mass * pos.y;
        local_com_z[thread_id] = mass * pos.z;
    } else {
        local_mass[thread_id] = 0.0;
        local_com_x[thread_id] = 0.0;
        local_com_y[thread_id] = 0.0;
        local_com_z[thread_id] = 0.0;
    }
    
    workgroupBarrier();
    
    // Parallel reduction
    var stride = 32u;
    while (stride > 0u) {
        if (thread_id < stride) {
            local_mass[thread_id] += local_mass[thread_id + stride];
            local_com_x[thread_id] += local_com_x[thread_id + stride];
            local_com_y[thread_id] += local_com_y[thread_id + stride];
            local_com_z[thread_id] += local_com_z[thread_id + stride];
        }
        stride /= 2u;
        workgroupBarrier();
    }
    
    // Write result
    if (thread_id == 0u) {
        let total_mass = local_mass[0];
        
        if (total_mass > 0.0) {
            octree_nodes[node_idx].com_x = local_com_x[0] / total_mass;
            octree_nodes[node_idx].com_y = local_com_y[0] / total_mass;
            octree_nodes[node_idx].com_z = local_com_z[0] / total_mass;
        }
        
        octree_nodes[node_idx].total_mass = total_mass;
    }
}