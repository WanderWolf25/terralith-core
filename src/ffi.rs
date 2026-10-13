// src/ffi.rs

use crate::core::{TerrainVertex, TerralithConfig, TerralithGenerator};
use std::mem;

/// The payload passed across the FFI boundary.
/// It contains raw pointers and lengths for the vertices, indices, and heights.
#[repr(C)]
pub struct ChunkPayload {
    pub vertices_ptr: *mut TerrainVertex,
    pub vertex_count: usize,
    pub indices_ptr: *mut u32,
    pub index_count: usize,
    pub heights_ptr: *mut f32,
    pub height_count: usize,
}

/// Creates a new generator instance and returns a raw pointer to it.
/// The caller is responsible for eventually calling `terralith_generator_free`.
#[no_mangle]
pub extern "C" fn terralith_generator_new(
    seed: u32,
    chunk_size: usize,
    spacing: f32,
    height_scale: f32,
    noise_scale: f64,
) -> *mut TerralithGenerator {
    let config = TerralithConfig {
        seed,
        chunk_size,
        spacing,
        height_scale,
        noise_scale,
    };
    
    // Allocate generator on the heap and surrender its pointer to C/C#
    let generator = Box::new(TerralithGenerator::new(config));
    Box::into_raw(generator)
}

/// Frees the generator memory.
#[no_mangle]
pub unsafe extern "C" fn terralith_generator_free(ptr: *mut TerralithGenerator) {
    if !ptr.is_null() {
        // Taking the pointer back into a Box will automatically drop it
        let _ = Box::from_raw(ptr); 
    }
}

/// Generates a chunk and leaks its memory into a C-ABI compatible payload.
/// The caller MUST pass the returned payload to `terralith_free_chunk` when done.
#[no_mangle]
pub unsafe extern "C" fn terralith_generate_chunk(
    generator_ptr: *mut TerralithGenerator,
    chunk_x: i32,
    chunk_z: i32,
) -> ChunkPayload {
    if generator_ptr.is_null() {
        return ChunkPayload {
            vertices_ptr: std::ptr::null_mut(),
            vertex_count: 0,
            indices_ptr: std::ptr::null_mut(),
            index_count: 0,
            heights_ptr: std::ptr::null_mut(),
            height_count: 0,
        };
    }

    // Safely dereference the generator pointer
    let generator = &*generator_ptr;
    let chunk_data = generator.generate_chunk(chunk_x, chunk_z);

    // Convert Vecs into boxed slices to lock their size in memory, then get raw pointers
    let mut vertices = chunk_data.vertices.into_boxed_slice();
    let mut indices = chunk_data.indices.into_boxed_slice();
    let mut heights = chunk_data.heights.into_boxed_slice();

    let payload = ChunkPayload {
        vertices_ptr: vertices.as_mut_ptr(),
        vertex_count: vertices.len(),
        indices_ptr: indices.as_mut_ptr(),
        index_count: indices.len(),
        heights_ptr: heights.as_mut_ptr(),
        height_count: heights.len(),
    };

    // mem::forget tells Rust NOT to free this memory when the function ends.
    // Unity/Godot will hold this memory until they call `terralith_free_chunk`.
    mem::forget(vertices);
    mem::forget(indices);
    mem::forget(heights);

    payload
}

/// Takes ownership of the memory allocated for a chunk payload and frees it.
#[no_mangle]
pub unsafe extern "C" fn terralith_free_chunk(payload: ChunkPayload) {
    // Reconstruct the slices from raw parts so Rust knows exactly how much memory to free
    if !payload.vertices_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(payload.vertices_ptr, payload.vertex_count));
    }
    if !payload.indices_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(payload.indices_ptr, payload.index_count));
    }
    if !payload.heights_ptr.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(payload.heights_ptr, payload.height_count));
    }
}
