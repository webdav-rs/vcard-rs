use lazy_static;
use regex::{self, Regex};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    io::{self, BufReader, Read},
    rc::Rc,
    str::FromStr,
};

use strum_macros;

use errors::VCardError;
mod errors;
use vcard_macro::{vcard, AltID, Pref};

pub trait Alternative {
    fn get_alt_id(&self) -> &str;
}

pub trait Preferable {
    fn get_pref(&self) -> u8;
}
pub struct MultiAltIDContainer<T: Alternative>(HashMap<String, AltIDContainer<T>>);

impl<T: Alternative> Default for MultiAltIDContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Alternative + Display> Display for MultiAltIDContainer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for val in self.0.values() {
            val.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Alternative> MultiAltIDContainer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_value(&mut self, value: T) -> Result<(), VCardError> {
        if self.0.contains_key(value.get_alt_id()) {
            let container = self.0.get_mut(value.get_alt_id()).unwrap();
            container.add_value(value)?;
        } else {
            let altid = value.get_alt_id().to_string();
            let container = AltIDContainer::from_vec(vec![value]);
            self.0.insert(altid, container);
        }

        Ok(())
    }

    pub fn values(&self) -> &HashMap<String, AltIDContainer<T>> {
        &self.0
    }

    pub fn take_values(self) -> HashMap<String, AltIDContainer<T>> {
        self.0
    }
}

impl<T: Alternative + Preferable> MultiAltIDContainer<T> {
    /// returns the prefered value.
    ///
    /// Preference values are ascending. No guarantees are made when multiple values have the same `pref`
    pub fn get_prefered_value(&self) -> Option<&T> {
        let mut prefered_item = None;
        for (_key, container) in self.0.iter() {
            let container_prefered_item = if let Some(cpi) = container.get_prefered_value() {
                cpi
            } else {
                continue;
            };
            if prefered_item.is_none() {
                prefered_item = Some(container_prefered_item);
            } else if prefered_item.unwrap().get_alt_id() > container_prefered_item.get_alt_id() {
                prefered_item = Some(container_prefered_item);
            }
        }

        prefered_item
    }
}

/// In vcard, if multiple entries share the same type and altid, they are considered
/// to be one record. This means, all entries in an `AltIDContainer` are considered one record as well.
#[derive(Default)]
pub struct AltIDContainer<T: Alternative>(Vec<T>);

impl<T> Display for AltIDContainer<T>
where
    T: Alternative + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.0.iter() {
            item.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Alternative> AltIDContainer<T> {
    pub fn new() -> Self {
        AltIDContainer(Vec::new())
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        AltIDContainer(items)
    }

    /// Adds a new value to this container.
    ///
    /// This will fail if `item` has a different `altid` than previous elements of this container.
    /// In case the container does not have any elemts, it will simply be added to the collection.
    pub fn add_value(&mut self, item: T) -> Result<(), VCardError> {
        if self.0.len() == 0 {
            self.0.push(item);
            return Ok(());
        }
        let prev_altid = self.0.get(0).unwrap().get_alt_id();
        if prev_altid != item.get_alt_id() {
            return Err(VCardError::InvalidAltID {
                expected_altid: prev_altid.to_string(),
                actual_altid: item.get_alt_id().to_owned(),
            });
        }

        self.0.push(item);

        Ok(())
    }

    pub fn values(&self) -> &[T] {
        &self.0
    }

    pub fn take_values(self) -> Vec<T> {
        self.0
    }
}

impl<T> AltIDContainer<T>
where
    T: Alternative + Preferable,
{
    /// returns the prefered value.
    ///
    /// Preference values are ascending. No guarantees are made when multiple values have the same `pref`
    pub fn get_prefered_value(&self) -> Option<&T> {
        let mut prefered_item = None;
        for item in self.0.iter() {
            if prefered_item.is_none() {
                prefered_item = Some(item);
                continue;
            }

            if prefered_item.unwrap().get_alt_id() > item.get_alt_id() {
                prefered_item = Some(item);
            }
        }

        prefered_item
    }
}

/// Represents a single VCard.
///
/// For more informatin about the fields, see https://datatracker.ietf.org/doc/html/rfc6350#section-6
#[derive(Default)]
pub struct VCard {
    pub version: Version,
    pub source: MultiAltIDContainer<Source>,
    pub kind: Option<Kind>,
    pub xml: MultiAltIDContainer<Xml>,
    pub fn_property: MultiAltIDContainer<FN>,

    pub n: AltIDContainer<N>,

    pub nickname: MultiAltIDContainer<Nickname>,

    pub photo: MultiAltIDContainer<Photo>,

    pub bday: AltIDContainer<BDay>,
    pub anniversary: AltIDContainer<Anniversary>,
    pub gender: Option<Gender>,
    pub adr: MultiAltIDContainer<Adr>,
    pub tel: MultiAltIDContainer<Tel>,
    pub email: MultiAltIDContainer<Email>,
    pub impp: MultiAltIDContainer<Impp>,
    pub lang: MultiAltIDContainer<Lang>,

    pub tz: MultiAltIDContainer<Tz>,
    pub geo: MultiAltIDContainer<Geo>,
    pub title: MultiAltIDContainer<Title>,
    pub role: MultiAltIDContainer<Role>,
    pub logo: MultiAltIDContainer<Logo>,
    pub org: MultiAltIDContainer<Org>,
    pub member: MultiAltIDContainer<Member>,
    pub related: MultiAltIDContainer<Related>,
    pub categories: MultiAltIDContainer<Categories>,
    pub note: MultiAltIDContainer<Note>,

    pub prodid: Option<ProdId>,
    pub rev: Option<Rev>,
    pub sound: MultiAltIDContainer<Sound>,
    pub uid: Option<Uid>,
    pub clientpidmap: Option<ClientPidMap>,

    pub url: MultiAltIDContainer<VcardURL>,
    pub key: MultiAltIDContainer<Key>,
    pub fburl: MultiAltIDContainer<FbURL>,
    pub caluri: MultiAltIDContainer<CalURI>,
    pub caladuri: MultiAltIDContainer<CalAdURI>,

    pub proprietary_properties: Vec<ProprietaryProperty>,
}

fn write_vcard_property<D: Display>(
    f: &mut std::fmt::Formatter<'_>,
    input: &Option<D>,
) -> std::fmt::Result {
    if let Some(item) = input {
        item.fmt(f)?;
    }
    Ok(())
}

impl Display for VCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VCARD\r\n")?;
        write_vcard_property(f, &Some(&self.version))?;

        self.source.fmt(f)?;
        write_vcard_property(f, &self.kind)?;

        self.xml.fmt(f)?;
        self.fn_property.fmt(f)?;
        self.n.fmt(f)?;
        self.nickname.fmt(f)?;
        self.photo.fmt(f)?;
        self.bday.fmt(f)?;
        self.anniversary.fmt(f)?;

        write_vcard_property(f, &self.gender)?;

        self.adr.fmt(f)?;
        self.tel.fmt(f)?;
        self.email.fmt(f)?;
        self.impp.fmt(f)?;
        self.lang.fmt(f)?;
        self.tz.fmt(f)?;
        self.geo.fmt(f)?;
        self.title.fmt(f)?;
        self.role.fmt(f)?;
        self.logo.fmt(f)?;
        self.org.fmt(f)?;
        self.member.fmt(f)?;
        self.related.fmt(f)?;
        self.categories.fmt(f)?;
        self.note.fmt(f)?;

        write_vcard_property(f, &self.prodid)?;
        write_vcard_property(f, &self.rev)?;
        write_vcard_property(f, &self.uid)?;
        write_vcard_property(f, &self.clientpidmap)?;

        self.sound.fmt(f)?;
        self.url.fmt(f)?;
        self.key.fmt(f)?;
        self.fburl.fmt(f)?;
        self.caluri.fmt(f)?;
        self.caladuri.fmt(f)?;
        for prop in self.proprietary_properties.iter() {
            prop.fmt(f)?;
        }
        write!(f, "END:VCARD\r\n")
    }
}

/// A reader that reads vcard properties one by one.
///
/// Vcard properties can span accross multiple lines called "logical lines".
/// The `max_logical_line_length` field acts as a safety net to prevent memory overflows.
/// An `std::io::BufReader` is used internally.
pub struct VCardReader<R: io::Read> {
    inner: PushbackReader<R>,
    discard_buf: Rc<RefCell<Vec<u8>>>,
    pub max_logical_line_length: u64,
}

/// See https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.9
#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum VersionValue {
    #[strum(serialize = "3.0")]
    V3,
    #[strum(serialize = "4.0")]
    V4,
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

#[vcard]
#[derive(Debug, PartialEq)]
pub struct Kind {
    pub group: Option<String>,
    pub value: KindValue,
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum KindValue {
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

impl FromStr for KindValue {
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

#[vcard]
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

#[vcard]
#[derive(Debug, PartialEq)]
pub struct Version {
    pub value: VersionValue,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            value: VersionValue::V4,
        }
    }
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Source {
    pub group: Option<String>,
    pub pid: Option<Pid>,
    pub altid: Option<String>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct FN {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,
    pub language: Option<String>,
    pub pref: Option<u8>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID)]
pub struct N {
    pub altid: Option<String>,
    pub language: Option<String>,
    pub sort_as: Option<Vec<String>>,
    pub group: Option<String>,

    pub surenames: Vec<String>,
    pub given_names: Vec<String>,
    pub additional_names: Vec<String>,
    pub honorific_prefixes: Vec<String>,
    pub honorific_suffixes: Vec<String>,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Nickname {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,
    pub pref: Option<u8>,
    pub pid: Option<Pid>,
    pub value: Vec<String>,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Photo {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,
    pub mediatype: Option<String>,
    pub pref: Option<u8>,
    pub pid: Option<Pid>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID)]
pub struct BDay {
    pub altid: Option<String>,
    pub calscale: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub language: Option<String>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID)]
pub struct Anniversary {
    pub altid: Option<String>,
    pub calscale: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Adr {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub label: Option<String>,
    pub language: Option<String>,
    pub geo: Option<String>,
    pub tz: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub po_box: Vec<String>,
    pub extended_address: Vec<String>,
    pub street: Vec<String>,
    pub city: Vec<String>,
    pub region: Vec<String>,
    pub postal_code: Vec<String>,
    pub country: Vec<String>,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct Tel {
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub altid: Option<String>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct Email {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct Impp {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub mediatype: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct Lang {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, Default, AltID, Pref)]
pub struct Tz {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Geo {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Title {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Role {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Logo {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Org {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,
    pub sort_as: Option<Vec<String>>,

    pub value: Vec<String>,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Member {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID, Pref)]
pub struct Related {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Categories {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub value: Vec<String>,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Note {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq)]
pub struct ProdId {
    pub group: Option<String>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq)]
pub struct Rev {
    pub group: Option<String>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Sound {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub language: Option<String>,
    pub mediatype: Option<String>,

    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq)]
pub struct Uid {
    pub group: Option<String>,
    pub value_data_type: Option<ValueDataType>,
    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq)]
pub struct ClientPidMap {
    pub group: Option<String>,
    pub pid_digit: u8,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct VcardURL {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct FbURL {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct CalAdURI {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct CalURI {
    pub group: Option<String>,
    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,
    pub mediatype: Option<String>,
    pub value: url::Url,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Key {
    pub group: Option<String>,

    pub altid: Option<String>,
    pub pid: Option<Pid>,
    pub pref: Option<u8>,
    pub value_data_type: Option<ValueDataType>,
    pub type_param: Option<Vec<String>>,

    pub mediatype: Option<String>,

    pub value: String,
}

#[vcard]
#[derive(Debug, PartialEq, AltID)]
pub struct Xml {
    pub altid: Option<String>,
    pub group: Option<String>,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct ProprietaryProperty {
    pub name: String,
    pub group: Option<String>,
    pub value: String,
    pub parameters: Vec<Parameter>,
}

impl Display for ProprietaryProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(grp) = &self.group {
            write!(f, "{}.", grp)?;
        }
        write!(f, "{}", self.name)?;

        for param in self.parameters.iter() {
            write!(f, ";{}", param.to_string())?;
        }

        write!(f, ":{}\r\n", self.value)?;

        Ok(())
    }
}

#[derive(strum_macros::AsRefStr, Debug, PartialEq)]
pub enum Property {
    #[strum(serialize = "begin")]
    Begin {
        value: String,
    },
    #[strum(serialize = "end")]
    End {
        value: String,
    },
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
    Adr(Adr),
    #[strum(serialize = "tel")]
    Tel(Tel),
    #[strum(serialize = "email")]
    Email(Email),
    #[strum(serialize = "impp")]
    Impp(Impp),
    #[strum(serialize = "lang")]
    Lang(Lang),
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
    Proprietary(ProprietaryProperty),
}

fn filter_and_transform(input: &str) -> Option<String> {
    if input.is_empty() {
        None
    } else {
        Some(String::from(input))
    }
}

fn parse_url<A: AsRef<str>>(input: A) -> Result<url::Url, VCardError> {
    input
        .as_ref()
        .parse()
        .map_err(|e| VCardError::url_parse_error(e, input.as_ref()))
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
        let name = name.trim_matches(char::from(0)).to_lowercase();
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
        let mut type_param: Option<Vec<String>> = None;
        let mut value_data_type = None;
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
                Parameter::Value(t) => value_data_type = Some(t),
                Parameter::Type(mut t) => {
                    if let Some(tp) = type_param.as_mut() {
                        tp.append(&mut t);
                    } else {
                        type_param = Some(t);
                    }
                }
                Parameter::Language(l) => language = Some(l),
                Parameter::Pref(p) => pref = Some(p),
                Parameter::Label(l) => label = Some(l),
                Parameter::Proprietary(p) => proprietary_parameters.push(Parameter::Proprietary(p)),
            }
        }

        let prop =
            match &name[..] {
                "begin" => Self::Begin { value },
                "end" => Self::End { value },
                "version" => {
                    let value = match &value[..] {
                        "4.0" => VersionValue::V4,
                        "3.0" => VersionValue::V3,
                        _ => return Err(VCardError::InvalidVersion(value)),
                    };
                    Self::Version(Version { value })
                }
                "source" => Self::Source(Source {
                    pid,
                    altid,
                    mediatype,
                    group,
                    value: parse_url(value)?,
                }),
                "kind" => Self::Kind(Kind {
                    group,
                    value: value.parse()?,
                }),
                "fn" => Self::FN(FN {
                    group,
                    altid,
                    type_param,
                    value_data_type,
                    value,
                    language,
                    pref,
                }),
                "n" => {
                    let mut split = value.split(";").map(|item| {
                        item.split(";")
                            .filter_map(filter_and_transform)
                            .collect::<Vec<String>>()
                    });
                    let surenames = split.next().unwrap_or_else(Vec::new);
                    let given_names = split.next().unwrap_or_else(Vec::new);
                    let additional_names = split.next().unwrap_or_else(Vec::new);
                    let honorific_prefixes = split.next().unwrap_or_else(Vec::new);
                    let honorific_suffixes = split.next().unwrap_or_else(Vec::new);
                    Self::N(N {
                        language,
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
                    value_data_type,
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
                    value_data_type,
                    pref,
                    value: parse_url(value)?,
                }),
                "bday" => Self::BDay(BDay {
                    altid,
                    calscale,
                    language,
                    value_data_type,
                    value,
                }),
                "anniversary" => Self::Anniversary(Anniversary {
                    altid,
                    calscale,
                    value_data_type,
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
                    let mut split = value.split(";").map(|item| {
                        item.split(",")
                            .filter_map(filter_and_transform)
                            .collect::<Vec<String>>()
                    });
                    let po_box = split.next().unwrap_or_else(|| Vec::new());
                    let extended_address = split.next().unwrap_or_else(|| Vec::new());
                    let street = split.next().unwrap_or_else(|| Vec::new());
                    let city = split.next().unwrap_or_else(|| Vec::new());
                    let region = split.next().unwrap_or_else(|| Vec::new());
                    let postal_code = split.next().unwrap_or_else(|| Vec::new());
                    let country = split.next().unwrap_or_else(|| Vec::new());
                    Self::Adr(Adr {
                        altid,
                        pid,
                        label,
                        language,
                        geo,
                        tz,
                        value_data_type,
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
                    value_data_type,
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
                    value_data_type,
                    type_param,
                    value,
                }),
                "impp" => Self::Impp(Impp {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value,
                }),

                "lang" => Self::Lang(Lang {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    group,
                    value,
                }),
                "tz" => Self::Tz(Tz {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    group,
                    value,
                }),
                "geo" => Self::Geo(Geo {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    group,
                    value: parse_url(value)?,
                }),
                "title" => Self::Title(Title {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    language,
                    group,
                    value,
                }),
                "role" => Self::Role(Role {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    language,
                    group,
                    value,
                }),
                "categories" => Self::Categories(Categories {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    group,
                    value: value.split(";").filter_map(filter_and_transform).collect(),
                }),
                "org" => Self::Org(Org {
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    language,
                    sort_as,
                    group,
                    value: value.split(";").filter_map(filter_and_transform).collect(),
                }),
                "member" => Self::Member(Member {
                    altid,
                    pid,
                    pref,
                    group,
                    mediatype,
                    value: parse_url(value)?,
                }),
                "related" => Self::Related(Related {
                    altid,
                    pid,
                    pref,
                    value_data_type,
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
                    value_data_type,
                    type_param,
                    language,
                    mediatype,
                    group,
                    value: parse_url(value)?,
                }),
                "note" => Self::Note(Note {
                    altid,
                    pid,
                    pref,
                    value_data_type,
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
                    value_data_type,
                    type_param,
                    language,
                    mediatype,
                    group,
                    value: parse_url(value)?,
                }),
                "uid" => Self::Uid(Uid {
                    value_data_type,
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
                    let global_identifier = split.next().map(parse_url).ok_or_else(|| {
                        VCardError::InvalidLine {
                            reason:
                                "expected clientpidmap value to have two parts separated by ';'",
                            raw_line: value.clone(),
                        }
                    })??;
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
                    value_data_type,
                    type_param,
                    mediatype,
                    value: value
                        .parse()
                        .map_err(|e| VCardError::url_parse_error(e, value))?,
                }),
                "key" => Self::Key(Key {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value,
                }),
                "fburl" => Self::FbUrl(FbURL {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value: parse_url(value)?,
                }),
                "caladuri" => Self::CalAdUri(CalAdURI {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value: parse_url(value)?,
                }),
                "caluri" => Self::CalUri(CalURI {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value: parse_url(value)?,
                }),
                "xml" => Self::Xml(Xml {
                    altid,
                    value,
                    group,
                }),
                _ => {
                    if !name.starts_with("X-") && !name.starts_with("x-") {
                        return Err(VCardError::InvalidName {
                            actual_name: name.into(),
                            raw_line: line.into(),
                        });
                    }

                    // let mut language = None;
                    if let Some(altid) = altid {
                        proprietary_parameters.push(Parameter::AltId(altid));
                    }

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

                    if let Some(sort_as) = sort_as {
                        proprietary_parameters.push(Parameter::SortAs(sort_as));
                    }

                    if let Some(calscale) = calscale {
                        proprietary_parameters.push(Parameter::CalScale(calscale));
                    }

                    if let Some(label) = label {
                        proprietary_parameters.push(Parameter::Label(label));
                    }

                    if let Some(type_param) = type_param {
                        proprietary_parameters.push(Parameter::Type(type_param));
                    }

                    if let Some(pref) = pref {
                        proprietary_parameters.push(Parameter::Pref(pref));
                    }

                    if let Some(l) = language {
                        proprietary_parameters.push(Parameter::Language(l));
                    }

                    Property::Proprietary(ProprietaryProperty {
                        name,
                        value: value.into(),
                        group,
                        parameters: proprietary_parameters,
                    })
                }
            };
        Ok(prop)
    }
}

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

fn parse_parameters(raw: &str) -> Result<Vec<Parameter>, VCardError> {
    let raw = raw.trim_start_matches(";");
    let mut result = Vec::new();
    let mut prev = 0;
    let mut buf = Vec::new();
    for char in raw.as_bytes() {
        // it is possible that a parameter contains an escaped semicolon (in the form \;).
        // We have to ensure those semicolons are not parsed as a separate parameter.
        if *char == b';' && prev != b'\\' {
            let s = std::str::from_utf8(&buf)?;
            let param = s.parse()?;
            result.push(param);
            buf.clear();
        } else {
            prev = *char;
            buf.push(*char);
        }
    }
    // ensure that the last entry gets added as well.
    let s = std::str::from_utf8(&buf)?;
    let param = s.parse()?;
    result.push(param);
    Ok(result)
}

lazy_static::lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<group>[^;:]+\.)?(?P<name>[^;:]+)(?P<parameter>;[^:]+)*:(?P<value>.+)").unwrap();
}

const DEFAULT_MAX_LINE_LENGTH: u64 = 5000;

enum LineInspection {
    NoMoreContent,
    Discard,
    LogicalLine,
    NewProperty,
}

macro_rules! add_single_value {
    ($result:expr,$prop:ident,$new_val:expr) => {{
        if $result.$prop.is_some() {
            return Err(VCardError::InvalidCardinality {
                expected: 1,
                property: stringify!($prop).into(),
            });
        }
        $result.$prop = Some($new_val);
    }};
}

impl<R: io::Read> VCardReader<R> {
    /// Creates a new `VCardReader` with the default logical line limit of 5000
    pub fn new(input: R) -> Self {
        Self::new_with_logical_line_limit(input, DEFAULT_MAX_LINE_LENGTH)
    }

    /// Creates a new `VCardReader` with a configurable line limit
    pub fn new_with_logical_line_limit(input: R, max_logical_line_length: u64) -> Self {
        Self {
            inner: PushbackReader {
                inner: io::BufReader::new(input),
                buf_index: 0,
                buf: [0, 0],
            },
            discard_buf: Rc::new(RefCell::new(Vec::with_capacity(1024))),
            max_logical_line_length,
        }
    }

    pub fn parse_vcard(&mut self) -> Result<VCard, VCardError> {
        match self.read_property()? {
            Property::Begin { value } => {
                if &value[..] != "VCARD" {
                    return Err(VCardError::InvalidBeginProperty);
                }
            }
            _ => return Err(VCardError::InvalidBeginProperty),
        }

        let version = match self.read_property()? {
            Property::Version(v) => v,
            _ => return Err(VCardError::InvalidVersionProperty),
        };

        let mut result = VCard {
            version,
            ..Default::default()
        };

        loop {
            let prop = self.read_property()?;
            match prop {
                Property::Version(_) => {
                    return Err(VCardError::InvalidCardinality {
                        expected: 1,
                        property: "VERSION".into(),
                    })
                }
                Property::Begin { value: _ } => {
                    return Err(VCardError::InvalidCardinality {
                        expected: 1,
                        property: "BEGIN".into(),
                    })
                }
                Property::End { value } => {
                    if &value[..] != "VCARD" {
                        return Err(VCardError::InvalidBeginProperty);
                    }
                    return Ok(result);
                }

                Property::Source(s) => result.source.add_value(s)?,
                Property::Kind(k) => add_single_value!(result, kind, k),
                Property::Xml(x) => result.xml.add_value(x)?,
                Property::FN(f) => result.fn_property.add_value(f)?,
                Property::N(n) => result.n.add_value(n)?,
                Property::NickName(n) => result.nickname.add_value(n)?,
                Property::Photo(p) => result.photo.add_value(p)?,
                Property::BDay(b) => result.bday.add_value(b)?,
                Property::Anniversary(a) => result.anniversary.add_value(a)?,
                Property::Gender(g) => add_single_value!(result, gender, g),
                Property::Adr(a) => result.adr.add_value(a)?,
                Property::Tel(t) => result.tel.add_value(t)?,
                Property::Email(e) => result.email.add_value(e)?,
                Property::Impp(i) => result.impp.add_value(i)?,
                Property::Lang(l) => result.lang.add_value(l)?,
                Property::Tz(t) => result.tz.add_value(t)?,
                Property::Geo(g) => result.geo.add_value(g)?,
                Property::Title(t) => result.title.add_value(t)?,
                Property::Role(r) => result.role.add_value(r)?,
                Property::Logo(l) => result.logo.add_value(l)?,
                Property::Org(o) => result.org.add_value(o)?,
                Property::Member(m) => result.member.add_value(m)?,
                Property::Related(r) => result.related.add_value(r)?,
                Property::Categories(c) => result.categories.add_value(c)?,
                Property::Note(n) => result.note.add_value(n)?,
                Property::ProdId(p) => add_single_value!(result, prodid, p),
                Property::Rev(r) => add_single_value!(result, rev, r),
                Property::Sound(s) => result.sound.add_value(s)?,
                Property::Uid(u) => add_single_value!(result, uid, u),
                Property::ClientPidMap(c) => add_single_value!(result, clientpidmap, c),
                Property::Url(u) => result.url.add_value(u)?,
                Property::Key(k) => result.key.add_value(k)?,
                Property::FbUrl(f) => result.fburl.add_value(f)?,
                Property::CalUri(c) => result.caluri.add_value(c)?,
                Property::CalAdUri(c) => result.caladuri.add_value(c)?,
                Property::Proprietary(p) => result.proprietary_properties.push(p),
            }
        }
    }

    fn inspect_next_line(&mut self) -> Result<LineInspection, VCardError> {
        let mut buf = [0, 0];
        // read the next two bytes. If the next byte continues with a whicespace char (space (U+0020) or horizontal tab (U+0009))
        // it counts as a logical continuation of this line.
        // If not, we indicate that those two bytes belong to the next line and return the line as is.
        if let Err(e) = self.inner.read_exact(&mut buf) {
            match e.kind() {
                // this means, there are no more bytes left. Most likely, this means we reached the END:VCARD line.
                io::ErrorKind::UnexpectedEof => {
                    return Ok(LineInspection::NoMoreContent);
                }
                _ => return Err(VCardError::Io(e)),
            }
        }

        if buf[0] != b' ' && buf[0] != b'\t' {
            self.inner.return_bytes(buf);
            return Ok(LineInspection::NewProperty);
        }

        // The spec tells us that we have to ensure that the start of a continued line does not have two whitespace characters in a  row
        match buf[1] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                self.inner.return_bytes(buf);
                return Ok(LineInspection::Discard);
            }
            _ => {
                return {
                    self.inner.return_byte(buf[1]);
                    Ok(LineInspection::LogicalLine)
                }
            }
        }
    }

    /// Reads the next Property from this vcard. In case the logical property line exceeds `max_logical_line_length`
    /// an `VCardError::MaxLineLengthExceeded` will be returned.
    /// see https://datatracker.ietf.org/doc/html/rfc6350#section-3.2 for more information about logical lines.
    pub fn read_property(&mut self) -> Result<Property, VCardError> {
        let line = self.read_logical_line()?;
        Property::from_str(&line[..])
    }
    fn read_logical_line(&mut self) -> Result<String, VCardError> {
        let mut logical_line_buf = Vec::new();

        // a logical line always starts with a new property declaration
        self.read_physical_line(&mut logical_line_buf)?;

        loop {
            match self.inspect_next_line()? {
                LineInspection::NewProperty => {
                    // a logical line expands only accross one property.
                    // if we encounter the declaration of the next property, the logical line has an end.
                    return Ok(String::from_utf8(logical_line_buf)?);
                }
                LineInspection::NoMoreContent => return Ok(String::from_utf8(logical_line_buf)?),
                LineInspection::Discard => self.discard_line()?,
                LineInspection::LogicalLine => {
                    self.read_physical_line(&mut logical_line_buf)?;
                }
            }
        }
    }
    fn discard_line(&mut self) -> Result<(), VCardError> {
        let rc = Rc::clone(&self.discard_buf.clone());
        let mut buf = rc.as_ref().borrow_mut();
        self.read_physical_line(&mut buf)?;
        Ok(())
    }

    fn read_physical_line(&mut self, buf: &mut Vec<u8>) -> Result<(), VCardError> {
        let mut tmp_buf = [0];

        loop {
            if buf.len() as u64 > self.max_logical_line_length {
                return Err(VCardError::MaxLineLengthExceeded(
                    self.max_logical_line_length,
                ));
            }
            // this should be okay since lines are usually short and we use a bufreader
            self.inner.read_exact(&mut tmp_buf)?;
            if tmp_buf[0] == b'\r' {
                // read one more byte to see if it is a \n char
                self.inner.read_exact(&mut tmp_buf)?;
                if tmp_buf[0] == b'\n' {
                    return Ok(());
                } else {
                    buf.extend(tmp_buf);
                }
            } else {
                buf.extend(tmp_buf);
            }
        }
    }
}

// This reader makes it possible to return a certain amount of bytes back to the reader itself.
// The use case is the inspection of bytes in order to determine the continuation/end of logical lines in a vcard.
struct PushbackReader<R> {
    inner: BufReader<R>,
    buf: [u8; 2],
    buf_index: usize,
}

impl<R: io::Read> PushbackReader<R> {
    fn return_byte(&mut self, b: u8) {
        if self.buf_index > 1 {
            self.buf_index = 0;
        }
        self.buf[self.buf_index] = b;
        self.buf_index = self.buf_index + 1;
    }

    fn return_bytes(&mut self, b: [u8; 2]) {
        self.buf = b;
        self.buf_index = 2;
    }
}
impl<R: io::Read> Read for PushbackReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buf_index == 0 {
            return self.inner.read(buf);
        }
        let first = &self.buf.as_ref()[0..self.buf_index];
        let mut chain = first.chain(&mut self.inner);
        let result = chain.read(buf)?;

        match result {
            1 => {
                self.buf[0] = self.buf[1];
                let new_index = self.buf_index - 1;
                self.buf_index = std::cmp::max(new_index, 0);
            }
            2 => {
                self.buf_index = 0;
            }
            _ => {}
        }
        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_display() -> Result<(), Box<dyn std::error::Error>> {
        let mut n = N::default();
        assert_eq!("N:;;;;\r\n", n.to_string());
        n.sort_as = Some(vec!["foo".into(), "bar".into()]);
        assert_eq!("N;SORT-AS=\"foo,bar\":;;;;\r\n", n.to_string());
        n.surenames = vec!["Vom Tosafjord".into()];
        n.given_names = vec!["Heinrich".into()];
        assert_eq!(
            "N;SORT-AS=\"foo,bar\":Vom Tosafjord;Heinrich;;;\r\n",
            n.to_string()
        );

        let mut e = Email::default();
        assert_eq!("EMAIL:\r\n", e.to_string());

        e.group = Some("foo".into());

        assert_eq!("foo.EMAIL:\r\n", e.to_string());

        e.altid = Some("asdf".into());

        assert_eq!("foo.EMAIL;ALTID=asdf:\r\n", e.to_string());

        e.value = "mail@example.com".into();

        assert_eq!("foo.EMAIL;ALTID=asdf:mail@example.com\r\n", e.to_string());

        Ok(())
    }

    #[test]
    fn test_multi_line() -> Result<(), Box<dyn std::error::Error>> {
        let testant = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/new_line.vcf",
        ))
        .to_vec();

        let mut reader = VCardReader::new(&testant[..]);

        let expected = vec![
            Property::Begin {
                value: "VCARD".into(),
            },
            Property::Version(Version {
                value: VersionValue::V3,
            }),
            Property::FN(FN {
                group: None,
                altid: None,
                value_data_type: None,
                type_param: None,
                language: None,
                pref: None,
                value: "Heinrich vom Tosafjordasdfsadfasdf".into(),
            }),
            Property::End {
                value: "VCARD".into(),
            },
        ];

        for expected_property in expected.iter() {
            let actual_property = reader.read_property()?;
            assert_eq!(expected_property, &actual_property);
        }
        let mut reader = VCardReader::new_with_logical_line_limit(&testant[..], 36);
        for _i in [0; 2] {
            reader.read_property()?;
        }

        let result = reader.read_property();

        if let Ok(_p) = result {
            panic!("expected MaxLineLengthExceeded error");
        }
        Ok(())
    }

    #[test]
    fn test_apple_icloud_format() -> Result<(), Box<dyn std::error::Error>> {
        let testant = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_assets/apple_icloud.vcf",
        ))
        .to_vec();
        let mut reader = VCardReader::new(&testant[..]);

        let expected = vec![
            Property::Begin {
                value: "VCARD".into(),
            },
            Property::Version(Version {
                value: VersionValue::V3,
            }),
            Property::N(N {
                altid: None,
                sort_as: None,
                language: None,
                group: None,
                surenames: vec!["vom Tosafjord".into()],
                given_names: vec!["Heinrich".into()],
                additional_names: Vec::new(),
                honorific_prefixes: Vec::new(),
                honorific_suffixes: Vec::new(),
            }),
            Property::FN(FN {
                group: None,
                altid: None,
                value_data_type: None,
                type_param: None,
                language: None,
                pref: None,
                value: "Heinrich vom Tosafjord".into(),
            }),
            Property::Org(Org {
                sort_as: None,
                pid: None,
                group: None,
                altid: None,
                value_data_type: None,
                type_param: None,
                language: None,
                pref: None,
                value: vec!["Richter GBR".into()],
            }),
            Property::BDay(BDay {
                altid: None,
                calscale: None,
                value_data_type: Some(ValueDataType::Date),
                language: None,
                value: "2017-01-03".into(),
            }),
            Property::Note(Note {
                pid: None,
                group: None,
                altid: None,
                value_data_type: None,
                type_param: None,
                language: None,
                pref: None,
                value: "ist eine Katze".into(),
            }),
            Property::Adr(Adr {
                group: Some("item1".into()),
                city: vec!["Katzenhausen".into()],
                street: vec!["am Katzenklo".into()],
                type_param: Some(vec!["HOME".into(), "pref".into()]),
                altid: None,
                label: None,
                language: None,
                geo: None,
                tz: None,
                pid: None,
                pref: None,
                value_data_type: None,
                po_box: Vec::new(),
                extended_address: Vec::new(),
                postal_code: vec!["23456".into()],
                country: vec!["Germany".into()],
                region: Vec::new(),
            }),
            Property::Proprietary(ProprietaryProperty {
                name: "x-abadr".into(),
                group: Some("item1".into()),
                value: "de".into(),
                parameters: Vec::new(),
            }),
            Property::Tel(Tel {
                type_param: Some(vec!["CELL".into(), "pref".into(), "VOICE".into()]),
                value_data_type: None,
                pid: None,
                pref: None,
                altid: None,
                value: "017610101520".into(),
            }),
            Property::Url(VcardURL {
                group: Some("item2".into()),
                type_param: Some(vec!["pref".into()]),
                value: "https://www.example.com/heinrich".parse()?,
                altid: None,
                pid: None,
                pref: None,
                value_data_type: None,
                mediatype: None,
            }),
            Property::Proprietary(ProprietaryProperty {
                name: "x-ablabel".into(),
                group: Some("item2".into()),
                value: "_$!<HomePage>!$_".into(),
                parameters: Vec::new(),
            }),
            Property::Email(Email {
                group: None,
                type_param: Some(vec!["HOME".into(), "pref".into(), "INTERNET".into()]),
                pid: None,
                altid: None,
                pref: None,
                value_data_type: None,
                value: "heinrich@tosafjord.com".into(),
            }),
            Property::ProdId(ProdId {
                group: None,
                value: "-//Apple Inc.//iCloud Web Address Book 2117B3//EN".into(),
            }),
            Property::Rev(Rev {
                group: None,
                value: "2021-09-23T05:51:29Z".into(),
            }),
            Property::End {
                value: "VCARD".into(),
            },
        ];

        for expected_prop in expected {
            let prop = match reader.read_property() {
                Ok(p) => p,
                Err(e) => {
                    panic!("expected prop {:#?} but got error {}", expected_prop, e);
                }
            };
            assert_eq!(expected_prop, prop);
        }
        Ok(())
    }
}
