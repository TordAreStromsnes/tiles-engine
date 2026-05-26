# Desktop Native Preview Launch

Issue #17 connects the Tauri desktop shell to the sibling native preview window.
This is a development-mode bridge only. Packaged installer behavior and embedded
native viewport work are intentionally deferred.

## Command

The desktop shell exposes `launch_native_preview` from
`apps/desktop/src-tauri/src/main.rs`.

The command:

- Resolves the repo root from the Tauri manifest directory.
- Looks for `target/debug/tiles-native-preview` or
  `target/debug/tiles-native-preview.exe`.
- Starts the binary as a sibling desktop process.
- Returns the process id, command path, and a user-facing message.
- Returns a clear error if the binary has not been built.

## Local Development Flow

Build the native preview binary first:

```powershell
cargo build -p tiles-native-preview
```

Run the desktop shell:

```powershell
npm run desktop:dev
```

In the desktop app, use `Open Preview` from the toolbar. The inspector shows the
launch result or the error returned from Rust.

## Deferred Work

- Packaged installer/runtime lookup for the preview binary: #53.
- Live scene streaming from editor to preview: #54.
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
