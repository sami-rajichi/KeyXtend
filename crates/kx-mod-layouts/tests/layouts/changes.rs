//! Layouts added or removed while running keep the list and the shown layout sane.

use crate::common::{GOLDEN, Heard, file, ids, service, source};
use kx_mod_layouts::LayoutsService;
use kx_module_api::ports::{LayoutId, LayoutSource, LayoutTable, Layouts, SwitchError};
use kx_platform_fake::{FakeLayoutSource, LayoutFile};
use proptest::collection::vec;
use proptest::prelude::*;

/// The golden tables, then copies of English and Arabic under ids no golden table has.
fn pool() -> Vec<LayoutFile> {
    let mut all: Vec<LayoutFile> = GOLDEN.iter().map(|n| file(n)).collect();
    for (from, id) in [(0, 0x0809_0809), (2, 0x0C01_0C01)] {
        let copy = LayoutFile {
            id: LayoutId::new(id),
            ..all[from].clone()
        };
        all.push(copy);
    }
    all
}

#[test]
fn an_added_layout_joins_the_end_and_the_pick_stays() {
    let [en, ..] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    let added = pool().remove(3);
    fake.add(added.clone());
    assert!(layouts.refresh(), "the list changed");
    assert_eq!(layouts.list().last().map(|l| l.id), Some(added.id));
    assert_eq!(layouts.current().label.id, en);
    assert!(heard.ids().is_empty(), "the shown layout did not change");
    assert!(!layouts.refresh());
}

#[test]
fn removing_the_shown_layout_moves_to_the_first_installed() {
    let [en, fr, ar] = ids();
    let fake = source();
    let (layouts, heard) = service(&fake);
    assert_eq!(layouts.pick(fr), Ok(()));
    assert!(layouts.refresh());
    fake.remove(fr);
    assert!(
        layouts.refresh(),
        "the app still says French, which is gone"
    );
    assert_eq!(layouts.current().label.id, en);
    let listed: Vec<LayoutId> = layouts.list().iter().map(|l| l.id).collect();
    assert_eq!(listed, [en, ar]);
    assert_eq!(heard.ids(), [fr, en]);
}

#[test]
fn with_nothing_installed_the_table_is_empty_and_requests_fail() {
    let fake = source();
    let (layouts, _) = service(&fake);
    for id in ids() {
        fake.remove(id);
    }
    assert!(layouts.refresh());
    assert!(layouts.list().is_empty());
    assert_eq!(*layouts.current(), LayoutTable::default());
    assert_eq!(layouts.step(1), Err(SwitchError::NotInstalled));
    fake.add(file(GOLDEN[1]));
    assert!(layouts.refresh());
    assert_eq!(layouts.current().label.id, file(GOLDEN[1]).id);
}

/// One thing the user or the app in front does.
#[derive(Clone, Debug)]
enum Op {
    Add(usize),
    Remove(usize),
    Front(Option<usize>),
    Follow(bool),
    Step(i32),
    Pick(usize),
    Refresh,
}

fn op(n: usize) -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..n).prop_map(Op::Add),
        (0..n).prop_map(Op::Remove),
        proptest::option::of(0..n).prop_map(Op::Front),
        any::<bool>().prop_map(Op::Follow),
        (-4i32..5).prop_map(Op::Step),
        (0..n).prop_map(Op::Pick),
        Just(Op::Refresh),
    ]
}

/// Runs `op`; a step or pick may fail, but never panics, and an unknown pick is never asked for.
fn apply(op: &Op, fake: &FakeLayoutSource, layouts: &LayoutsService, pool: &[LayoutFile]) {
    let listed = |id: LayoutId| layouts.list().iter().any(|l| l.id == id);
    match *op {
        Op::Add(i) => fake.add(pool[i].clone()),
        Op::Remove(i) => fake.remove(pool[i].id),
        Op::Front(i) => fake.set_foreground(i.map(|i| pool[i].id)),
        Op::Follow(on) => fake.set_follow(on),
        Op::Step(by) => {
            let got = layouts.step(by);
            if layouts.list().is_empty() {
                assert_eq!(got, Err(SwitchError::NotInstalled));
            }
        }
        Op::Pick(i) => {
            let asked = fake.requests().len();
            let got = layouts.pick(pool[i].id);
            if !listed(pool[i].id) {
                assert_eq!(got, Err(SwitchError::NotInstalled));
                assert_eq!(fake.requests().len(), asked);
            }
        }
        Op::Refresh => {
            layouts.refresh();
        }
    }
}

/// After a refresh: the list is the installed one, the shown layout is sane and its table right.
fn check(fake: &FakeLayoutSource, layouts: &LayoutsService, pool: &[LayoutFile]) {
    let mut installed: Vec<LayoutId> = Vec::new();
    for id in fake.installed() {
        if !installed.contains(&id) {
            installed.push(id);
        }
    }
    let list = layouts.list();
    let listed: Vec<LayoutId> = list.iter().map(|l| l.id).collect();
    assert_eq!(listed, installed);
    let current = layouts.current();
    let Some(shown) = installed.iter().find(|&&id| id == current.label.id) else {
        assert!(installed.is_empty(), "the shown layout is installed");
        assert_eq!(*current, LayoutTable::default());
        return;
    };
    if let Some(front) = fake.foreground().filter(|f| installed.contains(f)) {
        assert_eq!(*shown, front, "the app in front wins");
    }
    assert!(list.contains(&current.label), "one label per layout");
    let want = pool.iter().find(|f| f.id == *shown).map(|f| &f.keys);
    assert_eq!(Some(&current.keys), want);
}

/// A `LayoutChanged` came with every change of the shown layout, and only then.
fn heard_changes(heard: &Heard, before: (LayoutId, usize), now: LayoutId, installed: bool) {
    let (was, count) = before;
    let got = heard.ids();
    if was != now && installed {
        assert_eq!((got.len(), got.last()), (count + 1, Some(&now)));
    } else {
        assert_eq!(got.len(), count);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn any_changes_keep_the_list_and_the_pick_sane(ops in vec(op(5), 1..24)) {
        let pool = pool();
        let fake = source();
        let (layouts, heard) = service(&fake);
        for op in &ops {
            let before = (layouts.current().label.id, heard.ids().len());
            apply(op, &fake, &layouts, &pool);
            if matches!(op, Op::Refresh) {
                check(&fake, &layouts, &pool);
                let now = layouts.current().label.id;
                heard_changes(&heard, before, now, !layouts.list().is_empty());
                prop_assert!(!layouts.refresh(), "a second refresh sees nothing new");
            } else {
                let now = layouts.current().label.id;
                prop_assert_eq!(now, before.0, "only a refresh changes the shown layout");
            }
        }
    }
}
