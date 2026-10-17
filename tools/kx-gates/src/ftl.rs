//! kx-ui's texts as the product ships them, in English, French and Arabic, for gates that know a window by its title.
#![cfg(windows)]

/// Starts a comment line.
const COMMENT: char = '#';
/// Starts an attribute, and parts a message from it in a key.
const ATTR: char = '.';
/// Starts a term's name.
const TERM: char = '-';

/// Each language's translation file, by its tag; workspace paths from this file, all of kx-ui's (checked).
const FILES: [(&str, &str); 3] = [
    ("en", include_str!("../../../crates/kx-ui/i18n/en.ftl")),
    ("fr", include_str!("../../../crates/kx-ui/i18n/fr.ftl")),
    ("ar", include_str!("../../../crates/kx-ui/i18n/ar.ftl")),
];

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

/// `value` with each `{ -term }` filled from `ftl`; none for any other placeable or a term that holds one.
fn fill(ftl: &str, value: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}')?;
        let term = rest[open + 1..close].trim();
        let filled = term
            .starts_with(TERM)
            .then(|| written(ftl, term))
            .flatten()
            .filter(|t| !t.contains('{'))?;
        out.push_str(filled);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// The text of `key` in every shipped language, in file order; an error naming the first language that lacks it.
pub fn every(key: &str) -> Result<Vec<String>, String> {
    FILES
        .iter()
        .map(|(lang, ftl)| text(ftl, key).ok_or_else(|| format!("kx-ui has no {key} in {lang}")))
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
        let mut shipped: Vec<String> = entries
            .map(|e| {
                e.expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        shipped.sort();
        let mut ours: Vec<String> = FILES.iter().map(|(tag, _)| format!("{tag}.ftl")).collect();
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
