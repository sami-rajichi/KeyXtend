//! The order the language key steps through: the source's installed order, each layout once; `step_to` in
//! `kx_module_api::ports` counts the steps.

use kx_module_api::ports::LayoutId;
use std::collections::HashSet;

/// `installed` with each layout once, where it first appears.
#[must_use]
pub fn dedupe(installed: Vec<LayoutId>) -> Vec<LayoutId> {
    let mut seen = HashSet::new();
    installed
        .into_iter()
        .filter(|id| seen.insert(*id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    const EN: LayoutId = LayoutId::new(0x0409_0409);
    const FR: LayoutId = LayoutId::new(0x040C_040C);
    const AR: LayoutId = LayoutId::new(0x0401_0401);

    #[test]
    fn copies_keep_their_first_place() {
        assert_eq!(dedupe(vec![EN, FR, EN, AR, FR]), [EN, FR, AR]);
        assert_eq!(dedupe(vec![]), []);
    }

    proptest! {
        #[test]
        fn dedupe_keeps_each_layout_once_in_first_seen_order(raw in vec(0u64..6, 0..20)) {
            let ids: Vec<LayoutId> = raw.iter().copied().map(LayoutId::new).collect();
            let once = dedupe(ids.clone());
            for id in &ids {
                prop_assert_eq!(once.iter().filter(|&x| x == id).count(), 1);
            }
            let firsts: Vec<LayoutId> = ids
                .iter()
                .enumerate()
                .filter(|(i, id)| ids[..*i].iter().all(|x| x != *id))
                .map(|(_, id)| *id)
                .collect();
            prop_assert_eq!(once, firsts);
        }
    }
}
