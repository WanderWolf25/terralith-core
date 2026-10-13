// src/core.rs

/// Exact memory layout for a single vertex.
/// #[repr(C)] ensures C-ABI compatibility so C# and C++ read it correctly.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct TerrainVertex {
    pub position: [f32; 3], // 12 bytes
    pub normal: [f32; 3],   // 12 bytes
    pub uv: [f32; 2],       // 8 bytes
} // Total: 32 bytes per vertex

/// Configuration parameters for the terrain generation.
pub struct TerralithConfig {
    pub seed: u32,
    pub chunk_size: usize,
    pub spacing: f32,
    pub height_scale: f32,
    pub noise_scale: f64,
}

/// Pure Rust container for the generated data.
/// We use safe Vectors here. The FFI layer will later convert these into raw pointers.
pub struct ChunkData {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub heights: Vec<f32>,
}

/// The main generator engine.
pub struct TerralithGenerator {
    pub config: TerralithConfig,
}

impl TerralithGenerator {
    pub fn new(config: TerralithConfig) -> Self {
        Self { config }
    }

    /// Generates mesh and height data for a specific chunk coordinate.
    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> ChunkData {
        let size = self.config.chunk_size;
        let spacing = self.config.spacing;

        let num_vertices = (size + 1) * (size + 1);
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut heights = Vec::with_capacity(num_vertices);

        // 1. Calculate chunk half-size to center the mesh with HeightMapShape3D
        let half_size = (size as f32 * spacing) * 0.5;

        // 2. Generate Vertices and Heights
        for z in 0..=size {
            for x in 0..=size {
                let world_x = (chunk_x as f32 * size as f32 + x as f32) * spacing - half_size;
                let world_z = (chunk_z as f32 * size as f32 + z as f32) * spacing - half_size;

                let height = 0.0; // Placeholder for procedural math

                heights.push(height);
                vertices.push(TerrainVertex {
                    position: [world_x, height, world_z],
                    normal: [0.0, 1.0, 0.0],
                    uv: [x as f32 / size as f32, z as f32 / size as f32],
                });
            }
        }

        // 3. Generate Triangles (Indices) in counter-clockwise (CCW) winding order
        let num_indices = size * size * 6;
        let mut indices = Vec::with_capacity(num_indices);

        for z in 0..size {
            for x in 0..size {
                let top_left = (z * (size + 1) + x) as u32;
                let top_right = top_left + 1;
                let bot_left = ((z + 1) * (size + 1) + x) as u32;
                let bot_right = bot_left + 1;

                // Invert winding order so the surface faces upwards (+Y)
                // First triangle
                indices.push(top_left);
                indices.push(top_right);
                indices.push(bot_left);

                // Second triangle
                indices.push(top_right);
                indices.push(bot_right);
                indices.push(bot_left);
            }
        }

        ChunkData {
            vertices,
            indices,
            heights,
        }
    }
}
