fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    // Use absolute path - most reliable
    let lib_dir = "/home/paullopez/Desktop/cpp-workspace/MonteCarloLib/lib/build";
    
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib=mcoptions");
    println!("cargo:rerun-if-changed={}/libmcoptions.so", lib_dir);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
    
    Ok(())
}
