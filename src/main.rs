use clap::Parser;

mod objects;
mod utils;

mod commands;
use commands::cat_file::CatFileArg;
use commands::check_ignore::CheckIgnoreArg;
use commands::hash_object::HashObjectArg;
use commands::init::InitArg;
use commands::log::LogArg;
use commands::ls_files::LsFilesArg;
use commands::ls_tree::LsTreeArg;
use commands::rev_parse::RevParseArg;
use commands::show_ref::ShowRefArg;
use commands::tag::TagArg;
use commands::write_tree::WriteTreeArg;

use crate::commands::add::AddArg;
use crate::commands::branch::BranchArg;
use crate::commands::commit::CommitArg;
use crate::commands::read_tree::ReadTreeArg;
use crate::commands::remove::RemoveArg;
use crate::commands::status::StatusArg;
use crate::commands::switch::SwitchArg;
use crate::commands::testing::TestArg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    #[command(hide = true)]
    MarkdownExport,
    #[command(hide = true)]
    Test(TestArg),

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
    CheckIgnore(CheckIgnoreArg),
    Rm(RemoveArg),
    Add(AddArg),
    Status(StatusArg),
    Commit(CommitArg),
    ReadTree(ReadTreeArg),
    Branch(BranchArg),
    Switch(SwitchArg),
}

fn main() {
    let args = Args::parse();

    let result = match args {
        Args::MarkdownExport => Ok(clap_markdown::print_help_markdown::<Args>()),
        Args::Test(a) => a.run(),

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
        Args::CheckIgnore(a) => a.run(),
        Args::Rm(a) => a.run(),
        Args::Add(a) => a.run(),
        Args::Status(a) => a.run(),
        Args::Commit(a) => a.run(),
        Args::ReadTree(a) => a.run(),
        Args::Branch(a) => a.run(),
        Args::Switch(a) => a.run(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
