use clap::Parser;

mod util;

mod commands;
use commands::init::InitArg;
use commands::cat_file::CatFileArg;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init(InitArg),
    CatFile(CatFileArg),
}

fn main() {
    let args = Args::parse();

    match args {
        Args::Init(a) => a.run(),
        Args::CatFile(a) => a.run(),
    }
}
