use std::{
    env, fs,
    path::{Path, PathBuf},
};

const NATIVE_PREVIEW_SIDECAR_NAME: &str = "tiles-native-preview";

fn main() {
    ensure_dev_sidecar_placeholder();
    tauri_build::build();
}

fn ensure_dev_sidecar_placeholder() {
    if env::var("PROFILE").as_deref() == Ok("release") {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_dir) => PathBuf::from(manifest_dir),
        Err(_) => return,
    };
    let target_triple = env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_string());
    let placeholder_path = manifest_dir
        .join("binaries")
        .join(sidecar_binary_name(&target_triple));

    if placeholder_path.is_file() {
        return;
    }

    if let Some(parent) = placeholder_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(
        &placeholder_path,
        b"Tiles Engine development placeholder for Tauri sidecar validation.\n",
    );
    make_executable_on_unix(&placeholder_path);

    println!(
        "cargo:warning=Created development-only native preview sidecar placeholder at {}",
        placeholder_path.display()
    );
}

fn sidecar_binary_name(target_triple: &str) -> String {
    let extension = if target_triple.contains("windows") {
        ".exe"
    } else {
        ""
    };

    format!("{NATIVE_PREVIEW_SIDECAR_NAME}-{target_triple}{extension}")
}

#[cfg(unix)]
fn make_executable_on_unix(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn make_executable_on_unix(_path: &Path) {}
