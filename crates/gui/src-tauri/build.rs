fn main() {
    // generate_context! requires frontendDist to exist even when only type-checking
    // the Rust crate (CI, cargo check), so create it before tauri-build runs.
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    std::fs::create_dir_all(&dist).expect("failed to create frontend dist directory");
    tauri_build::build()
}
