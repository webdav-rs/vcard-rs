use regex::{self, Regex};
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    char,
    collections::HashMap,
    io::{self, BufRead, Read},
    rc::Rc,
    str::FromStr,
};
#[macro_use]
use lazy_static;

#[macro_use]
use strum_macros;
use strum;

use errors::VCardError;
mod errors;

pub struct VCardReader<R: io::Read> {
    inner: io::BufReader<R>,
    buf: [u8; 2],
    has_leftover_bytes: bool,
    discard_buf: Rc<RefCell<Vec<u8>>>,
}

const CRLF: [u8; 2] = [b'\r', b'\n'];

pub struct ContentLine {
    pub group: Option<String>,
}

pub enum VersionValue {
    V3,
    V4,
}

pub enum ValueType {
    Uri,
    Text,
    Date,
    Time,
    DateTime,
    DateAndOrTime,
    Timestamp,
    Boolean,
    Integer,
    Float,
    UtcOffset,
    LanguageTag,
    Proprietary(String),
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

impl FromStr for ValueType {
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

pub struct Pid {
    pub first_digit: u8,
    pub second_digit: Option<u8>,
}

pub enum Kind {
    Individual, //  default
    Group,
    Org,
    Location,
    Proprietary(String),
}

pub enum Gender {
    Male,
    Femal,
    Other,
    None,
    Unknown,
}
#[derive(strum_macros::AsRefStr)]
pub enum Property {
    #[strum(serialize = "begin")]
    Begin { value: String },
    #[strum(serialize = "end")]
    End { value: String },
    #[strum(serialize = "version")]
    Version { value: VersionValue },
    #[strum(serialize = "source")]
    Source {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "kind")]
    Kind {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: Kind, // Individual is default
    },
    #[strum(serialize = "fn")]
    FN {
        parameters: Vec<Parameter>,
        group: Option<String>,
        value: String,
    },
    #[strum(serialize = "n")]
    N {
        parameters: Vec<Parameter>,
        surenames: Vec<String>,
        given_names: Vec<String>,
        additional_names: Vec<String>,
        honorific_prefixes: Vec<String>,
        honorific_suffixes: Vec<String>,
    },
    #[strum(serialize = "nickname")]
    NickName {
        group: Option<String>,
        value: Vec<String>,
        parameters: Vec<Parameter>,
    },
    #[strum(serialize = "photo")]
    Photo {
        group: Option<String>,
        value: url::Url,
        parameters: Vec<Parameter>,
    },
    #[strum(serialize = "bday")]
    BDay {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "anniversary")]
    Anniversary {
        group: Option<String>,
        parameters: Vec<Parameter>,
    },
    #[strum(serialize = "gender")]
    Gender {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: Option<Gender>,
        identity_component: Option<String>,
    },
    #[strum(serialize = "adr")]
    Adr {
        group: Option<String>,
        parameters: Vec<Parameter>,
        po_box: Vec<String>,
        extended_address: Vec<String>,
        street: Vec<String>,
        city: Vec<String>,
        region: Vec<String>,
        postal_code: Vec<String>,
        country: Vec<String>,
    },
    #[strum(serialize = "tel")]
    Tel {
        parameters: Vec<Parameter>,
        group: Option<String>,
        value: String,
    },
    #[strum(serialize = "email")]
    Email {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "impp")]
    Impp {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "lang")]
    Lang {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "tz")]
    Tz {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "geo")]
    Geo {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "title")]
    Title {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "role")]
    Role {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "logo")]
    Logo {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "org")]
    Org {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: Vec<String>,
    },
    #[strum(serialize = "member")]
    Member {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "related")]
    Related {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "categories")]
    Categories {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: Vec<String>,
    },
    #[strum(serialize = "note")]
    Note {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "prodid")]
    ProdId {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "rev")]
    Rev {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "sound")]
    Sound {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "uid")]
    Uid {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "clientidmap")]
    ClientIdMap {
        group: Option<String>,
        parameters: Vec<Parameter>,
        pid: String,
        global_identifier: url::Url,
    },
    #[strum(serialize = "url")]
    Url {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "key")]
    Key {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: String,
    },
    #[strum(serialize = "fburl")]
    FbUrl {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "caladuri")]
    CalAdUri {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "caluri")]
    CalUri {
        group: Option<String>,
        parameters: Vec<Parameter>,
        value: url::Url,
    },
    #[strum(serialize = "xml")]
    Xml {
        value: String,
        group: Option<String>,
        parameters: Vec<Parameter>,
    },
    Proprietary {
        name: String,
        group: Option<String>,
        value: String,
        parameters: Vec<Parameter>,
    },
}

impl FromStr for Property {
    type Err = VCardError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let captures = if let Some(captures) = RE.captures(&line) {
            captures
        } else {
            return Err(VCardError::InvalidLine {
                reason: "does not match property pattern",
                raw_line: line.into(),
            });
        };
        let group = captures
            .name("group")
            .map(|m| m.as_str().trim_end_matches(".").to_string());
        let name =
            captures
                .name("name")
                .map(|m| m.as_str())
                .ok_or_else(|| VCardError::InvalidLine {
                    reason: "no name found",
                    raw_line: line.into(),
                })?;
        let parameter = captures.name("parameter").map(|m| m.as_str());
        let value =
            captures
                .name("value")
                .map(|m| m.as_str())
                .ok_or_else(|| VCardError::InvalidLine {
                    reason: "no value found",
                    raw_line: line.into(),
                })?;
        let name = name.to_lowercase();

        let prop = match &name[..] {
            _ => {
                if !name.starts_with("X-") && !name.starts_with("x-") {
                    return Err(VCardError::InvalidName {
                        actual_name: name.into(),
                        raw_line: line.into(),
                    });
                }

                let parameters = if let Some(raw_parameter) = parameter {
                    parse_parameters(raw_parameter)?
                } else {
                    Vec::new()
                };
                Property::Proprietary {
                    name,
                    value: value.into(),
                    group,
                    parameters,
                }
            }
        };
        Ok(prop)
    }
}

pub enum Parameter {
    Language(String),
    Value(ValueType),
    Pref(u8),
    AltId(String),
    Pid {
        first_digit: u8,
        second_digit: Option<u8>,
    },
    Type(String),
    MediaType(String),
    CalScale(String),
    SortAs(Vec<String>),
    Geo(String),
    TimeZone(String),
    Proprietary(String),
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
            VALUE => Parameter::Value(v.parse()?),
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
                Parameter::Pid {
                    first_digit,
                    second_digit,
                }
            }
            TYPE => Self::Type(v.into()),
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

fn parse_parameters(raw: &str) -> Result<Vec<Parameter>, VCardError> {
    raw.trim_start_matches(";")
        .split(";")
        .map(Parameter::from_str)
        .collect::<Result<Vec<Parameter>, VCardError>>()
}

lazy_static::lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<group>.+\.)?(?P<name>[^;]+)(?P<parameter>;.+)*:(?P<value>.+)").unwrap();
}

impl<R: io::Read> VCardReader<R> {
    pub fn new(input: R) -> Self {
        Self {
            inner: io::BufReader::new(input),
            has_leftover_bytes: false,
            buf: [0; 2],
            discard_buf: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn read_property(&mut self) -> Result<Property, VCardError> {
        let line = self.read_logical_line()?;
        Property::from_str(&line[..])
    }
    fn read_logical_line(&mut self) -> Result<String, VCardError> {
        let mut logical_line_buf = Vec::new();

        // append leftover bytes that we have falsely consumed for checking a logical line continuation.
        if !self.has_leftover_bytes {
            for i in self.buf {
                logical_line_buf.push(i);
            }
            self.has_leftover_bytes = false;
        }

        loop {
            self.read_until_crlf(&mut logical_line_buf)?;

            // read the next two bytes. If the next byte continues with a whicespace char (space (U+0020) or horizontal tab (U+0009))
            // it counts as a logical continuation of this line.
            // If not, we indicate that those two bytes belong to the next line and return the line as is.
            if let Err(e) = self.inner.read_exact(&mut self.buf) {
                match e.kind() {
                    // this means, there are no more bytes left. Most likely, this means we reached the END:VCARD line.
                    io::ErrorKind::UnexpectedEof => {
                        return Ok(String::from_utf8(logical_line_buf)?);
                    }
                    _ => return Err(VCardError::Io(e)),
                }
            }

            if self.buf[0] != b' ' && self.buf[0] != b'\t' {
                self.has_leftover_bytes = true;
                return Ok(String::from_utf8(logical_line_buf)?);
            }

            // The spec tells us that we have to ensure that the start of a continued line does not have two whitespace characters in a  row
            match self.buf[1] {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.discard_line()?;
                }
                _ => {
                    logical_line_buf.push(self.buf[1]);
                }
            }
        }
    }
    fn discard_line(&mut self) -> Result<(), VCardError> {
        let rc = Rc::clone(&self.discard_buf.clone());
        self.read_until_crlf(&mut rc.as_ref().borrow_mut())?;
        Ok(())
    }

    fn read_until_crlf(&mut self, mut buf: &mut Vec<u8>) -> Result<(), VCardError> {
        loop {
            self.inner.read_until(b'\n', &mut buf)?;
            if buf.ends_with(&CRLF) {
                // remove CRLF
                buf.pop();
                buf.pop();
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
