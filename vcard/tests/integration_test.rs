use std::{collections::HashMap, error::Error, path::PathBuf};

use vcard::*;

#[test]

fn test_vcards_from_big_services() -> Result<(), Box<dyn Error>> {
    let mut dir = PathBuf::new();
    dir.push(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_assets/good_vcards"
    ));

    let mut test_table = HashMap::new();

    test_table.insert(
        "apple_icloud.vcf",
        VCard::new(VersionValue::V3)
            .n(N {
                surenames: vec!["vom Tosafjord".into()],
                given_names: vec!["Heinrich".into()],
                ..Default::default()
            })?
            .fn_property(FN {
                value: "Heinrich vom Tosafjord".into(),
                ..Default::default()
            })
            .org(Org {
                value: vec!["Richter GBR".into()],
                ..Default::default()
            })
            .bday(BDay {
                value: "2017-01-03".into(),
                value_data_type: Some(ValueDataType::Date),
                ..Default::default()
            })?
            .note(Note {
                value: "ist eine Katze".into(),
                ..Default::default()
            })
            .adr(Adr {
                group: Some("item1".into()),
                city: vec!["Katzenhausen".into()],
                street: vec!["am Katzenklo".into()],
                type_param: Some(vec!["HOME".into(), "pref".into()]),
                postal_code: vec!["23456".into()],
                country: vec!["Germany".into()],
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABADR".into(),
                group: Some("item1".into()),
                value: "de".into(),
                parameters: Vec::new(),
            })
            .tel(Tel {
                type_param: Some(vec!["CELL".into(), "pref".into(), "VOICE".into()]),
                value: "017610101520".into(),
                ..Default::default()
            })
            .url(Url {
                group: Some("item2".into()),
                type_param: Some(vec!["pref".into()]),
                value: "https://www.example.com/heinrich".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLABEL".into(),
                group: Some("item2".into()),
                value: "_$!<HomePage>!$_".into(),
                parameters: Vec::new(),
            })
            .email(Email {
                type_param: Some(vec!["HOME".into(), "pref".into(), "INTERNET".into()]),
                value: "heinrich@tosafjord.com".into(),
                ..Default::default()
            })
            .prodid(ProdId {
                group: None,
                value: "-//Apple Inc.//iCloud Web Address Book 2117B3//EN".into(),
            })
            .rev(Rev {
                group: None,
                value: "2021-09-23T05:51:29Z".into(),
            })
            .build(),
    );

    test_table.insert(
        "google.vcf",
        VCard::new(VersionValue::V3)
            .n(N {
                given_names: vec!["Judith".to_string()],
                ..Default::default()
            })?
            .fn_property(FN {
                value: "Judith".to_string(),
                ..Default::default()
            })
            .email(Email {
                type_param: Some(vec!["INTERNET".into(), "HOME".into()]),
                value: "test@example.com".into(),
                ..Default::default()
            })
            .email(Email {
                type_param: Some(vec!["INTERNET".into()]),
                value: "test2@example.com".into(),
                ..Default::default()
            })
            .tel(Tel {
                type_param: Some(vec!["CELL".into()]),
                value: "+49123456789".into(),
                ..Default::default()
            })
            .tel(Tel {
                type_param: Some(vec!["HOME".into()]),
                value: "09999123456789".into(),
                ..Default::default()
            })
            .url(Url {
                group: Some("item1".into()),
                value: "http\\://www.google.com/profiles/xxxxx".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                group: Some("item1".into()),
                name: "X-ABLabel".into(),
                parameters: Vec::new(),
                value: "PROFILE".into(),
            })
            .photo(Photo {
                value: "https://lh3.example.com/-xxxx/xxxxxA/xxxxx/photo.jpg".parse()?,
                altid: None,
                group: None,
                value_data_type: None,
                type_param: None,
                mediatype: None,
                pref: None,
                pid: None,
            })
            .categories(Categories {
                value: vec!["Freunde".into(), "myContacts".into(), "starred".into()],
                ..Default::default()
            })
            .build(),
    );

    test_table.insert(
        "google_2.vcf",
        VCard::new(VersionValue::V3)
            .fn_property(FN {
                value: "Dr. Heinrich Kasper Vom Tosafjord Von und Zu".into(),
                ..Default::default()
            })
            .n(N {
                surenames: vec!["Vom Tosafjord".into()],
                given_names: vec!["Heinrich".into()],
                honorific_prefixes: vec!["Dr.".into()],
                honorific_suffixes: vec!["Von und Zu".into()],
                additional_names: vec!["Kasper".into()],
                ..Default::default()
            })?
            .nickname(Nickname {
                value: vec!["Knödel".into()],
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-PHONETIC-FIRST-NAME".into(),
                value: "Henry".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-PHONETIC-LAST-NAME".into(),
                value: "VTF".into(),
                ..Default::default()
            })
            .email(Email {
                type_param: Some(vec!["INTERNET".into(), "HOME".into()]),
                value: "heinrich@example.com".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-JABBER".into(),
                value: "heinrich@jabber.com".into(),
                group: Some("item1".into()),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLabel".into(),
                group: Some("item1".into()),
                ..Default::default()
            })
            .tel(Tel {
                type_param: Some(vec!["HOME".into()]),
                value: "00 0000".into(),
                ..Default::default()
            })
            .adr(Adr {
                type_param: Some(vec!["HOME".into()]),
                po_box: vec!["7".into()],
                postal_code: vec!["550201".into()],
                city: vec!["Achundkrach".into()],
                street: vec!["Auf dem Land 3".into()],
                country: vec!["DE".into()],
                ..Default::default()
            })
            .org(Org {
                value: vec!["test gmbh".into(), "it".into()],
                group: Some("item2".into()),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLabel".into(),
                group: Some("item2".into()),
                ..Default::default()
            })
            .title(Title {
                value: "manager".into(),
                group: Some("item3".into()),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLabel".into(),
                group: Some("item3".into()),
                ..Default::default()
            })
            .bday(BDay {
                value: "20180301".into(),
                ..Default::default()
            })?
            .url(Url {
                group: Some("item4".into()),
                value: "www.example.com".parse()?,
                altid: None,
                pid: None,
                mediatype: None,
                pref: None,
                type_param: None,
                value_data_type: None,
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLabel".into(),
                group: Some("item4".into()),
                value: "BLOG".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABRELATEDNAMES".into(),
                group: Some("item5".into()),
                value: "fiona".into(),
                ..Default::default()
            })
            .proprietary(ProprietaryProperty {
                name: "X-ABLabel".into(),
                group: Some("item5".into()),
                value: "_$!<Sister>!$_".into(),
                ..Default::default()
            })
            .note(Note {
                value: "ist eine katze\\nirgendeinlabel: testfeld".into(),
                ..Default::default()
            })
            .categories(Categories {
                value: vec!["myContacts".into()],
                ..Default::default()
            })
            .build(),
    );

    for (k, expected) in test_table {
        let mut path = dir.clone();
        path.push(k);
        let mut f = std::fs::File::open(path)?;
        let mut reader = VCardReader::new(&mut f);

        let actual = reader.parse_vcard()?;

        compare_vcards(&expected, &actual);

        // we test the Serialization by feeding it back into our reader.
        let new_val = expected.to_string();
        let new_card = VCardReader::new(new_val.as_bytes()).parse_vcard()?;

        compare_vcards(&expected, &new_card);
    }

    Ok(())
}

fn compare_vcards(expected: &VCard, actual: &VCard) {
    assert_eq!(expected.version, actual.version);
    assert_eq!(expected.source, actual.source);

    assert_eq!(expected.kind, actual.kind);
    assert_eq!(expected.xml, actual.xml);
    assert_eq!(expected.fn_property, actual.fn_property);
    assert_eq!(expected.n, actual.n);
    assert_eq!(expected.nickname, actual.nickname);
    assert_eq!(expected.photo, actual.photo);
    assert_eq!(expected.bday, actual.bday);
    assert_eq!(expected.anniversary, actual.anniversary);
    assert_eq!(expected.gender, actual.gender);

    assert_eq!(expected.adr, actual.adr);
    assert_eq!(expected.tel, actual.tel);
    assert_eq!(expected.email, actual.email);
    assert_eq!(expected.impp, actual.impp);
    assert_eq!(expected.lang, actual.lang);
    assert_eq!(expected.tz, actual.tz);
    assert_eq!(expected.geo, actual.geo);
    assert_eq!(expected.title, actual.title);
    assert_eq!(expected.role, actual.role);

    assert_eq!(expected.logo, actual.logo);
    assert_eq!(expected.org, actual.org);
    assert_eq!(expected.member, actual.member);
    assert_eq!(expected.related, actual.related);
    assert_eq!(expected.categories, actual.categories);
    assert_eq!(expected.note, actual.note);
    assert_eq!(expected.prodid, actual.prodid);
    assert_eq!(expected.rev, actual.rev);
    assert_eq!(expected.sound, actual.sound);

    assert_eq!(expected.uid, actual.uid);
    assert_eq!(expected.clientpidmap, actual.clientpidmap);
    assert_eq!(expected.url, actual.url);
    assert_eq!(expected.key, actual.key);
    assert_eq!(expected.fburl, actual.fburl);
    assert_eq!(expected.caluri, actual.caluri);
    assert_eq!(expected.caladuri, actual.caladuri);
    assert_eq!(
        expected.proprietary_properties,
        actual.proprietary_properties
    );
}
