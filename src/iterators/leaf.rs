use crate::chunk::Chunk;
use crate::direction::Direction;
use crate::voxel::Voxel;
use crate::index_path::IndexPath;
use crate::bounds::Bounds;
use crate::node::Node;

pub struct ChunkLeafIterator<'a, T> {
    chunk: &'a Chunk<T>,
    stack: Vec<(Direction, &'a Node<T>)>,
    index_path: IndexPath,
    bounds: Bounds,
    dir: u8, // Next voxel to emit
}

impl<'a, T> Iterator for ChunkLeafIterator<'a, T> {
    type Item = Voxel<'a, T>;

    /// Iterates all leaf nodes.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tuple) = self.stack.last() {
                // Peek the last node on the stack, which stores the indice to that arena node,
                // as well as how we get to this node (from the parent node)
                let (fromdir, node) = *tuple;

                if self.dir >= 8 {
                    // We've finished iterating all dirs on this node.
                    // Pop from stack, and continue from where we left off on the parent node
                    self.stack.pop();
                    if self.stack.is_empty() {
                        return None; // If we just popped the last item from the stack, fromdir is actually meaningless. Return directly.
                    }
                    self.index_path = self.index_path.del();
                    self.bounds = self.bounds.merge(fromdir);
                    self.dir = fromdir as u8 + 1;
                    continue;
                }

                if let Some(subnode) = &node.children[self.dir.into()] {
                    // Has a child on that dir, needs to go deeper
                    let dir: Direction = self.dir.into();
                    self.stack.push((dir, subnode));
                    self.index_path = self.index_path.put(dir);
                    self.bounds = self.bounds.half(dir);
                    self.dir = 0;
                    continue;
                } else {
                    let dir = self.dir;
                    self.dir += 1;
                    return Some(Voxel {
                        node,
                        index_path: self.index_path.put(dir.into()),
                        bounds: self.bounds.half(dir.into()),
                    });
                }
            } else {
                // The stack is empty meaning that there is no more nodes.
                return None;
            }
        }
    }
}
impl<T> Chunk<T> {
    pub fn iter_leaf(&self) -> ChunkLeafIterator<T> {
        ChunkLeafIterator {
            chunk: &self,
            stack: vec![(0.into(), &self.root)],
            index_path: IndexPath::new(),
            bounds: Bounds::new(),
            dir: 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index_path::IndexPath;
    use crate::world_builder::{WorldBuilder, Isosurface};
    use crate::bounds::{Bounds, BoundsSpacialRelationship};
    use crate::world::ChunkCoordinates;

    #[test]
    fn test_leaf_iterator() {
        let mut chunk: Chunk<u16> = Chunk::new();
        for i in 0..7 {
            chunk.set(IndexPath::new().push(i.into()), i as u16);
        }
        for i in 0..7 {
            chunk.set(IndexPath::new().push(i.into()).push(Direction::RearRightTop), i as u16 + 16);
        }

        for i in 0..8 {
            chunk.set(IndexPath::new().push(i.into()).push(Direction::RearRightTop).push(Direction::RearRightTop), i as u16 + 32);
        }

        let mut iter = chunk.iter_leaf();
        for (i, voxel) in iter.enumerate() {
            if i < 7 {
                assert_eq!(*voxel.get_value(), i as u16);
                assert_eq!(voxel.get_index_path(), IndexPath::new().push((i as u8).into()));
            } else if i < 14 {
                assert_eq!(*voxel.get_value(), i as u16 + 9);
                assert_eq!(
                    voxel.get_index_path(),
                    IndexPath::new()
                        .push((i as u8 - 7).into())
                        .push(Direction::RearRightTop)
                );
            } else {
                assert_eq!(*voxel.get_value(), i as u16 + 18);
                assert_eq!(
                    voxel.get_index_path(),
                    IndexPath::new()
                        .push((i as u8 - 14).into())
                        .push(Direction::RearRightTop)
                        .push(Direction::RearRightTop)
                );
            }
        }
    }

    #[test]
    fn test_leaf_iterator_cube_generator() {
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
        for (i, voxel) in chunk.iter_leaf().enumerate() {
        }
    }
}

