//! Our keyboard on top at a key before the gate clicks it, right after the window in front changed by itself: how long
//! the key stayed covered, and on a timeout where the key and the keyboard were, what lay over the key and why.
#![cfg(windows)]

use std::time::Instant;

use serde_json::{Value, json};
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOPMOST;

use crate::g9win::ex_of;
use crate::g19chips::cover;
use crate::product::{self, Keyboard};
use crate::win::{self, ms_since};

/// The ms from `probe`'s first no to its yes, asked every `poll_ms` for `wait_ms`: 0 for a yes at once, else its last
/// no.
pub fn covered_ms(
    wait_ms: u64,
    poll_ms: u64,
    mut probe: impl FnMut() -> Result<(), String>,
) -> Result<u64, String> {
    let (mut first, mut last) = (None, String::new());
    let mut ask = || match probe() {
        Ok(()) => Some(first.map_or(0, ms_since)),
        Err(e) => {
            first.get_or_insert_with(Instant::now);
            last = e;
            None
        }
    };
    win::poll_until(wait_ms, poll_ms, &mut ask).ok_or(last)
}

/// The screen numbers of key `id` on keyboard `kb`: the key's centre, both windows' boxes and the keyboard's band.
fn spot(kb: &Keyboard, id: &str) -> String {
    let at = kb.at(id).unwrap_or_default();
    let edges = |w| win::rect(w).map(|r| [r.left, r.top, r.right, r.bottom]);
    let top = ex_of(kb.hwnd) & WS_EX_TOPMOST.0 != 0;
    format!(
        "key at {},{}; keyboard {:?}, topmost {top}; over the key {:?}",
        at.x,
        at.y,
        edges(kb.hwnd),
        edges(win::root_at(at))
    )
}

/// The ms our keyboard's key `id` stayed covered, within one follow period of the keyboard, as the app in front just
/// changed; asked every `poll_ms`. A key still covered then gets the cover note, read once more.
pub fn wait(poll_ms: u64, id: &str) -> Result<u64, String> {
    let follow = product::follow_ms(poll_ms)?;
    let got = covered_ms(follow, poll_ms, || {
        let kb = Keyboard::find()?;
        kb.top_at(id)
            .map(drop)
            .map_err(|e| format!("{e}; {}", spot(&kb, id)))
    });
    got.map_err(|e| format!("{e}; cover {}", why(id)))
}

/// Why key `id` stayed covered, on the keyboard found afresh.
fn why(id: &str) -> Value {
    Keyboard::find().map_or_else(|e| json!({ "error": e }), |kb| cover::note(&kb, id))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn a_key_on_top_at_once_counts_nothing_and_one_covered_waits_or_says_why() {
        assert_eq!(covered_ms(0, 0, || Ok(())), Ok(0));
        let asked = Cell::new(0);
        let later = || {
            asked.set(asked.get() + 1);
            if asked.get() < 3 {
                Err("covered".into())
            } else {
                Ok(())
            }
        };
        assert!(
            covered_ms(1_000, 0, later).is_ok(),
            "on top at the third look"
        );
        assert_eq!(asked.get(), 3);
        let never = covered_ms(0, 0, || Err("[Notepad] covers the key".into()));
        assert_eq!(never, Err("[Notepad] covers the key".into()));
    }
}
