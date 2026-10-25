//! The layouts port's data, key and event, and the one stepping rule `step` and the language key share.

use super::*;
use proptest::prelude::*;

const EN: LayoutId = LayoutId::new(0x0409_0409);
const FR: LayoutId = LayoutId::new(0x040C_040C);
const AR: LayoutId = LayoutId::new(0x0401_0401);

fn table() -> LayoutTable {
    let q = KeyChars {
        base: "ض".into(),
        shift: "\u{64E}".into(),
        ..KeyChars::default()
    };
    LayoutTable {
        label: LangLabel {
            id: AR,
            short: "ع".into(),
            name: vec!["العربية".into(), "101".into()],
            iso: "ar".into(),
            rtl: true,
            arabic: true,
        },
        keys: [(Scan::new(0x10), q)].into_iter().collect(),
    }
}

#[test]
fn a_table_round_trips_through_json_and_toml() {
    let json = serde_json::to_string(&table()).unwrap();
    assert_eq!(serde_json::from_str::<LayoutTable>(&json).unwrap(), table());
    let text = toml::to_string(&table()).unwrap();
    assert_eq!(toml::from_str::<LayoutTable>(&text).unwrap(), table());
}

#[test]
fn the_layouts_service_needs_inject_input() {
    assert_eq!(LayoutsKey::ID.as_str(), "layouts");
    assert_eq!(LayoutsKey::CAPABILITY, Some(Capability::InjectInput));
}

#[test]
fn layout_changed_is_open_to_every_module() {
    assert_eq!(LayoutChanged::NAME, "layout-changed");
    assert_eq!(LayoutChanged::CAPABILITY, None);
}

#[test]
fn a_change_carries_the_labels_language_and_direction() {
    let label = table().label;
    let change = LayoutChanged::from(&label);
    let want = LayoutChanged {
        id: AR,
        iso: "ar".into(),
        rtl: true,
    };
    assert_eq!(change, want);
}

#[test]
fn the_ask_is_open_to_every_module() {
    assert_eq!(LayoutAsk::NAME, "layout-ask");
    assert_eq!(LayoutAsk::CAPABILITY, None);
}

#[test]
fn a_step_follows_the_list_and_wraps_both_ways() {
    let list = [EN, FR, AR];
    assert_eq!(step_to(&list, Some(EN), 1), Some(FR));
    assert_eq!(step_to(&list, Some(AR), 1), Some(EN));
    assert_eq!(step_to(&list, Some(EN), -1), Some(AR));
    assert_eq!(step_to(&list, Some(FR), 0), Some(FR));
    assert_eq!(step_to(&list, Some(FR), 7), Some(AR));
}

#[test]
fn a_lone_layout_steps_to_itself_and_none_steps_nowhere() {
    assert_eq!(step_to(&[EN], Some(EN), 1), Some(EN));
    assert_eq!(step_to(&[], Some(EN), 1), None);
    assert_eq!(step_to(&[], None, 0), None);
}

#[test]
fn an_unknown_start_counts_as_the_first_layout() {
    assert_eq!(step_to(&[EN, FR], Some(AR), 1), Some(FR));
    assert_eq!(step_to(&[EN, FR], None, 0), Some(EN));
}

proptest! {
    #[test]
    fn any_step_lands_where_the_index_maths_says(n in 1i64..8, at in 0i64..8, by in any::<i32>()) {
        let list: Vec<LayoutId> = (0..n).map(|i| LayoutId::new(i.unsigned_abs())).collect();
        let index = |i: i64| usize::try_from(i.rem_euclid(n)).unwrap();
        let want = list[index(at + i64::from(by))];
        prop_assert_eq!(step_to(&list, Some(list[index(at)]), by), Some(want));
    }
}
