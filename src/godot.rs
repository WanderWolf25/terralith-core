// src/godot.rs

use godot::prelude::*;
// Quitamos "Mesh" de aquí porque ya no lo usamos directamente
use godot::classes::{ArrayMesh, HeightMapShape3D}; 
use godot::classes::mesh::{ArrayType, PrimitiveType}; 

use crate::core::{TerralithConfig, TerralithGenerator};

struct TerralithExtension;

#[gdextension]
unsafe impl ExtensionLibrary for TerralithExtension {}

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct TerralithNode {
    generator: Option<TerralithGenerator>,
    base: Base<Node3D>,
    
    #[export]
    pub chunk_size: i32,
    
    #[export]
    pub spacing: f32,
    
    #[export]
    pub seed: i32,
}

#[godot_api]
impl INode3D for TerralithNode {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            generator: None,
            base,
            chunk_size: 32,
            spacing: 1.0,
            seed: 1337,
        }
    }

    fn ready(&mut self) {
        let config = TerralithConfig {
            seed: self.seed as u32,
            chunk_size: self.chunk_size as usize,
            spacing: self.spacing,
            height_scale: 12.0,
            noise_scale: 0.05,
        };
        
        self.generator = Some(TerralithGenerator::new(config));
        godot_print!("Terralith Generator initialized in Godot!");
    }
}

#[godot_api]
impl TerralithNode {
    #[func]
    pub fn build_chunk_mesh(&self, chunk_x: i32, chunk_z: i32) -> Gd<ArrayMesh> {
        let generator = self.generator.as_ref().expect("Generator not initialized");
        let chunk_data = generator.generate_chunk(chunk_x, chunk_z);

        let mut godot_vertices = PackedVector3Array::new();
        for v in &chunk_data.vertices {
            godot_vertices.push(Vector3::new(v.position[0], v.position[1], v.position[2]));
        }

        let mut godot_indices = PackedInt32Array::new();
        for &idx in &chunk_data.indices {
            godot_indices.push(idx as i32);
        }

        let mut arrays = godot::builtin::Array::<Variant>::new();
        let max_arrays = ArrayType::MAX.ord() as usize;
        for _ in 0..max_arrays {
            // Pasamos el Variant prestado con "&"
            arrays.push(&Variant::nil());
        }

        // Pasamos los Variants prestados con "&"
        arrays.set(ArrayType::VERTEX.ord() as usize, &godot_vertices.to_variant());
        arrays.set(ArrayType::INDEX.ord() as usize, &godot_indices.to_variant());

        let mut mesh = ArrayMesh::new_gd();
        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);

        mesh
    }

    #[func]
    pub fn build_chunk_collider(&self, chunk_x: i32, chunk_z: i32) -> Gd<HeightMapShape3D> {
        let generator = self.generator.as_ref().expect("Generator not initialized");
        let chunk_data = generator.generate_chunk(chunk_x, chunk_z);

        let mut godot_heights = PackedFloat32Array::new();
        for &h in &chunk_data.heights {
            godot_heights.push(h);
        }

        let mut shape = HeightMapShape3D::new_gd();
        let map_size = (self.chunk_size + 1) as i32;
        
        shape.set_map_width(map_size);
        shape.set_map_depth(map_size);
        shape.set_map_data(&godot_heights);

        shape
    }
}