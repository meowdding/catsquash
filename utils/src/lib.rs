use crate::error::SquashError;
use json::JsonValue;
use json::object::Object;

pub mod error;

pub fn identifier(object: &mut json::object::Object, field: &str) -> error::Result<()> {
    defaulted_identifier(object, field, "minecraft")
}
pub fn value_identifier(object: &mut json::JsonValue, field: &str) -> error::Result<()> {
    assume_object(object, |object| {
        defaulted_identifier(object, field, "minecraft")
    })
}

pub fn assume_object(
    value: &mut json::JsonValue,
    consumer: impl Fn(&mut json::object::Object) -> error::Result<()>,
) -> error::Result<()> {
    match *value {
        json::JsonValue::Object(ref mut object) => Ok(consumer(object)?),
        _ => expected_object(value),
    }
}

pub fn expected_object<T>(value: &json::JsonValue) -> error::Result<T> {
    Err(SquashError::ExpectedType {
        expected: "object".to_string(),
        actual: type_name(value),
    })
}

fn wrap_field_error<T>(result: error::Result<T>, field: &str) -> error::Result<T> {
    result.map_err(|err| SquashError::FieldError {
        field: field.to_string(),
        other: Box::new(err),
    })
}

pub fn get_field_as_object(value: &Object, field: &str) -> error::Result<Object> {
    wrap_field_error(get_option_object(value.get(field)), field)
}

pub fn get_field_as_array(value: &Object, field: &str) -> error::Result<Vec<JsonValue>> {
    wrap_field_error(get_option_array(value.get(field)), field)
}

pub fn get_field_as_string(value: &Object, field: &str) -> error::Result<String> {
    wrap_field_error(get_option_string(value.get(field)), field)
}

pub fn get_optional_field_as_object(value: &Object, field: &str) -> Option<error::Result<Object>> {
    value
        .get(field)
        .and_then(|entry| Some(wrap_field_error(get_object(entry), field)))
}

pub fn get_optional_field_as_array(
    value: &Object,
    field: &str,
) -> Option<error::Result<Vec<JsonValue>>> {
    value
        .get(field)
        .and_then(|entry| Some(wrap_field_error(get_array(entry), field)))
}

pub fn get_optional_field_as_string(value: &Object, field: &str) -> Option<error::Result<String>> {
    value
        .get(field)
        .and_then(|entry| Some(wrap_field_error(get_string(entry), field)))
}

pub fn get_object(value: &json::JsonValue) -> error::Result<Object> {
    match value {
        json::JsonValue::Object(object) => Ok(object.clone()),
        _ => Err(SquashError::ExpectedType {
            expected: "object".to_string(),
            actual: type_name(value),
        }),
    }
}
pub fn get_option_object(value: Option<&json::JsonValue>) -> error::Result<Object> {
    match value {
        Some(json::JsonValue::Object(object)) => Ok(object.clone()),
        _ => Err(SquashError::ExpectedType {
            expected: "object".to_string(),
            actual: match value {
                Some(value) => type_name(value),
                None => "None".to_string(),
            },
        }),
    }
}

pub fn get_array(value: &json::JsonValue) -> error::Result<Vec<JsonValue>> {
    match value {
        json::JsonValue::Array(array) => Ok(array.clone()),
        _ => Err(SquashError::ExpectedType {
            expected: "array".to_string(),
            actual: type_name(value),
        }),
    }
}
pub fn get_option_array(value: Option<&json::JsonValue>) -> error::Result<Vec<JsonValue>> {
    match value {
        Some(json::JsonValue::Array(array)) => Ok(array.clone()),
        _ => Err(SquashError::ExpectedType {
            expected: "array".to_string(),
            actual: match value {
                Some(value) => type_name(value),
                None => "None".to_string(),
            },
        }),
    }
}

pub fn get_string(value: &json::JsonValue) -> error::Result<String> {
    match value {
        json::JsonValue::String(string) => Ok(string.clone()),
        json::JsonValue::Short(string) => Ok(string.to_string()),
        _ => Err(SquashError::ExpectedType {
            expected: "string or short".to_string(),
            actual: type_name(value),
        }),
    }
}
pub fn get_option_string(value: Option<&json::JsonValue>) -> error::Result<String> {
    match value {
        Some(json::JsonValue::String(string)) => Ok(string.clone()),
        Some(json::JsonValue::Short(string)) => Ok(string.to_string()),
        _ => Err(SquashError::ExpectedType {
            expected: "string or short".to_string(),
            actual: match value {
                Some(value) => type_name(value),
                None => "None".to_string(),
            },
        }),
    }
}

fn type_name(value: &json::JsonValue) -> String {
    match value {
        json::JsonValue::Null => "null",
        json::JsonValue::Short(_) => "short",
        json::JsonValue::String(_) => "string",
        json::JsonValue::Number(_) => "number",
        json::JsonValue::Boolean(_) => "boolean",
        json::JsonValue::Object(_) => "object",
        json::JsonValue::Array(_) => "array",
    }
    .to_string()
}

pub fn defaulted_identifier(
    object: &mut json::object::Object,
    field: &str,
    default_namespace: &str,
) -> error::Result<()> {
    match get_optional_field_as_string(object, field) {
        Some(value) => {
            let value = value?;
            let mut namespace = default_namespace.to_string();
            namespace.push(':');
            let length = namespace.len();
            if value.starts_with(namespace.as_str()) && value.len() > length {
                object.insert(
                    field,
                    JsonValue::String(String::from_iter(value.chars().skip(length))),
                )
            }
        }
        _ => {}
    };

    Ok(())
}

#[derive(Clone)]
pub struct SquashOptions {
    pub oxipng: bool,
    pub gzip: bool,
    pub verbose: bool,
}
