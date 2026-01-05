mod json;

use crate::json::JsonFileProcessor;
use meta::Context;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use utils::error::{Result, SquashError};

pub fn process(input: &Path, output: &Path, context: &Context) -> utils::error::Result<()> {
    let mut prefix = input.display().to_string();
    prefix.push_str("/");
    println!("{prefix}");

    let processor = FileProcessors::new();

    'walk: for entry in walkdir::WalkDir::new(input).follow_links(false).into_iter() {
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
            if context.verbose {
                println!("Skipping non file {entry_path}!")
            }
            continue;
        }

        if context.verbose {
            println!("Processing file {entry_path}")
        }

        let data = fs::read(entry.path()).map_err(|err| SquashError::FailedToRead {
            path: entry.path().display().to_string(),
            error: err.to_string(),
        })?;
        for processor in &processor.processors {
            if processor.can_process(PathBuf::from(&entry_path).as_path()) {
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&path)
                    .map_err(|err| SquashError::FileError {
                        error: err.to_string(),
                    })?;

                let data = processor.process(data, entry.path())?;

                file.write_all(&data[..])
                    .map_err(|err| SquashError::FailedToWrite {
                        path: path.display().to_string(),
                        error: err.to_string(),
                    })?;

                continue 'walk;
            }
        }

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|err| SquashError::FileError {
                error: err.to_string(),
            })?;

        file.write_all(&data[..])
            .map_err(|err| SquashError::FailedToWrite {
                path: path.display().to_string(),
                error: err.to_string(),
            })?;
    }
    Ok(())
}

struct FileProcessors {
    processors: Vec<Box<dyn FileProcessor>>,
}

impl FileProcessors {
    fn new() -> Self {
        FileProcessors {
            processors: vec![Box::new(JsonFileProcessor::new())],
        }
    }
}

pub trait FileProcessor {
    fn can_process(&self, path: &Path) -> bool;

    fn process(&self, data: Vec<u8>, path: &Path) -> Result<Vec<u8>>;
}
