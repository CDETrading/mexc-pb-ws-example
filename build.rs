use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_root = "proto/mexc";
    
    // Recursively collect .proto files (or adjust as needed)
    let proto_files: Vec<PathBuf> = fs::read_dir(proto_root)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "proto" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for proto in &proto_files {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let out_dir = env::var("OUT_DIR")?;

    let mut config = prost_build::Config::new();
    config.out_dir(&out_dir);
    config.include_file("mexc_proto_build.rs"); // Rename output file here.
    config.compile_protos(&proto_files, &[proto_root])?;
    Ok(())
}