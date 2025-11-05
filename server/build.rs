fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);  //
    
    // Compile all protobuf files
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("proto_descriptor.bin"))
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
    
    // Link the Monte Carlo library using absolute path
    let lib_dir = "/home/paullopez/Desktop/cpp-workspace/MonteCarloLib/lib/build";
    
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib=mcoptions");
    println!("cargo:rerun-if-changed={}/libmcoptions.so", lib_dir);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
    
    Ok(())
}
