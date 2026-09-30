//! Which layout the keyboard shows: the app in front's while it is installed, else the one shown,
//! else the first installed (moved from the spike's `layout::Follow`).

use kx_module_api::ports::LayoutId;

/// Remembers the layout the keyboard shows, so it changes only when the app in front switches
/// or the shown layout is uninstalled.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Follow {
    shown: Option<LayoutId>,
}

impl Follow {
    /// Starts on the layout of the app in front, `front`, or, when that is not installed, the first of `installed`.
    #[must_use]
    pub fn new(front: Option<LayoutId>, installed: &[LayoutId]) -> Self {
        let mut follow = Self::default();
        follow.see(front, installed);
        follow
    }

    /// The layout shown; `None` only while nothing is installed.
    #[must_use]
    pub fn shown(&self) -> Option<LayoutId> {
        self.shown
    }

    /// Takes the layout of the app in front (`None` with no app in front) and the installed list;
    /// true when the shown layout changed.
    pub fn see(&mut self, front: Option<LayoutId>, installed: &[LayoutId]) -> bool {
        let has = |id: &LayoutId| installed.contains(id);
        let next = front
            .filter(has)
            .or(self.shown.filter(has))
            .or_else(|| installed.first().copied());
        let changed = next != self.shown;
        self.shown = next;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::option::of;
    use proptest::prelude::*;

    const EN: LayoutId = LayoutId::new(0x0409_0409);
    const FR: LayoutId = LayoutId::new(0x040C_040C);
    const AR: LayoutId = LayoutId::new(0x0401_0401);

    #[test]
    fn it_follows_a_new_layout_in_front_only() {
        let all = [EN, FR];
        let mut f = Follow::new(Some(EN), &all);
        assert_eq!(f.shown(), Some(EN));
        assert!(!f.see(Some(EN), &all), "already showing it");
        assert!(!f.see(None, &all), "no app in front keeps the layout");
        assert!(f.see(Some(FR), &all));
        assert_eq!(f.shown(), Some(FR));
        assert!(!f.see(Some(FR), &all));
    }

    #[test]
    fn a_layout_in_front_that_is_not_installed_is_not_shown() {
        let mut f = Follow::new(Some(AR), &[EN, FR]);
        assert_eq!(f.shown(), Some(EN), "the first installed");
        assert!(!f.see(Some(AR), &[EN, FR]));
    }

    #[test]
    fn an_uninstalled_shown_layout_gives_way() {
        let mut f = Follow::new(Some(FR), &[EN, FR]);
        assert!(f.see(None, &[AR, EN]), "to the first installed");
        assert_eq!(f.shown(), Some(AR));
        assert!(f.see(None, &[]), "nothing is installed");
        assert_eq!(f.shown(), None);
        assert!(f.see(Some(EN), &[EN]));
        assert_eq!(f.shown(), Some(EN));
    }

    fn id() -> impl Strategy<Value = LayoutId> {
        (0u64..5).prop_map(LayoutId::new)
    }

    proptest! {
        #[test]
        fn the_shown_layout_is_always_sane(
            start in of(id()),
            first in vec(id(), 0..5),
            steps in vec((of(id()), vec(id(), 0..5)), 1..12),
        ) {
            let mut f = Follow::new(start, &first);
            for (front, installed) in steps {
                let before = f.shown();
                let changed = f.see(front, &installed);
                let shown = f.shown();
                prop_assert_eq!(changed, shown != before);
                match shown {
                    None => prop_assert!(installed.is_empty()),
                    Some(s) => prop_assert!(installed.contains(&s)),
                }
                if let Some(front) = front.filter(|l| installed.contains(l)) {
                    prop_assert_eq!(shown, Some(front), "the app in front wins");
                } else if before.is_some_and(|b| installed.contains(&b)) {
                    prop_assert_eq!(shown, before, "nothing new to follow");
                }
            }
        }
    }
}
