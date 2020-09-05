#![feature(const_in_array_repeat_expressions)]
#![feature(generators, generator_trait)]
#![feature(alloc_layout_extra)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_uninit_array)]

pub mod direction;
pub mod node;
pub mod index_path;
pub mod chunk;
pub mod world;
pub mod world_builder;
pub mod bounds;
pub mod voxel;
pub mod mesher;
pub mod grid;
mod iterators;

pub trait VoxelData: Clone + Default {
    fn is_empty(&self) -> bool;
}
