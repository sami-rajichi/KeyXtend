//! The input injector: taps, chords and text for the app in front, every event tagged as ours.
//! Building the events and recovering from a partial send are pure; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows injector runs this logic; tests cover it everywhere."
    )
)]

use std::fmt;

use kx_module_api::ports::{InjectError, LayoutId, Scan, TextRoute};
use kx_module_api::{REDACTED_MARKER, Redacted};

use crate::keys::{self, ENTER, LINE_FEED};
use crate::layouts::TEXT_CAP;
use crate::strokes::{self, Key, KeyEvent};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{WinInjector, input, send, send_keys, vk_key};

/// The OS calls the injector makes: Win32 on Windows, a recorder in tests.
pub(crate) trait Os {
    /// Sends `events` in one batch and returns how many the OS took.
    fn send(&self, events: &[KeyEvent]) -> usize;

    /// The layout of the window with the keyboard in the app in front; `None` with no app in front.
    fn layout(&self) -> Option<LayoutId>;

    /// `VkKeyScanExW`: the key and shift state that type UTF-16 unit `unit` in `layout`.
    fn key_scan(&self, unit: u16, layout: LayoutId) -> i16;

    /// `MapVirtualKeyExW`: the scan code of virtual key `vk` in `layout`, with its prefix byte.
    fn scan_of(&self, vk: u16, layout: LayoutId) -> u32;

    /// True while Caps Lock is on.
    fn caps_on(&self) -> bool;

    /// `ToUnicodeEx` for `vk` with `held` down and Caps Lock `caps`, changing no keyboard state:
    /// the count (negative for a dead key) and the text in `buf`.
    fn to_unicode(
        &self,
        vk: u16,
        held: &[u16],
        caps: bool,
        layout: LayoutId,
        buf: &mut [u16],
    ) -> i32;
}

/// Sends `events` in one batch; after a partial send it tries to release what is still down and reports the count.
pub(crate) fn deliver<O: Os>(os: &O, events: &[KeyEvent]) -> Result<(), InjectError> {
    let total = events.len();
    if total == 0 {
        return Ok(());
    }
    let sent = os.send(events).min(total);
    if sent == total {
        return Ok(());
    }
    release(os, &strokes::releases(&events[..sent]));
    Err(InjectError::Refused { sent, total })
}

/// Sends `ups`; what the OS refuses is tried once more, one event at a time, so one refusal cannot hold the rest.
fn release<O: Os>(os: &O, ups: &[KeyEvent]) {
    if ups.is_empty() {
        return;
    }
    let went = os.send(ups).min(ups.len());
    for event in &ups[went..] {
        // Best effort: after this retry nothing more can be done.
        let _ = os.send(std::slice::from_ref(event));
    }
}

/// The events of `held` pressed in order around a tap of `key`, all as scan codes.
pub(crate) fn chord_events(held: &[Scan], key: Scan) -> Vec<KeyEvent> {
    let keys: Vec<Key> = held.iter().chain([&key]).map(|&s| Key::Scan(s)).collect();
    strokes::combo(&keys)
}

/// Types `text` by `route`; the Keys route checks every character before anything is sent.
pub(crate) fn type_text<O: Os>(
    os: &O,
    text: &Redacted<String>,
    route: TextRoute,
) -> Result<(), InjectError> {
    let events = match route {
        TextRoute::Unicode => strokes::unicode(text.expose()),
        TextRoute::Keys => keys_events(os, text.expose())?,
    };
    deliver(os, &events)
}

/// The key presses that type `text` in the layout of the app in front; `Unmappable` when a character has no key.
fn keys_events<O: Os>(os: &O, text: &str) -> Result<Vec<KeyEvent>, InjectError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let layout = os.layout().ok_or(InjectError::Unmappable)?;
    let caps = os.caps_on();
    let key = |vk| Key::vk(vk, os.scan_of(vk, layout));
    let mut events = Vec::new();
    for c in line_breaks(text) {
        let press = press_for(os, c, layout, caps)?;
        let mut pressed: Vec<Key> = held(press.shift, press.altgr)
            .into_iter()
            .map(key)
            .collect();
        pressed.push(key(press.vk));
        events.extend(strokes::combo(&pressed));
        if press.dead {
            events.extend(strokes::combo(&[key(keys::VK_SPACE)]));
        }
    }
    Ok(events)
}

/// How one character is typed: a key, the modifiers held around it, and whether it is a dead key.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Press {
    /// The virtual key.
    pub vk: u16,
    /// Shift is held.
    pub shift: bool,
    /// Ctrl and Alt are held, as `AltGr`.
    pub altgr: bool,
    /// A dead key, which types its own character when Space follows.
    pub dead: bool,
}

/// Shows no key, so a log never holds what was typed.
impl fmt::Debug for Press {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Press({REDACTED_MARKER})")
    }
}

/// How `c` is typed in `layout` with Caps Lock `caps`: the key `VkKeyScanEx` names, in the Shift
/// state that `ToUnicodeEx` confirms types `c`; `Unmappable` when neither state does.
pub(crate) fn press_for<O: Os>(
    os: &O,
    c: char,
    layout: LayoutId,
    caps: bool,
) -> Result<Press, InjectError> {
    let unit = u16::try_from(u32::from(c)).map_err(|_| InjectError::Unmappable)?;
    let found = keys::key_for(os.key_scan(unit, layout)).ok_or(InjectError::Unmappable)?;
    let confirm = |shift| {
        let (typed, dead) = typed(os, found.vk, shift, found.altgr, caps, layout)?;
        (typed == unit).then_some(Press {
            vk: found.vk,
            shift,
            altgr: found.altgr,
            dead,
        })
    };
    confirm(found.shift)
        .or_else(|| confirm(!found.shift))
        .ok_or(InjectError::Unmappable)
}

/// The one unit `vk` types with the modifiers given, and whether it is a dead key (whose unit is its own
/// character); `None` when it types nothing or more than one unit.
fn typed<O: Os>(
    os: &O,
    vk: u16,
    shift: bool,
    altgr: bool,
    caps: bool,
    layout: LayoutId,
) -> Option<(u16, bool)> {
    let mut buf = [0; TEXT_CAP];
    let n = os.to_unicode(vk, &held(shift, altgr), caps, layout, &mut buf);
    (n.unsigned_abs() == 1).then_some((buf[0], n < 0))
}

/// The virtual keys held around a key for Shift and for `AltGr` (Ctrl and Alt).
fn held(shift: bool, altgr: bool) -> Vec<u16> {
    let shift = shift.then_some(keys::VK_SHIFT);
    let altgr = keys::ALTGR_VKS.into_iter().filter(|_| altgr);
    shift.into_iter().chain(altgr).collect()
}

/// The characters of `text`, with each line break (LF or CR LF) as one Enter.
fn line_breaks(text: &str) -> impl Iterator<Item = char> + '_ {
    let mut after_enter = false;
    text.chars().filter_map(move |c| {
        let crlf = after_enter && c == LINE_FEED;
        after_enter = c == ENTER;
        match c {
            _ if crlf => None,
            LINE_FEED => Some(ENTER),
            _ => Some(c),
        }
    })
}

#[cfg(test)]
mod tests;
