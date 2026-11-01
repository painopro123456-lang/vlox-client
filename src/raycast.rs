use crate::world::{Block, World};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub block_position: [i32; 3],
    pub adjacent_position: [i32; 3],
    pub distance: f32,
}

/// Amanatides-Woo voxel traversal. Direction is normalized internally.
pub fn cast_block_ray(world: &World, origin: [f32; 3], direction: [f32; 3], max_distance: f32) -> Option<RayHit> {
    let length = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
    if length <= f32::EPSILON || max_distance <= 0.0 { return None; }
    let dir = [direction[0] / length, direction[1] / length, direction[2] / length];
    let mut cell = [origin[0].floor() as i32, origin[1].floor() as i32, origin[2].floor() as i32];
    let step = [dir[0].signum() as i32, dir[1].signum() as i32, dir[2].signum() as i32];
    let mut t_max = [0.0; 3];
    let mut t_delta = [f32::INFINITY; 3];

    for axis in 0..3 {
        if dir[axis] > 0.0 {
            t_max[axis] = ((cell[axis] + 1) as f32 - origin[axis]) / dir[axis];
            t_delta[axis] = 1.0 / dir[axis];
        } else if dir[axis] < 0.0 {
            t_max[axis] = (origin[axis] - cell[axis] as f32) / -dir[axis];
            t_delta[axis] = 1.0 / -dir[axis];
        }
    }

    let mut previous = cell;
    let mut distance = 0.0;
    loop {
        if world.get_block(cell[0], cell[1], cell[2]) != Block::Air {
            return Some(RayHit { block_position: cell, adjacent_position: previous, distance });
        }
        let axis = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] { 0 }
            else if t_max[1] <= t_max[2] { 1 } else { 2 };
        distance = t_max[axis];
        if distance > max_distance { return None; }
        previous = cell;
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_finds_first_solid_block_and_adjacent_cell() {
        let world = World::new_flat_test_world();
        let hit = cast_block_ray(&world, [4.5, 6.0, 4.5], [0.0, -1.0, 0.0], 6.0).unwrap();
        assert_eq!(hit.block_position, [4, 3, 4]);
        assert_eq!(hit.adjacent_position, [4, 4, 4]);
    }

    #[test]
    fn ray_respects_reach_distance() {
        let world = World::new_flat_test_world();
        assert!(cast_block_ray(&world, [4.5, 10.0, 4.5], [0.0, -1.0, 0.0], 1.0).is_none());
    }

    #[test]
    fn zero_direction_does_not_hit() {
        let world = World::new_flat_test_world();
        assert!(cast_block_ray(&world, [4.5, 6.0, 4.5], [0.0; 3], 5.0).is_none());
    }
}
