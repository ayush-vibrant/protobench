fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("../schemas")
        .file("../schemas/metrics.capnp")
        .run()
        .expect("compiling schema");
}