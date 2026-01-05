use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    // Migrations are embedded in the server so that we can initialize a db from scratch
    println!("cargo::rerun-if-changed=./migrations");

    // The VDS site code is embedded so that we only need to run a single binary
    // without additional dependencies.
    println!("cargo::rerun-if-changed=../vds-site/dist");
    resource_dir("../vds-site/dist").build()
}
