use lazy_static;
use regex::{self, Regex};
use std::{
    cell::RefCell,
    io::{self, BufRead, Read},
    rc::Rc,
    str::FromStr,
};

use strum_macros;

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

#[derive(Debug, PartialEq)]
pub enum VersionValue {
    V3,
    V4,
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum ValueType {
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

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum Kind {
    #[strum(serialize = "individual")]
    Individual, //  default
    #[strum(serialize = "group")]
    Group,
    #[strum(serialize = "org")]
    Org,
    #[strum(serialize = "location")]
    Location,
    Proprietary(String),
}

impl FromStr for Kind {
    type Err = VCardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match &s.to_lowercase()[..] {
            "individual" => Self::Individual,
            "group" => Self::Group,
            "org" => Self::Org,
            "location" => Self::Location,
            _ => Self::Proprietary(s.into()),
        };
        Ok(result)
    }
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum Gender {
    #[strum(serialize = "m")]
    Male,
    #[strum(serialize = "f")]
    Female,
    #[strum(serialize = "o")]
    Other,
    #[strum(serialize = "n")]
    None,
    #[strum(serialize = "u")]
    Unknown,
}

impl FromStr for Gender {
    type Err = VCardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let g = match &s.to_lowercase()[..] {
            "m" => Self::Male,
            "f" => Self::Female,
            "o" => Self::Other,
            "n" => Self::None,
            "u" => Self::Unknown,
            _ => return Err(VCardError::InvalidGenderError(s.into())),
        };
        Ok(g)
    }
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
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
        group: Option<String>,
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
    ClientPidMap {
        group: Option<String>,
        parameters: Vec<Parameter>,
        pid: u8,
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
        let value = captures
            .name("value")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| VCardError::InvalidLine {
                reason: "no value found",
                raw_line: line.into(),
            })?;
        let name = name.to_lowercase();
        let parameters = if let Some(raw_parameter) = parameter {
            parse_parameters(raw_parameter)?
        } else {
            Vec::new()
        };
        let prop =
            match &name[..] {
                "begin" => Self::Begin { value },
                "end" => Self::End { value },
                "version" => {
                    if value != "4.0" {
                        return Err(VCardError::InvalidVersion(value));
                    }
                    Self::Version {
                        value: VersionValue::V4,
                    }
                }
                "source" => Self::Source {
                    group,
                    parameters,
                    value,
                },
                "kind" => Self::Kind {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "fn" => Self::FN {
                    parameters,
                    group,
                    value,
                },
                "n" => {
                    let mut split = value
                        .split(";")
                        .map(|item| item.split(";").map(String::from).collect::<Vec<String>>());
                    let surenames = split.next().unwrap_or_else(Vec::new);
                    let given_names = split.next().unwrap_or_else(Vec::new);
                    let additional_names = split.next().unwrap_or_else(Vec::new);
                    let honorific_prefixes = split.next().unwrap_or_else(Vec::new);
                    let honorific_suffixes = split.next().unwrap_or_else(Vec::new);
                    Self::N {
                        additional_names,
                        honorific_prefixes,
                        honorific_suffixes,
                        given_names,
                        surenames,
                        group,
                        parameters,
                    }
                }
                "nickname" => Self::NickName {
                    group,
                    parameters,
                    value: value.split(",").map(String::from).collect(),
                },
                "photo" => Self::Photo {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "bday" => Self::BDay {
                    group,
                    parameters,
                    value,
                },
                "anniversary" => Self::Anniversary { group, parameters },
                "gender" => {
                    let mut split = value.split(";");
                    let value = if let Some(r) = split.next().map(Gender::from_str) {
                        Some(r?)
                    } else {
                        None
                    };
                    let identity_component = split.next().map(String::from);
                    Self::Gender {
                        group,
                        parameters,
                        value,
                        identity_component,
                    }
                }
                "adr" => {
                    let mut split = value
                        .split(";")
                        .map(|item| item.split(",").map(String::from).collect::<Vec<String>>());
                    let po_box = split.next().unwrap_or_else(|| Vec::new());
                    let extended_address = split.next().unwrap_or_else(|| Vec::new());
                    let street = split.next().unwrap_or_else(|| Vec::new());
                    let city = split.next().unwrap_or_else(|| Vec::new());
                    let region = split.next().unwrap_or_else(|| Vec::new());
                    let postal_code = split.next().unwrap_or_else(|| Vec::new());
                    let country = split.next().unwrap_or_else(|| Vec::new());
                    Self::Adr {
                        region,
                        po_box,
                        city,
                        group,
                        parameters,
                        extended_address,
                        street,
                        postal_code,
                        country,
                    }
                }
                "tel" => Self::Tel {
                    group,
                    parameters,
                    value,
                },
                "email" => Self::Email {
                    group,
                    parameters,
                    value,
                },
                "impp" => Self::Impp {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "lang" => Self::Lang {
                    group,
                    parameters,
                    value,
                },
                "tz" => Self::Tz {
                    group,
                    parameters,
                    value,
                },
                "geo" => Self::Geo {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "title" => Self::Title {
                    group,
                    parameters,
                    value,
                },
                "role" => Self::Role {
                    group,
                    parameters,
                    value,
                },
                "categories" => Self::Categories {
                    group,
                    parameters,
                    value: value.split(";").map(String::from).collect(),
                },
                "org" => Self::Org {
                    parameters,
                    group,
                    value: value.split(";").map(String::from).collect(),
                },
                "member" => Self::Member {
                    parameters,
                    group,
                    value: value.parse()?,
                },
                "related" => Self::Related {
                    parameters,
                    group,
                    value,
                },
                "logo" => Self::Logo {
                    parameters,
                    group,
                    value: value.parse()?,
                },
                "note" => Self::Note {
                    parameters,
                    group,
                    value,
                },
                "prodid" => Self::ProdId {
                    parameters,
                    group,
                    value,
                },
                "rev" => Self::Rev {
                    parameters,
                    group,
                    value,
                },
                "sound" => Self::Sound {
                    parameters,
                    group,
                    value: value.parse()?,
                },
                "uid" => Self::Uid {
                    parameters,
                    group,
                    value: value.parse()?,
                },
                "clientidmap" => {
                    let mut split = value.split(";");
                    let pid = split.next().map(u8::from_str).ok_or_else(|| {
                        VCardError::InvalidLine {
                            reason:
                                "expected clientpidmap value to have two parts separated by ';'",
                            raw_line: value.clone(),
                        }
                    })??;
                    let global_identifier = split.next().map(url::Url::from_str).ok_or_else(
                        || VCardError::InvalidLine {
                            reason:
                                "expected clientpidmap value to have two parts separated by ';'",
                            raw_line: value.clone(),
                        },
                    )??;
                    Self::ClientPidMap {
                        global_identifier,
                        pid,
                        group,
                        parameters,
                    }
                }
                "url" => Self::Url {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "key" => Self::Key {
                    group,
                    parameters,
                    value,
                },
                "fburl" => Self::FbUrl {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "caladuri" => Self::CalAdUri {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "caluri" => Self::CalUri {
                    group,
                    parameters,
                    value: value.parse()?,
                },
                "xml" => Self::Xml {
                    value,
                    group,
                    parameters,
                },
                _ => {
                    if !name.starts_with("X-") && !name.starts_with("x-") {
                        return Err(VCardError::InvalidName {
                            actual_name: name.into(),
                            raw_line: line.into(),
                        });
                    }
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

#[derive(Debug, PartialEq)]
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
