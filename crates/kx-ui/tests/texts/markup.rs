//! Text our QML draws is plain text, so no text from a file can make the uiAccess process load an image (security
//! review U-S1, ADR-0004).

use crate::qml_files;

/// Qt Quick's elements that draw or edit text, as their declarations start.
const TEXT_TYPES: [&str; 5] = [
    "Text {",
    "TextEdit {",
    "TextInput {",
    "TextArea {",
    "TextField {",
];
/// The property that sets how text is read, and the one value allowed.
const FORMAT: &str = "textFormat";
const PLAIN: &str = "Text.PlainText";
/// What starts a comment line.
const COMMENT: &str = "//";
/// The file that holds our text element, so a broken reader cannot pass the check.
const LABEL: &str = "Label.qml";

/// Whether trimmed line `l` starts a Qt text element, alone or as a property's value.
fn opens(l: &str) -> bool {
    TEXT_TYPES
        .iter()
        .any(|t| l == *t || l.ends_with(&format!(": {t}")))
}

/// Lines (from 0) of `qml` where a Qt text element starts.
fn texts(qml: &str) -> Vec<usize> {
    let lines = qml.lines().enumerate();
    lines
        .filter(|(_, l)| opens(l.trim()))
        .map(|(i, _)| i)
        .collect()
}

/// The trimmed lines of the block that opens on line `at`, at its own depth: its properties, not its children's.
fn own<'a>(lines: &[&'a str], at: usize) -> Vec<&'a str> {
    let mut depth = 0_usize;
    let mut out = Vec::new();
    for l in lines.iter().skip(at) {
        if depth == 1 {
            out.push(l.trim());
        }
        depth = (depth + l.matches('{').count()).saturating_sub(l.matches('}').count());
        if depth == 0 {
            break;
        }
    }
    out
}

/// Each line of `qml` where text may read markup, as `<name>:<line>`: a text element without its own plain format, or
/// any line that sets another format.
fn markup(name: &str, qml: &str) -> Vec<String> {
    let lines: Vec<&str> = qml.lines().collect();
    let plain = format!("{FORMAT}: {PLAIN}");
    let bare = texts(qml)
        .into_iter()
        .filter(|&at| !own(&lines, at).contains(&plain.as_str()));
    let other = lines.iter().enumerate().filter(|(_, l)| {
        let l = l.trim();
        !l.starts_with(COMMENT) && l.contains(FORMAT) && !l.contains(PLAIN)
    });
    let mut at: Vec<usize> = bare.chain(other.map(|(i, _)| i)).collect();
    at.sort_unstable();
    at.dedup();
    at.iter().map(|i| format!("{name}:{}", i + 1)).collect()
}

#[test]
fn every_text_element_in_our_qml_reads_plain_text() {
    let mut found = Vec::new();
    for (name, text) in qml_files() {
        assert_eq!(markup(&name, &text), Vec::<String>::new());
        if !texts(&text).is_empty() {
            found.push(name);
        }
    }
    assert!(found.iter().any(|n| n == LABEL), "found {found:?}");
}

/// A text element without its own plain format, under a property, with the format only on a child, or any text asking
/// for another format, is found.
#[test]
fn a_text_that_could_read_markup_is_found() {
    let qml = [
        "Item {",
        "    Text {",
        "        textFormat: Text.PlainText",
        "    }",
        "    Text {",
        "        text: name",
        "    }",
        "    contentItem: TextEdit {",
        "        Rectangle {",
        "            textFormat: Text.PlainText",
        "        }",
        "    }",
        "    Label {",
        "        textFormat: Text.RichText",
        "    }",
        "    // textFormat: Text.RichText in a comment is fine.",
        "}",
    ]
    .join("\n");
    assert_eq!(texts(&qml), [1, 4, 7]);
    assert_eq!(markup("x.qml", &qml), ["x.qml:5", "x.qml:8", "x.qml:14"]);
}
