// src/core.rs

use noise::{NoiseFn, Perlin};

/// Exact memory layout for a single vertex.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Configuration parameters for terrain generation.
pub struct TerralithConfig {
    pub seed: u32,
    pub chunk_size: usize,
    pub spacing: f32,
    pub height_scale: f32,
    pub noise_scale: f64,
}

/// Pure Rust container for the generated data.
pub struct ChunkData {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub heights: Vec<f32>,
}

pub struct TerralithGenerator {
    pub config: TerralithConfig,
    noise: Perlin,
}

impl TerralithGenerator {
    pub fn new(config: TerralithConfig) -> Self {
        let noise = Perlin::new(config.seed);
        Self { config, noise }
    }

    /// Evaluates noise height at world coordinates.
    fn sample_height(&self, wx: f32, wz: f32) -> f32 {
        let nx = wx as f64 * self.config.noise_scale;
        let nz = wz as f64 * self.config.noise_scale;
        
        // Sample Perlin noise (range roughly -1.0 to 1.0) and scale to desired elevation
        let n = self.noise.get([nx, nz]) as f32;
        n * self.config.height_scale
    }

    /// Computes smooth analytical normal via central finite differences.
    fn calculate_normal(&self, wx: f32, wz: f32, delta: f32) -> [f32; 3] {
        let hl = self.sample_height(wx - delta, wz);
        let hr = self.sample_height(wx + delta, wz);
        let hd = self.sample_height(wx, wz - delta);
        let hu = self.sample_height(wx, wz + delta);

        // Vector cross product approximation (-dh/dx, 2*delta, -dh/dz)
        let vx = hl - hr;
        let vy = 2.0 * delta;
        let vz = hd - hu;

        let len = (vx * vx + vy * vy + vz * vz).sqrt();
        if len > 0.0 {
            [vx / len, vy / len, vz / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }

    /// Generates mesh and height data for a specific chunk coordinate.
    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> ChunkData {
        let size = self.config.chunk_size;
        let spacing = self.config.spacing;
        
        let num_vertices = (size + 1) * (size + 1);
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut heights = Vec::with_capacity(num_vertices);
        
        let half_size = (size as f32 * spacing) * 0.5;

        // 1. Generate Vertices and Heights with analytical normals
        for z in 0..=size {
            for x in 0..=size {
                let world_x = (chunk_x as f32 * size as f32 + x as f32) * spacing - half_size;
                let world_z = (chunk_z as f32 * size as f32 + z as f32) * spacing - half_size;
                
                let height = self.sample_height(world_x, world_z);
                let normal = self.calculate_normal(world_x, world_z, spacing * 0.5);
                
                heights.push(height);
                vertices.push(TerrainVertex {
                    position: [world_x, height, world_z],
                    normal,
                    uv: [x as f32 / size as f32, z as f32 / size as f32],
                });
            }
        }

        // 2. Generate Triangles (Indices) in CCW winding order
        let num_indices = size * size * 6;
        let mut indices = Vec::with_capacity(num_indices);
        
        for z in 0..size {
            for x in 0..size {
                let top_left = (z * (size + 1) + x) as u32;
                let top_right = top_left + 1;
                let bot_left = ((z + 1) * (size + 1) + x) as u32;
                let bot_right = bot_left + 1;

                // Triangle 1
                indices.push(top_left);
                indices.push(top_right);
                indices.push(bot_left);

                // Triangle 2
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