//! A recording fake of the injector's OS calls: a French AZERTY layout with Caps Lock and dead keys.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use super::super::Os;
use crate::keys::{self, VK_SHIFT, VK_SPACE};
use crate::strokes::KeyEvent;
use kx_module_api::ports::LayoutId;

/// The layout the fake reports in front; lookups in any other layout fail the test.
pub const FRONT: LayoutId = LayoutId::new(0x040C_040C);
/// `VK_A`, `VK_0`, `VK_1`, `VK_7`, `VK_RETURN` and `VK_OEM_6` (the dead `^` key on AZERTY).
pub const VK_A: u16 = 0x41;
pub const VK_0: u16 = 0x30;
pub const VK_1: u16 = 0x31;
pub const VK_7: u16 = 0x37;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_DEAD: u16 = 0xDD;

/// `VkKeyScanEx` answers: plain, Shift, `AltGr`, controls, Ctrl-only ones and dead keys.
const KEYMAP: [(char, i16); 10] = [
    ('a', 0x0041),
    ('A', 0x0141),
    ('1', 0x0131),
    ('&', 0x0031),
    ('@', 0x0630),
    ('\r', 0x000D),
    ('\n', 0x020D),
    ('\u{1}', 0x0241),
    ('^', 0x00DD),
    ('`', 0x0637),
];

/// What each key types: (vk, Shift, `AltGr`, Caps Lock flips Shift, unit, dead).
/// `AltGr`+7 is a dead key whose own character is not a plain backtick.
const TYPES: [(u16, bool, bool, bool, char, bool); 10] = [
    (VK_A, false, false, true, 'a', false),
    (VK_A, true, false, true, 'A', false),
    (VK_1, false, false, true, '&', false),
    (VK_1, true, false, true, '1', false),
    (VK_0, false, true, false, '@', false),
    (VK_RETURN, false, false, false, '\r', false),
    (VK_SPACE, false, false, false, ' ', false),
    (VK_DEAD, false, false, false, '^', true),
    (VK_DEAD, true, false, false, '¨', true),
    (VK_7, false, true, false, '\u{2CB}', true),
];

/// `MapVirtualKeyExW` scan codes: Shift, Ctrl, Alt, A (the Q place), 0, 1, 7, Enter, the dead key and Space.
const SCANS: [(u16, u32); 10] = [
    (VK_SHIFT, 0x2A),
    (keys::ALTGR_VKS[0], 0x1D),
    (keys::ALTGR_VKS[1], 0x38),
    (VK_A, 0x10),
    (VK_0, 0x0B),
    (VK_1, 0x02),
    (VK_7, 0x08),
    (VK_RETURN, 0x1C),
    (VK_DEAD, 0x1A),
    (VK_SPACE, 0x39),
];

/// Records each batch; each send takes at most the next of `takes`, or all once they run out.
#[derive(Default)]
pub struct Fake {
    pub takes: RefCell<VecDeque<usize>>,
    pub sends: RefCell<Vec<Vec<KeyEvent>>>,
    pub layouts: Cell<usize>,
    pub caps: bool,
    pub away: bool,
}

impl Fake {
    /// A fake whose sends take at most `takes`, one after the other.
    pub fn taking(takes: &[usize]) -> Self {
        let fake = Self::default();
        fake.takes.borrow_mut().extend(takes);
        fake
    }

    /// A fake with Caps Lock on.
    pub fn caps() -> Self {
        Self {
            caps: true,
            ..Self::default()
        }
    }
}

impl Os for Fake {
    fn send(&self, events: &[KeyEvent]) -> usize {
        self.sends.borrow_mut().push(events.to_vec());
        let take = self.takes.borrow_mut().pop_front();
        take.map_or(events.len(), |t| t.min(events.len()))
    }

    fn layout(&self) -> Option<LayoutId> {
        self.layouts.set(self.layouts.get() + 1);
        (!self.away).then_some(FRONT)
    }

    fn key_scan(&self, unit: u16, layout: LayoutId) -> i16 {
        assert_eq!(layout, FRONT);
        let hit = KEYMAP
            .iter()
            .find(|(c, _)| u32::from(*c) == u32::from(unit));
        hit.map_or(-1, |&(_, raw)| raw)
    }

    fn scan_of(&self, vk: u16, layout: LayoutId) -> u32 {
        assert_eq!(layout, FRONT);
        SCANS.iter().find(|(v, _)| *v == vk).map_or(0, |&(_, s)| s)
    }

    fn caps_on(&self) -> bool {
        self.caps
    }

    fn to_unicode(
        &self,
        vk: u16,
        held: &[u16],
        caps: bool,
        layout: LayoutId,
        buf: &mut [u16],
    ) -> i32 {
        assert_eq!(layout, FRONT);
        assert_eq!(caps, self.caps, "the Caps state read once is used");
        let shift = held.contains(&VK_SHIFT);
        let altgr = keys::ALTGR_VKS.iter().all(|k| held.contains(k));
        let hit = TYPES.iter().find(|&&(v, s, a, flips, ..)| {
            v == vk && a == altgr && s == (shift != (caps && flips))
        });
        let Some(&(.., c, dead)) = hit else {
            return 0;
        };
        buf[0] = u16::try_from(u32::from(c)).unwrap();
        if dead { -1 } else { 1 }
    }
}
