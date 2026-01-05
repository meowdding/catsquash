mod item;
mod model;
mod block_replacements;

use crate::json::item::JsonItemOptimizer;
use crate::json::model::JsonModelOptimizer;
use crate::json::block_replacements::JsonBlockReplacementOptimizer;
use crate::FileProcessor;
use json::JsonValue;
use std::path::Path;
use utils::error::{Result, SquashError};

pub struct JsonFileProcessor {
    processors: Vec<Box<dyn JsonProcessor>>,
}

impl JsonFileProcessor {
    pub fn new() -> Self {
        JsonFileProcessor {
            processors: vec![
                Box::new(JsonModelOptimizer::new()),
                Box::new(JsonItemOptimizer::new()),
                Box::new(JsonBlockReplacementOptimizer::new()),
            ],
        }
    }
}

pub(crate) trait JsonProcessor {
    fn can_process(&self, path: &Path, json: &JsonValue) -> bool;
    fn process(&self, path: &Path, json: &JsonValue) -> Result<JsonValue>;
}

impl FileProcessor for JsonFileProcessor {
    fn can_process(&self, path: &Path) -> bool {
        match path.extension() {
            None => false,
            Some(str) => match str.to_str().unwrap_or("").to_lowercase().as_str() {
                "json" => true,
                "mcmeta" => true,
                _ => false,
            },
        }
    }

    fn process(&self, input: Vec<u8>, path: &Path) -> Result<Vec<u8>> {
        let mut json = json::parse(
            String::from_utf8(input)
                .map_err(|x| SquashError::FileError {
                    error: x.to_string(),
                })?
                .as_str(),
        )
        .map_err(SquashError::failed_to_parse_json(path))?;

        for file_processor in &self.processors {
            if file_processor.can_process(path, &json) {
                return file_processor
                    .process(path, &mut json)
                    .map(|json| json.dump().into_bytes())
                    .map_err(SquashError::context(path));
            }
        }

        Ok(json.dump().into_bytes())
    }
}
