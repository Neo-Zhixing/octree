mod marching_cubes;
mod mc_table;
use super::world::{World, ChunkCoordinates};
use super::chunk::Chunk;
use glam as math;
pub use mc_table::MC_TABLE;

pub struct Mesh {
    vertices: Vec<math::Vec3>,
    indices: Vec<u32>,
}

pub trait Mesher<'a, T> {
    fn new(world: &'a World<T>) -> Self;
    fn build(&self, chunk_location: &ChunkCoordinates, lod: u8) -> Mesh;
}
