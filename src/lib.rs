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
#[derive(Debug, PartialEq)]
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
pub enum Sex {
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

#[derive(Debug, PartialEq)]
pub struct Gender {
    pub sex: Option<Sex>,
    pub identity_component: Option<String>,
}

impl FromStr for Sex {
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

#[derive(Debug, PartialEq)]
pub struct Version {
    pub value: VersionValue,
}
#[derive(Debug, PartialEq)]
pub struct Source {
    pub group: Option<String>,
    pub pid: Option<Pid>,
    pub altid: String,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct FN {
    pub altid: String,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub pref: Option<u8>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct N {
    pub altid: String,
    pub sort_as: Vec<String>,
    pub group: Option<String>,
    pub surenames: Vec<String>,
    pub given_names: Vec<String>,
    pub additional_names: Vec<String>,
    pub honorific_prefixes: Vec<String>,
    pub honorific_suffixes: Vec<String>,
}
#[derive(Debug, PartialEq)]
pub struct Nickname {
    pub group: Option<String>,
    pub altid: String,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub pref: Option<u8>,
    pub pid: Option<Pid>,
    pub value: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Photo {
    pub group: Option<String>,
    pub altid: String,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,
    pub pref: Option<u8>,
    pub pid: Option<Pid>,
    pub value: url::Url,
}
#[derive(Debug, PartialEq)]
pub struct BDay {
    pub altid: String,
    pub calscale: Option<String>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Anniversary {
    pub altid: String,
    pub calscale: Option<String>,
    pub type_param: Option<ValueType>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Address {
    pub group: Option<String>,
    pub altid: String,
    pub label: Option<String>,
    pub language: Option<String>,
    pub geo: Option<String>,
    pub tz: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub po_box: Vec<String>,
    pub extended_address: Vec<String>,
    pub street: Vec<String>,
    pub city: Vec<String>,
    pub region: Vec<String>,
    pub postal_code: Vec<String>,
    pub country: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Tel {
    pub type_param: Option<ValueType>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub altid: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Email {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Impp {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub mediatype: Option<String>,
    pub type_param: Option<ValueType>,
    pub value: String,
}
#[derive(Debug, PartialEq)]
pub struct Language {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Tz {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Geo {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct Title {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Role {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Logo {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct Org {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub sort_as: Vec<String>,

    pub value: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Member {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct Related {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Categories {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,

    pub value: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct ProdId {
    pub group: Option<String>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Rev {
    pub group: Option<String>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Sound {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct Uid {
    pub group: Option<String>,
    pub type_param: Option<ValueType>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct ClientPidMap {
    pub group: Option<String>,
    pub pid_digit: u8,
    pub value: url::Url,
}
#[derive(Debug, PartialEq)]
pub struct VcardURL {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct FbURL {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct CalAdURI {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct CalURI {
    pub group: Option<String>,
    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[derive(Debug, PartialEq)]
pub struct Key {
    pub group: Option<String>,

    pub altid: String,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub type_param: Option<ValueType>,
    pub mediatype: Option<String>,

    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Xml {
    pub group: Option<String>,
    pub value: String,
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum Property {
    #[strum(serialize = "begin")]
    Begin { value: String },
    #[strum(serialize = "end")]
    End { value: String },
    #[strum(serialize = "version")]
    Version(Version),
    #[strum(serialize = "source")]
    Source(Source),
    #[strum(serialize = "kind")]
    Kind(Kind),
    #[strum(serialize = "fn")]
    FN(FN),
    #[strum(serialize = "n")]
    N(N),
    #[strum(serialize = "nickname")]
    NickName(Nickname),
    #[strum(serialize = "photo")]
    Photo(Photo),
    #[strum(serialize = "bday")]
    BDay(BDay),
    #[strum(serialize = "anniversary")]
    Anniversary(Anniversary),
    #[strum(serialize = "gender")]
    Gender(Gender),
    #[strum(serialize = "adr")]
    Adr(Address),
    #[strum(serialize = "tel")]
    Tel(Tel),
    #[strum(serialize = "email")]
    Email(Email),
    #[strum(serialize = "impp")]
    Impp(Impp),
    #[strum(serialize = "lang")]
    Lang(Language),
    #[strum(serialize = "tz")]
    Tz(Tz),
    #[strum(serialize = "geo")]
    Geo(Geo),
    #[strum(serialize = "title")]
    Title(Title),
    #[strum(serialize = "role")]
    Role(Role),
    #[strum(serialize = "logo")]
    Logo(Logo),
    #[strum(serialize = "org")]
    Org(Org),
    #[strum(serialize = "member")]
    Member(Member),
    #[strum(serialize = "related")]
    Related(Related),
    #[strum(serialize = "categories")]
    Categories(Categories),
    #[strum(serialize = "note")]
    Note(Note),
    #[strum(serialize = "prodid")]
    ProdId(ProdId),
    #[strum(serialize = "rev")]
    Rev(Rev),
    #[strum(serialize = "sound")]
    Sound(Sound),
    #[strum(serialize = "uid")]
    Uid(Uid),
    #[strum(serialize = "clientidmap")]
    ClientPidMap(ClientPidMap),
    #[strum(serialize = "url")]
    Url(VcardURL),
    #[strum(serialize = "key")]
    Key(Key),
    #[strum(serialize = "fburl")]
    FbUrl(FbURL),
    #[strum(serialize = "caladuri")]
    CalAdUri(CalAdURI),
    #[strum(serialize = "caluri")]
    CalUri(CalURI),
    #[strum(serialize = "xml")]
    Xml(Xml),
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

        let mut pid = None;
        let mut altid = None;
        let mut mediatype = None;
        let mut tz = None;
        let mut geo = None;
        let mut sort_as = None;
        let mut calscale = None;
        let mut type_param = None;
        let mut pref = None;
        let mut language = None;
        let mut label = None;
        let mut proprietary_parameters = Vec::new();
        for param in parameters {
            match param {
                Parameter::Pid(p) => pid = Some(p),
                Parameter::AltId(a) => altid = Some(a),
                Parameter::MediaType(m) => mediatype = Some(m),
                Parameter::TimeZone(t) => tz = Some(t),
                Parameter::Geo(g) => geo = Some(g),
                Parameter::SortAs(s) => sort_as = Some(s),
                Parameter::CalScale(c) => calscale = Some(c),
                Parameter::Type(t) => type_param = Some(ValueType::from_str(&t[..])?),
                Parameter::Language(l) => language = Some(l),
                Parameter::Pref(p) => pref = Some(p),
                Parameter::Label(l) => label = Some(l),
                Parameter::Proprietary(p) => {
                    proprietary_parameters.push(Parameter::Proprietary(p))
                }
                _ => return Err(VCardError::UnknownParameter(param.as_ref().into())),
            }
        }

        let sort_as = sort_as.unwrap_or_default();
        let altid = altid.unwrap_or_default();

        let prop =
            match &name[..] {
                "begin" => Self::Begin { value },
                "end" => Self::End { value },
                "version" => {
                    if value != "4.0" {
                        return Err(VCardError::InvalidVersion(value));
                    }
                    Self::Version(Version {
                        value: VersionValue::V4,
                    })
                }
                "source" => Self::Source(Source {
                    pid,
                    altid,
                    mediatype,
                    group,
                    value: value.parse()?,
                }),
                "kind" => Self::Kind(value.parse()?),
                "fn" => Self::FN(FN {
                    altid,
                    type_param,
                    value,
                    language,
                    pref,
                }),
                "n" => {
                    let mut split = value
                        .split(";")
                        .map(|item| item.split(";").map(String::from).collect::<Vec<String>>());
                    let surenames = split.next().unwrap_or_else(Vec::new);
                    let given_names = split.next().unwrap_or_else(Vec::new);
                    let additional_names = split.next().unwrap_or_else(Vec::new);
                    let honorific_prefixes = split.next().unwrap_or_else(Vec::new);
                    let honorific_suffixes = split.next().unwrap_or_else(Vec::new);
                    Self::N(N {
                        sort_as,
                        altid,
                        additional_names,
                        honorific_prefixes,
                        honorific_suffixes,
                        given_names,
                        surenames,
                        group,
                    })
                }
                "nickname" => Self::NickName(Nickname {
                    altid,
                    pref,
                    type_param,
                    language,
                    pid,
                    group,
                    value: value.split(",").map(String::from).collect(),
                }),
                "photo" => Self::Photo(Photo {
                    group,
                    altid,
                    pid,
                    mediatype,
                    type_param,
                    pref,
                    value: value.parse()?,
                }),
                "bday" => Self::BDay(BDay {
                    altid,
                    calscale,
                    language,
                    type_param,
                    value,
                }),
                "anniversary" => Self::Anniversary(Anniversary {
                    altid,
                    calscale,
                    type_param,
                    value,
                }),
                "gender" => {
                    let mut split = value.split(";");
                    let value = if let Some(r) = split.next().map(Sex::from_str) {
                        Some(r?)
                    } else {
                        None
                    };
                    let identity_component = split.next().map(String::from);
                    Self::Gender(Gender {
                        sex: value,
                        identity_component,
                    })
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
                    Self::Adr(Address {
                        altid,
                        pid,
                        label,
                        language,
                        geo,
                        tz,
                        type_param,
                        pref,
                        region,
                        po_box,
                        city,
                        group,
                        extended_address,
                        street,
                        postal_code,
                        country,
                    })
                }
                "tel" => Self::Tel(Tel {
                    type_param,
                    pid,
                    pref,
                    altid,
                    value,
                }),
                "email" => Self::Email(Email {
                    altid,
                    group,
                    pid,
                    pref,
                    type_param,
                    value,
                }),
                "impp" => Self::Impp(Impp {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value,
                }),

                "lang" => Self::Lang(Language {
                    altid,
                    pid,
                    pref,
                    type_param,
                    group,
                    value,
                }),
                "tz" => Self::Tz(Tz {
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    group,
                    value,
                }),
                "geo" => Self::Geo(Geo {
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    group,
                    value: value.parse()?,
                }),
                "title" => Self::Title(Title {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    group,
                    value,
                }),
                "role" => Self::Role(Role {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    group,
                    value,
                }),
                "categories" => Self::Categories(Categories {
                    altid,
                    pid,
                    pref,
                    type_param,
                    group,
                    value: value.split(";").map(String::from).collect(),
                }),
                "org" => Self::Org(Org {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    sort_as,
                    group,
                    value: value.split(";").map(String::from).collect(),
                }),
                "member" => Self::Member(Member {
                    altid,
                    pid,
                    pref,
                    group,
                    mediatype,
                    value: value.parse()?,
                }),
                "related" => Self::Related(Related {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    mediatype,
                    group,
                    value,
                }),
                "logo" => Self::Logo(Logo {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    mediatype,
                    group,
                    value: value.parse()?,
                }),
                "note" => Self::Note(Note {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    group,
                    value,
                }),
                "prodid" => Self::ProdId(ProdId { group, value }),
                "rev" => Self::Rev(Rev { group, value }),
                "sound" => Self::Sound(Sound {
                    altid,
                    pid,
                    pref,
                    type_param,
                    language,
                    mediatype,
                    group,
                    value: value.parse()?,
                }),
                "uid" => Self::Uid(Uid {
                    type_param,
                    group,
                    value,
                }),
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
                    Self::ClientPidMap(ClientPidMap {
                        value: global_identifier,
                        pid_digit: pid,
                        group,
                    })
                }
                "url" => Self::Url(VcardURL {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value: value.parse()?,
                }),
                "key" => Self::Key(Key {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value,
                }),
                "fburl" => Self::FbUrl(FbURL {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value: value.parse()?,
                }),
                "caladuri" => Self::CalAdUri(CalAdURI {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value: value.parse()?,
                }),
                "caluri" => Self::CalUri(CalURI {
                    group,
                    altid,
                    pid,
                    pref,
                    type_param,
                    mediatype,
                    value: value.parse()?,
                }),
                "xml" => Self::Xml(Xml { value, group }),
                _ => {
                    if !name.starts_with("X-") && !name.starts_with("x-") {
                        return Err(VCardError::InvalidName {
                            actual_name: name.into(),
                            raw_line: line.into(),
                        });
                    }

                    // let mut language = None;

                    proprietary_parameters.push(Parameter::AltId(altid));
                    if let Some(pid) = pid {
                        proprietary_parameters.push(Parameter::Pid(pid));
                    }
                    if let Some(mediatype) = mediatype {
                        proprietary_parameters.push(Parameter::MediaType(mediatype));
                    }
                    if let Some(tz) = tz {
                        proprietary_parameters.push(Parameter::TimeZone(tz));
                    }

                    if let Some(geo) = geo {
                        proprietary_parameters.push(Parameter::Geo(geo));
                    }

                    proprietary_parameters.push(Parameter::SortAs(sort_as));

                    if let Some(calscale) = calscale {
                        proprietary_parameters.push(Parameter::CalScale(calscale));
                    }

                    if let Some(label) = label {
                        proprietary_parameters.push(Parameter::Label(label));
                    }

                    if let Some(t) = type_param {
                        proprietary_parameters.push(Parameter::Type(t.as_ref().into()));
                    }

                    if let Some(pref) = pref {
                        proprietary_parameters.push(Parameter::Pref(pref));
                    }

                    if let Some(l) = language {
                        proprietary_parameters.push(Parameter::Language(l));
                    }

                    Property::Proprietary {
                        name,
                        value: value.into(),
                        group,
                        parameters: proprietary_parameters,
                    }
                }
            };
        Ok(prop)
    }
}

#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum Parameter {
    Label(String),
    Language(String),
    Value(ValueType),
    Pref(u8),
    AltId(String),
    Pid(Pid),
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
                Parameter::Pid(Pid {
                    first_digit,
                    second_digit,
                })
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
