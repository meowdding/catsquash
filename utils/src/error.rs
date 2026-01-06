use meta::error::CatError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone)]
pub enum SquashError {
    // packing error
    PackingError(CatError),
    Context {
        path: String,
        other: Box<SquashError>,
    },

    // os level errors
    FailedToRead {
        path: String,
        error: String,
    },
    FailedToWrite {
        path: String,
        error: String,
    },
    FileError {
        error: String,
    },

    // data errors
    FailedToParseJson {
        path: String,
        error: String,
    },
    ExpectedType {
        expected: String,
        actual: String,
    },
    FieldError {
        field: String,
        other: Box<SquashError>,
    },
    FailedToParsePng {
        path: String,
        error: String,
    },
    InvalidPngFile {
        path: String,
        reason: String,
    },
    OxipngError {
        path: String,
        error: String,
    },
}

impl SquashError {
    pub fn failed_to_parse_json<E>(path: &Path) -> impl Fn(E) -> SquashError
    where
        E: Error,
    {
        return |error| SquashError::FailedToParseJson {
            error: error.to_string(),
            path: path.display().to_string(),
        };
    }

    pub fn failed_to_parse_png<E>(path: &Path) -> impl Fn(E) -> SquashError
    where
        E: Error,
    {
        return |error| SquashError::FailedToParsePng {
            error: error.to_string(),
            path: path.display().to_string(),
        };
    }

    pub fn context(path: &Path) -> impl Fn(SquashError) -> SquashError {
        return |error| SquashError::Context {
            other: Box::new(error),
            path: path.display().to_string(),
        };
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
            SquashError::FileError { .. } => -402,

            SquashError::FailedToParseJson { .. } => 200,
            SquashError::FailedToParsePng { .. } => 201,
            SquashError::InvalidPngFile { .. } => 202,
            SquashError::ExpectedType { .. } => 203,
            SquashError::OxipngError { .. } => 204,
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
            SquashError::Context { path, other } => {
                f.write_str("An error occurred while processing '")?;
                std::fmt::Display::fmt(path, f)?;
                f.write_str("'\n")?;
                Debug::fmt(other, f)
            }
            SquashError::FieldError { field, other } => {
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
            SquashError::FileError { error } => {
                f.write_str("An unknown file error occurred due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }

            SquashError::FailedToParseJson { path, error } => {
                f.write_str("Failed to parse json file '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }
            SquashError::FailedToParsePng { path, error } => {
                f.write_str("Failed to parse png file '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(&error, f)
            }
            SquashError::InvalidPngFile { path, reason } => {
                f.write_str("Png file at '")?;
                std::fmt::Display::fmt(&path, f)?;
                f.write_str("' is invalid beacause of: ")?;
                std::fmt::Display::fmt(&reason, f)
            }
            SquashError::ExpectedType { expected, actual } => {
                f.write_str("Expected '")?;
                std::fmt::Display::fmt(expected, f)?;
                f.write_str("' but got '")?;
                std::fmt::Display::fmt(actual, f)?;
                f.write_str("'!")
            }
            SquashError::OxipngError { path, error } => {
                f.write_str("Failed to apply oxipng on '")?;
                std::fmt::Display::fmt(path, f)?;
                f.write_str("' due to: ")?;
                std::fmt::Display::fmt(error, f)
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
