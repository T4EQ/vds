use std::path::Path;

use static_files::resource_dir;

fn embed_static_files(src_path: &Path, namespace: &str) -> std::io::Result<()> {
    println!("cargo::rerun-if-changed={}", src_path.display());

    let mut frontend = resource_dir(src_path);
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir).join(namespace).join("generated.rs");
    if let Some(parent) = out_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    frontend.with_generated_filename(out_dir);
    frontend.build()
}

fn main() -> std::io::Result<()> {
    // Migrations are embedded in the server so that we can initialize a db from scratch
    println!("cargo::rerun-if-changed=./migrations");

    // Record build-time information
    built::write_built_file()?;

    // The LEAP site code is embedded so that we only need to run a single binary
    // without additional dependencies.
    let frontend_path: std::path::PathBuf = std::env::var("LEAP_SERVER_FRONTEND_PATH")
        .unwrap_or_else(|_| "../leap-site/dist".to_string())
        .into();
    embed_static_files(&frontend_path, "site")?;

    let provisioning_path: std::path::PathBuf = std::env::var("LEAP_SERVER_PROVISIONING_PATH")
        .unwrap_or_else(|_| "../leap-provision-site/dist".to_string())
        .into();
    embed_static_files(&provisioning_path, "provisioning")?;

    Ok(())
}
