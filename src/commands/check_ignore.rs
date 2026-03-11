use clap::Args;

#[derive(Args, Debug)]
pub struct CheckIgnoreArgs {
    pub path: Option<String>,
}

impl CheckIgnoreArgs {
    pub fn run(self) -> anyhow::Result<()> {
        Ok(())
    }
}

