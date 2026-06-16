// We compile the .proto files with `protox` (a pure-Rust protobuf compiler) and
// feed the resulting FileDescriptorSet to tonic-build. This avoids requiring a
// `protoc` binary on the build host (none is installed here).
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = ["proto/worker.proto", "proto/api.proto"];
    let includes = ["proto"];

    for p in &protos {
        println!("cargo:rerun-if-changed={p}");
    }

    let file_descriptors = protox::compile(protos, includes)?;

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_fds(file_descriptors)?;

    Ok(())
}
