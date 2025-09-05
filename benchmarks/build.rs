fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile gRPC protobuf schema
    tonic_build::compile_protos("../schemas/metrics.proto")?;
    
    // Compile Cap'n Proto schema
    capnpc::CompilerCommand::new()
        .src_prefix("../schemas")
        .file("../schemas/metrics.capnp")
        .run()?;
    
    Ok(())
}