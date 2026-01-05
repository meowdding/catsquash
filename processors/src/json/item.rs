use crate::json::JsonProcessor;
use json::JsonValue;
use regex::Regex;
use std::path::Path;
use utils::error::Result;
use utils::{get_array, get_object, get_option_object};

pub struct JsonItemOptimizer {
    regex: Regex,
}

impl JsonItemOptimizer {
    pub fn new() -> JsonItemOptimizer {
        JsonItemOptimizer {
            regex: Regex::new(r"^(?:.+/)?assets/.+/items/.+\.json$").unwrap(),
        }
    }

    fn optimize(&self, object: &mut json::object::Object) -> Result<JsonValue> {
        utils::identifier(object, "type")?;
        match utils::get_field_as_string(object, "type")?.as_str() {
            "model" => match object.get("tints") {
                Some(data) => {
                    let vec = get_array(data)?;
                    let mut new_vec = Vec::<JsonValue>::new();

                    for tint in vec {
                        let mut new_tint = tint.clone();
                        utils::value_identifier(&mut new_tint, "type")?;
                        new_vec.push(new_tint)
                    }

                    object.insert("tints", JsonValue::Array(new_vec))
                }
                _ => {}
            },
            "condition" => {
                match utils::get_optional_field_as_object(&object, "on_true") {
                    Some(data) => {
                        object.insert("on_true", self.optimize(&mut data?)?);
                    }
                    _ => {}
                };
                match utils::get_optional_field_as_object(&object, "on_false") {
                    Some(data) => {
                        object.insert("on_false", self.optimize(&mut data?)?);
                    }
                    _ => {}
                };
            }
            "select" => {
                utils::identifier(object, "property")?;
                let mut new_vec = Vec::<JsonValue>::new();
                for value in utils::get_field_as_array(object, "cases")? {
                    let mut case = value.clone();
                    let mut object = get_object(&mut case)?;

                    object.insert(
                        "model",
                        self.optimize(&mut utils::get_field_as_object(&object, "model")?)?,
                    );

                    new_vec.push(JsonValue::Object(object));
                }

                object.insert("cases", JsonValue::Array(new_vec))
            }
            "range_dispatch" => {
                utils::identifier(object, "property")?;
                let mut new_vec = Vec::<JsonValue>::new();
                for value in utils::get_field_as_array(object, "entries")? {
                    let mut case = value.clone();
                    let mut object = get_object(&mut case)?;

                    object.insert(
                        "model",
                        self.optimize(&mut utils::get_field_as_object(&object, "model")?)?,
                    );

                    new_vec.push(JsonValue::Object(object));
                }
                object.insert("property", JsonValue::Array(new_vec));

                match utils::get_optional_field_as_object(&object, "fallback") {
                    Some(value) => {
                        object.insert("fallback", self.optimize(&mut value?)?)
                    }
                    _ => {}
                }
            }
            "composite" => {
                let mut new_vec = Vec::<JsonValue>::new();
                for value in utils::get_field_as_array(&object, "models")? {
                    let mut model = value.clone();

                    new_vec.push(self.optimize(&mut get_object(&mut model)?)?);
                }
                object.insert("models", JsonValue::Array(new_vec));

            }
            _ => {}
        }

        Ok(JsonValue::Object(object.clone()))
    }
}

impl JsonProcessor for JsonItemOptimizer {
    fn can_process(&self, path: &Path, _: &JsonValue) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

    fn process(&self, _: &Path, object: &JsonValue) -> Result<JsonValue> {
        if object.has_key("model") {
            let mut data = get_object(&object)?.clone();

            let mut model = utils::get_field_as_object(&data, "model")?;
            data.insert("model", self.optimize(&mut model)?);

            return Ok(JsonValue::Object(data));
        }
        Ok(object.clone())
    }
}
