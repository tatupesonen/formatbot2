use axum::{http::StatusCode, response::{IntoResponse, Response}};
use std::{fmt::Error, str::FromStr};
use thiserror::Error;

pub trait Formatter {
    fn format(code: &str) -> Result<String, FormatError>;
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Formatter failed: {0}")]
    FormatterFailed(std::io::Error),
    #[error("{0}")]
    FormatterError(String),
    #[error("Formatter replied with status code: {0}")]
    FormatterStatusCode(i32),
    #[error("Cannot create temporary file")]
    CannotCreateTempFile(#[from] std::io::Error),
    #[error("Unable to read formatted file")]
    CannotReadFileContents(std::io::Error),
    #[error("Formatter error output was not valid UTF-8")]
    FormatterOutputNotUTF8,
}

impl IntoResponse for FormatError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            format!("{}", self),
        )
            .into_response()
    }
}

enum Language {
    Rust,
    PHP,
    JavaScript,
    TypeScript,
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub code: Option<&'a str>,
    pub language: Option<&'a str>,
}

#[derive(Error, Debug)]
enum LanguageParseError {
    #[error("Language not found")]
    LanguageNotFound,
}

impl FromStr for Language {
    type Err = LanguageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rs" | "rust" => Ok(Language::Rust),
            "php" => Ok(Language::PHP),
            "js" | "jsx" => Ok(Language::JavaScript),
            "ts" | "tsx" => Ok(Language::TypeScript),
            _ => Err(LanguageParseError::LanguageNotFound),
        }
    }
}
