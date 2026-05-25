# Local Setup

Tiles Engine is set up as a Rust workspace with a Tauri desktop app and React
frontend.

## Prerequisites

Install:

- Rust with the MSVC toolchain on Windows.
- Node.js LTS and npm.
- Microsoft C++ Build Tools with the Desktop development with C++ workload.
- Microsoft Edge WebView2 runtime if it is not already installed.

The official Tauri prerequisites page is the source of truth for platform setup:
https://v2.tauri.app/start/prerequisites/

## First Install

From the repo root:

```powershell
npm install --prefix apps/desktop
```

Then verify the Rust core:

```powershell
cargo test -p tiles-core
```

If the MSVC linker is not installed yet, pure Rust core tests can be run with
the temporary GNU toolchain:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
rustup component add --toolchain stable-x86_64-pc-windows-gnu rustfmt
cargo +stable-x86_64-pc-windows-gnu test -p tiles-core
```

The native preview can be smoke-tested with the MSVC toolchain:

```powershell
cargo run -p tiles-native-preview -- --smoke-test
```

Run it interactively without the smoke-test flag:

```powershell
cargo run -p tiles-native-preview
```

## Run The Desktop Shell

```powershell
npm run desktop:dev
```

This starts Vite on `http://localhost:5173` and launches Tauri. The shell calls a
Rust command named `engine_status` exposed by `apps/desktop/src-tauri/src/main.rs`
and implemented by `crates/tiles-core`.

## Current Machine Note

On 2026-05-25:

- Node.js LTS was installed with winget.
- Rustup was installed with winget.
- Visual Studio 2022 Build Tools was installed with the C++ workload by launching
  the cached `vs_BuildTools.exe` bootstrapper directly in passive mode.
- `npm install --prefix apps/desktop` completed after putting
  `C:\Program Files\nodejs` first in `PATH`.
- `npm --prefix apps/desktop run check` passed.
- `npm --prefix apps/desktop run build` passed.
- `cargo test -p tiles-core` passed with the default MSVC toolchain after
  loading the Visual Studio developer environment.
- `cargo test --workspace --exclude tiles-engine-desktop` passed with the
  default MSVC toolchain after loading the Visual Studio developer environment.
- `cargo check -p tiles-engine-desktop` passed with the default MSVC toolchain
  after adding the required `apps/desktop/src-tauri/icons/icon.ico` placeholder.
- `cargo check -p tiles-native-preview` passed with the default MSVC toolchain.
- `cargo run -p tiles-native-preview -- --smoke-test` passed with the default
  MSVC toolchain and rendered a native `wgpu` preview window.
- `cargo +stable-x86_64-pc-windows-gnu test -p tiles-core` passed.
- `cargo +stable-x86_64-pc-windows-gnu test --workspace --exclude tiles-engine-desktop` passed.
- `cargo +stable-x86_64-pc-windows-gnu fmt --all -- --check` passed.

In a fresh terminal, `link.exe` may still not appear on the normal PATH. Use the
Visual Studio developer shell, or load the environment with
`Common7\Tools\VsDevCmd.bat`, before running MSVC builds.

Next verification step:

```powershell
cargo test -p tiles-core
npm run desktop:dev
cargo run -p tiles-native-preview
```
