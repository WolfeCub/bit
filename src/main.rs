use clap::Parser;

mod commit;
mod config;
// mod errors;
mod object;
mod tag;
mod tree;
mod util;
mod index;

mod commands;
use commands::cat_file::CatFileArg;
use commands::hash_object::HashObjectArg;
use commands::init::InitArg;

use crate::commands::log::LogArg;
use crate::commands::ls_files::LsFilesArg;
use crate::commands::ls_tree::LsTreeArg;
use crate::commands::rev_parse::RevParseArg;
use crate::commands::show_ref::ShowRefArg;
use crate::commands::tag::TagArg;
use crate::commands::write_tree::WriteTreeArg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    #[command(hide = true)]
    MarkdownExport,
    Init(InitArg),
    CatFile(CatFileArg),
    HashObject(HashObjectArg),
    Log(LogArg),
    LsTree(LsTreeArg),
    WriteTree(WriteTreeArg),
    ShowRef(ShowRefArg),
    Tag(TagArg),
    RevParse(RevParseArg),
    LsFiles(LsFilesArg),
}

fn main() {
    let args = Args::parse();

    let result = match args {
        Args::MarkdownExport => {
            clap_markdown::print_help_markdown::<Args>();
            Ok(())
        }
        Args::Init(a) => a.run(),
        Args::CatFile(a) => a.run(),
        Args::HashObject(a) => a.run(),
        Args::Log(a) => a.run(),
        Args::LsTree(a) => a.run(),
        Args::WriteTree(a) => a.run(),
        Args::ShowRef(a) => a.run(),
        Args::Tag(a) => a.run(),
        Args::RevParse(a) => a.run(),
        Args::LsFiles(a) => a.run(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
