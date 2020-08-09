use crate::direction::{DirectionMapper, Direction};
use crate::index_path::{TopDownIndexPath, IndexPath};

pub struct Node<T> {
    // A pointer pointing towards 8 child nodes
    pub(crate) children: Box<DirectionMapper<Option<Node<T>>>>,
    pub(crate) data: DirectionMapper<T>,
}

impl<T> Node<T> {
    /// Get the data on the specified index path. If
    pub fn get(&self, index_path: TopDownIndexPath) -> &T {
        let dir = index_path.peek();
        let index_path = index_path.pop();
        if index_path.is_empty() {
            return &self.data[dir];
        } else if let Some(child) = &self.children[dir] {
            return child.get(index_path);
        } else {
            // Trying to access a child while the node is already a leaf node.
            return &self.data[dir];
        }
    }
}

impl<T: Copy + PartialEq> Node<T> {
    pub fn new_all(item: T) -> Node<T> {
        Node {
            children: Box::new(DirectionMapper::new([None; 8])),
            data: DirectionMapper::new([item; 8])
        }
    }
    /// Set location on the index path to data.
    /// If the index path goes deeper than the tree does, new subnodes will be created as needed.
    pub fn set(&mut self, index_path: TopDownIndexPath, data: T) {
        let dir = index_path.peek();
        let index_path = index_path.pop();
        if index_path.is_empty() {
            self.data[dir] = data;
            return;
        } else if let Some(child) = &mut self.children[dir] {
            child.set(index_path, data);
        } else {
            // Trying to access a child while the node is already a leaf node.
            let mut child = Node::<T>::new_all(self.data[dir]);
            child.set(index_path, data);
            self.children[dir] = Some(child);
        }

        let child = self.children[dir].as_ref().unwrap();
        if child.data.data.windows(2).all(|w| w[0] == w[1]) {
            // Merge child cell
            self.data[dir] = child.data.data[0];
            self.children[dir] = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use crate::direction::Direction;
    use crate::index_path::{TopDownIndexPath, IndexPath};

    #[test]
    fn test_sizes() {
        assert_eq!(std::mem::size_of::<Node::<u16>>(), 24);
        assert_eq!(std::mem::size_of::<Option<Node::<u16>>>(), 24);
    }

    #[test]
    fn test_get_set() {
        let mut node: Node<u16> = Node::new_all(0);
        for dir in &Direction::map(|d| d).data {
            assert_eq!(*node.get(TopDownIndexPath::new().push(*dir)), 0);
        }
        for (index, dir) in Direction::map(|d| d).data.iter().enumerate() {
            let index_path = TopDownIndexPath::new().push(*dir).push(Direction::RearLeftTop);
            assert_eq!(*node.get(index_path), 0);
            node.set(index_path, 1);

            if index == 7 {
                assert!(node.children[Direction::RearLeftTop].is_none());
            } else {
                assert!(node.children[Direction::RearLeftTop].is_some());
            }
        }
    }
}
