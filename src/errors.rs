use std::{io, string::FromUtf8Error};
use thiserror::Error;

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
    InvalidPID {
        provided: String,
    }   
}
