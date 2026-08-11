use anyhow::Result;

fn main() -> Result<()> {
    tonic_prost_build::compile_protos("proto/ipc.proto")?;

    Ok(())
}