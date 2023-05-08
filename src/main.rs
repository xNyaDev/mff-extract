use std::error::Error;
use std::path::PathBuf;

use binrw::BinRead;
use clap::{Parser, Subcommand};

mod display;
mod extract;
mod list;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all files in the archive
    #[clap(visible_alias = "l", visible_alias = "ls")]
    List(list::Arguments),
    /// Extract all files from the archive
    #[clap(visible_alias = "e", visible_alias = "x")]
    Extract(extract::Arguments),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input archive file name
    input_file: PathBuf,
    /// Folder to extract files to
    output_folder: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List(arguments) => list::run(arguments, &mut std::io::stdout()),
        Commands::Extract(arguments) => extract::run(arguments),
    }
}

#[derive(BinRead)]
#[br(big)]
struct FFArchive {
    /// File data starts here
    data_start: u16,
    /// Files in this archive section
    file_count: u16,
    /// File info
    #[br(count = file_count)]
    archived_files: Vec<ArchivedFile>,
}

#[derive(BinRead, Debug)]
#[br(big)]
struct ArchivedFile {
    /// Length of the file name
    _file_name_length: u16,
    /// File name
    #[br(count = _file_name_length, map = |bytes: Vec<u8>| { String::from_utf8_lossy(&bytes).to_string() })]
    file_name: String,
    /// File offset from data_start
    offset: u32,
    /// File size
    size: u16,
}
