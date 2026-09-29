//! The `--layout` switch: an installed layout the window types into, so a gate knows the layout.
//!
//! Windows 8+ switches the whole session's input language, so the previous layout goes back at close.

use kx_module_api::ports::{CodeError, LayoutId};
use thiserror::Error;

/// Why the window could not take the layout it was given.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    /// The switch has no layout id after it.
    #[error("{0} needs a layout id after it")]
    Missing(String),
    /// The text after the switch is not a layout id.
    #[error(transparent)]
    Bad(#[from] CodeError),
    /// The layout is not installed on this PC; the window never installs one.
    #[error("layout {0} is not installed")]
    NotInstalled(LayoutId),
    /// Windows did not switch to the layout; the previous one is already back.
    #[error("Windows did not switch to layout {0}")]
    Refused(LayoutId),
}

/// The layout named after switch `flag` in `args`; `None` when the switch is absent.
///
/// # Errors
/// `Missing` when nothing follows the switch, `Bad` when the text is not a `0x` hex layout id.
pub fn parse(args: &[String], flag: &str) -> Result<Option<LayoutId>, LayoutError> {
    let Some(at) = args.iter().position(|a| a == flag) else {
        return Ok(None);
    };
    let text = args
        .get(at + 1)
        .ok_or_else(|| LayoutError::Missing(flag.to_owned()))?;
    Ok(Some(text.parse()?))
}

/// Switches the input layout and reads it back: Win32 in the window, a fake in tests.
pub trait Switcher {
    /// Makes `id` the input layout; the previous one, or `None` when Windows refused.
    fn activate(&self, id: LayoutId) -> Option<LayoutId>;
    /// The layout this thread types in now.
    fn current(&self) -> LayoutId;
}

/// The layout to put back at close, since Windows 8+ switches the whole session, not only the window.
#[must_use = "the previous layout must be put back at close"]
#[derive(Debug, PartialEq, Eq)]
pub struct Restore(LayoutId);

/// Switches to `id`; when the switch does not take, the previous layout is put back at once.
///
/// # Errors
/// `Refused` when Windows refused the switch or this thread's layout did not change.
pub fn switch(s: &impl Switcher, id: LayoutId) -> Result<Restore, LayoutError> {
    let back = Restore(s.activate(id).ok_or(LayoutError::Refused(id))?);
    if s.current() == id {
        return Ok(back);
    }
    restore(s, back);
    Err(LayoutError::Refused(id))
}

/// Puts back the layout from before `switch`; call it while the window still has the keyboard.
pub fn restore(s: &impl Switcher, Restore(previous): Restore) {
    let _ = s.activate(previous);
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use proptest::prelude::*;

    use super::*;

    const FLAG: &str = "--layout";
    const EN: LayoutId = LayoutId::new(0x0409_0409);
    const FR: LayoutId = LayoutId::new(0x040C_040C);

    /// A session whose layout the thread follows unless `stuck`; `deaf` refuses every switch.
    struct Fake {
        session: Cell<LayoutId>,
        thread: Cell<LayoutId>,
        stuck: bool,
        deaf: bool,
        calls: RefCell<Vec<LayoutId>>,
    }

    impl Fake {
        fn new(stuck: bool, deaf: bool) -> Self {
            let (session, thread) = (Cell::new(EN), Cell::new(EN));
            let calls = RefCell::default();
            Self {
                session,
                thread,
                stuck,
                deaf,
                calls,
            }
        }
    }

    impl Switcher for Fake {
        fn activate(&self, id: LayoutId) -> Option<LayoutId> {
            self.calls.borrow_mut().push(id);
            if self.deaf {
                return None;
            }
            if !self.stuck {
                self.thread.set(id);
            }
            Some(self.session.replace(id))
        }

        fn current(&self) -> LayoutId {
            self.thread.get()
        }
    }

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn without_the_switch_there_is_no_layout() {
        assert_eq!(parse(&args(""), FLAG), Ok(None));
        assert_eq!(parse(&args("--password"), FLAG), Ok(None));
    }

    #[test]
    fn the_id_after_the_switch_is_read_as_hex() {
        let fr = LayoutId::new(0x040C_040C);
        assert_eq!(parse(&args("--layout 0x040C040C"), FLAG), Ok(Some(fr)));
        let ar = LayoutId::new(0xFFFF_FFFF_F029_1C01);
        let text = "--password --layout 0xFFFFFFFFF0291C01";
        assert_eq!(parse(&args(text), FLAG), Ok(Some(ar)));
    }

    #[test]
    fn a_switch_without_an_id_is_missing_one() {
        let err = parse(&args("--layout"), FLAG).unwrap_err();
        assert_eq!(err, LayoutError::Missing(FLAG.into()));
        assert!(err.to_string().contains(FLAG), "{err}");
    }

    #[test]
    fn text_that_is_not_an_id_is_refused() {
        for bad in ["--layout 040C040C", "--layout 0xZZ", "--layout --password"] {
            let err = parse(&args(bad), FLAG).unwrap_err();
            assert!(matches!(err, LayoutError::Bad(_)), "{bad}: {err}");
        }
    }

    proptest! {
        #[test]
        fn any_arguments_parse_without_panic(
            words in prop::collection::vec(".{0,12}", 0..6),
            flag in ".{0,12}",
        ) {
            let _ = parse(&words, &flag);
        }

        #[test]
        fn every_id_after_the_switch_reads_back(
            raw: u64,
            before in prop::collection::vec("[a-z]{0,8}", 0..3),
        ) {
            let id = LayoutId::new(raw);
            let after = [FLAG.into(), id.to_string()];
            let words: Vec<String> = before.into_iter().chain(after).collect();
            prop_assert_eq!(parse(&words, FLAG), Ok(Some(id)));
        }
    }

    #[test]
    fn the_previous_layout_comes_back_at_close() {
        let fake = Fake::new(false, false);
        let back = switch(&fake, FR).expect("the switch takes");
        assert_eq!(fake.session.get(), FR);
        restore(&fake, back);
        assert_eq!(fake.session.get(), EN);
        assert_eq!(*fake.calls.borrow(), [FR, EN]);
    }

    #[test]
    fn a_switch_that_does_not_take_puts_the_previous_layout_back() {
        let fake = Fake::new(true, false);
        assert_eq!(switch(&fake, FR), Err(LayoutError::Refused(FR)));
        assert_eq!(fake.session.get(), EN);
        assert_eq!(*fake.calls.borrow(), [FR, EN]);
    }

    #[test]
    fn a_refused_switch_changes_nothing_to_put_back() {
        let fake = Fake::new(false, true);
        assert_eq!(switch(&fake, FR), Err(LayoutError::Refused(FR)));
        assert_eq!(fake.session.get(), EN);
        assert_eq!(*fake.calls.borrow(), [FR]);
    }

    #[test]
    fn each_error_names_the_layout() {
        let id = LayoutId::new(0x0419_0419);
        for err in [LayoutError::NotInstalled(id), LayoutError::Refused(id)] {
            assert!(err.to_string().contains("0x0000000004190419"), "{err}");
        }
    }
}
