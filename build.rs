use std::io;

fn main() -> io::Result<()> {
    // Build Controller API (with client & server)
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(&["proto/controller.proto"], &["proto"])?;

    // Build Judger API (for customize judger)
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile(&["proto/judger.proto"], &["proto"])?;

    Ok(())
}
