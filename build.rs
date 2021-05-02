use std::io;

const DEFS: &[&str] = &["proto/github.com/moby/buildkit/frontend/gateway/pb/gateway.proto"];
const PATHS: &[&str] = &["proto"];

fn main() -> io::Result<()> {
    // Local RPC
    tonic_build::compile_protos("proto/judger.proto")?;

    // BuildKit Client
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(DEFS, PATHS)?;

    Ok(())
}
