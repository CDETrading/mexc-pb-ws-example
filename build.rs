// build.rs
extern crate glob;
use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect all proto files.
    let mut proto_files: Vec<String> = Vec::new();
    for entry in glob("proto/mexc/*.proto")? {
        if let Ok(path) = entry {
            println!("cargo:rerun-if-changed={}", path.display());
            proto_files.push(path.display().to_string());
        }
    }

    // Configure prost to generate a single file (mexc.rs) in OUT_DIR.
    let mut config = prost_build::Config::new();
    config.include_file("mexc.rs"); // This tells prost to combine all generated modules into "mexc.rs"
    config.compile_protos(&proto_files, &["proto/mexc"])?;
    Ok(())
}