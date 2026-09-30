//! The order the language key steps through: the source's installed order, each layout once.

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

/// The layout `by` places after `from` in `list`, wrapping both ways; `None` for an empty list.
/// A `from` that is not in the list counts as the first layout.
#[must_use]
pub fn wrap(list: &[LayoutId], from: Option<LayoutId>, by: i32) -> Option<LayoutId> {
    let len = i64::try_from(list.len()).ok().filter(|&n| n > 0)?;
    let at = from.and_then(|f| list.iter().position(|&l| l == f));
    let at = i64::try_from(at.unwrap_or_default()).ok()?;
    let to = usize::try_from((at + i64::from(by)).rem_euclid(len)).ok()?;
    list.get(to).copied()
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
    fn a_step_follows_the_list_and_wraps_both_ways() {
        let list = [EN, FR, AR];
        assert_eq!(wrap(&list, Some(EN), 1), Some(FR));
        assert_eq!(wrap(&list, Some(AR), 1), Some(EN));
        assert_eq!(wrap(&list, Some(EN), -1), Some(AR));
        assert_eq!(wrap(&list, Some(FR), 0), Some(FR));
        assert_eq!(wrap(&list, Some(FR), 7), Some(AR));
    }

    #[test]
    fn a_lone_layout_steps_to_itself_and_none_steps_nowhere() {
        assert_eq!(wrap(&[EN], Some(EN), 1), Some(EN));
        assert_eq!(wrap(&[], Some(EN), 1), None);
        assert_eq!(wrap(&[], None, 0), None);
    }

    #[test]
    fn an_unknown_start_counts_as_the_first_layout() {
        assert_eq!(wrap(&[EN, FR], Some(AR), 1), Some(FR));
        assert_eq!(wrap(&[EN, FR], None, 0), Some(EN));
    }

    #[test]
    fn copies_keep_their_first_place() {
        assert_eq!(dedupe(vec![EN, FR, EN, AR, FR]), [EN, FR, AR]);
        assert_eq!(dedupe(vec![]), []);
    }

    proptest! {
        #[test]
        fn any_step_lands_where_the_index_maths_says(
            n in 1i64..8, at in 0i64..8, by in any::<i32>(),
        ) {
            let list: Vec<LayoutId> = (0..n).map(|i| LayoutId::new(i.unsigned_abs())).collect();
            let index = |i: i64| usize::try_from(i.rem_euclid(n)).unwrap();
            let want = list[index(at + i64::from(by))];
            prop_assert_eq!(wrap(&list, Some(list[index(at)]), by), Some(want));
        }

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
