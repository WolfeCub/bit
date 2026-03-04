use clap::Parser;

mod util;
mod object;
mod errors;
mod commit;
mod tree;

mod commands;
use commands::init::InitArg;
use commands::cat_file::CatFileArg;
use commands::hash_object::HashObjectArg;

use crate::commands::log::LogArg;
use crate::commands::ls_tree::LsTreeArg;
use crate::commands::write_tree::WriteTreeArg;
use crate::errors::BitError;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init(InitArg),
    CatFile(CatFileArg),
    HashObject(HashObjectArg),
    Log(LogArg),
    LsTree(LsTreeArg),
    WriteTree(WriteTreeArg),
}

fn main() {
    let args = Args::parse();

    let result = match args {
        Args::Init(a) => a.run(),
        Args::CatFile(a) => a.run(),
        Args::HashObject(a) => a.run(),
        Args::Log(a) => a.run(),
        Args::LsTree(a) => a.run(),
        Args::WriteTree(a) => a.run(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
