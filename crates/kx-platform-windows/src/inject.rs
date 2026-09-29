//! The input injector: taps, chords and text for the app in front, every event tagged as ours.
//! Building the events and recovering from a partial send are pure; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows injector runs this logic; tests cover it everywhere."
    )
)]

use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, Scan, TextRoute};

use crate::keys::{self, ENTER, LINE_FEED};
use crate::strokes::{self, Key, KeyEvent};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{WinInjector, input, send, send_keys, vk_key};

/// The OS calls the injector makes: Win32 on Windows, a recorder in tests.
pub(crate) trait Os {
    /// A keyboard layout handle.
    type Layout: Copy;

    /// Sends `events` in one batch and returns how many the OS took.
    fn send(&self, events: &[KeyEvent]) -> usize;

    /// The layout of the app in front.
    fn layout(&self) -> Self::Layout;

    /// `VkKeyScanExW`: the key and shift state that type UTF-16 unit `unit` in `layout`.
    fn key_scan(&self, unit: u16, layout: Self::Layout) -> i16;

    /// `MapVirtualKeyExW`: the scan code of virtual key `vk` in `layout`, with its prefix byte.
    fn scan_of(&self, vk: u16, layout: Self::Layout) -> u32;
}

/// Sends `events` in one batch; after a partial send it releases what is still down and reports the count.
pub(crate) fn deliver<O: Os>(os: &O, events: &[KeyEvent]) -> Result<(), InjectError> {
    let total = events.len();
    if total == 0 {
        return Ok(());
    }
    let sent = os.send(events).min(total);
    if sent == total {
        return Ok(());
    }
    let held = strokes::still_down(&events[..sent]);
    if !held.is_empty() {
        // Best effort: when this is refused too, nothing more can be done.
        let _ = os.send(&held);
    }
    Err(InjectError::Refused { sent, total })
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
    let layout = os.layout();
    let key = |vk| Key::vk(vk, os.scan_of(vk, layout));
    let mut events = Vec::new();
    for c in line_breaks(text) {
        let unit = u16::try_from(u32::from(c)).map_err(|_| InjectError::Unmappable)?;
        let found = keys::key_for(os.key_scan(unit, layout)).ok_or(InjectError::Unmappable)?;
        let mut held = Vec::new();
        if found.shift {
            held.push(key(keys::VK_SHIFT));
        }
        if found.altgr {
            held.extend(keys::ALTGR_VKS.map(key));
        }
        held.push(key(found.vk));
        events.extend(strokes::combo(&held));
    }
    Ok(events)
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
