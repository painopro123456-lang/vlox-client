use crate::world::{Block, World};

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const GRAVITY: f32 = 24.0;
const JUMP_SPEED: f32 = 8.4;
const WALK_SPEED: f32 = 4.3;
const SPRINT_SPEED: f32 = 5.6;

/// A minimal survival-style player body. Position is the centre of the feet.
#[derive(Clone, Copy, Debug)]
pub struct PhysicsBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub grounded: bool,
}

impl PhysicsBody {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position, velocity: [0.0; 3], grounded: false }
    }

    pub fn tick(&mut self, world: &World, input: [f32; 3], jump: bool, dt: f32) {
        self.tick_with_sprint(world, input, jump, false, dt);
    }

    pub fn tick_with_sprint(&mut self, world: &World, input: [f32; 3], jump: bool, sprint: bool, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        let mut ix = input[0];
        let mut iz = input[2];
        let length = (ix * ix + iz * iz).sqrt();
        if length > 1.0 { ix /= length; iz /= length; }

        let speed = if sprint { SPRINT_SPEED } else { WALK_SPEED };
        self.velocity[0] = ix * speed;
        self.velocity[2] = iz * speed;
        if jump && self.grounded {
            self.velocity[1] = JUMP_SPEED;
            self.grounded = false;
        }
        self.velocity[1] -= GRAVITY * dt;

        self.move_axis(world, 0, self.velocity[0] * dt);
        self.move_axis(world, 2, self.velocity[2] * dt);
        self.move_axis(world, 1, self.velocity[1] * dt);
    }

    fn move_axis(&mut self, world: &World, axis: usize, amount: f32) {
        if amount == 0.0 { return; }
        self.position[axis] += amount;
        if self.collides(world) {
            self.position[axis] -= amount;
            if axis == 1 {
                if amount < 0.0 { self.grounded = true; }
                self.velocity[1] = 0.0;
            } else {
                self.velocity[axis] = 0.0;
            }
        } else if axis == 1 && amount < 0.0 {
            self.grounded = false;
        }
    }

    fn collides(&self, world: &World) -> bool {
        let min_x = (self.position[0] - PLAYER_WIDTH * 0.5).floor() as i32;
        let max_x = (self.position[0] + PLAYER_WIDTH * 0.5 - 0.0001).floor() as i32;
        let min_y = self.position[1].floor() as i32;
        let max_y = (self.position[1] + PLAYER_HEIGHT - 0.0001).floor() as i32;
        let min_z = (self.position[2] - PLAYER_WIDTH * 0.5).floor() as i32;
        let max_z = (self.position[2] + PLAYER_WIDTH * 0.5 - 0.0001).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.get_block(x, y, z) != Block::Air { return true; }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn gravity_lands_player_on_flat_ground() {
        let world = World::new_flat_test_world();
        let mut player = PhysicsBody::new([4.5, 8.0, 4.5]);
        for _ in 0..240 { player.tick(&world, [0.0; 3], false, 1.0 / 60.0); }
        assert!(player.grounded);
        assert!((player.position[1] - 4.0).abs() < 0.001);
    }

    #[test]
    fn player_cannot_walk_through_solid_blocks() {
        let mut world = World::new_flat_test_world();
        world.set_block(5, 4, 4, Block::Stone);
        let mut player = PhysicsBody::new([4.5, 4.0, 4.5]);
        player.grounded = true;
        for _ in 0..60 { player.tick(&world, [1.0, 0.0, 0.0], false, 1.0 / 60.0); }
        assert!(player.position[0] < 4.71);
    }

    #[test]
    fn sprint_moves_faster_than_walking() {
        let world = World::new_flat_test_world();
        let mut walker = PhysicsBody::new([4.5, 4.0, 4.5]);
        let mut sprinter = walker;
        walker.tick(&world, [1.0, 0.0, 0.0], false, 1.0 / 60.0);
        sprinter.tick_with_sprint(&world, [1.0, 0.0, 0.0], false, true, 1.0 / 60.0);
        assert!(sprinter.velocity[0] > walker.velocity[0]);
    }

    #[test]
    fn jump_only_starts_when_grounded() {
        let world = World::new_flat_test_world();
        let mut player = PhysicsBody::new([4.5, 4.0, 4.5]);
        player.grounded = true;
        player.tick(&world, [0.0; 3], true, 1.0 / 60.0);
        assert!(player.velocity[1] > 0.0);
    }
}
