# VLOX CLIENT

VLOX CLIENT is a custom Minecraft Java client experience with two parts:

- **VLOX Desktop** — a native desktop companion with a launcher-style interface.
- **VLOX In-game** — a Fabric client mod that adds the VLOX menu and client-side UI options.

This is an independent client-side add-on, not a replacement Minecraft game engine. It does not bundle Mojang assets.

## Current foundation

The desktop app can open the installed Minecraft Launcher and copy a locally built VLOX mod JAR into the default Windows `.minecraft/mods` directory. The in-game menu opens with **Right Shift** and currently contains Home, HUD, Modules and Settings sections.

This is an early foundation. Direct Microsoft authentication, automatic profile/version installation, direct game launching, persistent settings and the full feature set are not implemented yet.

## Build

Desktop (stable Rust):
```sh
cargo run
cargo build --release
```

Fabric mod (JDK 21 and Gradle):
```sh
cd client-mod
gradle build
```

The mod JAR is created in `client-mod/build/libs`.

## Install

1. Install Fabric Loader for Minecraft Java **1.21.1**.
2. Build the desktop app and Fabric mod, or download the matching GitHub Actions artifacts.
3. In VLOX Desktop, open **Mod Library** and install the mod.
4. Start the Fabric 1.21.1 profile from the official Minecraft Launcher.

Compatibility with other Minecraft versions is not claimed until separately built and tested.
