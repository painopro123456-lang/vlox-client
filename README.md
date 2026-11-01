# VLOX CLIENT

VLOX CLIENT is a native Minecraft Java launcher and client for **Minecraft Java 26.2**.

## Included features

- Microsoft account sign-in through Microsoft OAuth, Xbox Live, XSTS and Minecraft Services.
- Minecraft profile validation before launch.
- Saved Microsoft session in Windows Credential Manager (not a plaintext token file).
- Direct Minecraft Java 26.2 launch from VLOX; the official Minecraft Launcher is not required to start the game.
- Automatic download and installation of Minecraft assets, libraries, natives and the matching Java runtime.
- Automatic Fabric Loader profile setup.
- Bundled VLOX Fabric mod and Fabric API, installed into `.minecraft/mods` before launch.
- In-game VLOX menu opened with **Right Shift**.

VLOX does not bundle Mojang's game files. It downloads required Minecraft files from the official metadata and asset sources when you launch.

## Build requirements

- Windows 10/11 x64 for the current desktop build.
- Rust stable.
- JDK 25.
- Gradle 9.5.1.
- Network access to Microsoft, Mojang, Fabric and Maven download services.

The Windows workflow builds the Fabric mod and downloads the matching Fabric API first, then embeds both JARs into the VLOX executable. The EXE can therefore install its client components without requiring the repository checkout.

## Local build

Build the mod first with Gradle in `client-mod`. Download Fabric API 0.161.0+26.2 to `client-mod/build/libs/fabric-api-0.161.0+26.2.jar`, then run `cargo build --release` from the repository root.

The executable is created at `target/release/vlox-client.exe`.

## First launch

1. Open VLOX CLIENT.
2. Select **Sign in with Microsoft**.
3. Complete the displayed device-code sign-in in your browser.
4. After your Minecraft Java profile is verified, select **Launch Minecraft**.
5. On the first launch VLOX downloads the 26.2 game files, Java runtime and Fabric components. Later launches reuse the installed files and saved account session.

## Account security

The Microsoft account session is stored using Windows Credential Manager. Signing out removes the saved VLOX credential. Never share your Microsoft access or refresh tokens.

## Current target

- Minecraft Java: **26.2**
- Java runtime: **25**
- Fabric Loader: **0.19.3 or latest compatible**
- Fabric API bundled: **0.161.0+26.2**
