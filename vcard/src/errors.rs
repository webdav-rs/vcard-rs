use std::{io, str::Utf8Error, string::FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VCardError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    FromUTF8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    UTF8Error(#[from] Utf8Error),
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

    #[error("Invalid version {0}, only version 3.0 and 4.0 are valid")]
    InvalidVersion(String),

    #[error("Invalid gender {0}, expected one of (m,f,o,n,u)")]
    InvalidGenderError(String),

    #[error("Unknown parameter {0}")]
    UnknownParameter(String),

    #[error("Exceeded maximum logical line length of {0}")]
    MaxLineLengthExceeded(u64),

    #[error("first property of a vcard must be BEGIN:VCARD")]
    InvalidBeginProperty,

    #[error("second property of a vcard must be VERSION:3.0 or VERSION:4.0")]
    InvalidVersionProperty,

    #[error("last property of a vcard must be END:VCARD")]
    InvalidEndProperty,

    #[error("only {expected} amount of {property} are valid in a vcard")]
    InvalidCardinality { expected: u64, property: String },

    #[error("expected item to have altid {expected_altid}, but got {actual_altid}")]
    InvalidAltID {
        expected_altid: String,
        actual_altid: String,
    },
    #[error("invalid syntax for property {property}: {message}")]
    InvalidSyntax { message: String, property: String },
}
