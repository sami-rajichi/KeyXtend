//! The shipped text's lines found by their key within a row and their fields by name, never by a value, so a
//! check-day edit of a value breaks no test that edits that line.

use super::TEXT;

/// What starts each row, on a line of its own: the table-wide lines come before the first.
const ROW: &str = "\n[[services]]\n";

/// Where the line `key = …` starts in `part` and its length, found by its key alone.
fn line_at(part: &str, key: &str) -> (usize, usize) {
    let named = format!("{key} = ");
    let mut at = 0;
    for line in part.split_inclusive('\n') {
        if line.starts_with(&named) {
            return (at, line.trim_end_matches('\n').len());
        }
        at += line.len();
    }
    panic!("no {key} line")
}

/// The line `key = …` in `part`.
fn keyed(part: &'static str, key: &str) -> &'static str {
    let (at, len) = line_at(part, key);
    &part[at..at + len]
}

/// Where row `id`'s part of `text` starts, at its id line, and its length up to the next row.
fn row_span(text: &str, id: &str) -> (usize, usize) {
    let at = text.find(&format!("\nid = \"{id}\"\n"));
    let at = at.unwrap_or_else(|| panic!("no row {id}")) + 1;
    (at, text[at..].find(ROW).unwrap_or(text.len() - at))
}

/// Row `id`'s line `key = …` in `text`, either file of the table.
pub(crate) fn row_line(text: &'static str, id: &str, key: &str) -> &'static str {
    let (at, len) = row_span(text, id);
    keyed(&text[at..at + len], key)
}

/// Groq's line `key = …`; Groq is the first row, so the line's first match in the shipped text is its own.
pub(crate) fn groq_line(key: &str) -> &'static str {
    row_line(TEXT, "groq", key)
}

/// The table-wide line `key = …`, above the rows.
pub(crate) fn head_line(key: &str) -> &'static str {
    keyed(TEXT.split(ROW).next().unwrap_or_default(), key)
}

/// `text` with row `id`'s line of each key set to its `to`, in turn; a missing row or key fails the test.
pub(crate) fn row_edit(text: &str, id: &str, edits: &[(&str, &str)]) -> String {
    let (at, len) = row_span(text, id);
    let row = edits
        .iter()
        .fold(text[at..at + len].to_owned(), |row, (key, to)| {
            let (start, size) = line_at(&row, key);
            format!("{}{to}{}", &row[..start], &row[start + size..])
        });
    format!("{}{row}{}", &text[..at], &text[at + len..])
}

/// `text` with row `id`'s terms number raised by one, as newer terms raise it.
pub(crate) fn raised(text: &'static str, id: &str) -> String {
    let terms: u16 = field_of(row_line(text, id, "terms"), "terms")
        .parse()
        .unwrap();
    row_edit(text, id, &[("terms", &format!("terms = {}", terms + 1))])
}

/// Where field `field`'s value sits in `line`: the field's name starts the line or follows a space.
fn span(line: &str, field: &str) -> (usize, usize) {
    let named = format!(" {field} = ");
    let at = format!(" {line}").find(&named);
    let start = at.unwrap_or_else(|| panic!("no {field} in {line}")) + named.len() - 1;
    (start, value_len(&line[start..]))
}

/// The length of the value `rest` starts with: a quoted text, array or inline table whole, else up to the next comma,
/// space or close.
fn value_len(rest: &str) -> usize {
    let (mut depth, mut quote, mut escaped) = (0_usize, None, false);
    for (at, c) in rest.char_indices() {
        match quote {
            Some(_) if escaped => escaped = false,
            Some('"') if c == '\\' => escaped = true,
            Some(open) if c == open => quote = None,
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '[' || c == '{' => depth += 1,
            None if depth > 0 && (c == ']' || c == '}') => depth -= 1,
            None if depth == 0 && matches!(c, ',' | ' ' | ']' | '}') => return at,
            _ => {}
        }
    }
    rest.len()
}

/// Field `field`'s value in `line`, as written.
pub(crate) fn field_of<'a>(line: &'a str, field: &str) -> &'a str {
    let (start, len) = span(line, field);
    &line[start..start + len]
}

/// `line` with field `field`'s value set to `value`, as written.
pub(crate) fn with_field(line: &str, field: &str, value: &str) -> String {
    let (start, len) = span(line, field);
    format!("{}{value}{}", &line[..start], &line[start + len..])
}
