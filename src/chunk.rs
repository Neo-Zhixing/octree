use crate::node::Node;

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