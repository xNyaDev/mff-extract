use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::{fs, io};

use binrw::BinRead;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use crate::display::display_size;
use crate::FFArchive;

#[derive(Parser)]
pub struct Arguments {
    /// FF archive file name
    archive: PathBuf,
    /// Output directory
    output: PathBuf,
    /// Print names of extracted files
    #[clap(short, long)]
    verbose: bool,
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let file = File::open(&arguments.archive)?;
    let mut file_reader = BufReader::new(file);

    let archive = FFArchive::read(&mut file_reader)?;

    let bar = ProgressBar::new(archive.file_count as u64);

    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] {wide_bar} [{pos}/{len}]")
            .unwrap()
            .progress_chars("##-"),
    );

    fs::create_dir_all(&arguments.output)?;

    archive
        .archived_files
        .iter()
        .try_for_each(|archived_file| {
            file_reader.seek(SeekFrom::Start(
                archive.data_start as u64 + archived_file.offset as u64,
            ))?;

            let output_file_path = arguments.output.join(&archived_file.file_name);
            let mut output_file = File::create(output_file_path)?;

            extract_data(
                &mut file_reader,
                &mut output_file,
                archived_file.size as u64,
            )?;

            if arguments.verbose {
                bar.println(format!(
                    "{} [{}]",
                    archived_file.file_name,
                    display_size(&(archived_file.size as u64))
                ));
            }
            bar.inc(1);
            Ok::<(), io::Error>(())
        })?;

    bar.finish_and_clear();

    println!(
        "Extracted {}.",
        if bar.length() == Some(1) {
            "1 file".to_string()
        } else {
            format!("{} files", bar.length().unwrap_or_default())
        }
    );

    Ok(())
}

pub fn extract_data<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    size: u64,
) -> io::Result<u64> {
    let mut data = reader.take(size);
    io::copy(&mut data, writer)
}
