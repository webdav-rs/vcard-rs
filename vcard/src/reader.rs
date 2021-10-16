use std::{
    cell::RefCell,
    io::{self, BufReader, Read},
    rc::Rc,
    str::FromStr,
};

use crate::{errors::VCardError, Property, VCard};

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
                num_returned_bytes: 0,
                buf: [0, 0],
            },
            discard_buf: Rc::new(RefCell::new(Vec::with_capacity(1024))),
            max_logical_line_length,
        }
    }

    pub fn parse_vcard(&mut self) -> Result<VCard, VCardError> {
        let (prop, more) = self.read_property()?;
        match prop {
            Property::Begin { value } => {
                if &value[..] != "VCARD" {
                    return Err(VCardError::InvalidBeginProperty);
                }
            }
            _ => return Err(VCardError::InvalidBeginProperty),
        }

        if !more {
            return Err(VCardError::InvalidVersionProperty);
        }
        let (prop, more) = self.read_property()?;
        let version = match prop {
            Property::Version(v) => v,
            _ => return Err(VCardError::InvalidVersionProperty),
        };

        if !more {
            return Err(VCardError::InvalidEndProperty);
        }

        let mut result = VCard {
            version,
            ..Default::default()
        };

        loop {
            let (prop, more) = self.read_property()?;
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
                    if &value[..] != "VCARD" || more {
                        return Err(VCardError::InvalidEndProperty);
                    }
                    return Ok(result);
                }

                Property::Source(s) => result.source.add_value(s),
                Property::Kind(k) => add_single_value!(result, kind, k),
                Property::Xml(x) => result.xml.add_value(x),
                Property::FN(f) => result.fn_property.add_value(f),
                Property::N(n) => result.n.add_value(n)?,
                Property::NickName(n) => result.nickname.add_value(n),
                Property::Photo(p) => result.photo.add_value(p),
                Property::BDay(b) => result.bday.add_value(b)?,
                Property::Anniversary(a) => result.anniversary.add_value(a)?,
                Property::Gender(g) => add_single_value!(result, gender, g),
                Property::Adr(a) => result.adr.add_value(a),
                Property::Tel(t) => result.tel.add_value(t),
                Property::Email(e) => result.email.add_value(e),
                Property::Impp(i) => result.impp.add_value(i),
                Property::Lang(l) => result.lang.add_value(l),
                Property::Tz(t) => result.tz.add_value(t),
                Property::Geo(g) => result.geo.add_value(g),
                Property::Title(t) => result.title.add_value(t),
                Property::Role(r) => result.role.add_value(r),
                Property::Logo(l) => result.logo.add_value(l),
                Property::Org(o) => result.org.add_value(o),
                Property::Member(m) => result.member.add_value(m),
                Property::Related(r) => result.related.add_value(r),
                Property::Categories(c) => result.categories.add_value(c),
                Property::Note(n) => result.note.add_value(n),
                Property::ProdId(p) => add_single_value!(result, prodid, p),
                Property::Rev(r) => add_single_value!(result, rev, r),
                Property::Sound(s) => result.sound.add_value(s),
                Property::Uid(u) => add_single_value!(result, uid, u),
                Property::ClientPidMap(c) => add_single_value!(result, clientpidmap, c),
                Property::Url(u) => result.url.add_value(u),
                Property::Key(k) => result.key.add_value(k),
                Property::FbUrl(f) => result.fburl.add_value(f),
                Property::CalUri(c) => result.caluri.add_value(c),
                Property::CalAdUri(c) => result.caladuri.add_value(c),
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
    pub fn read_property(&mut self) -> Result<(Property, bool), VCardError> {
        let (line, more) = self.read_logical_line()?;
        Ok((Property::from_str(&line[..])?, more))
    }
    fn read_logical_line(&mut self) -> Result<(String, bool), VCardError> {
        let mut logical_line_buf = Vec::new();

        // a logical line always starts with a new property declaration
        let result = self.read_physical_line(&mut logical_line_buf);

        match result {
            Ok(()) => {}
            Err(e) => match e {
                VCardError::Io(io_err) => match io_err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        // some implementations like google contacts and icloud do not respect the standard
                        // and omit the trailing \r\n
                        if b"END:VCARD" != &logical_line_buf[..] {
                            return Err(io_err.into());
                        }
                    }
                    _ => return Err(io_err.into()),
                },
                _ => return Err(e),
            },
        }

        loop {
            match self.inspect_next_line()? {
                LineInspection::NewProperty => {
                    // a logical line expands only accross one property.
                    // if we encounter the declaration of the next property, the logical line has an end.
                    return Ok((String::from_utf8(logical_line_buf)?, true));
                }
                LineInspection::NoMoreContent => {
                    return Ok((String::from_utf8(logical_line_buf)?, false))
                }
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

// This reader makes it possible to return a certain amount of bytes back to the reader itself (two to be precise).
// The use case is the inspection of bytes in order to determine the continuation/end of logical lines in a vcard.
struct PushbackReader<R> {
    inner: BufReader<R>,
    buf: [u8; 2],

    // num_buf_bytes can be 2 at maximum
    num_returned_bytes: usize,
}

impl<R: io::Read> PushbackReader<R> {
    // a maximum of two bytes can be returned.
    // If more bytes are returned, the buffer will be filled again from the beginning
    // and already present bytes will be discarded.
    fn return_byte(&mut self, b: u8) {
        if self.num_returned_bytes >= 2 {
            self.num_returned_bytes = 0;
        }
        // this is safe because num_retruned_bytes can be at max 1 here.
        self.buf[self.num_returned_bytes] = b;
        self.num_returned_bytes = self.num_returned_bytes + 1;
    }

    fn return_bytes(&mut self, b: [u8; 2]) {
        self.buf = b;
        self.num_returned_bytes = 2;
    }
}
impl<R: io::Read> Read for PushbackReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.num_returned_bytes == 0 {
            return self.inner.read(buf);
        }
        let first = &self.buf.as_ref()[0..self.num_returned_bytes];
        let mut chain = first.chain(&mut self.inner);
        let result = chain.read(buf)?;

        // if only one byte was read, we have to emulate a cursor move by removing the consumed byte.
        // in case more than one bytes where read, we just invalidate the whole buffer.
        if result == 1 {
            self.buf[0] = self.buf[1];
            self.num_returned_bytes = self.num_returned_bytes - 1;
        } else {
            self.num_returned_bytes = 0;
        }

        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

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
            let (actual_property, _more) = reader.read_property()?;
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
                name: "X-ABADR".into(),
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
                name: "X-ABLABEL".into(),
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
                Ok((p,_more)) => p,
                Err(e) => {
                    panic!("expected prop {:#?} but got error {}", expected_prop, e);
                }
            };
            assert_eq!(expected_prop, prop);
        }
        Ok(())
    }
}
