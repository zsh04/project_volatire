fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define paths to protos
    let proto_root = "../../protos";
    let reflex_proto = format!("{}/reflex.proto", proto_root);
    let brain_proto = format!("{}/brain.proto", proto_root);

    // 2. Compile Protos
    // We want to generate server code for Reflex, and potentially client code for Brain
    tonic_build::configure()
        .build_server(true)
        .build_client(true) // Reflex acts as client to Brain
        .compile(
            &[reflex_proto, brain_proto],
            &[proto_root],
        )?;

    Ok(())
}
