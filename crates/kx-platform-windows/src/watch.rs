//! Watches the system look: a hidden window on its own thread hears Windows' change messages and reports each real change once.
//! Telling a real change from a repeat is pure and tested everywhere; the window and its thread are in `win`.

use kx_platform::ui::SysLook;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub(crate) use win::start;

/// `now` when it differs from `last`, which then becomes `now`; a first look is always new.
#[must_use]
pub fn changed(last: &mut Option<SysLook>, now: SysLook) -> Option<SysLook> {
    if *last == Some(now) {
        return None;
    }
    *last = Some(now);
    Some(now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark() -> SysLook {
        SysLook {
            dark: true,
            ..SysLook::default()
        }
    }

    #[test]
    fn a_first_look_is_new_and_each_real_change_is_reported_once() {
        let mut last = None;
        assert_eq!(
            changed(&mut last, SysLook::default()),
            Some(SysLook::default())
        );
        assert_eq!(changed(&mut last, SysLook::default()), None, "no change");
        assert_eq!(changed(&mut last, dark()), Some(dark()));
        assert_eq!(changed(&mut last, dark()), None, "the same change again");
        assert_eq!(last, Some(dark()));
    }

    #[test]
    fn a_new_pointer_size_alone_is_a_change() {
        let mut last = Some(dark());
        let bigger = SysLook {
            pointer_px: dark().pointer_px * 2,
            ..dark()
        };
        assert_eq!(changed(&mut last, bigger), Some(bigger));
    }
}
