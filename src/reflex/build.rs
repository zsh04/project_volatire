fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true) // Helper: Now we are a server too!
        .compile(

            &["../../protos/brain.proto", "../../protos/reflex.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
