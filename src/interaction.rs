use crate::raycast::RayHit;
use crate::world::{Block, World};

/// Remove the block selected by a valid world ray hit.
pub fn mine_block(world: &mut World, hit: RayHit) -> Option<Block> {
    let [x, y, z] = hit.block_position;
    let previous = world.get_block(x, y, z);
    if previous == Block::Air { return None; }
    world.set_block(x, y, z, Block::Air).then_some(previous)
}

/// Place a block in the cell adjacent to a ray hit, unless it intersects the player.
pub fn place_block(
    world: &mut World,
    hit: RayHit,
    block: Block,
    player_feet: [f32; 3],
) -> bool {
    if block == Block::Air { return false; }
    let [x, y, z] = hit.adjacent_position;
    if world.get_block(x, y, z) != Block::Air { return false; }

    let overlaps_player = (x as f32) + 1.0 > player_feet[0] - 0.3
        && (x as f32) < player_feet[0] + 0.3
        && (y as f32) + 1.0 > player_feet[1]
        && (y as f32) < player_feet[1] + 1.8
        && (z as f32) + 1.0 > player_feet[2] - 0.3
        && (z as f32) < player_feet[2] + 0.3;

    !overlaps_player && world.set_block(x, y, z, block)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface_hit() -> RayHit {
        RayHit {
            block_position: [4, 3, 4],
            adjacent_position: [4, 4, 4],
            distance: 2.0,
        }
    }

    #[test]
    fn mining_removes_target_and_returns_dropped_block_type() {
        let mut world = World::new_flat_test_world();
        assert_eq!(mine_block(&mut world, surface_hit()), Some(Block::Grass));
        assert_eq!(world.get_block(4, 3, 4), Block::Air);
    }

    #[test]
    fn placement_adds_block_to_adjacent_cell() {
        let mut world = World::new_flat_test_world();
        assert!(place_block(&mut world, surface_hit(), Block::Stone, [8.0, 4.0, 8.0]));
        assert_eq!(world.get_block(4, 4, 4), Block::Stone);
    }

    #[test]
    fn placement_cannot_put_block_inside_player() {
        let mut world = World::new_flat_test_world();
        assert!(!place_block(&mut world, surface_hit(), Block::Stone, [4.5, 4.0, 4.5]));
        assert_eq!(world.get_block(4, 4, 4), Block::Air);
    }

    #[test]
    fn air_cannot_be_placed() {
        let mut world = World::new_flat_test_world();
        assert!(!place_block(&mut world, surface_hit(), Block::Air, [8.0, 4.0, 8.0]));
    }
}
