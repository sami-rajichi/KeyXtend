use super::*;
use kx_module_api::ports::Scan;

const Q: Scan = Scan::new(0x10);

fn fake() -> FakeLayouts {
    FakeLayouts::golden(&["en-us", "fr-fr", "ar-tn"]).unwrap()
}

fn ids(fake: &FakeLayouts) -> Vec<LayoutId> {
    fake.list().into_iter().map(|l| l.id).collect()
}

#[test]
fn it_shows_the_first_table_and_lists_all_in_order() {
    let fake = fake();
    let files = ["en-us", "fr-fr", "ar-tn"].map(|n| LayoutFile::golden(n).unwrap());
    assert_eq!(ids(&fake), files.each_ref().map(|f| f.id));
    let shown = fake.current();
    assert_eq!(*shown, LayoutTable::from(&files[0]));
    assert_eq!(shown.keys[&Q].base, "q");
    assert!(fake.list()[2].rtl, "Arabic is right to left");
    let isos: Vec<String> = fake.list().into_iter().map(|l| l.iso).collect();
    assert_eq!(isos, ["en", "fr", "ar"], "each label carries its language");
}

#[test]
fn a_step_or_pick_only_asks_until_the_test_shows_the_layout() {
    let fake = fake();
    let fr = ids(&fake)[1];
    assert_eq!((fake.step(-1), fake.pick(fr)), (Ok(()), Ok(())));
    assert_eq!(fake.asked(), [Ask::Step(-1), Ask::Pick(fr)]);
    assert!(!fake.refresh());
    assert_eq!(fake.current().keys[&Q].base, "q", "not switched yet");
    assert!(fake.show(fr));
    assert_eq!(fake.current().keys[&Q].base, "a", "AZERTY");
}

#[test]
fn a_switch_lands_on_the_next_refresh_which_counts_its_calls() {
    let fake = fake();
    let fr = ids(&fake)[1];
    assert!(fake.switch(fr));
    assert_eq!(fake.current().keys[&Q].base, "q", "not seen yet");
    assert!(fake.refresh(), "the refresh sees the switch");
    assert_eq!(fake.current().keys[&Q].base, "a");
    assert!(!fake.refresh(), "nothing more changed");
    assert_eq!(fake.refreshes(), 2);
    assert!(!fake.switch(LayoutId::new(1)));
}

#[test]
fn a_refresh_that_changes_the_layout_runs_the_listener_once_it_shows() {
    let fake = Arc::new(fake());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (weak, log) = (Arc::downgrade(&fake), Arc::clone(&seen));
    fake.on_change(move || {
        let shown = weak.upgrade().map(|f| f.current().label.id);
        log.lock().unwrap().extend(shown);
    });
    let fr = ids(&fake)[1];
    assert!(!fake.refresh());
    fake.switch(fr);
    assert!(fake.refresh());
    assert_eq!(*seen.lock().unwrap(), [fr], "it reads the new layout");
}

#[test]
fn an_error_the_test_sets_refuses_steps_and_picks_until_cleared() {
    let fake = fake();
    let fr = ids(&fake)[1];
    fake.set_error(Some(SwitchError::Refused));
    assert_eq!(fake.step(1), Err(SwitchError::Refused));
    assert_eq!(fake.pick(fr), Err(SwitchError::Refused));
    assert!(fake.asked().is_empty());
    fake.set_error(None);
    assert_eq!(fake.step(1), Ok(()));
}

#[test]
fn a_step_that_lands_on_the_shown_layout_asks_nothing() {
    let lone = FakeLayouts::golden(&["ar-tn"]).unwrap();
    assert_eq!((lone.step(1), lone.step(-1)), (Ok(()), Ok(())));
    let fake = fake();
    assert_eq!(
        (fake.step(0), fake.step(3)),
        (Ok(()), Ok(())),
        "a full turn"
    );
    assert!(lone.asked().is_empty() && fake.asked().is_empty());
    assert_eq!(fake.step(2), Ok(()));
    assert_eq!(fake.asked(), [Ask::Step(2)]);
}

#[test]
fn a_removed_layout_leaves_the_list_and_the_first_shows_in_its_place() {
    let fake = fake();
    let [en, fr, ar] = [0, 1, 2].map(|i| ids(&fake)[i]);
    assert!(fake.show(ar) && fake.switch(fr));
    assert!(fake.remove(ar), "the shown one");
    assert_eq!(ids(&fake), [en, fr]);
    assert_eq!(fake.current().label.id, en);
    assert!(
        fake.remove(fr) && !fake.refresh(),
        "a switch to it is dropped"
    );
    assert_eq!((ids(&fake), fake.current().label.id), (vec![en], en));
    assert!(!fake.remove(fr), "not listed any more");
    assert_eq!(fake.step(1), Ok(()));
    assert!(fake.asked().is_empty(), "a lone layout asks nothing");
}

#[test]
fn an_unknown_layout_or_an_empty_list_is_refused() {
    let fake = fake();
    let none = LayoutId::new(1);
    assert_eq!(fake.pick(none), Err(SwitchError::NotInstalled));
    assert!(!fake.show(none));
    let empty = FakeLayouts::new(Vec::new());
    assert_eq!(empty.step(1), Err(SwitchError::NotInstalled));
    assert_eq!(*empty.current(), LayoutTable::default());
    assert!(fake.asked().is_empty() && empty.asked().is_empty());
}
