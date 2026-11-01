# VLOX CLIENT

VLOX CLIENT is an independent Rust implementation project targeting the Minecraft Java Edition experience. It uses its own engine and code; it does not contain Mojang's proprietary source or bundled Minecraft assets.

## Current implementation

The repository currently contains an early playable voxel-engine prototype with:

- Rust application and Macroquad 3D renderer.
- Seeded procedural terrain with rolling height variation, grass/dirt/stone layers, sandy regions, generated oak trees, underground cave tunnels, and coal/iron ore deposits.
- 16 × 16 × 64 block chunks with world-coordinate access across chunk boundaries.
- First-person movement, gravity, jumping, solid-block collision, and sprinting.
- Survival health and food meters, fall damage, food-based regeneration, and starvation damage.
- Voxel raycasting with a five-block reach.
- Hold-to-mine behavior with different mining times for grass, dirt, sand, and stone.
- Block placement adjacent to the targeted block, with player-overlap prevention.
- Nine-slot survival hotbar, stack sizes up to 64, mined-block pickup, and item consumption when placing blocks.
- Versioned binary world saving and loading.
- Automated Rust tests for world data, generation, physics, raycasting, inventory, interactions, and persistence.

This is still an early foundation, **not a complete Minecraft Java Edition replacement**. It does not yet implement the full Java Edition feature set, assets, world format, crafting, mobs, dimensions, redstone, or multiplayer/server protocol. Compatibility is not claimed until those systems are independently implemented and tested.

## Build locally

Install the stable Rust toolchain, then run:

```sh
cargo run
```

Release build:

```sh
cargo build --release
```

## Controls

| Input | Action |
|---|---|
| W A S D | Move |
| Left Control | Sprint |
| Space | Jump |
| Hold right mouse button | Look around |
| Hold left mouse button | Mine targeted block |
| F | Place selected block in the adjacent cell |
| 1–9 | Select hotbar slot |
| F5 | Save world and inventory (`vlox-world.dat`, `vlox-inventory.dat`) |
| F9 | Load world and inventory from their save files |
| Escape | Pause / resume |

Mined blocks are added to the hotbar inventory. Placing a block consumes one item from the selected stack.

## Windows executable and tests

Every push to `main` runs the Windows x64 workflow. It runs `cargo test` before compiling and publishing the release executable. The workflow publishes a direct `.exe` release asset and also uploads an Actions artifact named `VLOX-CLIENT-Windows-x64-EXE`.

## Development roadmap

1. Improve chunk rendering with cached face/mesh generation and reduce per-frame world traversal.
2. Expand terrain generation with more biomes, caves, water, ores, structures, and deterministic world features.
3. Add complete inventory screens, crafting, tools, durability, health, hunger, and survival/creative modes.
4. Implement entity simulation, passive/hostile mobs, combat, and AI.
5. Add dimensions, redstone, lighting, weather, and time-of-day systems.
6. Implement resource-pack handling, audio, UI, and save compatibility.
7. Develop Java Edition networking and protocol compatibility in explicitly tested version stages.

## Project principles

- Reimplement behavior independently; do not copy proprietary game source.
- Do not redistribute copyrighted game assets. Users must provide assets they are legally entitled to use where required.
- Keep performance and low-end hardware support in mind.
- Mark compatibility honestly and test each supported version rather than claiming broad compatibility prematurely.
