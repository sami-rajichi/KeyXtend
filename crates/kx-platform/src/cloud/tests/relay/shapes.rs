//! Each request shape reaches the frame whole: an upload form with its paywall and region, `ElevenLabs`' own form, and
//! `AssemblyAI`'s model header.

use super::*;
use kx_module_api::ports::{FormField, OwnFields};

/// `OpenAI`'s error code for a country or region it does not serve.
const REGION: &str = "unsupported_country_region_territory";

#[test]
fn the_rows_form_reaches_the_frame_whole_with_its_paywall_and_region() {
    let (host, script, _) = host();
    let keep = FormField {
        name: "provider".into(),
        value: r#"{"zdr":true}"#.into(),
    };
    let shape = CloudShape::Form(FormFields {
        lang: "languages".into(),
        list: true,
        fixed: vec![keep],
        paywall: true,
        region: Some(REGION.into()),
    });
    let (_run, _) = sent(
        &host,
        CloudAsk {
            shape,
            ..ask(&[PER_S])
        },
    );
    let NetAsk::Send { call, .. } = &nets(&script)[0] else {
        panic!("a send first");
    };
    let NetShape::Form(form) = &call.shape else {
        panic!("a form, not {:?}", call.shape);
    };
    assert_eq!(
        (
            form.lang.as_ref(),
            form.list,
            form.paywall,
            form.region.as_deref()
        ),
        ("languages", true, true, Some(REGION))
    );
    let fixed: Vec<(&str, &str)> = form
        .fixed
        .iter()
        .map(|f| (f.name.as_ref(), f.value.as_ref()))
        .collect();
    assert_eq!(fixed, [("provider", r#"{"zdr":true}"#)]);
}

#[test]
fn elevenlabs_form_reaches_the_frame_in_its_own_shape() {
    let (host, script, _) = host();
    let tags = FormField {
        name: "tag_audio_events".into(),
        value: "false".into(),
    };
    let shape = CloudShape::Eleven(OwnFields {
        lang: "language_code".into(),
        fixed: vec![tags],
    });
    let ask = CloudAsk {
        shape,
        header: KeyHeader::Named("xi-api-key".into()),
        ..ask(&[PER_S])
    };
    let (_run, _) = sent(&host, ask);
    let NetAsk::Send { call, .. } = &nets(&script)[0] else {
        panic!("a send first");
    };
    let NetShape::Eleven(form) = &call.shape else {
        panic!("its own shape, not {:?}", call.shape);
    };
    let fixed: Vec<(&str, &str)> = (form.fixed.iter())
        .map(|f| (f.name.as_ref(), f.value.as_ref()))
        .collect();
    assert_eq!(form.lang, "language_code");
    assert_eq!(fixed, [("tag_audio_events", "false")]);
}

#[test]
fn assemblyais_model_header_reaches_the_frame_in_its_own_shape() {
    let (host, script, _) = host();
    let name = "X-AAI-Model";
    let model_header = name.to_owned();
    let mut asked = ask(&[PER_S]);
    asked.shape = CloudShape::Assembly { model_header };
    let (_run, _) = sent(&host, asked);
    let NetAsk::Send { call, .. } = &nets(&script)[0] else {
        panic!("a send first");
    };
    assert!(matches!(&call.shape, NetShape::Assembly { model_header } if model_header == name));
}
