pub mod generation;
pub mod interaction;
pub mod inventory;
pub mod physics;
pub mod raycast;
pub mod survival;
pub mod time;
pub mod world;

pub use generation::{generate_chunk, generate_world};
pub use interaction::{mine_block, place_block};
pub use inventory::{Inventory, ItemStack, HOTBAR_SLOTS, MAX_STACK};
pub use physics::PhysicsBody;
pub use raycast::{cast_block_ray, RayHit};
pub use survival::Vitals;
pub use time::{WorldClock, DAY_LENGTH_SECONDS};
pub use world::{Block, Chunk, World, CHUNK_SIZE, WORLD_HEIGHT};
