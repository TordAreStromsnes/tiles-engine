# Spike 001: Tauri + React + Rust Shell

## Question

Can Tiles Engine use Tauri + React for the local editor shell while calling into
a reusable Rust engine core?

## Current Answer

Provisionally yes. The repo now includes:

- Root Rust workspace.
- `crates/tiles-core` Rust crate.
- `apps/desktop` React/Vite frontend.
- `apps/desktop/src-tauri` Tauri shell.
- A Tauri command bridge named `engine_status`.

## Validation Completed

- Node.js LTS installed.
- Rustup installed.
- Desktop npm dependencies installed.
- Frontend TypeScript check passed.
- Frontend production build passed.
- Visual Studio Build Tools installed with the C++ workload.
- `tiles-core` tests passed on the default MSVC toolchain.
- `tiles-engine-desktop` compile-check passed on the default MSVC toolchain.
- `tiles-core` tests passed with `stable-x86_64-pc-windows-gnu`.
- Rust formatting check passed with `stable-x86_64-pc-windows-gnu`.
- Renderer and ECS options are compared in
  [003-renderer-ecs-options.md](003-renderer-ecs-options.md).

## Validation Still Needed

- Run `npm run desktop:dev`.
- Confirm the desktop app displays `Rust bridge connected`.

## Notes

Tauri's normal Windows development path requires the MSVC linker. The linker is
available after loading the Visual Studio developer environment, but it is not on
the ordinary shell PATH by default.

## Definition Of Done

- The desktop shell launches locally.
- React UI renders the starter editor workspace.
- The UI receives stack status from Rust through Tauri.
- Verification commands and screenshots are recorded in the issue or PR.

## Follow-Up

After this shell works, the next spike should render a moving sprite or tile map
inside the editor preview area.
