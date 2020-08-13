use std::fmt::Write;
use std::num::NonZeroU64;
use super::direction::Direction;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct IndexPath(NonZeroU64);

impl IndexPath {
    const MAX_SIZE: u8 = 21;

    pub fn new() -> Self {
        unsafe {
            Self::from(NonZeroU64::new_unchecked(1))
        }
    }

    pub fn is_empty(&self) -> bool {
        Into::<u64>::into(*self) == 1
    }
    pub fn is_full(&self) -> bool {
        // Check highest bit
        (Into::<u64>::into(*self) >> 63) == 1
    }
    pub fn peek(&self) -> Direction {
        assert!(!self.is_empty());
        (Into::<u64>::into(*self) as u8 & 0b111).into()
    }
    pub fn pop(&self) -> Self {
        assert!(!self.is_empty());
        unsafe {
            let num = Into::<u64>::into(*self) >> 3;
            Self::from(NonZeroU64::new_unchecked(num))
        }
    }
    pub fn push(&self, octant: Direction) -> Self {
        assert!(!self.is_full(), "The index path is full");
        unsafe {
            let num = (Into::<u64>::into(*self) << 3) | (octant as u64);
            Self::from(NonZeroU64::new_unchecked(num))
        }
    }
    pub fn count(&self) -> u8 {
        Self::MAX_SIZE - (Into::<u64>::into(*self).leading_zeros() / 3) as u8
    }
    pub fn put(&self, octant: Direction) -> Self {
        assert!(!self.is_full(), "The index path is full");
        let mut val = Into::<u64>::into(*self);
        let num_bits = 64 - val.leading_zeros() - 1;
        val &= !(0b111 << num_bits); // clear those bits
        val |= (octant as u64 | 0b1000) << num_bits; // Set back those bits
        unsafe {
            Self::from(NonZeroU64::new_unchecked(val))
        }
    }
    pub fn get(&self) -> Direction {
        assert!(!self.is_empty());
        let val = Into::<u64>::into(*self);
        let num_bits = 64 - val.leading_zeros() - 4;
        let dir_bin: u8 = (val >> num_bits) as u8 & 0b111_u8;
        dir_bin.into()
    }
    pub fn del(&self) -> Self {
        assert!(!self.is_empty());
        let val = Into::<u64>::into(*self);
        let num_bits = 64 - val.leading_zeros() - 1;
        let dir_bin: u64 = Into::<u64>::into(*self) & !(std::u64::MAX << num_bits);
        let dir_bin = dir_bin | (1 << num_bits);
        unsafe {
            Self::from(NonZeroU64::new_unchecked(dir_bin))
        }
    }
    pub fn replace(&self, octant: Direction) -> Self {
        unsafe {
            Self::from(NonZeroU64::new_unchecked((Into::<u64>::into(*self) & !0b111) | (octant as u64)))
        }
    }
    pub fn len(&self) -> u8 {
        let num_empty_slots = Into::<u64>::into(*self).leading_zeros() as u8 / 3;
        Self::MAX_SIZE - num_empty_slots
    }
}

impl From<NonZeroU64> for IndexPath {
    fn from(val: NonZeroU64) -> Self { Self(val) }
}
impl From<IndexPath> for NonZeroU64 {
    fn from(index_path: IndexPath) -> NonZeroU64 { index_path.0 }
}
impl From<IndexPath> for u64 {
    fn from(index_path: IndexPath) -> u64 { index_path.0.get() }
}

impl Iterator for IndexPath {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let dir = self.peek();
            self.0 = self.pop().0;
            Some(dir)
        }
    }
}

impl std::fmt::Debug for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut current = self.clone();
        f.write_str("(Root)")?;
        while !current.is_empty() {
            f.write_char('/')?;
            f.write_char((current.peek() as u8 + '0' as u8).into())?;
            current = current.pop();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_index_path() {
        assert_eq!(size_of::<IndexPath>(), size_of::<u64>());
        assert_eq!(size_of::<Option<IndexPath>>(), size_of::<u64>());

        let mut path = IndexPath::new();
        for i in 0..IndexPath::MAX_SIZE {
            assert_eq!(path.len(), i);
            path = path.push(Direction::FrontLeftBottom);
        }
        assert_eq!(path.len(), IndexPath::MAX_SIZE);
    }

    #[test]
    fn test_iterator() {
        let mut index_path = IndexPath::new();
        for i in 0..7 {
            index_path = index_path.push(i.into());
        }
        for i in (0..7).rev() {
            let dir: Direction = i.into();
            assert_eq!(index_path.next(), Some(dir));
        }

        let mut index_path = IndexPath::new();
        for i in 0..7 {
            index_path = index_path.put(i.into());
        }
        for i in 0..7 {
            let dir: Direction = i.into();
            assert_eq!(index_path.next(), Some(dir));
        }

        assert_eq!(index_path.next(), None);
    }
}
