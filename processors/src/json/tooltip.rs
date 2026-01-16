use json::JsonValue;
use regex::Regex;
use utils::error::Result;
use utils::{get_object};

use crate::json::JsonProcessor;

pub struct JsonTooltipOptimizer {
    regex: Regex,
}

impl JsonTooltipOptimizer {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"assets/catharsis/tooltip.json$").unwrap(),
        }
    }

    fn optimize(&self, object: &mut json::object::Object) -> Result<JsonValue> {
        utils::identifier(object, "type")?;
        match utils::get_field_as_string(object, "type")?.as_str() {
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
                object.insert("entries", JsonValue::Array(new_vec));

                match utils::get_optional_field_as_object(&object, "fallback") {
                    Some(value) => object.insert("fallback", self.optimize(&mut value?)?),
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(JsonValue::Object(object.clone()))
    }
}

impl JsonProcessor for JsonTooltipOptimizer {
    fn can_process(&self, path: &std::path::Path, _: &json::JsonValue) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

    fn process(
        &self,
        _: &std::path::Path,
        object: &json::JsonValue,
    ) -> utils::error::Result<json::JsonValue> {
        let mut data = get_object(&object)?.clone();
        self.optimize(&mut data)
    }
}
