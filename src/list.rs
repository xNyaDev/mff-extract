use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use binrw::BinRead;
use clap::Parser;
use tabled::settings::object::{Columns, Segment};
use tabled::settings::{Alignment, Modify, Style};
use tabled::{Table, Tabled};

use crate::display::{display_offset, display_size};
use crate::FFArchive;

#[derive(Parser)]
pub struct Arguments {
    /// FF archive file name
    archive: PathBuf,
}

#[derive(Tabled, Eq, PartialEq)]
pub struct TableFileInfo {
    #[tabled(rename = "Size", display_with = "display_size")]
    pub size: u64,

    #[tabled(rename = "Offset", display_with = "display_offset")]
    pub offset: u32,

    #[tabled(rename = "File Name")]
    pub file_name: String,
}

pub fn run(arguments: Arguments, mut writer: impl std::io::Write) -> Result<(), Box<dyn Error>> {
    let file = File::open(&arguments.archive)?;
    let mut file_reader = BufReader::new(file);

    let archive = FFArchive::read(&mut file_reader)?;

    let table_contents = archive
        .archived_files
        .into_iter()
        .map(|file_info| TableFileInfo {
            size: file_info.size as u64,
            offset: file_info.offset + archive.data_start as u32,
            file_name: file_info.file_name,
        })
        .collect::<Vec<TableFileInfo>>();

    writeln!(
        writer,
        "Listing archive: {}",
        arguments.archive.to_string_lossy()
    )?;
    writeln!(
        writer,
        "Physical size: {}",
        display_size(&fs::metadata(&arguments.archive).unwrap().len())
    )?;
    writeln!(writer, "File count: {}", archive.file_count)?;
    writeln!(
        writer,
        "{}",
        Table::new(table_contents)
            .with(Style::markdown())
            .with(Modify::new(Segment::all()).with(Alignment::right()))
            .with(Modify::new(Columns::single(1)).with(Alignment::center()))
            .with(Modify::new(Columns::last()).with(Alignment::left()))
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn listing_test() -> Result<(), Box<dyn Error>> {
        let mut result = Vec::new();
        let arguments = Arguments {
            archive: PathBuf::from("test_data/resource0.bin"),
        };
        run(arguments, &mut result)?;

        let mut expected_result_file = File::open("test_data/list.txt")?;
        let mut expected_result = Vec::new();
        expected_result_file.read_to_end(&mut expected_result)?;

        // Compare results as strings for pretty diff when mismatching
        //
        // Ignore mismatching line breaks when comparing (assume \r\n and \n are equal) by
        // removing all occurrences of \r
        assert_eq!(
            String::from_utf8_lossy(&result)
                .to_string()
                .replace('\r', ""),
            String::from_utf8_lossy(&expected_result)
                .to_string()
                .replace('\r', "")
        );

        Ok(())
    }
}
