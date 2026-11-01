# VLOX CLIENT

VLOX CLIENT is an independent voxel-game/client project written in Rust. The long-term goal is to build a polished, performant client with its own engine and a compatibility roadmap. This repository does **not** contain Mojang source code or bundled Minecraft assets.

## Current status

The first commit establishes a small, playable 3D voxel prototype:
- Rust application using Macroquad
- Procedurally generated block test area
- First-person free-fly movement and mouse look
- Pause overlay and FPS counter
- Windows x64 release build in GitHub Actions

This is an early engine prototype, **not yet a Minecraft Java Edition replacement**. Java Edition compatibility involves a large amount of independently implemented game behavior, asset handling, rendering, networking, authentication, and version-specific protocol work. Those systems will be developed and tested in stages.

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
| Space | Fly upward |
| Left Shift | Fly downward |
| Hold right mouse button | Look around |
| Escape | Pause / resume |

## Windows executable

Every push to `main` and every manual workflow dispatch runs **Actions → Windows Build**. The workflow builds the x64 release executable and uploads it as the artifact **VLOX-CLIENT-Windows-x64-EXE**.

## Roadmap

1. Stabilize the Rust engine foundation and automated Windows builds.
2. Add chunk storage, block definitions, meshing, and terrain generation.
3. Add collision, physics, inventory, block interaction, and world saving.
4. Build rendering, UI, resource-pack, and audio systems.
5. Develop multiplayer protocol and compatibility in clearly versioned stages.
6. Add account/session integration and mod/plugin APIs only after the core systems are reliable.

## Project principles

- Implement independently; do not copy proprietary game source.
- Do not redistribute copyrighted game assets. Users must provide assets they are legally entitled to use where required.
- Keep performance and low-end hardware support in mind.
- Mark compatibility honestly and test each supported version rather than claiming broad compatibility prematurely.
