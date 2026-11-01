# VLOX CLIENT

VLOX CLIENT is a custom Minecraft Java client experience with two parts:

- **VLOX Desktop** — a native desktop companion with a launcher-style interface.
- **VLOX In-game** — a Fabric client mod with the VLOX menu and client-side HUD options.

This is an independent client-side add-on, not a replacement Minecraft game engine. It does not bundle Mojang assets.

## Target version

**Minecraft Java 26.2**, Fabric Loader 0.19.3, Fabric API 0.161.0+26.2, and Java 25.

The in-game menu opens with **Right Shift**. The current foundation includes Home, HUD, Modules and Settings sections, plus FPS and coordinates HUD toggles.

## Current desktop foundation

The desktop app can open the installed Minecraft Launcher and copy the locally built VLOX mod JAR into the default Windows `.minecraft/mods` directory. It does not yet perform Microsoft authentication, install the Minecraft 26.2 profile automatically, or launch the game directly.

## Build

Desktop (stable Rust):
```sh
cargo run
cargo build --release
```

Fabric mod (JDK 25 and Gradle 9.5.1):
```sh
cd client-mod
gradle build
```

The mod JAR is created in `client-mod/build/libs`.

## Install

1. Install Fabric Loader for Minecraft Java **26.2**.
2. Install Fabric API **0.161.0+26.2**.
3. Build the desktop app and Fabric mod, or download the matching GitHub Actions artifacts.
4. In VLOX Desktop, open **Mod Library** and install the VLOX mod.
5. Start the Fabric 26.2 profile from the official Minecraft Launcher.

The Fabric mod is version-locked to 26.2 and will not load on 1.21.x or other Minecraft versions.
