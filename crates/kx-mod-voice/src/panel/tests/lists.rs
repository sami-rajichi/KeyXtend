//! History and Options through the panel: texts ticked in order, then Insert, Copy, Pin or Delete; Delete all asks
//! first; an option's switch and stepper write its setting.

use kx_module_api::ports::{
    HistorySheet, OptionValue, PanelButton, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::rig::Rig;
use super::Do;

/// The History page as shown.
fn history(rig: &mut Rig) -> HistorySheet {
    match rig.view().unwrap().sheet {
        VoiceSheet::History(sheet) => sheet,
        other => panic!("not History: {other:?}"),
    }
}

/// The rig on the History page with `n` texts.
fn on_history(n: u32) -> Rig {
    let mut rig = Rig::new();
    rig.texts(n);
    rig.click(&VoiceClick::Open(VoiceTab::History));
    rig
}

#[test]
fn texts_are_ticked_in_order_then_inserted_in_that_order() {
    let mut rig = on_history(12);
    let sheet = history(&mut rig);
    let (third, first) = (sheet.rows[2].id, sheet.rows[0].id);
    assert!(
        !rig.view().unwrap().acts.contains(&PanelButton::Insert),
        "nothing ticked"
    );
    rig.click(&VoiceClick::Row(2));
    rig.click(&VoiceClick::Row(0));
    let view = rig.view().unwrap();
    assert_eq!(view.ticked, 2);
    let sheet = history(&mut rig);
    assert_eq!((sheet.rows[2].tick, sheet.rows[0].tick), (Some(1), Some(2)));
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Insert)),
        [Do::Insert(vec![third, first])]
    );
    assert_eq!(rig.view().unwrap().ticked, 0, "Insert clears the ticks");
}

#[test]
fn a_row_click_on_rows_that_moved_since_is_dropped() {
    let mut rig = on_history(3);
    rig.view();
    let seen = rig.round;
    rig.click(&VoiceClick::Row(1));
    rig.click_at(seen, &VoiceClick::Row(2));
    assert_eq!(rig.view().unwrap().ticked, 2, "a tick keeps the rows");
    let seen = rig.round;
    let id = history(&mut rig).rows[2].id;
    rig.history.pin(&[id]);
    rig.view();
    rig.click_at(seen, &VoiceClick::Row(0));
    assert_eq!(
        rig.view().unwrap().ticked,
        2,
        "the moved rows' click is dropped"
    );
    rig.click(&VoiceClick::Row(0));
    assert_eq!(rig.view().unwrap().ticked, 1, "the pinned text unticked");
}

#[test]
fn copy_keeps_the_ticks_and_pin_reads_unpin_on_pinned_texts() {
    let mut rig = on_history(3);
    let id = history(&mut rig).rows[1].id;
    rig.click(&VoiceClick::Row(1));
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Copy)),
        [Do::Copy(vec![id])]
    );
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Pin)),
        [Do::Pin(vec![id])]
    );
    rig.history.pin(&[id]);
    let pinned = history(&mut rig).rows[0].id;
    assert_eq!(pinned, id, "pinned texts come first");
    rig.click(&VoiceClick::Row(0));
    assert!(history(&mut rig).unpin);
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Pin)),
        [Do::Unpin(vec![id])]
    );
}

#[test]
fn delete_all_asks_first_counting_the_unpinned_texts() {
    let mut rig = on_history(3);
    let id = history(&mut rig).rows[0].id;
    rig.history.pin(&[id]);
    assert!(
        rig.click(&VoiceClick::Button(PanelButton::DeleteAll))
            .is_empty()
    );
    let sheet = history(&mut rig);
    assert!(sheet.confirm);
    assert_eq!((sheet.total, sheet.unpinned), (3, 2));
    assert!(
        rig.view()
            .unwrap()
            .acts
            .iter()
            .all(|a| matches!(a, PanelButton::Back | PanelButton::Close))
    );
    let out = rig.click(&VoiceClick::Confirm(true));
    assert!(matches!(&out[..], [Do::DeleteAll(asked)] if asked.count() == 2));
    assert!(!history(&mut rig).confirm);
}

#[test]
fn no_to_delete_all_keeps_everything() {
    let mut rig = on_history(2);
    rig.click(&VoiceClick::Button(PanelButton::DeleteAll));
    assert!(rig.click(&VoiceClick::Confirm(false)).is_empty());
    assert!(!history(&mut rig).confirm);
}

#[test]
fn a_ticked_text_gone_from_history_loses_its_tick() {
    let mut rig = on_history(3);
    let id = history(&mut rig).rows[0].id;
    rig.click(&VoiceClick::Row(0));
    rig.history.delete(&[id]);
    assert_eq!(rig.view().unwrap().ticked, 0);
}

#[test]
fn the_history_lists_stored_texts_with_history_off() {
    let mut rig = on_history(2);
    rig.settings.history = false;
    assert_eq!(history(&mut rig).rows.len(), 2);
}

/// The Options page's rows as shown.
fn options(rig: &mut Rig) -> Vec<kx_module_api::ports::OptionRow> {
    match rig.view().unwrap().sheet {
        VoiceSheet::Options(rows) => rows,
        other => panic!("not Options: {other:?}"),
    }
}

#[test]
fn an_options_switch_and_stepper_write_their_setting() {
    let mut rig = Rig::new();
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    let rows = options(&mut rig);
    assert_eq!(rows.len(), 5);
    assert_eq!(
        (rows[0].key, &rows[0].value),
        ("sounds", &OptionValue::Switch(true))
    );
    let out = rig.click(&VoiceClick::Flip(0));
    assert_eq!(out, [Do::Set("sounds", toml::Value::Boolean(false))]);
    let step = rig.click(&VoiceClick::Step { row: 3, up: false });
    assert!(
        matches!(&step[..], [Do::Set("done_ms", toml::Value::Integer(_))]),
        "{step:?}"
    );
    assert!(
        rig.click(&VoiceClick::Row(0)).is_empty(),
        "an option row ticks nothing"
    );
}

#[test]
fn a_stepper_under_a_switch_that_is_off_does_nothing() {
    let mut rig = Rig::new();
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    assert!(!options(&mut rig)[2].live, "silence_stop is off by default");
    assert!(rig.click(&VoiceClick::Step { row: 2, up: true }).is_empty());
    rig.settings.silence_stop = true;
    let out = rig.click(&VoiceClick::Step { row: 2, up: true });
    assert!(
        matches!(
            &out[..],
            [Do::Set("silence_stop_s", toml::Value::Integer(4))]
        ),
        "{out:?}"
    );
}
