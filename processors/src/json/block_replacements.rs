use crate::json::JsonProcessor;
use json::JsonValue;
use regex::Regex;
use std::path::Path;
use utils::error::Result;
use utils::get_object;

pub struct JsonBlockReplacementOptimizer {
    regex: Regex,
}

impl JsonBlockReplacementOptimizer {
    pub fn new() -> JsonBlockReplacementOptimizer {
        JsonBlockReplacementOptimizer {
            regex: Regex::new(r"^(?:[^/]+/)?assets/[^/]+/catharsis/block_replacements/.+?\.json$")
                .unwrap(),
        }
    }

    fn optimize_optional_field(
        &self,
        object: &mut json::object::Object,
        field: &str,
    ) -> Result<()> {
        match utils::get_optional_field_as_object(&object, field) {
            Some(data) => {
                object.insert(field, self.optimize(&mut data?)?);
            }
            _ => {}
        };

        Ok(())
    }

    fn optimize_field(&self, object: &mut json::object::Object, field: &str) -> Result<()> {
        object.insert(
            field,
            self.optimize(&mut utils::get_field_as_object(&object, field)?)?,
        );

        Ok(())
    }

    fn optimize(&self, object: &mut json::object::Object) -> Result<JsonValue> {
        utils::defaulted_identifier(object, "type", "catharsis")?;
        match utils::get_field_as_string(object, "type")?.as_str() {
            "random" => {}
            "condition" => {
                self.optimize_field(object, "definition")?;
                self.optimize_optional_field(object, "fallback")?;
            }
            "per_area" => {
                let mut entries = utils::get_field_as_object(object, "entries")?;
                for (key, _) in entries.clone().iter() {
                    entries.insert(key, self.optimize(&mut utils::get_field_as_object(&entries, key)?)?)
                }
            }
            "conditional" => {
                self.optimize_field(object, "definition")?;
                self.optimize_optional_field(object, "fallback")?;
            }
            "select" => {
                let mut new_vec = Vec::<JsonValue>::new();
                for mut value in utils::get_field_as_array(&object, "definitions")? {
                    new_vec.push(self.optimize(&mut utils::get_object(&mut value)?)?)
                }
                object.insert("definitions", JsonValue::Array(new_vec));
                self.optimize_optional_field(object, "fallback")?;
            }
            _ => {}
        }

        Ok(JsonValue::Object(object.clone()))
    }
}

impl JsonProcessor for JsonBlockReplacementOptimizer {
    fn can_process(&self, path: &Path, _: &JsonValue) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

    fn process(&self, path: &Path, object: &JsonValue) -> Result<JsonValue> {
        let mut data = get_object(object)?.clone();
        return Ok(self.optimize(&mut data)?);
    }
}
