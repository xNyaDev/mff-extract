use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
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

            let output_file_path = sanitize_path(&arguments.output, &archived_file.file_name)?;
            if let Some(parent) = output_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
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

/// Resolves an archived entry's stored file name into a safe path inside the
/// output directory.
///
/// File names come from the archive and are therefore untrusted. Without
/// checks, an entry such as `../ESCAPED.txt` or an absolute path could be
/// written outside the chosen output directory (a "tarslip"/"zip slip" path
/// traversal). To prevent this, only normal path components are kept: absolute
/// paths, drive/root prefixes and `..` components are rejected, guaranteeing the
/// returned path stays within `output`.
fn sanitize_path(output: &Path, file_name: &str) -> io::Result<PathBuf> {
    let mut sanitized = PathBuf::new();

    for component in Path::new(file_name).components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            // A leading `./` is harmless and simply ignored.
            Component::CurDir => {}
            // `..`, absolute roots and Windows drive prefixes could all escape
            // the output directory, so refuse the entry entirely.
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("refusing to extract file with unsafe name: {file_name:?}"),
                ));
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("refusing to extract file with empty or invalid name: {file_name:?}"),
        ));
    }

    Ok(output.join(sanitized))
}

pub fn extract_data<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    size: u64,
) -> io::Result<u64> {
    let mut data = reader.take(size);
    io::copy(&mut data, writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_normal_names_inside_output() {
        let output = Path::new("out");
        assert_eq!(
            sanitize_path(output, "file.txt").unwrap(),
            output.join("file.txt")
        );
        assert_eq!(
            sanitize_path(output, "sub/dir/file.txt").unwrap(),
            output.join("sub").join("dir").join("file.txt")
        );
        // A leading `./` is stripped and harmless.
        assert_eq!(
            sanitize_path(output, "./file.txt").unwrap(),
            output.join("file.txt")
        );
    }

    #[test]
    fn rejects_parent_directory_traversal() {
        let output = Path::new("out");
        assert!(sanitize_path(output, "../ESCAPED.txt").is_err());
        assert!(sanitize_path(output, "sub/../../ESCAPED.txt").is_err());
        assert!(sanitize_path(output, "..").is_err());
    }

    #[test]
    fn rejects_absolute_paths() {
        let output = Path::new("out");
        assert!(sanitize_path(output, "/etc/passwd").is_err());
    }

    #[test]
    fn rejects_empty_names() {
        let output = Path::new("out");
        assert!(sanitize_path(output, "").is_err());
        assert!(sanitize_path(output, ".").is_err());
    }
}
