use super::{Mesher, Mesh};
use crate::world::{ChunkCoordinates, World};
use crate::grid::Grid;
use crate::VoxelData;
use crate::direction::{Edge, DirectionMapper};
use glam as math;

pub struct MarchingCubesMesher<'a, T> {
    world: &'a World<T>
}

impl<'a, T: VoxelData> Mesher<'a, T> for MarchingCubesMesher<'a, T> {
    fn new(world: &'a World<T>) -> Self {
        MarchingCubesMesher {
            world
        }
    }

    fn build(&self, chunk_location: &ChunkCoordinates, lod: u8) -> Mesh {
        let chunk = self.world.get_chunk_ref(chunk_location)
            .expect(&format!("Trying to build a chunk that doesn't exist at {:?}", chunk_location));

        let mut mesh = Mesh {
            vertices: vec![],
            indices: vec![]
        };

        let mut count: u32 = 0;

        let grid = Grid::new(&chunk, lod);

        for (position, cell) in grid.iter_grouped() {
            let mut edge_index: u8 = 0;
            for node in cell.iter().rev() {
                edge_index <<= 1;
                if !node.is_empty() {
                    edge_index |= 1;
                }
            }

            let edge_bin = super::MC_TABLE[edge_index as usize];

            for edges in edge_bin.iter() {
                let edges = *edges;
                if edges == std::u16::MAX {
                    // Marks the end of array
                    break;
                }

                // Each element here represents an edge to connect.
                debug_assert_eq!(edges >> 12, 0); // Highest 4 bits are always 0
                let edge1: Edge = ((edges & 0b1111) as u8).into();
                let edge2: Edge = (((edges >> 4) & 0b1111) as u8).into();
                let edge3: Edge = ((edges >> 8) as u8).into();

                // We need to connect the midpoints of these three edges
                let edges = [edge1, edge2, edge3];
                for edge in &edges {
                    let (v1, v2) = edge.vertices();
                    let v1 = v1.breakdown();
                    let v2 = v2.breakdown();
                    let midpoint = math::Vec3::new(
                        (v1.0 + v2.0) as f32,
                        (v1.1 + v2.1) as f32,
                        (v1.2 + v2.2) as f32,
                    ) / 2.0;
                    mesh.vertices.push(midpoint);
                }
                mesh.indices.push(count);
                mesh.indices.push(count + 1);
                mesh.indices.push(count + 2);
                count += 3;
            }
        }
        mesh
    }
}
