use glam as math;
use super::index_path::IndexPath;
use super::direction::Direction;
use std::convert::TryInto;

#[derive(Clone)]
pub struct Bounds {
    x: u32,
    y: u32,
    z: u32,
    width: u32,
}

#[derive(Debug)]
pub enum BoundsSpacialRelationship {
    Disjoint,
    Contain,
    Intersect,
}

impl Bounds {
    const MAX_WIDTH: u32 = (1 << 31);
    pub fn new() -> Self {
        Bounds {
            x: 0,
            y: 0,
            z: 0,
            width: Self::MAX_WIDTH,
        }
    }
    pub fn from_discrete_grid(location: (u64, u64, u64), width: u64, gridsize: u64) -> Self {
        Bounds {
            x: (location.0 * Self::MAX_WIDTH as u64 / gridsize).try_into().unwrap(),
            y: (location.1 * Self::MAX_WIDTH as u64 / gridsize).try_into().unwrap(),
            z: (location.2 * Self::MAX_WIDTH as u64 / gridsize).try_into().unwrap(),
            width: (width * Self::MAX_WIDTH as u64 / gridsize).try_into().unwrap(),
        }
    }
    pub fn get_position_with_gridsize(&self, gridsize: u64) -> (u64, u64, u64) {
        (
            (self.x as u64 * gridsize / Self::MAX_WIDTH as u64),
            (self.y as u64 * gridsize / Self::MAX_WIDTH as u64),
            (self.z as u64 * gridsize / Self::MAX_WIDTH as u64),
        )
    }
    pub fn get_width_with_gridsize(&self, gridsize: u64) -> u64 {
        self.width as u64 * gridsize / Self::MAX_WIDTH as u64
    }
    pub fn get_position(&self) -> math::Vec3A {
        math::Vec3A::new(
            self.x as f32,
            self.y as f32,
            self.z as f32,
        ) / (Self::MAX_WIDTH as f32)
    }
    pub fn get_width(&self) -> f32 {
        self.width as f32 / Self::MAX_WIDTH as f32
    }
    pub fn center(&self) -> math::Vec3A {
        let half_width = self.get_width() / 2.0;
        self.get_position() + math::Vec3A::splat(half_width)
    }

    pub fn half(&self, dir: Direction) -> Bounds {
        let mut bounds = self.clone();
        bounds.width >>= 1; // half the width
        if dir.is_max_x() {
            bounds.x += bounds.width;
        }
        if dir.is_max_y() {
            bounds.y += bounds.width;
        }
        if dir.is_max_z() {
            bounds.z += bounds.width;
        }
        bounds
    }

    pub fn merge(&self, dir: Direction) -> Bounds {
        let mut bounds = self.clone();
        if dir.is_max_x() {
            bounds.x -= bounds.width;
        }
        if dir.is_max_y() {
            bounds.y -= bounds.width;
        }
        if dir.is_max_z() {
            bounds.z -= bounds.width;
        }
        bounds.width <<= 1;
        bounds
    }

    pub fn intersects(&self, other: &Self) -> BoundsSpacialRelationship {
        // Check for disjoint
        if (self.x + self.width <= other.x || other.x + other.width <= self.x) ||
            (self.y + self.width <= other.y || other.y + other.width <= self.y) ||
            (self.z + self.width <= other.z || other.z + other.width <= self.z) {
            return BoundsSpacialRelationship::Disjoint;
        }

        // Other is smaller
        if (other.x >= self.x && other.x + other.width <= self.x + self.width) &&
            (other.y >= self.y && other.y + other.width <= self.y + self.width) &&
            (other.z >= self.z && other.z + other.width <= self.z + self.width) {
            return BoundsSpacialRelationship::Contain;
        }
        return BoundsSpacialRelationship::Intersect;
    }
}


impl From<IndexPath> for Bounds {
    fn from(index_path: IndexPath) -> Self {
        let mut bounds = Bounds::new();
        for dir in index_path {
            bounds = bounds.half(dir);
        }
        bounds
    }
}

impl std::fmt::Debug for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let pos = self.get_position_with_gridsize(256);
        let width = self.get_width_with_gridsize(256);
        write!(f, "Bounds({}, {}, {})[{}]", pos.0, pos.1, pos.2, width)
    }
}
