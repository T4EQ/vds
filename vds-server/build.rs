use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    // Migrations are embedded in the server so that we can initialize a db from scratch
    println!("cargo::rerun-if-changed=./migrations");

    // Record build-time information
    built::write_built_file()?;

    let frontend_path = std::env::var("VDS_SERVER_FRONTEND_PATH")
        .unwrap_or_else(|_| "../vds-site/dist".to_string());

    // The VDS site code is embedded so that we only need to run a single binary
    // without additional dependencies.
    println!("cargo::rerun-if-changed={frontend_path}");
    resource_dir(&frontend_path).build()
}
