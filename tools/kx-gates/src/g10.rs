//! G10 screen readers, the automatic part (roadmap G10): every key is found by UI Automation, named (character keys
//! after the layout in front, ADR-0020) and pressable, and Invoke on each, as Narrator and NVDA press, types into
//! target-window; Narrator and NVDA themselves are the owner's.
#![cfg(windows)]

use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use serde_json::{Value, json};

use crate::apps::{self, Ctx};
use crate::keymap::{KeyRead, Reader};
use crate::product::{self, Keyboard};
use crate::win::sleep_ms;
use crate::{board, launch, tlog, walk};

/// What the walk found wrong, by board key.
#[derive(Debug, Default, PartialEq, serde::Serialize)]
struct Findings {
    missing: Vec<String>,
    unnamed: Vec<String>,
    not_invokable: Vec<String>,
    misnamed: Vec<String>,
}

impl Findings {
    /// The walk's findings over `read` for the board's `ids` and its character keys.
    fn of(read: &[KeyRead], ids: &[String], want: &[walk::CharKey]) -> Self {
        Self {
            missing: walk::missing(read, ids),
            unnamed: walk::unnamed(read, ids),
            not_invokable: walk::not_invokable(read, ids),
            misnamed: walk::misnamed(read, want),
        }
    }

    /// True when nothing is wrong.
    fn clean(&self) -> bool {
        *self == Self::default()
    }
}

/// The character keys to press and the text they type, in board order: keys that type a printable character at once.
fn to_press(keys: &[walk::CharKey]) -> (Vec<String>, String) {
    let live = keys
        .iter()
        .filter_map(|k| Some((k.id.clone(), k.types.as_deref()?)));
    live.fold(
        (Vec::new(), String::new()),
        |(mut ids, mut text), (id, t)| {
            ids.push(id);
            text.push_str(t);
            (ids, text)
        },
    )
}

/// True when target-window got exactly what the presses should type.
fn typed_all(expected: &str, got: &str) -> bool {
    !expected.is_empty() && expected == got
}

/// Presses `ids` through UI Automation on keyboard `kb`, then reads what target-window got.
fn press_all(ctx: &Ctx, kb: &Keyboard, ids: &[String]) -> Result<String, String> {
    let reader = Reader::new()?;
    for id in ids {
        reader.invoke(kb.hwnd, id)?;
        sleep_ms(ctx.cfg.g10.press_gap_ms);
    }
    sleep_ms(ctx.cfg.g10.drain_ms);
    let log = std::fs::read_to_string(&ctx.target.log).map_err(|e| format!("target log: {e}"))?;
    Ok(tlog::decode(&tlog::parse(&log)))
}

/// Walks and presses the keyboard with target-window in front, in its layout.
fn check(ctx: &Ctx, kb: &Keyboard) -> Result<Value, String> {
    let follow_ms = product::follow_ms(ctx.cfg.timing.poll_ms)?;
    sleep_ms(follow_ms);
    let layout = WinLayouts
        .foreground()
        .ok_or("no app in front to read a layout from")?;
    let want = walk::char_keys(layout)?;
    let read = Reader::new()?.walk(kb.hwnd)?;
    let found = Findings::of(&read, &board::ids()?, &want);
    let (ids, expected) = to_press(&want);
    let got = press_all(ctx, kb, &ids)?;
    let typed = typed_all(&expected, &got);
    let names: Vec<(&str, &str)> = read
        .iter()
        .map(|k| (k.id.as_str(), k.name.as_str()))
        .collect();
    Ok(json!({
        "layout": layout.to_string(), "found": found, "pressed": ids.len(), "typed_ok": typed,
        "expected": expected, "got": got, "names": names, "follow_ms": follow_ms, "pass": found.clean() && typed,
    }))
}

/// Runs G10 on the product, called `name`; the names it records are for the owner to read in each language.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let target = launch::to_front(ctx, launch::start_target(ctx)?)?;
    let out = check(ctx, &kb);
    let left_open = apps::clean_up(ctx, target);
    let mut v = out.map_err(|e| format!("{e}; left open: {left_open:?}"))?;
    v["gate"] = json!("G10");
    v["keyboard"] = json!(name);
    v["left_open"] = json!(left_open);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::RECT;

    use super::*;

    use crate::walk::CharKey;

    #[test]
    fn only_keys_that_type_at_once_are_pressed_in_board_order() {
        let keys = [
            CharKey::new("c-0x0029", "`", Some("`")),
            CharKey::new("c-0x001A", "^", None),
            CharKey::new("c-0x0056", "", None),
            CharKey::new("c-0x0010", "ض", Some("ض")),
        ];
        let (ids, text) = to_press(&keys);
        assert_eq!(
            ids,
            ["c-0x0029", "c-0x0010"],
            "a dead key waits; an empty key types nothing"
        );
        assert_eq!(text, "`ض");
    }

    #[test]
    fn the_presses_pass_only_when_the_whole_text_arrived() {
        assert!(typed_all("`1ض", "`1ض"));
        assert!(!typed_all("`1ض", "`1"), "a lost character");
        assert!(!typed_all("", ""), "nothing pressed checks nothing");
    }

    #[test]
    fn findings_are_clean_only_when_every_key_is_found_named_pressable_and_named_right() {
        let k = |id: &str, name: &str| KeyRead {
            id: id.into(),
            name: name.into(),
            rect: RECT::default(),
            invokable: true,
        };
        let read = [k("esc", "Escape"), k("c-0x0010", "q")];
        let ids = ["esc".to_string(), "c-0x0010".to_string()];
        assert!(Findings::of(&read, &ids, &[CharKey::new("c-0x0010", "q", Some("q"))]).clean());
        let f = Findings::of(&read, &ids, &[CharKey::new("c-0x0010", "a", Some("a"))]);
        assert_eq!(f.misnamed.len(), 1);
        assert!(!f.clean());
    }
}
