use std::{fmt::Display, str::FromStr};

use crate::errors::VCardError;

#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum Parameter {
    Label(String),
    Language(String),
    Value(ValueDataType),
    Pref(u8),
    AltId(String),
    Pid(Pid),
    Type(Vec<String>),
    MediaType(String),
    CalScale(String),
    SortAs(Vec<String>),
    Geo(String),
    TimeZone(String),
    Proprietary(String),
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label(l) => write!(f, "LABEL={}", l)?,
            Self::Language(l) => write!(f, "LANGUAGE={}", l)?,
            Self::Value(v) => write!(f, "VALUE={}", v.to_string())?,
            Self::Pref(p) => write!(f, "PREF={}", p)?,
            Self::AltId(a) => write!(f, "ALTID={}", a)?,
            Self::Pid(p) => write!(f, "PID={}", p)?,
            Self::Type(t) => write!(f, "TYPE={}", t.join(","))?,
            Self::MediaType(m) => write!(f, "MEDIATYPE={}", m)?,
            Self::CalScale(c) => write!(f, "CALSCALE={}", c)?,
            Self::SortAs(s) => write!(f, "SORT-AS={}", s.join(","))?,
            Self::Geo(g) => write!(f, "GEO={}", g)?,
            Self::TimeZone(t) => write!(f, "TZ={}", t)?,
            Self::Proprietary(p) => write!(f, "{}", p)?,
        }

        Ok(())
    }
}

const LANGUAGE: &str = "language";
const VALUE: &str = "value";
const PREF: &str = "pref";
const ALTID: &str = "altid";
const PID: &str = "pid";
const TYPE: &str = "type";
const MEDIATYPE: &str = "mediatype";
const CALSCALE: &str = "calscale";
const SORT_AS: &str = "sort-as";
const GEO: &str = "geo";
const TZ: &str = "tz";

impl FromStr for Parameter {
    type Err = VCardError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (k, v) = raw.split_once("=").ok_or_else(|| VCardError::InvalidLine {
            reason: "parameter has no = sign",
            raw_line: raw.into(),
        })?;
        let identifier = k.to_lowercase();
        let param = match &identifier[..] {
            LANGUAGE => Parameter::Language(v.into()),
            PREF => Parameter::Pref(v.parse()?),
            ALTID => Parameter::AltId(v.into()),
            PID => {
                let mut split = v.split(".");
                let first_digit = split
                    .next()
                    .map(u8::from_str)
                    .ok_or_else(|| VCardError::InvalidPID { provided: v.into() })??;
                let second_digit = if let Some(item) = split.next() {
                    Some(u8::from_str(item)?)
                } else {
                    None
                };
                Parameter::Pid(Pid {
                    first_digit,
                    second_digit,
                })
            }
            VALUE => Self::Value(ValueDataType::from_str(v)?),
            TYPE => Self::Type(v.split(",").map(String::from).collect()),
            MEDIATYPE => Self::MediaType(v.into()),
            CALSCALE => Self::CalScale(v.into()),
            SORT_AS => Self::SortAs(v.split(",").map(String::from).collect()),
            GEO => Self::Geo(v.into()),
            TZ => Self::TimeZone(v.into()),
            _ => Self::Proprietary(v.into()),
        };
        Ok(param)
    }
}


#[derive(Debug, PartialEq)]
pub struct Pid {
    pub first_digit: u8,
    pub second_digit: Option<u8>,
}

impl Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(d) = self.second_digit {
            write!(f, "{}.{}", self.first_digit, d)
        } else {
            write!(f, "{}", self.first_digit)
        }
    }
}

/// See https://datatracker.ietf.org/doc/html/rfc6350#section-5.2
#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum ValueDataType {
    #[strum(serialize = "uri")]
    Uri,
    #[strum(serialize = "text")]
    Text,
    #[strum(serialize = "date")]
    Date,
    #[strum(serialize = "time")]
    Time,
    #[strum(serialize = "date-time")]
    DateTime,
    #[strum(serialize = "date-and-or-time")]
    DateAndOrTime,
    #[strum(serialize = "timestamp")]
    Timestamp,
    #[strum(serialize = "boolean")]
    Boolean,
    #[strum(serialize = "integer")]
    Integer,
    #[strum(serialize = "float")]
    Float,
    #[strum(serialize = "utc-offset")]
    UtcOffset,
    #[strum(serialize = "language-tag")]
    LanguageTag,
    Proprietary(String),
}

impl Display for ValueDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proprietary(p) => write!(f, "{}", p),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

const URI: &str = "uri";
const TEXT: &str = "text";
const DATE: &str = "date";
const TIME: &str = "time";
const DATE_TIME: &str = "date-time";
const DATE_AND_OR_TIME: &str = "date-and-or-time";
const TIMESTAMP: &str = "timestamp";
const BOOLEAN: &str = "boolean";
const INTEGER: &str = "integer";
const FLOAT: &str = "float";
const UTC_OFFSET: &str = "utc-offset";
const LANGUAGE_TAG: &str = "language-tag";

impl FromStr for ValueDataType {
    type Err = VCardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s {
            URI => Self::Uri,
            TEXT => Self::Text,
            DATE => Self::Date,
            TIME => Self::Time,
            DATE_TIME => Self::DateTime,
            DATE_AND_OR_TIME => Self::DateAndOrTime,
            TIMESTAMP => Self::Timestamp,
            BOOLEAN => Self::Boolean,
            INTEGER => Self::Integer,
            FLOAT => Self::Float,
            UTC_OFFSET => Self::UtcOffset,
            LANGUAGE_TAG => Self::LanguageTag,
            _ => {
                if !s.starts_with("X-") && !s.starts_with("x-") {
                    return Err(VCardError::UnknownType {
                        given_type: s.into(),
                    });
                }
                Self::Proprietary(s.into())
            }
        };
        Ok(t)
    }
}