#[tauri::command]
fn engine_status() -> tiles_core::EngineStatus {
    tiles_core::engine_status()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![engine_status])
        .run(tauri::generate_context!())
        .expect("failed to run Tiles Engine desktop app");
}
