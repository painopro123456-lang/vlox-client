use crate::world::{Block, Chunk, World, CHUNK_SIZE, WORLD_HEIGHT};

/// Generate a deterministic set of chunks around the origin.
pub fn generate_world(seed: u64, chunk_radius: i32) -> World {
    let mut world = World::default();
    for chunk_z in -chunk_radius..=chunk_radius {
        for chunk_x in -chunk_radius..=chunk_radius {
            world.insert_chunk(chunk_x, chunk_z, generate_chunk(seed, chunk_x, chunk_z));
        }
    }
    world
}

/// Generate one complete terrain chunk, including caves, ores, and oak trees.
/// This is also used by the runtime chunk streamer when the player explores.
pub fn generate_chunk(seed: u64, chunk_x: i32, chunk_z: i32) -> Chunk {
    let mut chunk = Chunk::default();

    // Terrain surface and biome layers.
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

    // Carve two deterministic cave tunnels without writing outside this chunk.
    for worm in 0..2_u64 {
        let mut rng = seed
            ^ (chunk_x as i64 as u64).wrapping_mul(0x9E3779B185EBCA87)
            ^ (chunk_z as i64 as u64).wrapping_mul(0xC2B2AE3D27D4EB4F)
            ^ worm.wrapping_mul(0xA24BAED4963EE407);
        let mut x = 3 + (next_random(&mut rng) % 10) as i32;
        let mut z = 3 + (next_random(&mut rng) % 10) as i32;
        let mut y = 7 + (next_random(&mut rng) % 15) as i32;
        let mut dir_x = 0.0_f32;
        let mut dir_z = 0.0_f32;

        for _ in 0..22 {
            let radius: i32 = if next_random(&mut rng) % 5 == 0 { 2 } else { 1 };
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx * dx + dy * dy + dz * dz > radius * radius + 1 { continue; }
                        let bx = x + dx;
                        let by = y + dy;
                        let bz = z + dz;
                        if bx < 0 || bx >= CHUNK_SIZE as i32 || bz < 0 || bz >= CHUNK_SIZE as i32 { continue; }
                        if matches!(chunk.get(bx as usize, by.max(0) as usize, bz as usize), Block::Stone | Block::Dirt)
                            && by >= 0 && by < WORLD_HEIGHT as i32 {
                            chunk.set(bx as usize, by as usize, bz as usize, Block::Air);
                        }
                    }
                }
            }
            dir_x = (dir_x + ((next_random(&mut rng) % 200) as f32 / 100.0 - 1.0) * 0.35).clamp(-1.0, 1.0);
            dir_z = (dir_z + ((next_random(&mut rng) % 200) as f32 / 100.0 - 1.0) * 0.35).clamp(-1.0, 1.0);
            x = (x + dir_x.round() as i32).clamp(2, 13);
            z = (z + dir_z.round() as i32).clamp(2, 13);
            y = (y + (next_random(&mut rng) % 3) as i32 - 1).clamp(5, 23);
        }
    }

    // Generate coal and iron deposits inside underground stone.
    for local_z in 0..CHUNK_SIZE {
        for local_x in 0..CHUNK_SIZE {
            let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
            let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
            let top = chunk.surface_height(local_x, local_z).unwrap_or(0) as i32;
            for y in 4..(top - 3).max(4) {
                if chunk.get(local_x, y as usize, local_z) != Block::Stone { continue; }
                let depth_seed = seed ^ (y as i64 as u64).wrapping_mul(0xD1B54A32D192ED03);
                let value = hash_value(depth_seed, world_x, world_z);
                if value > 0.997 {
                    chunk.set(local_x, y as usize, local_z, Block::IronOre);
                } else if value > 0.991 {
                    chunk.set(local_x, y as usize, local_z, Block::CoalOre);
                }
            }
        }
    }

    // Place oak trees well inside chunk borders so canopies remain in this chunk.
    let mut trees = Vec::new();
    for local_z in 2..14 {
        for local_x in 2..14 {
            if let Some(surface) = chunk.surface_height(local_x, local_z) {
                let world_x = chunk_x * CHUNK_SIZE as i32 + local_x as i32;
                let world_z = chunk_z * CHUNK_SIZE as i32 + local_z as i32;
                if chunk.get(local_x, surface, local_z) == Block::Grass
                    && hash_value(seed ^ 0xA24BAED4963EE407, world_x, world_z) > 0.985 {
                    trees.push((local_x, surface + 1, local_z));
                }
            }
        }
    }

    for (x, base_y, z) in trees {
        if base_y + 5 >= WORLD_HEIGHT { continue; }
        let mut clear = true;
        for y in base_y..base_y + 6 {
            if chunk.get(x, y, z) != Block::Air { clear = false; }
        }
        if !clear { continue; }
        for y in 0..4 {
            chunk.set(x, base_y + y, z, Block::OakLog);
        }
        for dy in 2..=4 {
            let radius: i32 = if dy == 4 { 1 } else { 2 };
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    if dx.abs() == radius && dz.abs() == radius && dy != 4 { continue; }
                    let lx = (x as i32 + dx) as usize;
                    let lz = (z as i32 + dz) as usize;
                    let ly = base_y + dy;
                    if chunk.get(lx, ly, lz) == Block::Air {
                        chunk.set(lx, ly, lz, Block::OakLeaves);
                    }
                }
            }
        }
    }

    chunk
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
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
    fn distant_chunks_generate_deterministically() {
        let a = generate_chunk(54321, -7, 11);
        let b = generate_chunk(54321, -7, 11);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..WORLD_HEIGHT {
                    assert_eq!(a.get(x, y, z), b.get(x, y, z));
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
    fn generated_world_contains_underground_caves_and_ores() {
        let world = generate_world(0x564C4F585F323032, 1);
        let mut cave = false;
        let mut coal = false;
        let mut iron = false;
        for x in -16..32 {
            for z in -16..32 {
                let top = world.surface_height(x, z).unwrap_or(0) as i32;
                for y in 5..top.saturating_sub(3) {
                    match world.get_block(x, y, z) {
                        Block::Air => cave = true,
                        Block::CoalOre => coal = true,
                        Block::IronOre => iron = true,
                        _ => {}
                    }
                }
            }
        }
        assert!(cave);
        assert!(coal);
        assert!(iron);
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
