//! Core voxel data model. Kept independent of rendering so gameplay and
//! networking can share the same world representation.

use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
}

impl Block {
    pub const fn is_solid(self) -> bool {
        !matches!(self, Self::Air)
    }
}

#[derive(Clone)]
pub struct Chunk {
    blocks: Vec<Block>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self { blocks: vec![Block::Air; CHUNK_SIZE * WORLD_HEIGHT * CHUNK_SIZE] }
    }
}

impl Chunk {
    fn index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= CHUNK_SIZE || y >= WORLD_HEIGHT || z >= CHUNK_SIZE {
            return None;
        }
        Some((y * CHUNK_SIZE + z) * CHUNK_SIZE + x)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        Self::index(x, y, z).map_or(Block::Air, |i| self.blocks[i])
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) -> bool {
        if let Some(i) = Self::index(x, y, z) {
            self.blocks[i] = block;
            true
        } else {
            false
        }
    }

    pub fn surface_height(&self, x: usize, z: usize) -> Option<usize> {
        (0..WORLD_HEIGHT).rev().find(|&y| self.get(x, y, z).is_solid())
    }
}

#[derive(Default)]
pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new_flat_test_world() -> Self {
        let mut world = Self::default();
        let mut chunk = Chunk::default();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..=3 {
                    let block = match y {
                        3 => Block::Grass,
                        1..=2 => Block::Dirt,
                        _ => Block::Stone,
                    };
                    chunk.set(x, y, z, block);
                }
            }
        }

        world.chunks.insert((0, 0), chunk);
        world
    }

    pub fn chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    pub fn chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> bool {
        if y < 0 || y >= WORLD_HEIGHT as i32 {
            return false;
        }

        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = z.div_euclid(CHUNK_SIZE as i32);
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.chunks.entry((chunk_x, chunk_z)).or_default()
            .set(local_x, y as usize, local_z, block)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        if y < 0 || y >= WORLD_HEIGHT as i32 {
            return Block::Air;
        }

        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = z.div_euclid(CHUNK_SIZE as i32);
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.chunk(chunk_x, chunk_z)
            .map_or(Block::Air, |chunk| chunk.get(local_x, y as usize, local_z))
    }

    pub fn surface_height(&self, x: i32, z: i32) -> Option<usize> {
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = z.div_euclid(CHUNK_SIZE as i32);
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;
        self.chunk(chunk_x, chunk_z)?.surface_height(local_x, local_z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_world_has_grass_surface() {
        let world = World::new_flat_test_world();
        assert_eq!(world.get_block(0, 3, 0), Block::Grass);
        assert_eq!(world.get_block(0, 2, 0), Block::Dirt);
        assert_eq!(world.get_block(0, 0, 0), Block::Stone);
    }

    #[test]
    fn block_coordinates_cross_chunk_boundaries() {
        let mut world = World::default();
        assert!(world.set_block(16, 5, -1, Block::Stone));
        assert_eq!(world.get_block(16, 5, -1), Block::Stone);
        assert_eq!(world.get_block(15, 5, -1), Block::Air);
    }

    #[test]
    fn out_of_range_y_is_rejected() {
        let mut world = World::default();
        assert!(!world.set_block(0, -1, 0, Block::Stone));
        assert!(!world.set_block(0, WORLD_HEIGHT as i32, 0, Block::Stone));
    }
}
