use std::{io, string::FromUtf8Error};
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VCardError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    UTF8Error(#[from] FromUtf8Error),
    #[error("{reason} - complete line is:\n{raw_line}")]
    InvalidLine {
        reason: &'static str,
        raw_line: String,
    },

    #[error("unexpected name {actual_name} - raw line is \n{raw_line}")]
    InvalidName {
        actual_name: String,
        raw_line: String,
    },

    #[error("expected one of the following values [{expected_values}] but got value {actual_value} - raw line is \n{raw_line}")]
    InvalidValue {
        expected_values: String,
        actual_value: String,
        raw_line: String,
    },

    #[error("Unknown type {given_type}")]
    UnknownType { given_type: String },

    #[error("Invalid PID parameter. Expected parameter to have the form digit[.digit] (e.g: 1 or 1.2) but got {provided}")]
    InvalidPID { provided: String },

    #[error("Invalid version {0}, only version 4.0 is valid")]
    InvalidVersion(String),

    #[error("Invalid gender {0}, expected one of (m,f,o,n,u)")]
    InvalidGenderError(String),

    #[error(transparent)]
    UrlParseError(#[from] ParseError),

    #[error("Unknown parameter {0}")]
    UnknownParameter(String),
}
