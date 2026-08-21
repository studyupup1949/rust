fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/ada/v1/browser_session.proto",
        "proto/ada/v1/common.proto",
        "proto/ada/v1/data.proto",
        "proto/ada/v1/events.proto",
        "proto/ada/v1/ingest.proto",
        "proto/ada/v1/query.proto",
        "proto/ada/v1/signals.proto",
        "proto/ada/v1/summary.proto",
    ];
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&protos, &["proto"])?;
    Ok(())
}
