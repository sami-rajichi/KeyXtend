//! The shipped text's lines found by their key within a row and their fields by name, never by a value, so a
//! check-day edit of a value breaks no test that edits that line.

use super::{OWN, TEXT};

/// What starts each row, on a line of its own: the table-wide lines come before the first.
const ROW: &str = "\n[[services]]\n";

/// The line `key = …` in `part`, by its key alone.
fn keyed(part: &'static str, key: &str) -> &'static str {
    let named = format!("{key} = ");
    let line = part.lines().find(|l| l.starts_with(&named));
    line.unwrap_or_else(|| panic!("no {key} line"))
}

/// Row `id`'s part of `text`, from its id line to the next row.
fn row_of(text: &'static str, id: &str) -> &'static str {
    let at = text.find(&format!("id = \"{id}\""));
    let at = at.unwrap_or_else(|| panic!("no row {id}"));
    text[at..].split(ROW).next().unwrap_or_default()
}

/// Groq's line `key = …`; Groq is the first row, so the line's first match in the shipped text is its own.
pub(super) fn groq_line(key: &str) -> &'static str {
    keyed(row_of(TEXT, "groq"), key)
}

/// The table-wide line `key = …`, above the rows.
pub(super) fn head_line(key: &str) -> &'static str {
    keyed(TEXT.split(ROW).next().unwrap_or_default(), key)
}

/// Row `id`'s line `key = …` in the second file.
pub(super) fn own_line(id: &str, key: &str) -> &'static str {
    keyed(row_of(OWN, id), key)
}

/// The edit that turns row `id`'s line `key = …` into `to`: the row from its id line through that line, so its first
/// match is that row's even where an earlier row has the same line.
pub(super) fn row_edit(id: &str, key: &str, to: &str) -> (&'static str, String) {
    let row = row_of(TEXT, id);
    let line = keyed(row, key);
    let end = row.find(line).unwrap_or_default() + line.len();
    (&row[..end], format!("{}{to}", &row[..end - line.len()]))
}

/// Where field `field`'s value sits in `line`: a quoted text whole, else up to the next comma, space or brace.
fn span(line: &str, field: &str) -> (usize, usize) {
    let named = format!("{field} = ");
    let at = line
        .find(&named)
        .unwrap_or_else(|| panic!("no {field} in {line}"));
    let (start, rest) = (at + named.len(), &line[at + named.len()..]);
    let len = match rest.strip_prefix('"') {
        Some(text) => text.find('"').map_or(rest.len(), |end| end + 2),
        None => rest.find([',', ' ', '}']).unwrap_or(rest.len()),
    };
    (start, len)
}

/// Field `field`'s value in `line`, as written.
pub(super) fn field_of<'a>(line: &'a str, field: &str) -> &'a str {
    let (start, len) = span(line, field);
    &line[start..start + len]
}

/// `line` with field `field`'s value set to `value`, as written.
pub(super) fn with_field(line: &str, field: &str, value: &str) -> String {
    let (start, len) = span(line, field);
    format!("{}{value}{}", &line[..start], &line[start + len..])
}
