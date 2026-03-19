use clap::Args;

/// Shows changes between working directory, index, commits etc.
#[derive(Args, Debug)]
pub struct DiffArg {
    pub path: Option<String>,
}

impl DiffArg {
    pub fn run(self) -> anyhow::Result<()> {

        Ok(())
    }
}
