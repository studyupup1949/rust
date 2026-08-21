fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_client(false) // only build server code
        .compile_protos(&["proto/agent.proto"], &["proto"])?;
    Ok(())
}
