# Packaged Preview Binary Lookup

Issue #53 decides how packaged Tiles Engine builds should find and launch the
native preview/playtest binary after the current development-only
`target/debug` lookup.

## Decision

Use a dual lookup strategy:

- Development builds keep the existing `target/debug/tiles-native-preview`
  lookup.
- Packaged desktop builds launch `tiles-native-preview` as a Tauri sidecar.

React should not locate or spawn the preview binary directly. The Tauri/Rust
command layer owns launch behavior and returns a user-facing launch result or
error to the editor UI.

## Options Compared

### Keep `target/debug` Lookup

This is the current development path.

Pros:

- Simple for contributors.
- Easy to test with `cargo build -p tiles-native-preview`.
- Keeps the preview binary outside the desktop app while renderer work changes
  quickly.

Cons:

- Packaged apps cannot assume a repository checkout or Cargo target directory.
- Users would see a broken `Open Preview` button after installation.
- It hides a development assumption inside production launch behavior.

Decision: keep only for local development.

### Tauri Sidecar

Tauri supports bundling external binaries as sidecars through `bundle.externalBin`.
The official Tauri sidecar docs say each supported architecture needs a binary
with the configured name plus a `-$TARGET_TRIPLE` suffix. The same docs show
Rust-side launch through `app.shell().sidecar(...)` and JavaScript-side launch
through `Command.sidecar(...)`.

Pros:

- Purpose-built for packaged external binaries.
- Works with Tauri's bundle/install layout instead of guessing platform paths.
- Keeps process spawning in the desktop shell boundary.
- Allows a clear capability/permission model if launched from JavaScript later.

Cons:

- Requires per-target binary naming and placement before packaging.
- Requires adding the Tauri shell plugin to the desktop app.
- macOS signing/notarization may need extra verification for the bundled binary.

Decision: use this for packaged desktop preview/playtest launch.

### Bundle As A Resource And Spawn By Path

The native preview binary could be copied as a bundled resource and resolved at
runtime by path.

Pros:

- Keeps launch code close to the current `std::process::Command` path.
- Could work for internal experiments.

Cons:

- Recreates sidecar path handling manually.
- Easier to get wrong across `.app`, AppImage, deb/rpm, and Windows installer
  layouts.
- Does not use Tauri's documented sidecar model.

Decision: avoid for MVP unless sidecar constraints block us.

## Recommended Packaged Flow

1. Build `apps/native-preview` for the target platform.
2. Copy or rename the binary into `apps/desktop/src-tauri/binaries/` using the
   Tauri sidecar target-triple suffix convention.
3. Add `binaries/tiles-native-preview` to `apps/desktop/src-tauri/tauri.conf.json`
   under `bundle.externalBin`.
4. Add `tauri-plugin-shell` to the desktop Rust app.
5. Update `launch_native_preview`:
   - In development, use the current workspace `target/debug` lookup.
   - In packaged builds, call `app.shell().sidecar("tiles-native-preview")`.
6. Return the same `PreviewLaunch` response shape to React.

## Platform Caveats

Windows:

- Sidecar binaries need the Windows target triple suffix and `.exe` handling.
- Missing sidecar errors should mention packaged preview installation rather
  than Cargo build commands.

macOS:

- App bundle layout differs from Windows and Linux, so avoid manual path
  assumptions.
- Signing/notarization should verify the bundled native preview binary.
- Apple Silicon and Intel builds need correctly suffixed sidecar binaries.

Linux:

- AppImage/deb/rpm layouts should use Tauri's sidecar resolution.
- Build scripts must preserve executable permissions for sidecar binaries.

## Current Limits

- No sidecar config is implemented yet.
- No cross-target copy/rename script exists yet.
- No packaged installer verification exists yet.
- Live scene streaming to the preview remains separate from launch lookup.
- Exported game launch remains separate from editor preview launch.

## Follow-Ups

- #54: Prototype live scene streaming to native preview.
- #55: Design exported game launch path.
- #87: Implement packaged native preview sidecar launch.

## Sources

- [Tauri: Embedding External Binaries](https://v2.tauri.app/develop/sidecar/)
- [Tauri shell plugin reference](https://v2.tauri.app/reference/javascript/shell/)
