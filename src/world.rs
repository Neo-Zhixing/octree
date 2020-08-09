use std::collections::HashMap;
use crate::chunk::Chunk;

pub struct ChunkCoordinates(i64, i64, i64);

impl ChunkCoordinates {
    pub fn new() -> Self {
        Self(0, 0, 0)
    }
}
pub struct World<T> {
    nodes: HashMap<ChunkCoordinates, Chunk<T>>,
}
