fn main() -> std::io::Result<()> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(&["proto/judger.proto"], &["proto"])
}
