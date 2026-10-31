//! kx-ui's texts as the product ships them, in English, French and Arabic, for gates that know a window by its title
//! or read a word or a number back out of an accessible name.
#![cfg(windows)]

/// Starts a comment line.
const COMMENT: char = '#';
/// Starts an attribute, and parts a message from it in a key.
const ATTR: char = '.';
/// Starts a term's name.
const TERM: char = '-';
/// Starts a variable's name.
const VAR: char = '$';
/// Opens and closes a string literal placeable.
const QUOTE: char = '"';
/// Starts an escape inside a string literal, which this reader never reads.
const ESCAPE: char = '\\';

/// Each language's tag, main file and clipboard panel file, which kx-ui loads into one bundle; workspace paths from
/// this file, all of kx-ui's (checked).
const FILES: [(&str, &str, &str); 3] = [
    (
        "en",
        include_str!("../../../crates/kx-ui/i18n/en.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/en.ftl"),
    ),
    (
        "fr",
        include_str!("../../../crates/kx-ui/i18n/fr.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/fr.ftl"),
    ),
    (
        "ar",
        include_str!("../../../crates/kx-ui/i18n/ar.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/ar.ftl"),
    ),
];

/// Each language's tag and its files as one text, as kx-ui's bundle holds them.
fn bundles() -> impl Iterator<Item = (&'static str, String)> {
    FILES
        .iter()
        .map(|(lang, main, clip)| (*lang, format!("{main}\n{clip}")))
}

/// The text of `key` in `ftl`, with each `{ -term }` filled in: `group.name` names a message's attribute, a plain name
/// a message or a term; none when it is missing.
pub fn text(ftl: &str, key: &str) -> Option<String> {
    fill(ftl, written(ftl, key)?)
}

/// The value of `key` in `ftl` as written, on one line; none when it is missing or spans lines.
fn written<'a>(ftl: &'a str, key: &str) -> Option<&'a str> {
    let (msg, attr) = key
        .split_once(ATTR)
        .map_or((key, None), |(m, a)| (m, Some(a)));
    let mut current = "";
    for line in ftl.lines() {
        let entry = line.trim();
        if entry.starts_with(COMMENT) {
            continue;
        }
        let Some((name, value)) = entry.split_once('=') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        let wanted = if line.starts_with(char::is_whitespace) {
            current == msg && attr.is_some() && name.strip_prefix(ATTR) == attr
        } else {
            current = name;
            attr.is_none() && name == msg
        };
        if wanted {
            return (!value.is_empty()).then_some(value);
        }
    }
    None
}

/// The text of string literal `term`, quotes and all, when it holds no escape; none for anything else.
fn literal(term: &str) -> Option<&str> {
    let inner = term.strip_prefix(QUOTE)?.strip_suffix(QUOTE)?;
    (!inner.contains([QUOTE, ESCAPE])).then_some(inner)
}

/// `value` with each `{ -term }` filled from `ftl` and each `{"literal"}` by its text; none for any other placeable or
/// a term that holds one.
fn fill(ftl: &str, value: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}')?;
        let term = rest[open + 1..close].trim();
        let named = || {
            let found = term.starts_with(TERM).then(|| written(ftl, term));
            found.flatten().filter(|t| !t.contains('{'))
        };
        let filled = literal(term).or_else(named)?;
        out.push_str(filled);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// The text of `key` in `ftl` around its one placeable `{ $var }`: what comes before it and after it, terms filled;
/// none when the key, the placeable or a term is missing.
fn split(ftl: &str, key: &str, var: &str) -> Option<(String, String)> {
    let value = written(ftl, key)?;
    let mut from = 0;
    while let Some(open) = value[from..].find('{').map(|at| from + at) {
        let close = open + value[open..].find('}')?;
        if value[open + 1..close].trim().strip_prefix(VAR) == Some(var) {
            return Some((fill(ftl, &value[..open])?, fill(ftl, &value[close + 1..])?));
        }
        from = close + 1;
    }
    None
}

/// The text of `key` in every shipped language split around `{ $var }`, in file order; an error naming the first
/// language that lacks it.
pub fn around(key: &str, var: &str) -> Result<Vec<(String, String)>, String> {
    bundles()
        .map(|(lang, ftl)| {
            split(&ftl, key, var).ok_or_else(|| format!("kx-ui has no {key} with ${var} in {lang}"))
        })
        .collect()
}

/// The text of `key` in every shipped language, in file order; an error naming the first language that lacks it.
pub fn every(key: &str) -> Result<Vec<String>, String> {
    bundles()
        .map(|(lang, ftl)| text(&ftl, key).ok_or_else(|| format!("kx-ui has no {key} in {lang}")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A translation file with a comment, a term, a message with attributes and a plain message.
    const FTL: &str = "## The product's name.\n-app = KeyXtend\n\nwindow =\n    .keyboard = { -app } keyboard\n    \
                       .outline = Scroll outline\ncatch = Click to end\nother =\n    .outline = Not this one\n";

    #[test]
    fn a_key_names_an_attribute_or_a_plain_message_and_its_terms_are_filled() {
        assert_eq!(
            text(FTL, "window.keyboard").as_deref(),
            Some("KeyXtend keyboard")
        );
        assert_eq!(
            text(FTL, "window.outline").as_deref(),
            Some("Scroll outline")
        );
        assert_eq!(text(FTL, "other.outline").as_deref(), Some("Not this one"));
        assert_eq!(text(FTL, "catch").as_deref(), Some("Click to end"));
        assert_eq!(text(FTL, "-app").as_deref(), Some("KeyXtend"));
    }

    #[test]
    fn a_missing_message_attribute_or_term_gives_none() {
        assert_eq!(text(FTL, "window.nowhere"), None);
        assert_eq!(text(FTL, "nowhere.outline"), None);
        assert_eq!(
            text(FTL, "window"),
            None,
            "a message with no value of its own"
        );
        assert_eq!(text("x =\n    .a = { -gone } here\n", "x.a"), None);
    }

    #[test]
    fn arabic_text_comes_back_as_written() {
        let ar = "-app = KeyXtend\nwindow =\n    .keyboard = لوحة مفاتيح { -app }\n";
        assert_eq!(
            text(ar, "window.keyboard").as_deref(),
            Some("لوحة مفاتيح KeyXtend")
        );
    }

    #[test]
    fn the_languages_are_exactly_the_files_kx_ui_ships() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/i18n");
        let entries = std::fs::read_dir(dir).expect("kx-ui's i18n folder");
        // Feature folders, such as the clipboard panel's, hold no window title, so only the main files count.
        let files = entries
            .map(|e| e.expect("an entry"))
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()));
        let mut shipped: Vec<String> = files
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        shipped.sort();
        let mut ours: Vec<String> = FILES.iter().map(|(tag, ..)| format!("{tag}.ftl")).collect();
        ours.sort();
        assert_eq!(shipped, ours);
    }

    #[test]
    fn a_variables_text_splits_around_it_with_its_terms_filled() {
        let ftl = "-app = KeyXtend\nchip =\n    .name = { -app } pick: { $word }!\n";
        let got = split(ftl, "chip.name", "word");
        assert_eq!(got, Some(("KeyXtend pick: ".into(), "!".into())));
        assert_eq!(split(ftl, "chip.name", "other"), None, "another variable");
        assert_eq!(split(ftl, "chip.none", "word"), None, "no such key");
        let ar = around("chip.name", "word").expect("in all three");
        assert_eq!(ar.len(), FILES.len());
        assert!(ar.iter().all(|(before, _)| !before.is_empty()));
        let e = around("chip.name", "nothing").expect_err("no such variable");
        assert!(e.contains("$nothing") && e.contains("en"), "{e}");
    }

    #[test]
    fn a_string_literal_placeable_is_its_text_and_an_escaped_one_gives_none() {
        let ftl = "x =\n    .join = {\", \"}\n    .ar = {\"، \"}\n    .esc = {\"a\\u0020\"}\n";
        assert_eq!(text(ftl, "x.join").as_deref(), Some(", "));
        assert_eq!(text(ftl, "x.ar").as_deref(), Some("، "));
        assert_eq!(text(ftl, "x.esc"), None, "no escapes are read");
    }

    #[test]
    fn the_clip_panels_texts_are_read_in_every_language_beside_the_main_ones() {
        let insert = around("clip.insert", "count").expect("in all three");
        assert_eq!(insert[0], ("Insert (".into(), ")".into()));
        assert!(
            insert
                .iter()
                .all(|(before, after)| !before.is_empty() && after == ")")
        );
        assert_eq!(
            every("clip.join").expect("in all three"),
            [", ", ", ", "، "]
        );
        assert!(
            every("window.outline").is_ok(),
            "the main files still count"
        );
    }

    #[test]
    fn the_clip_folder_holds_exactly_the_languages_of_the_main_files() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/kx-ui/i18n/clip");
        let entries = std::fs::read_dir(dir).expect("kx-ui's clip folder");
        let mut shipped: Vec<String> = entries
            .map(|e| {
                e.expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        shipped.sort();
        let mut ours: Vec<String> = FILES.iter().map(|(tag, ..)| format!("{tag}.ftl")).collect();
        ours.sort();
        assert_eq!(shipped, ours);
    }

    #[test]
    fn every_shipped_language_has_the_outline_title_and_a_missing_key_names_the_language() {
        let titles = every("window.outline").expect("in all three");
        assert_eq!(titles.len(), FILES.len());
        assert!(titles.iter().all(|t| !t.is_empty() && !t.contains('{')));
        let e = every("window.nowhere").expect_err("in none");
        assert!(e.contains("window.nowhere") && e.contains("en"), "{e}");
    }
}
