fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false) // only build client code
        .compile_protos(
            &["src/workflow/actions/agent/proto/agent.proto"],
            &["src/workflow/actions/agent/proto"],
        )?;
    Ok(())
}
