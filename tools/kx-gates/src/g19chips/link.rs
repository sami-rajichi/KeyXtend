//! The translate page's address as the browser shows it: the text's query key, found where the site's own link puts a
//! marker, and that key's value decoded, with `%XX` bytes and form spaces, whether the browser shows it encoded or not.
#![cfg(windows)]

use kx_mod_selection::{SelSettings, sites};

/// Starts a query.
const QUERY: char = '?';
/// Joins query parts.
const AND: char = '&';
/// Joins a key to its value.
const IS: char = '=';
/// Ends the query.
const FRAGMENT: char = '#';
/// Starts an encoded byte.
const PERCENT: u8 = b'%';
/// A form's space.
const PLUS: u8 = b'+';
/// The marker a site's link holds where its text goes; it encodes as itself.
pub const MARKER: &str = "kxmark";

/// The value of hex digit `b`.
fn hex(b: u8) -> Option<u8> {
    char::from(b)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

/// `s` with each `%XX` as its byte and each `+` as a space, read as UTF-8; none for a broken escape or bytes.
pub fn decode(s: &str) -> Option<String> {
    let (mut out, mut bytes) = (Vec::with_capacity(s.len()), s.bytes());
    while let Some(b) = bytes.next() {
        out.push(match b {
            PERCENT => hex(bytes.next()?)?
                .checked_mul(16)?
                .checked_add(hex(bytes.next()?)?)?,
            PLUS => b' ',
            other => other,
        });
    }
    String::from_utf8(out).ok()
}

/// The query parts of `link`: each key and its raw value, after the query mark and before the fragment.
fn parts(link: &str) -> impl Iterator<Item = (&str, &str)> {
    let query = link.split_once(QUERY).map_or("", |(_, q)| q);
    let query = query.split_once(FRAGMENT).map_or(query, |(q, _)| q);
    query.split(AND).filter_map(|p| p.split_once(IS))
}

/// The query key whose value is `marker` in `link`, a link the site built around that marker.
pub fn key_of<'a>(link: &'a str, marker: &str) -> Option<&'a str> {
    parts(link).find(|(_, v)| *v == marker).map(|(k, _)| k)
}

/// The raw value of query key `key` in `address`, up to the next part or the fragment.
pub fn value<'a>(address: &'a str, key: &str) -> Option<&'a str> {
    parts(address).find(|(k, _)| *k == key).map(|(_, v)| v)
}

/// The decoded value of query key `key` in `address`.
pub fn text_of(address: &str, key: &str) -> Option<String> {
    value(address, key).and_then(decode)
}

/// The query key the module's site puts the text in, found where its link puts `MARKER`.
pub fn site_key(s: &SelSettings) -> Option<String> {
    let site = sites().ok()?.get(&s.translate_site)?;
    let marked = site.link(MARKER, &site.target(&s.translate_to, ""), s.translate_chars)?;
    key_of(marked.expose(), MARKER).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A marker that encodes as itself.
    const MARK: &str = MARKER;

    #[test]
    fn the_modules_site_puts_its_text_in_the_key_its_link_names() {
        let s = SelSettings::from_defaults().expect("defaults");
        let key = site_key(&s).expect("a key");
        let site = sites()
            .expect("the table")
            .get(&s.translate_site)
            .expect("the site");
        let link = site.link("abc", "en", s.translate_chars).expect("a link");
        assert_eq!(text_of(link.expose(), &key), Some("abc".to_string()));
    }

    #[test]
    fn bytes_and_form_spaces_decode_and_a_broken_escape_is_none() {
        assert_eq!(decode("a%20b+c"), Some("a b c".to_string()));
        assert_eq!(decode("%D8%AD%D8%AF%D8%AF"), Some("حدد".to_string()));
        assert_eq!(
            decode("حدد هذا"),
            Some("حدد هذا".to_string()),
            "shown decoded"
        );
        for bad in ["%", "%2", "%G0", "%FF"] {
            assert_eq!(decode(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_sites_link_names_the_texts_key_and_the_address_gives_it_back() {
        let site = sites().expect("the table").first().expect("a site");
        let marked = site.link(MARK, "en", 50).expect("a link");
        let key = key_of(marked.expose(), MARK).expect("the key");
        let arabic = "حدد هذا السطر ثم اضغط زر الترجمة.";
        let sent = site.link(arabic, "en", 1000).expect("a link");
        assert_eq!(text_of(sent.expose(), key), Some(arabic.to_string()));
        let shown = sent.expose().trim_start_matches("https://");
        assert_eq!(text_of(shown, key), Some(arabic.to_string()), "no scheme");
    }

    #[test]
    fn a_key_is_read_whole_and_only_in_the_query() {
        let a = "https://x.test/p?atext=1&text=a%20b&op=t#text=no";
        assert_eq!(value(a, "text"), Some("a%20b"));
        assert_eq!(value("https://x.test/p?q=1", "text"), None);
        assert_eq!(value("https://x.test/text=1", "text"), None, "no query");
        assert_eq!(value("https://x.test/?text=", "text"), Some(""));
        assert_eq!(key_of("https://x.test/?a=1&b=kxmark", MARK), Some("b"));
        assert_eq!(key_of("https://x.test/?b=1", MARK), None);
    }
}
