//! The Win32 side of shortcuts: the table's keys through the injector's `SendInput`, and the session lock.

use kx_module_api::ports::{Scan, ShortcutError, ShortcutId, ShortcutInfo, Shortcuts};
use windows::Win32::System::Shutdown::LockWorkStation;

use super::{Call, Plan, deliver_keys, plan, table};
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
    use crate::shortcut::plan_in;
    use crate::strokes::Key;

    /// `VK_A`, as `keynames.toml` maps `a`.
    const VK_A: u16 = 0x41;
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
