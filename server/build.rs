fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    
    // Compile all protobuf files
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                "../protos/common.proto",
                "../protos/trading.proto",
                "../protos/pricing.proto",
            ],
            &["../protos"],
        )?;
    
    println!("cargo:rerun-if-changed=../protos/common.proto");
    println!("cargo:rerun-if-changed=../protos/trading.proto");
    println!("cargo:rerun-if-changed=../protos/pricing.proto");
    
    // Debug: print where files are generated
    println!("cargo:warning=Generated files in: {}", out_dir.display());
    
    Ok(())
}
