use clap::Parser;

mod commit;
mod errors;
mod object;
mod tree;
mod util;
mod tag;

mod commands;
use commands::cat_file::CatFileArg;
use commands::hash_object::HashObjectArg;
use commands::init::InitArg;

use crate::commands::log::LogArg;
use crate::commands::ls_tree::LsTreeArg;
use crate::commands::show_ref::ShowRefArg;
use crate::commands::tag::TagArg;
use crate::commands::write_tree::WriteTreeArg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Init(InitArg),
    CatFile(CatFileArg),
    HashObject(HashObjectArg),
    Log(LogArg),
    LsTree(LsTreeArg),
    WriteTree(WriteTreeArg),
    ShowRef(ShowRefArg),
    Tag(TagArg),
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
        Args::ShowRef(a) => a.run(),
        Args::Tag(a) => a.run(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
