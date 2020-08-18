use crate::chunk::Chunk;
use crate::index_path::IndexPath;
use crate::node::Node;
use crate::direction::{Direction, DirectionMapper};
use crate::bounds::Bounds;
use std::alloc::{alloc, dealloc, Layout};
use std::ops::{Index, IndexMut};

// Because this is a n x n x n array where n is 2^lod,
// We specify that there's 2^(3*lod) elements in the array.
// So the array can be indexed by a binary number with 3*lod digits.
pub struct Grid<T> {
    data: *mut T,
    lod: u8,
}

impl<T: Clone + std::fmt::Display> Grid<T> {
    pub fn new(chunk: &Chunk<T>, lod: u8) -> Grid<T> {
        assert!(lod > 0);
        let (layout, padding) = Layout::new::<T>().repeat(1 << (lod * 3)).unwrap();
        let mut grid = Self {
            data: unsafe { alloc(layout) as *mut T },
            lod,
        };
        grid.build_chunk_recursive(&chunk.root, lod, (0, 0, 0));
        grid
    }

    fn build_chunk_recursive(&mut self, node: &Node<T>, lod: u8, location: (usize, usize, usize)) {
        if lod == 1 { // base case
            // Copy data into the grid
            for (dir, data) in node.data.enumerate() {
                let offset = dir.breakdown();
                let coords = (location.0 + offset.0 as usize, location.1 + offset.1 as usize, location.2 + offset.2 as usize);
                self[coords] = data.clone();
            }
            return;
        }
        // Inductive steps.
        let new_lod = lod - 1;
        let size: usize = 1 << new_lod;
        for (dir, child) in node.children.enumerate() {

            let mut newlocation = location;
            if dir.is_max_x() {
                newlocation.0 += size;
            }
            if dir.is_max_y() {
                newlocation.1 += size;
            }
            if dir.is_max_z() {
                newlocation.2 += size;
            }

            if let Some(child) = child {
                self.build_chunk_recursive(child, new_lod, newlocation);
            } else {
                // Fill area
                let fakedata = &node.data[dir];

                for i in 0 .. size {
                    for j in 0 .. size {
                        for k in 0 .. size {
                            let mut newlocation = newlocation;
                            newlocation.0 += i;
                            newlocation.1 += j;
                            newlocation.2 += k;
                            self[newlocation] = fakedata.clone();
                        }
                    }
                }
            }
        }
    }
}

impl<T> Drop for Grid<T> {
    fn drop(&mut self) {
        let size: usize = 1 << self.lod;
        let (layout, _) = Layout::new::<T>().repeat(size * size * size).unwrap();
        unsafe { dealloc(self.data as *mut u8, layout) }
    }
}

impl<T> Index<(usize, usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        debug_assert!(index.0 < (1 << self.lod));
        debug_assert!(index.1 < (1 << self.lod));
        debug_assert!(index.2 < (1 << self.lod));
        unsafe {
            &*self.data.offset((index.2 | (index.1 << self.lod) | (index.0 << (2 * self.lod))) as isize)
        }
    }
}
impl<T> IndexMut<(usize, usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        let offset = (index.2 | (index.1 << self.lod) | (index.0 << (2 * self.lod))) as isize;
        unsafe {
            &mut *self.data.offset(offset)
        }
    }
}

pub struct GridIterator<'a, T> {
    grid: &'a Grid<T>,
    location: usize,
}

impl<'a, T> Iterator for GridIterator<'a, T> {
    type Item = ((usize, usize, usize), &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        let lod = self.grid.lod;
        let size: usize = 1 << lod;
        let capacity = 1 << (lod * 3);
        if self.location >= capacity {
            None
        } else {
            let item = unsafe { &*self.grid.data.offset(self.location as isize) };

            let mask = (1 << lod) - 1;
            let z = self.location & mask;
            let y = (self.location >> lod) & mask;
            let x = self.location >> (lod * 2);

            self.location += 1;
            Some(((x, y, z),item))
        }
    }
}

pub struct GridGroupedIterator<'a, T> {
    grid: &'a Grid<T>,
    location: usize,
}

impl<'a, T> Iterator for GridGroupedIterator<'a, T> {
    type Item = ((usize, usize, usize), DirectionMapper<&'a T>);
    fn next(&mut self) -> Option<Self::Item> {
        let lod = self.grid.lod;
        let size: usize = 1 << lod;
        let capacity = 1 << (lod * 3);
        if self.location >= capacity {
            None
        } else {
            let mask = (1 << lod) - 1;
            let z = self.location & mask;
            let y = (self.location >> lod) & mask;
            let x = self.location >> (lod * 2);
            self.location += 1;

            if z + 1 >= size || y + 1 >= size || x + 1 >= size {
                return self.next();
            }

            let mapper = DirectionMapper::from_mapper(|dir| {
                let offset = dir.breakdown();
                let new_location: (usize, usize, usize) = (
                    x + offset.0 as usize,
                    y + offset.1 as usize,
                    z + offset.2 as usize,
                );
                &self.grid[new_location]
            });
            Some(((x, y, z), mapper))
        }
    }
}

impl<'a, T> Grid<T> {
    pub fn iter(&'a self) -> GridIterator<'a, T> {
        GridIterator {
            grid: self,
            location: 0,
        }
    }
    pub fn iter_grouped(&'a self) -> GridGroupedIterator<'a, T> {
        GridGroupedIterator {
            grid: self,
            location: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::Chunk;
    use crate::index_path::IndexPath;
    use super::Grid;
    use crate::direction::Direction;

    #[test]
    fn test_base_case() {
        let mut chunk: Chunk<u16> = Chunk::new();
        for i in 0..=7 {
            chunk.set(IndexPath::new().push(i.into()), i as u16);
        }
        let grid = Grid::new(&chunk, 1); // lod = 1 for the base case

        let mut iter = grid.iter();

        for (location, expected_value) in &[
            ((0, 0, 0), 0),
            ((0, 0, 1), 1),
            ((0, 1, 0), 2),
            ((0, 1, 1), 3),
            ((1, 0, 0), 4),
            ((1, 0, 1), 5),
            ((1, 1, 0), 6),
            ((1, 1, 1), 7)
        ] {
            let (coords, value) = iter.next().unwrap();
            assert_eq!(coords, *location);
            assert_eq!(*value, *expected_value);
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_inductive_step() {
        let mut chunk: Chunk<u16> = Chunk::new();
        for i in 0..7 {
            chunk.set(IndexPath::new().push(i.into()), i as u16);
        }
        for i in 0..8 {
            chunk.set(IndexPath::new().push(i.into()).push(Direction::RearRightTop), i as u16 + 16);
        }
        let grid = Grid::new(&chunk, 2); // lod = 1 for the base case
        let mut iter = grid.iter();

        for i in &[0, 0, 4, 4, 0, 0, 4, 4,
        2, 2, 6, 6, 2, 2, 6, 6,
        0, 0, 4, 4, 0, 0, 4, 4,
        2, 2, 6, 6, 2, 2, 6, 6,
        1, 1, 5, 5, 1, 1, 5, 5,
        3, 3, 16, 17, 3, 3, 18, 19,
        1, 1, 5, 5, 1, 1, 5, 5,
        3, 3, 20, 21, 3, 3, 22, 23] {
            assert_eq!(*iter.next().unwrap().1, *i);
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_grouped_iterator() {
        let mut chunk: Chunk<u16> = Chunk::new();
        for i in 0..=7 {
            chunk.set(IndexPath::new().push(i.into()), i as u16);
        }
        let grid = Grid::new(&chunk, 1); // lod = 1 for the base case
        let mut iter = grid.iter_grouped();

        let (location, mapper) = iter.next().unwrap();
        for (i, value) in mapper.enumerate() {
            assert_eq!(i, (**value as u8).into());
        }

        assert!(iter.next().is_none());
    }
}
