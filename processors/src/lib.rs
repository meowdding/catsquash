mod json;
mod png;

use crate::json::JsonFileProcessor;
use crate::png::PngFileProcessor;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use utils::SquashOptions;
use utils::error::{Result, SquashError};

pub async fn process_file(
    input: PathBuf,
    entry_path: String,
    path: PathBuf,
    options: SquashOptions,
) -> Result<()> {
    if options.verbose {
        println!("Processing file {entry_path}")
    }
    let mut entry = input.clone();
    entry.push(&entry_path);

    let path = &path;
    let data = fs::read(entry).map_err(|err| SquashError::FailedToRead {
        path: path.display().to_string(),
        error: err.to_string(),
    })?;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|err| SquashError::FileError {
            error: err.to_string(),
        })?;

    for processor in FileProcessors::new().processors {
        if processor.can_process(PathBuf::from(&entry_path).as_path()) {
            let data = processor.process(data, PathBuf::from(entry_path).as_path(), &options)?;

            file.write_all(&data[..])
                .map_err(|err| SquashError::FailedToWrite {
                    path: path.display().to_string(),
                    error: err.to_string(),
                })?;

            return Ok(());
        }
    }

    file.write_all(&data[..])
        .map_err(|err| SquashError::FailedToWrite {
            path: path.display().to_string(),
            error: err.to_string(),
        })
}

pub async fn process(
    input: &Path,
    output: &Path,
    options: SquashOptions,
) -> utils::error::Result<()> {
    let mut prefix = input.display().to_string();
    if !prefix.ends_with("/") {
        prefix.push_str("/");
    }

    let mut paths = Vec::<(String, PathBuf)>::new();

    for entry in walkdir::WalkDir::new(input).follow_links(false).into_iter() {
        let entry = entry.map_err(|err| SquashError::FileError {
            error: err.to_string(),
        })?;

        if input == entry.path() {
            continue;
        }

        let entry_path = entry
            .path()
            .display()
            .to_string()
            .strip_prefix(prefix.as_str())
            .unwrap()
            .to_string();

        let path = output.join(&entry_path);
        fs::create_dir_all(&path.parent().unwrap()).map_err(|err| SquashError::FileError {
            error: err.to_string(),
        })?;
        if !entry.path().is_file() {
            if options.verbose {
                println!("Skipping non file {entry_path}!")
            }
            continue;
        }

        if entry
            .path()
            .file_name()
            .and_then(|ostr| ostr.to_str())
            .map(|str| str.to_string().eq_ignore_ascii_case(".DS_Store"))
            .unwrap_or(false)
        {
            println!("Skipping ds_store {entry_path}!");
            continue;
        }

        paths.push((entry_path.clone(), path.to_path_buf().clone()));
    }

    let input = input.to_path_buf();
    let futures = paths.iter().map(|(entry_path, path)| {
        tokio::spawn(process_file(
            input.clone(),
            entry_path.clone(),
            path.clone(),
            options.clone(),
        ))
    });

    for ele in futures {
        ele.await.map_err(|_| SquashError::JoinError)??;
    }

    Ok(())
}

struct FileProcessors {
    processors: Vec<Box<dyn FileProcessor>>,
}

impl FileProcessors {
    fn new() -> Self {
        FileProcessors {
            processors: vec![
                Box::new(JsonFileProcessor::new()),
                Box::new(PngFileProcessor::new()),
            ],
        }
    }
}

pub trait FileProcessor {
    fn can_process(&self, path: &Path) -> bool;

    fn process(&self, data: Vec<u8>, path: &Path, options: &SquashOptions) -> Result<Vec<u8>>;
}
