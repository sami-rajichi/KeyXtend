//! What the upload form refuses as out of range: a value that would break it, a fixed field repeating one of its own
//! names or a list language's, and another shape's call.

use super::*;

#[test]
fn a_value_that_would_break_the_form_is_refused() {
    let call = call();
    for bad in ["a\r\nb", "a\nb", "a\rb", "a\0b"] {
        let got = Form.build(&piece(&call, Some(bad), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "model {bad:?}");
        let got = Form.build(&piece(&call, Some(MODEL), bad, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "language {bad:?}");
    }
    let fixed = vec![NetField {
        name: Cow::Borrowed("prompt"),
        value: Cow::Borrowed("a\r\n--x"),
    }];
    let call = call_to(URL, NetShape::Form(NetForm { fixed, ..form() }));
    let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
    assert_eq!(got, Some(NetFault::Range), "a fixed value");
}

#[test]
fn a_fixed_field_may_not_repeat_the_forms_own_names() {
    for name in ["model", "file", "response_format", "language", "MODEL"] {
        let fixed = vec![NetField {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed("x"),
        }];
        let call = call_to(URL, NetShape::Form(NetForm { fixed, ..form() }));
        let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
}

#[test]
fn a_fixed_field_may_not_repeat_a_list_languages_name() {
    for name in ["languages[]", "LANGUAGES[]", "languages"] {
        let fixed = vec![NetField {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed("x"),
        }];
        let shape = NetShape::Form(NetForm {
            lang: Cow::Borrowed("languages"),
            list: true,
            fixed,
            paywall: false,
            region: None,
            admin: Vec::new(),
        });
        let call = call_to(URL, shape);
        let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
}

#[test]
fn another_shapes_call_is_out_of_range() {
    let call = call_to(URL, NetShape::Eleven(super::super::eleven::own_form()));
    let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
    assert_eq!(got, Some(NetFault::Range));
}
