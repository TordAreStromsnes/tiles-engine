# Desktop Native Playtest Launch

Issue #17 connected the Tauri desktop shell to the sibling native preview
window. Issue #152 upgrades that path into the first native playtest snapshot
launcher: React sends editor state to Rust, Rust writes a temporary snapshot
file, and the native process loads only that file.

This is still a development-mode bridge for the editor. Packaged installer
behavior and embedded native viewport work are intentionally deferred. The
packaged lookup decision is documented in
[packaged-preview-binary-lookup.md](packaged-preview-binary-lookup.md).

## Command

The desktop shell exposes `launch_native_playtest` from
`apps/desktop/src-tauri/src/main.rs`.

The command:

- Resolves the repo root from the Tauri manifest directory.
- Accepts and validates the current editor `SceneDocument`.
- Builds the native preview snapshot and validates it against the standard
  runtime safety budget before launch.
- Blocks hard errors with severity-coded diagnostics and allows warning-only
  diagnostics to launch.
- Looks for `target/debug/tiles-native-preview` or
  `target/debug/tiles-native-preview.exe`.
- Writes a unique temporary snapshot folder under the OS temp directory.
- Starts the binary as a sibling desktop process with `--snapshot <path>`.
- Returns the process id, command path, snapshot root/path, cleanup count, and a
  user-facing message with the validation report.
- Returns `launched: false` with diagnostics when validation blocks launch,
  leaving snapshot paths empty because no process was started.
- Returns a clear error if the binary has not been built.
- Returns a clear error if the snapshot cannot be validated, serialized, or
  written.
- Leaves project files unchanged unless the user explicitly saves.

Successful launches are marked with `launch-ok.json`. Cleanup removes older
successful snapshot folders while retaining unmarked folders, which are useful
when a launch fails after snapshot creation.

## Local Development Flow

Build the native preview binary first:

```powershell
cargo build -p tiles-native-preview
```

Run the desktop shell:

```powershell
npm run desktop:dev
```

In the desktop app, use `Playtest` from the toolbar. The inspector shows the
launch result, snapshot path, cleanup count, or the error returned from Rust.

## Deferred Work

- Packaged sidecar implementation for the preview binary: #87.
- Replace the launch snapshot with true live editor streaming after #54.
- Embedded native viewport feasibility: #18.
- Exported game build launch behavior: #55.

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p tiles-engine-desktop`
- `cargo check -p tiles-engine-desktop`
- `cargo build -p tiles-native-preview`
- `npm --prefix apps/desktop run check`
- `npm --prefix apps/desktop run build`

Manual desktop launch should be verified from `npm run desktop:dev` after
`cargo build -p tiles-native-preview`.
