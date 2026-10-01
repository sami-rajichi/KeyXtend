//! What a screen reader meets on the product keyboard, by board key: keys UI Automation lacks, keys with no name or no
//! way to press them, and character keys whose name is not what their layout types. G10 and G26 check these; G2 and
//! G4 click the character keys that type.
#![cfg(windows)]

use kx_module_api::ports::{KeyChars, LayoutId, LayoutSource, Scan, Slot};
use kx_platform_windows::{WinLayouts, layouts};

use crate::board;
use crate::keymap::KeyRead;

/// A character key in one layout: its scan code and id, the name it should have (what it types with no modifier, empty
/// when it types nothing), and what one press types at once (none for a dead key or one that types nothing printable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharKey {
    /// Its scan code.
    pub scan: Scan,
    /// Its board id.
    pub id: String,
    /// The name it should have.
    pub name: String,
    /// What one press types at once.
    pub types: Option<String>,
}

impl CharKey {
    /// A key named `name` that types `types`, for tests, which find keys by id; its scan code is none.
    #[cfg(test)]
    pub fn new(id: &str, name: &str, types: Option<&str>) -> Self {
        Self {
            scan: Scan::new(0),
            id: id.into(),
            name: name.into(),
            types: types.map(Into::into),
        }
    }
}

/// What a key types with no modifier, when that is a printable character at once; a dead key waits for the next.
pub fn typed_by(k: &KeyChars) -> Option<String> {
    let live = !k.dead.contains(&Slot::Base);
    (live && layouts::is_printable(&k.base)).then(|| k.base.clone())
}

/// The board's character keys in `layout`.
pub fn char_keys(layout: LayoutId) -> Result<Vec<CharKey>, String> {
    let keys = board::char_keys()?;
    let scans: Vec<_> = keys.iter().map(|(sc, _)| *sc).collect();
    let chars = WinLayouts.keys(layout, &scans);
    let key = |((scan, id), c)| CharKey {
        scan,
        id,
        types: typed_by(&c),
        name: c.base,
    };
    Ok(keys.into_iter().zip(chars).map(key).collect())
}

/// The first button UI Automation lists with board id `id`.
fn find<'a>(read: &'a [KeyRead], id: &str) -> Option<&'a KeyRead> {
    read.iter().find(|k| k.id == id)
}

/// The board ids UI Automation did not list.
pub fn missing(read: &[KeyRead], ids: &[String]) -> Vec<String> {
    ids.iter()
        .filter(|id| find(read, id).is_none())
        .cloned()
        .collect()
}

/// The board keys a screen reader would call only "button"; a character key in `want` that types nothing may.
pub fn unnamed(read: &[KeyRead], ids: &[String], want: &[CharKey]) -> Vec<String> {
    let types_nothing = |id: &str| want.iter().any(|c| c.id == id && c.name.is_empty());
    let blank = |id: &&String| {
        let empty = find(read, id).is_some_and(|k| k.name.trim().is_empty());
        empty && !types_nothing(id)
    };
    ids.iter().filter(blank).cloned().collect()
}

/// The board keys Narrator and NVDA cannot press: no Invoke pattern.
pub fn not_invokable(read: &[KeyRead], ids: &[String]) -> Vec<String> {
    let stuck = |id: &&String| find(read, id).is_some_and(|k| !k.invokable);
    ids.iter().filter(stuck).cloned().collect()
}

/// Character keys named unlike what their layout types, as `id: "got" for "wanted"`.
pub fn misnamed(read: &[KeyRead], want: &[CharKey]) -> Vec<String> {
    let wrong = |c: &CharKey| {
        let k = find(read, &c.id)?;
        (k.name != c.name).then(|| format!("{}: {:?} for {:?}", c.id, k.name, c.name))
    };
    want.iter().filter_map(wrong).collect()
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::RECT;

    use super::*;

    fn key(id: &str, name: &str, invokable: bool) -> KeyRead {
        KeyRead {
            id: id.into(),
            name: name.into(),
            rect: RECT::default(),
            invokable,
        }
    }

    /// Part of the walk recorded from the debug app on 2026-10-01 (US layout in front).
    fn recorded() -> Vec<KeyRead> {
        vec![
            key(
                "QGuiApplication.KbWindow_QMLTYPE_17.Button",
                "Smaller",
                true,
            ),
            key("esc", "Escape", true),
            key("c-0x0029", "`", true),
            key("c-0x0010", "q", true),
            key("caps", "Caps Lock", true),
            key("c-0x0056", "\\", true),
            key("power", "Power", true),
        ]
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_key_counts_only_when_it_types_a_printable_character_at_once() {
        let k = |base: &str, dead: Vec<Slot>| KeyChars {
            base: base.into(),
            dead,
            ..KeyChars::default()
        };
        assert_eq!(typed_by(&k("a", vec![])), Some("a".to_string()));
        assert_eq!(typed_by(&k("ض", vec![])), Some("ض".to_string()));
        assert_eq!(
            typed_by(&k("^", vec![Slot::Base])),
            None,
            "a dead key waits"
        );
        assert_eq!(typed_by(&k("^", vec![Slot::Shift])), Some("^".to_string()));
        for nothing in ["", " ", "\r", "\u{1b}"] {
            assert_eq!(typed_by(&k(nothing, vec![])), None, "{nothing:?}");
        }
    }

    #[test]
    fn the_recorded_walk_names_and_presses_every_key_it_lists() {
        let read = recorded();
        let board = ids(&["esc", "c-0x0029", "c-0x0010", "caps", "c-0x0056", "power"]);
        assert!(missing(&read, &board).is_empty());
        assert!(unnamed(&read, &board, &[]).is_empty());
        assert!(not_invokable(&read, &board).is_empty());
        let want = [
            CharKey::new("c-0x0010", "q", Some("q")),
            CharKey::new("c-0x0056", "\\", Some("\\")),
            CharKey::new("c-0x0029", "`", Some("`")),
        ];
        assert!(misnamed(&read, &want).is_empty());
    }

    #[test]
    fn a_lost_unnamed_stuck_or_misnamed_key_is_named() {
        let mut read = recorded();
        read[2].name = " ".into();
        read[3].invokable = false;
        let board = ids(&["esc", "c-0x0029", "c-0x0010", "lang"]);
        assert_eq!(missing(&read, &board), ["lang"]);
        assert_eq!(unnamed(&read, &board, &[]), ["c-0x0029"]);
        assert_eq!(not_invokable(&read, &board), ["c-0x0010"]);
        let want = [
            CharKey::new("c-0x0010", "й", Some("й")),
            CharKey::new("lang", "x", None),
        ];
        assert_eq!(misnamed(&read, &want), [r#"c-0x0010: "q" for "й""#]);
    }

    #[test]
    fn a_key_that_types_nothing_may_go_unnamed() {
        let mut read = recorded();
        read[2].name = String::new();
        let board = ids(&["c-0x0029", "c-0x0010"]);
        let want = [
            CharKey::new("c-0x0029", "", None),
            CharKey::new("c-0x0010", "q", Some("q")),
        ];
        assert!(unnamed(&read, &board, &want).is_empty());
        assert!(misnamed(&read, &want).is_empty());
        let types = [CharKey::new("c-0x0029", "`", Some("`"))];
        assert_eq!(unnamed(&read, &board, &types), ["c-0x0029"]);
    }
}
