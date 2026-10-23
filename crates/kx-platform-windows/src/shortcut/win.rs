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

    #[test]
    fn select_alls_a_goes_as_the_key_each_installed_layout_maps_it_to() {
        let select_all = ShortcutId::new("select-all").unwrap();
        for layout in WinLayouts.installed() {
            let Ok(Plan::Keys(events)) = plan_in(&Win32, layout, table(), &select_all, &[]) else {
                panic!("{layout}: no keys planned");
            };
            let mapped = Win32.scan_of(VK_A, layout);
            let a = events
                .iter()
                .find(|e| matches!(e.key, Key::Vk { vk: VK_A, .. }));
            assert_eq!(a.map(|e| e.key), Some(Key::vk(VK_A, mapped)), "{layout}");
            if layout == FRENCH {
                assert_eq!(mapped, AZERTY_A, "{layout}");
            } else if QWERTY_LANGS.contains(&(layout.get() & PRIMARY)) {
                assert_eq!(mapped, QWERTY_A, "{layout}");
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
