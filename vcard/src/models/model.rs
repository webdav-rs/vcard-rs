use std::{fmt::Display, str::FromStr};

use vcard_macro::{vcard, AltID, Pref};

use crate::{AltIDContainer, MultiAltIDContainer, Parameter, Pid, ValueDataType, errors::VCardError};

pub trait Alternative {
    fn get_alt_id(&self) -> &str;
}

pub trait Preferable {
    fn get_pref(&self) -> u8;
}

/// See https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.9
#[derive(Debug, PartialEq, strum_macros::AsRefStr)]
pub enum VersionValue {
    #[strum(serialize = "3.0")]
    V3,
    #[strum(serialize = "4.0")]
    V4,
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
}