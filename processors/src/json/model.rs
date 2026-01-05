use crate::json::JsonProcessor;
use crate::json::JsonValue;
use crate::Path;
use regex::Regex;

pub struct JsonModelOptimizer {
    regex: Regex,
}

impl JsonModelOptimizer {
    pub fn new() -> JsonModelOptimizer {
        JsonModelOptimizer {
            regex: Regex::new(r"assets/.+/models/.+\.json$").unwrap()
        }
    }
}

impl JsonProcessor for JsonModelOptimizer {
    fn can_process(&self, path: &Path, _: &JsonValue) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

    fn process(&self, _: &Path, object: &JsonValue) -> utils::error::Result<JsonValue> {
        let mut object = object.clone();
        utils::value_identifier(&mut object, "parent")?;
        Ok(object)
    }
}
