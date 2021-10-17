use regex::Regex;
use std::str::FromStr;

use crate::errors::VCardError;

use super::*;

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
    Url(Url),
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

fn filter_and_transform<A: AsRef<str>>(input: A) -> Option<String> {
    if input.as_ref().is_empty() {
        None
    } else {
        Some(String::from(input.as_ref()))
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

fn escaped_split(item: &str, split: char) -> impl Iterator<Item = String> {
    let escape_char = '\\';
    let mut result = Vec::new();
    let mut escaped_value = false;
    let mut buf = String::new();
    for c in item.chars() {
        // add escaped values no matter what
        if escaped_value {
            buf.push(c);
            escaped_value = false;
            continue;
        }

        if c == escape_char {
            escaped_value = true
        } else if c == split {
            result.push(buf);
            buf = String::new();
        } else {
            buf.push(c)
        }
    }
    result.push(buf);

    result.into_iter()
}

lazy_static::lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<group>[^;:]+\.)?(?P<name>[^;:]+)(?P<parameter>;[^:]+)*:(?P<value>.*)").unwrap();
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
        let name = name.trim_matches(char::from(0));
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
            match &name.to_lowercase()[..] {
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
                    value: value,
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
                    let mut split = escaped_split(&value, ';').map(|item| {
                        escaped_split(&item, ',')
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
                    value: escaped_split(&value, ',').map(String::from).collect(),
                }),
                "photo" => Self::Photo(Photo {
                    group,
                    altid,
                    pid,
                    mediatype,
                    type_param,
                    value_data_type,
                    pref,
                    value: value,
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
                    let (sex, identity) =
                        value
                            .split_once(";")
                            .ok_or_else(|| VCardError::InvalidSyntax {
                                property: "Gender".into(),
                                message: "gender property must include a semicolon (;)".into(),
                            })?;
                    let value = if sex.is_empty() {
                        None
                    } else {
                        Some(Sex::from_str(sex)?)
                    };
                    let identity_component = Some(identity.to_string());
                    Self::Gender(Gender {
                        sex: value,
                        identity_component,
                    })
                }
                "adr" => {
                    let mut split = escaped_split(&value, ';').map(|item| {
                        escaped_split(&item, ',')
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
                    value,
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
                    value: escaped_split(&value, ',')
                        .filter_map(filter_and_transform)
                        .collect(),
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
                    value: escaped_split(&value, ';')
                        .filter_map(filter_and_transform)
                        .collect(),
                }),
                "member" => Self::Member(Member {
                    altid,
                    pid,
                    pref,
                    group,
                    mediatype,
                    value,
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
                    value,
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
                    value,
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
                    let global_identifier = split.next().map(String::from).ok_or_else(|| {
                        VCardError::InvalidLine {
                            reason:
                                "expected clientpidmap value to have two parts separated by ';'",
                            raw_line: value.clone(),
                        }
                    })?;
                    Self::ClientPidMap(ClientPidMap {
                        value: global_identifier,
                        pid_digit: pid,
                        group,
                    })
                }
                "url" => Self::Url(Url {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value,
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
                    value,
                }),
                "caladuri" => Self::CalAdUri(CalAdURI {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value,
                }),
                "caluri" => Self::CalUri(CalURI {
                    group,
                    altid,
                    pid,
                    pref,
                    value_data_type,
                    type_param,
                    mediatype,
                    value,
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
                        name: name.into(),
                        value: value.into(),
                        group,
                        parameters: proprietary_parameters,
                    })
                }
            };
        Ok(prop)
    }
}
