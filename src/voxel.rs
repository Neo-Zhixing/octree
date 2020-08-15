use crate::index_path::IndexPath;
use crate::node::Node;
use crate::bounds::Bounds;
use crate::direction::Direction;

#[derive(Clone)]
pub struct Voxel<'a, T> {
    pub(crate) node: &'a Node<T>,
    pub(crate) index_path: IndexPath, // when empty, voxel is the root node
    pub(crate) bounds: Bounds,
}

impl<'a, T> Voxel<'a, T> {
    pub fn is_root(&self) -> bool {
        self.index_path.is_empty() // Voxel is root if and only if index path is empty
    }
    pub fn get_value(&self) -> &T{
        if self.is_root() {
            todo!();
        } else {
            &self.node.data[self.index_path.get()]
        }
    }
    pub fn is_leaf(&self) -> bool {
        if self.is_root() {
            self.node.children.iter().all(|c| c.is_none())
        } else {
            self.node.children[self.index_path.get()].is_none()
        }
    }
    pub fn is_subdivided(&self) -> bool {
        !self.is_leaf()
    }
    pub fn get_bounds(&self) -> &Bounds {
        &self.bounds
    }
    pub fn get_index_path(&self) -> IndexPath {
        self.index_path
    }
    pub fn get_child(&self, dir: Direction) -> Voxel<'a, T> {
        if self.is_root() {
            Voxel {
                node: self.node,
                index_path: self.index_path.put(dir),
                bounds: self.bounds.half(dir),
            }
        } else if let Some(node) = self.node.children[self.index_path.get()].as_ref() {
            Voxel {
                node,
                index_path: self.index_path.put(dir),
                bounds: self.bounds.half(dir),
            }
        } else {
            Voxel {
                node: self.node,
                index_path: self.index_path,
                bounds: self.bounds.clone(),
            }
        }
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for Voxel<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.get_value())
    }
}
