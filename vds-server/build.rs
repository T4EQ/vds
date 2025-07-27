use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    println!("cargo::rerun-if-changed=../vds-site/dist");
    resource_dir("../vds-site/dist").build()
}
