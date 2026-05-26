# Embedded Native Viewport Feasibility

Issue #18 asks whether Tiles Engine should replace the sibling native preview
window with an embedded native GPU viewport inside the Tauri editor.

## Recommendation

Defer embedded native viewport work. Keep the sibling native preview/playtest
window as the MVP path.

Do not create an implementation prototype issue yet. Revisit after:

- Development preview launch is stable: #17.
- Packaged preview lookup is designed: #53.
- Live scene streaming exists: #54.
- Overlay primitives are separated from hardcoded preview markers: #50.

The next embedded-viewport step should be another scoped prototype issue only
after those prerequisites are done.

## Options Reviewed

### Sibling Native Preview Window

This is the current MVP path. The Rust `wgpu` renderer owns its own `winit`
window, event loop, surface, and GPU lifecycle. The desktop shell launches it and
will later stream scene data to it.

Pros:

- Lowest platform risk.
- Keeps React out of the render loop.
- Preserves native Rust renderer ownership.
- Easy to smoke-test as a separate binary.

Cons:

- Less integrated editor workflow.
- Needs scene streaming and window coordination.
- Selection/inspector UX must bridge between editor and sibling preview.

### Tauri Webview Window Or Child Webview

Tauri supports creating additional `WebviewWindow`s, and Tauri webviews have
desktop APIs such as bounds, reparenting, and platform-specific webview access.
That is useful for multi-window editor UI, but it is not the same thing as
embedding a Rust-owned `wgpu` surface inside a React panel.

This option is useful for editor panels, docs, asset browsers, or secondary UI.
It does not satisfy the native renderer goal by itself.

### Native Child Window Inside The Editor

`wgpu` can create surfaces from window/display handles, and `winit` has an
unsafe parent-window path on some platforms. This is the closest technical route
to an embedded Rust-native viewport, but it carries the highest risk.

Risks:

- Parent window handles are unsafe and platform-specific.
- Wayland child windows are not supported by the checked `winit` API.
- macOS/Metal surface creation has main-thread requirements.
- Tauri/WebView2 window creation has known Windows command/event-handler
  deadlock warnings.
- Focus, input routing, DPI scaling, resize synchronization, and z-ordering need
  custom work across Windows, macOS, and Linux.

### Offscreen Rendering Streamed Into React

The renderer could draw offscreen and stream images or textures into the editor
UI. This avoids native child-window parenting, but it adds copy/latency cost and
does not give the editor a direct native swapchain. It may be useful for small
thumbnails or asset previews later, but it is not the best primary game preview
surface for MVP.

## Platform Notes

### Windows

Owned windows and child windows are technically possible, but Tauri documents
Windows deadlock risk when creating windows from synchronous commands/event
handlers. WebView2 focus and z-order behavior would need dedicated testing.

### macOS

AppKit child windows/views are possible in principle, but `wgpu`/Metal surface
creation must respect main-thread rules. DPI, resizing, and input focus would
need native code beyond the current Tauri command bridge.

### Linux

X11 has child-window behavior, but Wayland support is the important blocker:
the checked `winit` parent-window API lists Wayland as unsupported. This makes a
cross-Linux embedded viewport risky for a first engine MVP.

## Decision

Keep the sibling native preview window. Treat embedded native viewport as a
post-MVP research direction, not a near-term implementation.

The engine should continue to invest in:

- A stable renderer contract.
- Desktop launch and process lifecycle.
- Scene/runtime snapshot streaming.
- Editor overlay primitives and selection state.

Those investments are useful for both sibling and embedded preview futures.

## References

- Tauri `WebviewWindowBuilder` docs: https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
- Tauri `Webview` docs: https://docs.rs/tauri/latest/tauri/webview/struct.Webview.html
- `wgpu` `SurfaceTarget` docs: https://wgpu.rs/doc/wgpu/api/surface/enum.SurfaceTarget.html
- `winit` parent window API docs: https://docs.rs/winit-gtk/latest/winit/window/struct.WindowBuilder.html#method.with_parent_window
