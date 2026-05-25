fn main() {
    let renderer = tiles_renderer::native_renderer_plan();
    let runtime = tiles_runtime::native_runtime_boundary();

    println!("Tiles Engine native preview scaffold");
    println!("renderer: {}", renderer.backend_summary());
    println!("preview: {}", renderer.preview_strategy);
    println!("runtime: {}", runtime.game_loop_owner);
}
