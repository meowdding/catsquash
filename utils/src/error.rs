use meta::error::CatError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone)]
pub enum SquashError {
    // packing error
    PackingError(CatError),
    Context { path: String, other: Box<SquashError> },

    // os level errors
    FailedToRead { path: String, error: String },
    FailedToWrite { path: String, error: String },
    FileError { error: String },

    // data errors
    FailedToParseJson { path: String, error: String },
    ExpectedType { expected: String, actual: String },
    FieldError { field: String, other: Box<SquashError> },
}

impl SquashError {
    fn failed_helper<T, E>(
        result: std::result::Result<T, E>,
        path: &Path,
        mapper: fn(String, String) -> SquashError,
    ) -> Result<T>
    where
        E: Error,
    {
        result.map_err(|err| mapper(path.display().to_string(), err.to_string()))
    }

    pub fn failed_to_parse_json<T, E>(result: std::result::Result<T, E>, path: &Path) -> Result<T>
    where
        E: Error,
    {
        Self::failed_helper(result, path, |path, error| SquashError::FailedToParseJson {
            path,
            error,
        })
    }

    pub fn context<T>(result: Result<T>, path: &Path) -> Result<T> {
        result.map_err(|err| SquashError::Context {
            path: path.display().to_string(),
            other: Box::new(err),
        })
    }
}

impl Into<i32> for SquashError {
    fn into(self) -> i32 {
        match self {
            SquashError::PackingError(cat) => cat.into(),
            SquashError::Context { other, .. } => <SquashError>::into(*other),
            SquashError::FieldError { other, .. } => <SquashError>::into(*other),

            SquashError::FailedToRead { .. } => -400,
            SquashError::FailedToWrite { .. } => -401,
            SquashError::FileError{ .. } => -402,

            SquashError::FailedToParseJson { .. } => 200,
            SquashError::ExpectedType { .. } => 201,
        }
    }
}
impl Display for SquashError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SquashError::PackingError(cat) => {
                f.write_str("Failed to squash due to packing error: ")?;
                std::fmt::Display::fmt(&cat, f)
            }
            SquashError::Context { path, other} => {
                f.write_str("An error occurred while processing '")?;
                std::fmt::Display::fmt(path, f)?;
                f.write_str("'\n")?;
                Debug::fmt(other, f)
            }
            SquashError::FieldError { field, other} => {
                f.write_str("An error occurred at '")?;
                std::fmt::Display::fmt(field, f)?;
                f.write_str("'\n")?;
                Debug::fmt(other, f)
            }

            SquashError::FailedToRead { path, error } => {
                f.write_str("Failed to read file '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }
            SquashError::FailedToWrite { path, error } => {
                f.write_str("Failed to write file '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }
            SquashError::FileError {  error } => {
                f.write_str("An unknown file error occurred due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }

            SquashError::FailedToParseJson { path, error } => {
                f.write_str("Failed to parse json file '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }
            SquashError::ExpectedType { expected, actual } => {
                f.write_str("Expected '")?;
                std::fmt::Display::fmt(expected, f)?;
                f.write_str("' but got '")?;
                std::fmt::Display::fmt(actual, f)?;
                f.write_str("'!")
            }
        }
    }
}

impl Into<SquashError> for meta::error::CatError {
    fn into(self) -> SquashError {
        SquashError::PackingError(self)
    }
}

pub trait SquashErrorWrappable<T> {
    fn wrap(self) -> Result<T>;
}

impl<T> SquashErrorWrappable<T> for std::result::Result<T, CatError> {
    fn wrap(self) -> std::result::Result<T, SquashError> {
        self.map_err(|err| SquashError::PackingError(err))
    }
}

impl<T> Into<std::result::Result<T, SquashError>> for SquashError {
    fn into(self) -> std::result::Result<T, SquashError> {
        Err(self)
    }
}

impl Error for SquashError {}

pub type Result<T> = std::result::Result<T, SquashError>;
