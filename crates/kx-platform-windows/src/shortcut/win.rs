//! The Win32 side of shortcuts: the table's keys through the injector's `SendInput`, the session lock, and the history
//! chord read in the layout of the app in front.

use kx_module_api::ports::{Scan, ShortcutError, ShortcutId, ShortcutInfo, Shortcuts};
use windows::Win32::System::Shutdown::LockWorkStation;

use super::{Call, Plan, deliver_keys, history, plan, table};
use crate::inject::Win32;

/// The shortcuts service on Windows: the compiled-in table, sent to the app in front.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinShortcuts;

impl Shortcuts for WinShortcuts {
    fn known(&self) -> Vec<ShortcutInfo> {
        table().known()
    }

    fn send(&self, id: &ShortcutId, held: &[Scan]) -> Result<(), ShortcutError> {
        match plan(&Win32, table(), id, held)? {
            Plan::Keys(events) => deliver_keys(&Win32, &events),
            Plan::Call(call) => run(call),
        }
    }

    fn is_history(&self, held: &[Scan], key: Scan) -> bool {
        history(&Win32, table(), held, key)
    }
}

/// Has Windows do `call`.
#[allow(unsafe_code, reason = "Locks the session, as Win+L does.")]
fn run(call: Call) -> Result<(), ShortcutError> {
    match call {
        // SAFETY: a plain call with no arguments; Windows locks the session or says why not.
        Call::Lock => unsafe { LockWorkStation() }.map_err(|_| ShortcutError::Failed),
    }
}

#[cfg(test)]
mod tests {
    use kx_module_api::ports::{LayoutId, LayoutSource};

    use super::*;
    use crate::WinLayouts;
    use crate::inject::Os;
    use crate::shortcut::{history_in, plan_in};
    use crate::strokes::Key;

    /// `VK_A` and `VK_V`, as `keynames.toml` maps `a` and `v`.
    const VK_A: u16 = 0x41;
    const VK_V: u16 = 0x56;
    /// The keyboard's Win latch: left Win, an extended key.
    const WIN: Scan = Scan::new(0xE05B);
    /// French (France) AZERTY, whose A sits at the QWERTY Q place.
    const FRENCH: LayoutId = LayoutId::new(0x040C_040C);
    /// The primary-language bits of a layout id's language word.
    const PRIMARY: u64 = 0x3FF;
    /// English and Arabic, by primary language: their layouts keep A at the QWERTY place.
    const QWERTY_LANGS: [u64; 2] = [0x09, 0x01];
    /// A's scan code on QWERTY and on French AZERTY.
    const QWERTY_A: u32 = 0x1E;
    const AZERTY_A: u32 = 0x10;
    /// `VK_ADD`, `VK_SUBTRACT` and `VK_OEM_PERIOD`, as `keynames.toml` maps `numplus`, `numminus` and `period`.
    const VK_ADD: u16 = 0x6B;
    const VK_SUBTRACT: u16 = 0x6D;
    const VK_PERIOD: u16 = 0xBE;
    /// The number pad's + and − scan codes, the same on every layout.
    const NUM_PLUS: u32 = 0x4E;
    const NUM_MINUS: u32 = 0x4A;

    /// The key shortcut `id` sends for virtual key `vk` in `layout`, and the scan code `layout` maps `vk` to.
    fn sent(layout: LayoutId, id: &str, vk: u16) -> (Option<Key>, u32) {
        let id = ShortcutId::new(id).unwrap();
        let Ok(Plan::Keys(events)) = plan_in(&Win32, layout, table(), &id, &[]) else {
            panic!("{layout}: no keys planned for {}", id.as_str());
        };
        let key = events
            .iter()
            .map(|e| e.key)
            .find(|k| matches!(k, Key::Vk { vk: v, .. } if *v == vk));
        (key, Win32.scan_of(vk, layout))
    }

    #[test]
    fn select_alls_a_goes_as_the_key_each_installed_layout_maps_it_to() {
        for layout in WinLayouts.installed() {
            let (a, mapped) = sent(layout, "select-all", VK_A);
            assert_eq!(a, Some(Key::vk(VK_A, mapped)), "{layout}");
            if layout == FRENCH {
                assert_eq!(mapped, AZERTY_A, "{layout}");
            } else if QWERTY_LANGS.contains(&(layout.get() & PRIMARY)) {
                assert_eq!(mapped, QWERTY_A, "{layout}");
            }
        }
    }

    #[test]
    fn zoom_and_emoji_keys_go_as_each_installed_layout_maps_them() {
        let keys = [
            ("zoom-in", VK_ADD, Some(NUM_PLUS)),
            ("zoom-out", VK_SUBTRACT, Some(NUM_MINUS)),
            ("emoji", VK_PERIOD, None),
        ];
        for layout in WinLayouts.installed() {
            for (id, vk, pad) in keys {
                let (key, mapped) = sent(layout, id, vk);
                assert_eq!(key, Some(Key::vk(vk, mapped)), "{layout}: {id}");
                assert_ne!(mapped, 0, "{layout}: {id}'s key is on the layout");
                assert!(
                    pad.is_none_or(|p| p == mapped),
                    "{layout}: {id} on the number pad"
                );
            }
        }
    }

    #[test]
    fn the_history_chord_is_win_and_the_key_each_installed_layout_maps_v_to() {
        for layout in WinLayouts.installed() {
            let v = Win32.scan_of(VK_V, layout);
            let key = Scan::new(u16::try_from(v).unwrap());
            assert_ne!(v, 0, "{layout}: V is on the layout");
            assert!(history_in(&Win32, layout, table(), &[WIN], key), "{layout}");
            assert!(!history_in(&Win32, layout, table(), &[], key), "{layout}");
        }
    }

    #[test]
    fn the_service_lists_the_table_and_plans_lock_pc_as_the_call() {
        let port: &dyn Shortcuts = &WinShortcuts;
        assert_eq!(port.known(), table().known());
        let lock = ShortcutId::new("lock-pc").unwrap();
        assert_eq!(
            plan(&Win32, table(), &lock, &[]),
            Ok(Plan::Call(Call::Lock))
        );
    }
}
