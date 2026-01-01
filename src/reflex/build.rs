fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // Only client needed for Reflex
        .compile(
            &["../../protos/brain.proto", "../../protos/reflex.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
