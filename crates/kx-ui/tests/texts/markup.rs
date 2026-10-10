//! Text our QML draws is plain text, so no text from a file can make the uiAccess process load an image (security
//! review U-S1, ADR-0004).

use crate::qml_files;

/// Qt Quick's elements that draw or edit text.
const TEXT_TYPES: [&str; 5] = ["Text", "TextEdit", "TextInput", "TextArea", "TextField"];
/// The property that sets how text is read, and the one value allowed.
const FORMAT: &str = "textFormat";
const PLAIN: &str = "Text.PlainText";
/// What starts a comment, the quotes that start a string, and what escapes a quote inside one.
const COMMENT: &str = "//";
const QUOTES: [char; 3] = ['"', '\'', '`'];
const ESCAPE: char = '\\';
/// What ends a statement in a block: a line's end or a semicolon.
const LINE_END: char = '\n';
const STATEMENT_END: char = ';';
/// The file that holds our text element, so a broken reader cannot pass the check.
const LABEL: &str = "Label.qml";

/// Line `l` as code: each string emptied and any comment dropped, so braces and names inside them do not count.
pub fn code(l: &str) -> String {
    let (mut out, mut quote, mut escaped) = (String::new(), None, false);
    for (i, c) in l.char_indices() {
        match quote {
            Some(_) if escaped => escaped = false,
            Some(_) if c == ESCAPE => escaped = true,
            Some(q) if c == q => {
                quote = None;
                out.push(c);
            }
            Some(_) => {}
            None if l[i..].starts_with(COMMENT) => break,
            None => {
                if QUOTES.contains(&c) {
                    quote = Some(c);
                }
                out.push(c);
            }
        }
    }
    out
}

/// Where each Qt text element opens in code line `l`: the byte after its type name and any space, at its brace.
fn opens(l: &str) -> Vec<usize> {
    let word = |c: char| c.is_alphanumeric() || c == '_' || c == '.';
    let mut at = Vec::new();
    for t in TEXT_TYPES {
        for (i, _) in l.match_indices(t) {
            let alone = !l[..i].chars().next_back().is_some_and(word);
            let rest = l[i + t.len()..].trim_start();
            if alone && rest.starts_with('{') {
                at.push(l.len() - rest.len());
            }
        }
    }
    at.sort_unstable();
    at
}

/// Each Qt text element in code lines `code`, as its line (from 0) and the byte of its brace.
fn openings(code: &[String]) -> Vec<(usize, usize)> {
    let each = code.iter().enumerate();
    each.flat_map(|(i, l)| opens(l).into_iter().map(move |b| (i, b)))
        .collect()
}

/// The lines (from 0) of `qml` where a Qt text element opens.
fn texts(qml: &str) -> Vec<usize> {
    let code: Vec<String> = qml.lines().map(code).collect();
    openings(&code).into_iter().map(|(i, _)| i).collect()
}

/// The statements of the block whose brace is at `at` in `code`, at its own depth: its properties, not its children's.
fn own(code: &[String], at: (usize, usize)) -> Vec<String> {
    let (mut depth, mut out, mut cur) = (0_usize, Vec::new(), String::new());
    let lines = code.iter().skip(at.0).enumerate();
    let text = lines.map(|(i, l)| if i == 0 { &l[at.1..] } else { l.as_str() });
    for c in text.flat_map(|l| l.chars().chain([LINE_END])) {
        match c {
            '{' => depth += 1,
            '}' if depth == 1 => break,
            '}' => depth = depth.saturating_sub(1),
            LINE_END | STATEMENT_END if depth == 1 => out.push(std::mem::take(&mut cur)),
            _ if depth == 1 => cur.push(c),
            _ => {}
        }
    }
    out.push(cur);
    out.iter().map(|s| s.split_whitespace().collect()).collect()
}

/// Each line of `qml` where text may read markup, as `<name>:<line>`: a text element without its own plain format, or
/// any line that sets another format.
fn markup(name: &str, qml: &str) -> Vec<String> {
    let code: Vec<String> = qml.lines().map(code).collect();
    let plain = format!("{FORMAT}:{PLAIN}");
    let bare = openings(&code)
        .into_iter()
        .filter(|&at| !own(&code, at).contains(&plain))
        .map(|(i, _)| i);
    let other = code.iter().enumerate();
    let other = other.filter(|(_, l)| l.contains(FORMAT) && !l.contains(PLAIN));
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

/// A one-line element, one with no space before its brace, and braces or a type name inside strings are all read.
#[test]
fn one_line_texts_tight_braces_and_braces_in_strings_are_read() {
    let qml = [
        "Item {",
        "    Text { text: name; textFormat: Text.PlainText }",
        "    Text { text: name }",
        "    Text{",
        "        text: \"{\"",
        "        textFormat: Text.PlainText",
        "    }",
        "    Text { text: \"}\"; textFormat: Text.PlainText }",
        "    Label { text: \"Text { }\" }",
        "}",
    ]
    .join("\n");
    assert_eq!(texts(&qml), [1, 2, 3, 7]);
    assert_eq!(markup("x.qml", &qml), ["x.qml:3"]);
}
