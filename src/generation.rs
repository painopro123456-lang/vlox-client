use crate::world::{Block, Chunk, World, CHUNK_SIZE, WORLD_HEIGHT};

/// Generate a deterministic set of terrain chunks from a world seed.
pub fn generate_world(seed: u64, chunk_radius: i32) -> World {
    let mut world = World::default();
    for chunk_z in -chunk_radius..=chunk_radius {
        for chunk_x in -chunk_radius..=chunk_radius {
            let mut chunk = Chunk::default();
            for local_z in 0..CHUNK_SIZE {
                for local_x in 0..CHUNK_SIZE {
                    let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
                    let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
                    let continental = octave_noise(seed, world_x, world_z);
                    let detail = octave_noise(seed ^ 0x9E3779B97F4A7C15, world_x * 2, world_z * 2);
                    let height = (25.0 + continental * 8.0 + detail * 2.5)
                        .round()
                        .clamp(8.0, (WORLD_HEIGHT - 2) as f32) as usize;
                    let desert = octave_noise(seed ^ 0xD1B54A32D192ED03, world_x / 3, world_z / 3) > 0.28;

                    for y in 0..=height {
                        let depth = height - y;
                        let block = if depth == 0 {
                            if desert { Block::Sand } else { Block::Grass }
                        } else if desert && depth <= 4 {
                            Block::Sand
                        } else if depth <= 3 {
                            Block::Dirt
                        } else {
                            Block::Stone
                        };
                        chunk.set(local_x, y, local_z, block);
                    }
                }
            }
            world.insert_chunk(chunk_x, chunk_z, chunk);
        }
    }
    // Add oak trees on grassland. World-coordinate writes let trees cross chunk edges.
    let min_x = -chunk_radius * CHUNK_SIZE as i32 + 2;
    let max_x = (chunk_radius + 1) * CHUNK_SIZE as i32 - 2;
    let min_z = -chunk_radius * CHUNK_SIZE as i32 + 2;
    let max_z = (chunk_radius + 1) * CHUNK_SIZE as i32 - 2;
    let mut trees = Vec::new();
    for z in min_z..max_z {
        for x in min_x..max_x {
            if let Some(surface) = world.surface_height(x, z) {
                if world.get_block(x, surface as i32, z) == Block::Grass
                    && hash_value(seed ^ 0xA24BAED4963EE407, x, z) > 0.985 {
                    trees.push((x, surface as i32 + 1, z));
                }
            }
        }
    }

    for (x, base_y, z) in trees {
        let mut clear = true;
        for y in base_y..base_y + 6 {
            if world.get_block(x, y, z) != Block::Air { clear = false; }
        }
        if !clear { continue; }
        for y in 0..4 {
            world.set_block(x, base_y + y, z, Block::OakLog);
        }
        for dy in 2..=4 {
            let radius = if dy == 4 { 1 } else { 2 };
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    if dx.abs() == radius && dz.abs() == radius && dy != 4 { continue; }
                    let leaf_x = x + dx;
                    let leaf_y = base_y + dy;
                    let leaf_z = z + dz;
                    if world.get_block(leaf_x, leaf_y, leaf_z) == Block::Air {
                        world.set_block(leaf_x, leaf_y, leaf_z, Block::OakLeaves);
                    }
                }
            }
        }
    }
    world
}

fn octave_noise(seed: u64, x: i32, z: i32) -> f32 {
    let coarse = smooth_noise(seed, x, z);
    let fine = smooth_noise(seed ^ 0xA0761D6478BD642F, x * 2, z * 2);
    let micro = smooth_noise(seed ^ 0xE7037ED1A0B428DB, x * 4, z * 4);
    (coarse * 0.58 + fine * 0.29 + micro * 0.13) * 2.0 - 1.0
}

fn smooth_noise(seed: u64, x: i32, z: i32) -> f32 {
    let x0 = x.div_euclid(8);
    let z0 = z.div_euclid(8);
    let tx = smoothstep(x.rem_euclid(8) as f32 / 8.0);
    let tz = smoothstep(z.rem_euclid(8) as f32 / 8.0);
    let a = hash_value(seed, x0, z0);
    let b = hash_value(seed, x0 + 1, z0);
    let c = hash_value(seed, x0, z0 + 1);
    let d = hash_value(seed, x0 + 1, z0 + 1);
    lerp(lerp(a, b, tx), lerp(c, d, tx), tz)
}

fn hash_value(seed: u64, x: i32, z: i32) -> f32 {
    let mut value = seed
        ^ (x as i64 as u64).wrapping_mul(0x9E3779B185EBCA87)
        ^ (z as i64 as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58476D1CE4E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D049BB133111EB);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_generates_same_terrain() {
        let a = generate_world(12345, 0);
        let b = generate_world(12345, 0);
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..40 {
                    assert_eq!(a.get_block(x, y, z), b.get_block(x, y, z));
                }
            }
        }
    }

    #[test]
    fn different_seeds_can_generate_different_heights() {
        let a = generate_world(1, 0);
        let b = generate_world(2, 0);
        let mut different = false;
        for x in 0..16 {
            for z in 0..16 {
                if a.surface_height(x, z) != b.surface_height(x, z) {
                    different = true;
                    break;
                }
            }
        }
        assert!(different);
    }

    #[test]
    fn grassland_world_generates_oak_trees() {
        let world = generate_world(0x564C4F585F323032, 1);
        let mut found_log = false;
        let mut found_leaves = false;
        for x in -16..32 {
            for z in -16..32 {
                for y in 0..WORLD_HEIGHT as i32 {
                    match world.get_block(x, y, z) {
                        Block::OakLog => found_log = true,
                        Block::OakLeaves => found_leaves = true,
                        _ => {}
                    }
                }
            }
        }
        assert!(found_log);
        assert!(found_leaves);
    }

    #[test]
    fn generated_chunks_have_solid_surface_and_stone_below() {
        let world = generate_world(99, 0);
        let top = world.surface_height(8, 8).unwrap() as i32;
        assert!(world.get_block(8, top, 8).is_solid());
        assert_eq!(world.get_block(8, 0, 8), Block::Stone);
    }
}
