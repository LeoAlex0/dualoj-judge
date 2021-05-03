use std::io;

fn main() -> io::Result<()> {
    // Local RPC
    tonic_build::compile_protos("proto/judger.proto")?;

    Ok(())
}
