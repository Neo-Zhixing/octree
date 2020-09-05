use std::collections::HashMap;
use crate::chunk::Chunk;
use crate::VoxelData;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ChunkCoordinates(i64, i64, i64);

impl ChunkCoordinates {
    pub fn new() -> Self {
        Self(0, 0, 0)
    }
}
pub struct World<T> {
    nodes: HashMap<ChunkCoordinates, Chunk<T>>,
}
impl<T: VoxelData> World<T> {
    pub fn get_chunk_ref(&self, location: &ChunkCoordinates) -> Option<&Chunk<T>> {
        self.nodes.get(location)
    }
}
