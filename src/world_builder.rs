use crate::world::ChunkCoordinates;
use crate::index_path::IndexPath;
use crate::chunk::Chunk;
use crate::direction::Direction;
use crate::node::Node;
use crate::bounds::Bounds;

pub enum Isosurface<T> {
    Uniform(T), // Everything within the bounding box is T
    Surface, // There exist multiple materials within this bounding box
}

pub type WorldBuildIsosurfaceOracle<T> = Box<dyn Fn(&ChunkCoordinates, &Bounds) -> Isosurface<T>>;

pub struct WorldBuilder<T, ORACLE: Fn(&ChunkCoordinates, &Bounds) -> Isosurface<T>>{
    oracle: ORACLE
}

impl<T: Copy + Default + PartialEq, ORACLE> WorldBuilder<T, ORACLE>
    where ORACLE: Fn(&ChunkCoordinates, &Bounds) -> Isosurface<T> {
    pub fn new(oracle: ORACLE) -> WorldBuilder<T, ORACLE> {
        WorldBuilder {
            oracle
        }
    }
    pub fn build(&self, chunk_coords: &ChunkCoordinates) -> Chunk<T> {
        let mut chunk: Chunk<T> = Chunk::new();

        self.build_recurse(chunk_coords, &Bounds::new(), &mut chunk.root);
        chunk
    }

    fn build_recurse(&self, chunk_coords: &ChunkCoordinates, bounds: &Bounds, node: &mut Node<T>) {
        for (dir, subnode) in node.children.enumerate_mut() {
            let subbounds = bounds.half(dir);
            match (self.oracle)(chunk_coords, &subbounds) {
                Isosurface::Uniform(value) => {
                    node.data[dir] = value;
                    *subnode = None;
                }
                Isosurface::Surface => {
                    if let Some(subnode) = subnode.as_mut() {
                        self.build_recurse(chunk_coords, &subbounds, subnode);
                    } else {
                        let mut newnode = Node::new_all(Default::default());
                        self.build_recurse(chunk_coords, &subbounds, &mut newnode);
                        *subnode = Some(newnode);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    use crate::bounds::BoundsSpacialRelationship;

    #[test]
    fn test_cube() {
        let world_builder: WorldBuilder<u32, _> = WorldBuilder::new(
            |chunk: &ChunkCoordinates, bounds: &Bounds| {
                let target_bounds = Bounds::from_discrete_grid((32, 32, 32), 32, 128);
                match target_bounds.intersects(bounds) {
                    BoundsSpacialRelationship::Disjoint => Isosurface::Uniform(0),
                    BoundsSpacialRelationship::Contain => Isosurface::Uniform(1),
                    BoundsSpacialRelationship::Intersect => Isosurface::Surface,
                }
            }
        );
        let chunk = world_builder.build(&ChunkCoordinates::new());
    }
}

