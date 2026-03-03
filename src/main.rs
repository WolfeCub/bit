use clap::Parser;

mod util;
mod object;

mod commands;
use commands::init::InitArg;
use commands::cat_file::CatFileArg;
use commands::hash_object::HashObjectArg;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init(InitArg),
    CatFile(CatFileArg),
    HashObject(HashObjectArg),
}

fn main() {
    let args = Args::parse();

    match args {
        Args::Init(a) => a.run(),
        Args::CatFile(a) => a.run(),
        Args::HashObject(a) => a.run(),
    }
}
