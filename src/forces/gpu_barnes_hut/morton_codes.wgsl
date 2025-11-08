// Morton Code Generation for GPU Octree Construction
// Converts 3D positions to Morton codes for spatial ordering

struct BoundingBox {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec3<f32>>;
@group(0) @binding(1) var<storage, read_write> morton_codes: array<u64>;
@group(0) @binding(2) var<uniform> bounds: BoundingBox;

@compute @workgroup_size(64, 1, 1)
fn generate_morton_codes(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if (idx >= arrayLength(&positions)) {
        return;
    }
    
    let pos = positions[idx];
    
    // Normalize position to [0, 1] within bounding box
    let size = vec3<f32>(
        bounds.max_x - bounds.min_x,
        bounds.max_y - bounds.min_y,
        bounds.max_z - bounds.min_z
    );
    
    let normalized = (pos - vec3<f32>(bounds.min_x, bounds.min_y, bounds.min_z)) / size;
    
    // Clamp to [0, 1] to handle edge cases
    let clamped = clamp(normalized, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // Convert to integer coordinates (21-bit precision for each axis)
    let max_coord = 0x1FFFFFu; // 2^21 - 1
    let x_int = u32(clamped.x * f32(max_coord));
    let y_int = u32(clamped.y * f32(max_coord));
    let z_int = u32(clamped.z * f32(max_coord));
    
    // Generate Morton code by interleaving bits
    morton_codes[idx] = morton_encode_3d(x_int, y_int, z_int);
}

// Encode 3D coordinates into Morton code by interleaving bits
fn morton_encode_3d(x: u32, y: u32, z: u32) -> u64 {
    var result = 0u64;
    
    // Interleave 21 bits from each coordinate
    for (var i = 0u; i < 21u; i += 1u) {
        let bit_x = (x >> i) & 1u;
        let bit_y = (y >> i) & 1u;
        let bit_z = (z >> i) & 1u;
        
        result |= u64(bit_x) << (3u * i);
        result |= u64(bit_y) << (3u * i + 1u);
        result |= u64(bit_z) << (3u * i + 2u);
    }
    
    return result;
}

// Alternative implementation using bit manipulation tricks
fn morton_encode_3d_fast(x: u32, y: u32, z: u32) -> u64 {
    // Expand each 21-bit coordinate to 64 bits with zeros between
    let expanded_x = expand_bits(x);
    let expanded_y = expand_bits(y);
    let expanded_z = expand_bits(z);
    
    // Interleave the expanded coordinates
    return expanded_x | (expanded_y << 1u) | (expanded_z << 2u);
}

// Expand 21-bit integer to 64-bit with zeros between bits
fn expand_bits(x: u32) -> u64 {
    var result = u64(x) & 0x1FFFFFu64; // Keep only 21 bits
    
    // Magic bit manipulation for fast expansion
    result = (result | (result << 32u)) & 0x1F00000000FFFFu64;
    result = (result | (result << 16u)) & 0x1F0000FF0000FFu64;
    result = (result | (result << 8u)) & 0x100F00F00F00F00Fu64;
    result = (result | (result << 4u)) & 0x10C30C30C30C30C3u64;
    result = (result | (result << 2u)) & 0x1249249249249249u64;
    
    return result;
}

// Decode Morton code back to 3D coordinates (for debugging)
fn morton_decode_3d(morton: u64) -> vec3<u32> {
    let x = compact_bits(morton);
    let y = compact_bits(morton >> 1u);
    let z = compact_bits(morton >> 2u);
    
    return vec3<u32>(x, y, z);
}

// Compact 64-bit value with zeros between bits to 21-bit integer
fn compact_bits(x: u64) -> u32 {
    var result = x & 0x1249249249249249u64;
    
    result = (result ^ (result >> 2u)) & 0x10C30C30C30C30C3u64;
    result = (result ^ (result >> 4u)) & 0x100F00F00F00F00Fu64;
    result = (result ^ (result >> 8u)) & 0x1F0000FF0000FFu64;
    result = (result ^ (result >> 16u)) & 0x1F00000000FFFFu64;
    result = (result ^ (result >> 32u)) & 0x1FFFFFu64;
    
    return u32(result);
}