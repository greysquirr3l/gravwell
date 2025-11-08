// GPU Radix Sort for Morton Codes
// Implements parallel radix sort for ordering particles by Morton codes

// Constants for radix sort
const RADIX_BITS: u32 = 8u;           // Process 8 bits per pass
const RADIX_SIZE: u32 = 256u;         // 2^8 buckets
const WORKGROUP_SIZE: u32 = 256u;     // Match RADIX_SIZE for efficiency

// Shared memory for local reduction
var<workgroup> local_histogram: array<atomic<u32>, RADIX_SIZE>;
var<workgroup> local_prefix_sum: array<atomic<u32>, RADIX_SIZE>;

@group(0) @binding(0) var<storage, read> input_keys: array<u32>;
@group(0) @binding(1) var<storage, read> input_values: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_keys: array<u32>;
@group(0) @binding(3) var<storage, read_write> output_values: array<u32>;
@group(0) @binding(4) var<storage, read_write> global_histogram: array<u32>;
@group(0) @binding(5) var<uniform> pass_info: RadixPassInfo;

struct RadixPassInfo {
    num_elements: u32,
    bit_shift: u32,
    num_workgroups: u32,
}

// Phase 1: Compute local histograms
@compute @workgroup_size(256, 1, 1)
fn compute_histogram(@builtin(global_invocation_id) global_id: vec3<u32>,
                    @builtin(local_invocation_id) local_id: vec3<u32>,
                    @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let thread_id = local_id.x;
    let global_idx = global_id.x;
    
    // Initialize local histogram
    if (thread_id < RADIX_SIZE) {
        atomicStore(&local_histogram[thread_id], 0u);
    }
    workgroupBarrier();
    
    // Process elements assigned to this thread
    let elements_per_thread = (pass_info.num_elements + WORKGROUP_SIZE - 1u) / WORKGROUP_SIZE;
    let start_idx = global_idx * elements_per_thread;
    let end_idx = min(start_idx + elements_per_thread, pass_info.num_elements);
    
    for (var i = start_idx; i < end_idx; i += 1u) {
        let key = input_keys[i];
        let digit = extract_digit(key, pass_info.bit_shift);
        atomicAdd(&local_histogram[digit], 1u);
    }
    
    workgroupBarrier();
    
    // Write local histogram to global memory
    if (thread_id < RADIX_SIZE) {
        let global_offset = workgroup_id.x * RADIX_SIZE + thread_id;
        global_histogram[global_offset] = atomicLoad(&local_histogram[thread_id]);
    }
}

// Phase 2: Compute prefix sums (exclusive scan)
@compute @workgroup_size(256, 1, 1)
fn compute_prefix_sums(@builtin(global_invocation_id) global_id: vec3<u32>,
                      @builtin(local_invocation_id) local_id: vec3<u32>) {
    let thread_id = local_id.x;
    
    if (thread_id >= RADIX_SIZE) {
        return;
    }
    
    // Load histogram values for this digit across all workgroups
    var sum = 0u;
    for (var wg = 0u; wg < pass_info.num_workgroups; wg += 1u) {
        let idx = wg * RADIX_SIZE + thread_id;
        let count = global_histogram[idx];
        global_histogram[idx] = sum; // Store prefix sum
        sum += count;
    }
}

// Phase 3: Scatter elements to their final positions
@compute @workgroup_size(256, 1, 1)
fn scatter_elements(@builtin(global_invocation_id) global_id: vec3<u32>,
                   @builtin(local_invocation_id) local_id: vec3<u32>,
                   @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let thread_id = local_id.x;
    let global_idx = global_id.x;
    
    // Initialize local prefix sum array
    if (thread_id < RADIX_SIZE) {
        let global_offset = workgroup_id.x * RADIX_SIZE + thread_id;
        local_prefix_sum[thread_id] = global_histogram[global_offset];
    }
    workgroupBarrier();
    
    // Process elements assigned to this thread
    let elements_per_thread = (pass_info.num_elements + WORKGROUP_SIZE - 1u) / WORKGROUP_SIZE;
    let start_idx = global_idx * elements_per_thread;
    let end_idx = min(start_idx + elements_per_thread, pass_info.num_elements);
    
    for (var i = start_idx; i < end_idx; i += 1u) {
        let key = input_keys[i];
        let value = input_values[i];
        let digit = extract_digit(key, pass_info.bit_shift);
        
        // Get output position and increment counter
        let output_pos = atomicAdd(&local_prefix_sum[digit], 1u);
        
        // Store in output arrays
        output_keys[output_pos] = key;
        output_values[output_pos] = value;
    }
}

// Single-pass radix sort for smaller arrays (< 1024 elements)
@compute @workgroup_size(256, 1, 1)
fn single_pass_sort(@builtin(global_invocation_id) global_id: vec3<u32>,
                   @builtin(local_invocation_id) local_id: vec3<u32>) {
    let thread_id = local_id.x;
    
    // Initialize local arrays
    if (thread_id < RADIX_SIZE) {
        local_histogram[thread_id] = 0u;
        local_prefix_sum[thread_id] = 0u;
    }
    workgroupBarrier();
    
    // Build histogram
    if (global_id.x < pass_info.num_elements) {
        let key = input_keys[global_id.x];
        let digit = extract_digit(key, pass_info.bit_shift);
        atomicAdd(&local_histogram[digit], 1u);
    }
    workgroupBarrier();
    
    // Compute prefix sum using parallel scan
    parallel_prefix_sum();
    
    // Scatter elements
    if (global_id.x < pass_info.num_elements) {
        let key = input_keys[global_id.x];
        let value = input_values[global_id.x];
        let digit = extract_digit(key, pass_info.bit_shift);
        
        let output_pos = atomicAdd(&local_prefix_sum[digit], 1u);
        output_keys[output_pos] = key;
        output_values[output_pos] = value;
    }
}

// Extract digit from key for current radix pass
fn extract_digit(key: u32, bit_shift: u32) -> u32 {
    return (key >> bit_shift) & (RADIX_SIZE - 1u);
}

// Parallel prefix sum using up-sweep/down-sweep
fn parallel_prefix_sum() {
    let thread_id = u32(workgroupUniformLoad(&thread_id));
    
    if (thread_id >= RADIX_SIZE) {
        return;
    }
    
    // Copy histogram to prefix sum array
    local_prefix_sum[thread_id] = local_histogram[thread_id];
    workgroupBarrier();
    
    // Up-sweep phase
    var stride = 1u;
    while (stride < RADIX_SIZE) {
        if (thread_id % (stride * 2u) == 0u && (thread_id + stride) < RADIX_SIZE) {
            local_prefix_sum[thread_id + stride * 2u - 1u] += 
                local_prefix_sum[thread_id + stride - 1u];
        }
        stride *= 2u;
        workgroupBarrier();
    }
    
    // Clear the last element
    if (thread_id == 0u) {
        local_prefix_sum[RADIX_SIZE - 1u] = 0u;
    }
    workgroupBarrier();
    
    // Down-sweep phase
    stride = RADIX_SIZE / 2u;
    while (stride > 0u) {
        if (thread_id % (stride * 2u) == 0u && (thread_id + stride) < RADIX_SIZE) {
            let temp = local_prefix_sum[thread_id + stride - 1u];
            local_prefix_sum[thread_id + stride - 1u] = 
                local_prefix_sum[thread_id + stride * 2u - 1u];
            local_prefix_sum[thread_id + stride * 2u - 1u] += temp;
        }
        stride /= 2u;
        workgroupBarrier();
    }
}

// Bitonic sort for very small arrays (alternative approach)
@compute @workgroup_size(256, 1, 1)
fn bitonic_sort(@builtin(global_invocation_id) global_id: vec3<u32>,
               @builtin(local_invocation_id) local_id: vec3<u32>) {
    let thread_id = local_id.x;
    
    if (global_id.x >= pass_info.num_elements) {
        return;
    }
    
    // Load data into shared memory
    var local_keys: array<u32, 256>;
    var local_values: array<u32, 256>;
    
    if (thread_id < pass_info.num_elements) {
        local_keys[thread_id] = input_keys[thread_id];
        local_values[thread_id] = input_values[thread_id];
    } else {
        local_keys[thread_id] = 0xFFFFFFFFu; // Max value for padding
        local_values[thread_id] = 0u;
    }
    workgroupBarrier();
    
    // Bitonic sort implementation
    var step = 2u;
    while (step <= 256u) {
        var substep = step / 2u;
        while (substep > 0u) {
            let partner = thread_id ^ substep;
            
            if (partner > thread_id) {
                let ascending = ((thread_id & step) == 0u);
                let should_swap = (local_keys[thread_id] > local_keys[partner]) == ascending;
                
                if (should_swap) {
                    // Swap keys
                    let temp_key = local_keys[thread_id];
                    local_keys[thread_id] = local_keys[partner];
                    local_keys[partner] = temp_key;
                    
                    // Swap values
                    let temp_value = local_values[thread_id];
                    local_values[thread_id] = local_values[partner];
                    local_values[partner] = temp_value;
                }
            }
            
            workgroupBarrier();
            substep /= 2u;
        }
        step *= 2u;
    }
    
    // Write back results
    if (thread_id < pass_info.num_elements) {
        output_keys[thread_id] = local_keys[thread_id];
        output_values[thread_id] = local_values[thread_id];
    }
}