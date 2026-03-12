use clap::Args;

#[derive(Args, Debug)]
pub struct TestArg {}

impl TestArg {
    pub fn run(self) -> anyhow::Result<()> {
        Ok(())
    }
}
