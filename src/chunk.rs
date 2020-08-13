use crate::node::Node;
use crate::direction::Direction;
use crate::index_path::IndexPath;

pub struct Chunk<T> {
    pub(crate) root: Node<T>
}

impl<T: Default + Copy + PartialEq> Chunk<T> {
    pub fn new() -> Chunk<T> {
        Chunk {
            root: Node::new_all(Default::default())
        }
    }
}

pub struct Voxel<'a, T> {
    node: &'a Node<T>,
    index_path: IndexPath,
}

impl<'a, T> Voxel<'a, T> {
    pub fn get_value(&self) -> &T{
        &self.node.data[self.index_path.get()]
    }
    pub fn is_leaf(&self) -> bool {
        self.node.children[self.index_path.get()].is_none()
    }
    pub fn is_subdivided(&self) -> bool {
        self.node.children[self.index_path.get()].is_some()
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for Voxel<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.get_value())
    }
}

pub struct ChunkLeafIterator<'a, T> {
    chunk: &'a Chunk<T>,
    stack: Vec<(Direction, &'a Node<T>)>,
    index_path: IndexPath,
    dir: u8, // Next voxel to emit
}

impl<T> Chunk<T> {
    pub fn iter_leaf(&self) -> ChunkLeafIterator<T> {
        ChunkLeafIterator {
            chunk: &self,
            stack: vec![(0.into(), &self.root)],
            index_path: IndexPath::new(),
            dir: 0
        }
    }
    fn get(&self, index_path: IndexPath) -> &T {
        self.root.get(index_path)
    }
}

impl<T: Copy + PartialEq> Chunk<T> {
    fn set(&mut self, index_path: IndexPath, value: T) {
        self.root.set(index_path, value)
    }
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
                    self.index_path = self.index_path.del();
                    self.dir = fromdir as u8 + 1;
                    continue;
                }

                if let Some(subnode) = &node.children[self.dir.into()] {
                    // Has a child on that dir, needs to go deeper
                    let dir: Direction = self.dir.into();
                    self.stack.push((dir, subnode));
                    self.index_path = self.index_path.put(dir);
                    self.dir = 0;
                    continue;
                } else {
                    let dir = self.dir;
                    self.dir += 1;
                    return Some(Voxel {
                        node,
                        index_path: self.index_path.put(dir.into()),
                    });
                }
            } else {
                // The stack is empty meaning that there is no more nodes.
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index_path::IndexPath;

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
                assert_eq!(voxel.index_path, IndexPath::new().push((i as u8).into()));
            } else if i < 14 {
                assert_eq!(*voxel.get_value(), i as u16 + 9);
                assert_eq!(
                    voxel.index_path,
                    IndexPath::new()
                        .push((i as u8 - 7).into())
                        .push(Direction::RearRightTop)
                );
            } else {
                assert_eq!(*voxel.get_value(), i as u16 + 18);
                assert_eq!(
                    voxel.index_path,
                    IndexPath::new()
                        .push((i as u8 - 14).into())
                        .push(Direction::RearRightTop)
                        .push(Direction::RearRightTop)
                );
            }
        }
    }
}
