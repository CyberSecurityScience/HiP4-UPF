fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["proto/p4runtime.proto", "proto/bfruntime.proto"],
            &["proto/"],
        )
        .unwrap();
    Ok(())
}