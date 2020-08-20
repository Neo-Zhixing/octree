use crate::node::Node;
use crate::voxel::Voxel;
use crate::index_path::IndexPath;
use crate::bounds::Bounds;

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

impl<T> Chunk<T> {
    pub fn get(&self, index_path: IndexPath) -> &T {
        self.root.get(index_path)
    }
    pub fn get_root(&self) -> Voxel<T> {
        Voxel {
            node: &self.root,
            index_path: IndexPath::new(),
            bounds: Bounds::new(),
        }
    }
}

impl<T: Copy + PartialEq> Chunk<T> {
    pub fn set(&mut self, index_path: IndexPath, value: T) {
        self.root.set(index_path, value)
    }
}
