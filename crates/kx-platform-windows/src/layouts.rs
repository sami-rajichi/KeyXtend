//! The layout source: the installed layouts, what each key types in them, and the switch request.
//! The logic runs over `Os`, so it is tested on every system; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows source runs this logic; tests cover it everywhere."
    )
)]

use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, Scan, Slot, SwitchError};

use crate::keys::{self, ALTGR_READ, VK_CAPITAL, VK_SHIFT};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{WinLayouts, ask, hkl, id};

/// Bytes in the key-state array `ToUnicodeEx` reads: one per virtual key.
const KEY_STATES: usize = 256;
/// Key-state bit of a key held down.
const DOWN: u8 = 0x80;
/// Key-state bit of a toggle key that is on.
const TOGGLED: u8 = 0x01;
/// Longest text read for one key, in UTF-16 units.
const TEXT_CAP: usize = 32;
/// Keys held for the Shift slots.
const SHIFT: [u16; 1] = [VK_SHIFT];

/// The key-state array `ToUnicodeEx` reads.
pub(crate) type KeyState = [u8; KEY_STATES];

/// The OS calls the layout source makes: Win32 on Windows, a fake in tests.
pub(crate) trait Os {
    /// A window that can take the keyboard.
    type Window: Copy;

    /// Copies the installed layouts into `buf` and returns how many it copied; with an empty `buf`, how many there are.
    fn list(&self, buf: &mut [LayoutId]) -> usize;

    /// The window with the keyboard in the app in front; `None` with no app in front.
    fn focus(&self) -> Option<Self::Window>;

    /// The layout of `window`'s thread; `None` when the window is gone.
    fn layout_of(&self, window: Self::Window) -> Option<LayoutId>;

    /// `MapVirtualKeyExW`: the virtual key of `key` in `layout`; 0 when it has none.
    fn vk_of(&self, key: Scan, layout: LayoutId) -> u16;

    /// `ToUnicodeEx` without changing any keyboard state: the count (negative for a dead key), the text in `buf`.
    fn to_unicode(
        &self,
        vk: u16,
        key: Scan,
        state: &KeyState,
        layout: LayoutId,
        buf: &mut [u16],
    ) -> i32;

    /// The language facts of `layout`.
    fn facts(&self, layout: LayoutId) -> LangFacts;

    /// Posts the switch request for `layout` to `window`.
    fn post(&self, window: Self::Window, layout: LayoutId) -> Result<(), SwitchError>;
}

/// The installed layouts, however many there are: Windows gives the count first.
pub(crate) fn installed<O: Os>(os: &O) -> Vec<LayoutId> {
    let mut list = vec![LayoutId::default(); os.list(&mut [])];
    let copied = os.list(&mut list);
    list.truncate(copied);
    list
}

/// The layout of the window with the keyboard in the app in front.
pub(crate) fn foreground<O: Os>(os: &O) -> Option<LayoutId> {
    os.focus().and_then(|window| os.layout_of(window))
}

/// What each of `scans` types in `layout`; all empty when the layout is not installed.
pub(crate) fn keys<O: Os>(os: &O, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
    if !installed(os).contains(&layout) {
        return vec![KeyChars::default(); scans.len()];
    }
    scans
        .iter()
        .map(|&key| key_chars(os, layout, key))
        .collect()
}

/// What `key` types in `layout` in each slot, and which slots are dead keys.
fn key_chars<O: Os>(os: &O, layout: LayoutId, key: Scan) -> KeyChars {
    let vk = os.vk_of(key, layout);
    let read = Slot::ALL.map(|slot| {
        let (held, caps) = slot_keys(slot);
        let (n, buf) = answer(os, layout, key, vk, &state(held, caps));
        read_typed(n, &buf)
    });
    let dead = Slot::ALL.into_iter().zip(&read);
    let dead = dead
        .filter(|(_, (_, d))| *d)
        .map(|(slot, _)| slot)
        .collect();
    let [base, shift, altgr, caps, caps_shift] = read.map(|(text, _)| text);
    KeyChars {
        base,
        shift,
        altgr,
        caps,
        caps_shift,
        dead,
    }
}

/// The keys held and the Caps Lock state that read slot `slot`.
fn slot_keys(slot: Slot) -> (&'static [u16], bool) {
    match slot {
        Slot::Base => (&[], false),
        Slot::Shift => (&SHIFT, false),
        Slot::AltGr => (&ALTGR_READ, false),
        Slot::Caps => (&[], true),
        Slot::CapsShift => (&SHIFT, true),
    }
}

/// Windows' answer for `key`, virtual key `vk`, in `state`: the count and the text; nothing when `vk` is 0.
fn answer<O: Os>(
    os: &O,
    layout: LayoutId,
    key: Scan,
    vk: u16,
    state: &KeyState,
) -> (i32, [u16; TEXT_CAP]) {
    let mut buf = [0; TEXT_CAP];
    if vk == 0 {
        return (0, buf);
    }
    let n = os.to_unicode(vk, key, state, layout, &mut buf);
    (n, buf)
}

/// True when `key` types anything in `layout` with `held` down, even a control character.
pub(crate) fn types_any<O: Os>(os: &O, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
    if !installed(os).contains(&layout) {
        return false;
    }
    let vks: Vec<u16> = held.iter().map(|&h| os.vk_of(h, layout)).collect();
    let vk = os.vk_of(key, layout);
    answer(os, layout, key, vk, &state(&vks, false)).0 != 0
}

/// The facts of `layout`; empty when it is not installed.
pub(crate) fn facts<O: Os>(os: &O, layout: LayoutId) -> LangFacts {
    if installed(os).contains(&layout) {
        os.facts(layout)
    } else {
        LangFacts::default()
    }
}

/// Asks `target`, the window with the keyboard, to switch to `layout`; nothing is switched here.
pub(crate) fn request<O: Os>(
    os: &O,
    target: Option<O::Window>,
    layout: LayoutId,
) -> Result<(), SwitchError> {
    let window = target.ok_or(SwitchError::NoTarget)?;
    if !installed(os).contains(&layout) {
        return Err(SwitchError::NotInstalled);
    }
    os.post(window, layout)
}

/// The key state with `vks` held down, each sided modifier with its generic key, and Caps Lock on when `caps`.
pub(crate) fn state(vks: &[u16], caps: bool) -> KeyState {
    let mut state = [0; KEY_STATES];
    let held = vks.iter().filter(|&&vk| vk != 0);
    for vk in held.flat_map(|&vk| [Some(vk), keys::generic(vk)]).flatten() {
        if let Some(byte) = state.get_mut(usize::from(vk)) {
            *byte |= DOWN;
        }
    }
    if caps {
        state[usize::from(VK_CAPITAL)] |= TOGGLED;
    }
    state
}

/// A `ToUnicodeEx` answer `n` over `buf`: the visible text (empty for controls) and whether the key is dead.
#[must_use]
pub fn read_typed(n: i32, buf: &[u16]) -> (String, bool) {
    let len = usize::try_from(n.unsigned_abs()).unwrap_or(usize::MAX);
    let text = String::from_utf16_lossy(&buf[..len.min(buf.len())]);
    let shown = if is_printable(&text) {
        text
    } else {
        String::new()
    };
    (shown, n < 0)
}

/// True when `text` has a visible character, not only controls or spaces.
#[must_use]
pub fn is_printable(text: &str) -> bool {
    text.chars().any(|c| !c.is_control() && !c.is_whitespace())
}

#[cfg(test)]
mod tests;
