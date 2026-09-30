//! The tray icon: the icon in the taskbar's notification area, its clicks, and "the user clicked elsewhere".
//! Reading the shell's answers, the tooltip and the picture's bytes is pure and tested everywhere; the calls are in `win`.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "The tables and readers serve only the Windows calls in win."
    )
)]

use kx_module_api::ports::Point;
use kx_platform::ui::TrayEvent;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinTray;

/// The shell's windows that come to the front when the user clicks the taskbar or its hidden icons, so their coming
/// is no click elsewhere: the main and a second taskbar, and the hidden icons' flyout (Windows 10, then 11).
const TASKBAR_CLASSES: [&str; 4] = [
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
];

/// The shell's notifications that open the menu: `NIN_SELECT` (a click), `NIN_KEYSELECT` (Enter or Space on the
/// icon) and `WM_CONTEXTMENU` (the menu key or a right click, which never blocks the left one).
const OPENS: [u32; 3] = [0x0400, 0x0401, 0x007B];

/// The tooltip's room in UTF-16 units, its ending null included (`NOTIFYICONDATAW.szTip`).
pub const TIP_UNITS: usize = 128;

/// Bytes per pixel of the icon's picture: blue, green, red and alpha.
const BGRA: usize = 4;

/// True when a window of class `class` coming to the front means the user clicked elsewhere.
#[must_use]
pub fn elsewhere(class: &str) -> bool {
    !TASKBAR_CLASSES.contains(&class)
}

/// The event behind the shell's callback: the notification is `info`'s low word, and `anchor` packs where it
/// happened (x in the low word, y in the high word, both signed); other notifications are ignored.
#[must_use]
pub fn event(info: isize, anchor: usize) -> Option<TrayEvent> {
    let code = u32::try_from(info & 0xFFFF).unwrap_or_default();
    OPENS
        .contains(&code)
        .then(|| TrayEvent::Click(point(anchor)))
}

/// The signed x and y the shell packs into one value.
fn point(anchor: usize) -> Point {
    let word = |shift: u32| {
        let bits = u16::try_from((anchor >> shift) & 0xFFFF).unwrap_or_default();
        i32::from(bits.cast_signed())
    };
    Point {
        x: word(0),
        y: word(16),
    }
}

/// `text` as the tooltip's UTF-16 with its null, cut to fit without splitting a surrogate pair.
#[must_use]
pub fn tip(text: &str) -> [u16; TIP_UNITS] {
    let mut out = [0u16; TIP_UNITS];
    let mut used = 0;
    for c in text.chars() {
        let mut buf = [0u16; 2];
        let units = c.encode_utf16(&mut buf);
        if used + units.len() >= TIP_UNITS {
            break;
        }
        out[used..used + units.len()].copy_from_slice(units);
        used += units.len();
    }
    out
}

/// Straight RGBA pixels as the straight BGRA a 32-bit icon holds.
#[must_use]
pub fn bgra(rgba: &[u8]) -> Vec<u8> {
    let mut out = rgba.to_vec();
    for px in out.as_chunks_mut::<BGRA>().0 {
        px.swap(0, 2);
    }
    out
}

/// The icon's AND mask, all clear since the alpha shapes it: `height` rows of one bit per pixel, each row padded to
/// 16 bits as `CreateBitmap` reads them. `None` when it would not fit in memory.
#[must_use]
pub fn mask(width: u32, height: u32) -> Option<Vec<u8>> {
    let row = width.div_ceil(16) * 2;
    let bytes = u64::from(row) * u64::from(height);
    Some(vec![0; usize::try_from(bytes).ok()?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_taskbar_and_its_hidden_icons_are_not_elsewhere() {
        for class in TASKBAR_CLASSES {
            assert!(!elsewhere(class), "{class}");
        }
        for class in ["Notepad", "Progman", "WorkerW", "Chrome_WidgetWin_1", ""] {
            assert!(elsewhere(class), "{class}");
        }
    }

    #[test]
    fn a_click_a_key_or_the_menu_key_opens_the_menu_at_the_anchor() {
        let anchor = (0x0000_0438 << 16) | 0x0000_0780;
        let at = Point { x: 1920, y: 1080 };
        for code in OPENS {
            // The icon's id rides in the high word.
            let info = (1 << 16) | isize::try_from(code).unwrap();
            assert_eq!(event(info, anchor), Some(TrayEvent::Click(at)), "{code:#x}");
        }
    }

    #[test]
    fn other_notifications_are_ignored() {
        // Mouse moves and buttons, balloon and hover notices, and nonsense.
        for info in [
            0x0200, 0x0201, 0x0202, 0x0203, 0x0402, 0x0406, 0x0407, 0, -1,
        ] {
            assert_eq!(event(info, 0), None, "{info:#x}");
        }
    }

    #[test]
    fn an_anchor_left_of_or_above_the_main_screen_is_negative() {
        let anchor = (0xFFF6 << 16) | 0xFB00;
        let want = Some(TrayEvent::Click(Point { x: -1280, y: -10 }));
        assert_eq!(event(0x0400, anchor), want);
    }

    #[test]
    fn a_tooltip_ends_with_a_null_and_is_cut_without_splitting_a_pair() {
        let short = tip("KeyXtend");
        assert_eq!(
            &short[..9],
            "KeyXtend\0".encode_utf16().collect::<Vec<_>>().as_slice()
        );
        let long = "a".repeat(200);
        let cut = tip(&long);
        assert_eq!(cut[TIP_UNITS - 2], u16::from(b'a'));
        assert_eq!(cut[TIP_UNITS - 1], 0, "room for the null");
        let fits = tip(&format!("{}😀", "a".repeat(TIP_UNITS - 3)));
        assert!(
            char::decode_utf16(fits[TIP_UNITS - 3..TIP_UNITS - 1].iter().copied())
                .all(|c| c == Ok('😀'))
        );
        let over = tip(&format!("{}😀", "a".repeat(TIP_UNITS - 2)));
        assert_eq!(over[TIP_UNITS - 2], 0, "the pair did not fit whole");
        let arabic = tip("لوحة مفاتيح KeyXtend");
        assert_eq!(
            String::from_utf16_lossy(&arabic[..20]),
            "لوحة مفاتيح KeyXtend"
        );
    }

    #[test]
    fn pixels_turn_from_rgba_to_bgra_keeping_alpha() {
        assert_eq!(bgra(&[1, 2, 3, 4, 5, 6, 7, 8]), [3, 2, 1, 4, 7, 6, 5, 8]);
        assert!(bgra(&[]).is_empty());
    }

    #[test]
    fn the_mask_pads_each_row_to_sixteen_bits() {
        let len = |w, h| mask(w, h).map(|m| m.len());
        assert_eq!(len(16, 16), Some(32));
        assert_eq!(len(20, 20), Some(80), "20 bits round up to 32");
        assert_eq!(len(1, 3), Some(6));
        assert!(
            mask(24, 24).is_some_and(|m| m.iter().all(|b| *b == 0)),
            "the alpha shapes the icon"
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_notification_table_matches_windows() {
        use windows::Win32::UI::Shell::{NIN_SELECT, NINF_KEY};
        use windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU;
        assert_eq!(OPENS, [NIN_SELECT, NIN_SELECT | NINF_KEY, WM_CONTEXTMENU]);
    }
}
