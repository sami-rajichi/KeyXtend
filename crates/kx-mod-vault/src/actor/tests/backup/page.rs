//! The Backup page and the first offer over the actor: the offer's Yes and Later, Remove place and its question,
//! Change password with a place gone, Back up now, a write due at the stop, and the checks switch; `held` has the hold.

use std::path::Path;

use kx_module_api::ports::{
    BackupAsk, EntryKind, FillButton, FillClick, PanelKey, PlaceError, RowButton, Sheet,
};
use kx_module_api::{NoticeAsk, keys};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::Rig;
use super::{
    CLOUD_ROW, KEY, NEW_KEY, STICK_ROW, add, asked, box_in, kept, opens, press, rows, save_note,
    set_up, two_places, type_password, waits,
};
use crate::actor::Msg;
use crate::backup::Kept;
use crate::settings::MEMORY_CHECKS;

#[test]
fn the_first_entry_ever_saved_brings_one_offer_and_later_never_asks_again() {
    let mut rig = Rig::kept(Vec::new(), &Kept::default());
    rig.open();
    save_note(&mut rig, "Shed");
    let offers = rig.notices();
    assert_eq!(offers.len(), 1);
    assert_eq!(
        (offers[0].key, offers[0].ask),
        (keys::VAULT_OFFER, Some(&NoticeAsk::YES_LATER))
    );
    rig.send(Msg::Answered {
        key: keys::VAULT_OFFER,
        yes: false,
    });
    save_note(&mut rig, "Barn");
    assert_eq!(rig.notices().len(), 1);
    assert!(kept(&rig).offered, "kept for the next start");
}

#[test]
fn the_offers_yes_opens_the_panel_on_setup_but_never_over_an_unsaved_edit() {
    let mut rig = Rig::kept(Vec::new(), &Kept::default());
    rig.open();
    save_note(&mut rig, "Shed");
    press(&mut rig, FillButton::New);
    rig.send(Msg::Answered {
        key: keys::VAULT_OFFER,
        yes: true,
    });
    assert!(matches!(rig.page().sheet, Sheet::Edit(_)), "the edit stays");
    rig.send(Msg::Open);
    rig.send(Msg::Answered {
        key: keys::VAULT_OFFER,
        yes: true,
    });
    assert_eq!(rig.panels().last(), Some(&true));
    assert!(matches!(rig.page().sheet, Sheet::Places(_)));
}

#[test]
fn the_offers_yes_over_an_edit_opens_setup_once_the_edit_is_saved_or_cancelled() {
    for (name, button) in [("", FillButton::Cancel), ("Barn", FillButton::Save)] {
        let mut rig = Rig::kept(Vec::new(), &Kept::default());
        rig.open();
        save_note(&mut rig, "Shed");
        press(&mut rig, FillButton::New);
        rig.click(FillClick::Kind(EntryKind::Note));
        rig.text(name);
        rig.send(Msg::Answered {
            key: keys::VAULT_OFFER,
            yes: true,
        });
        assert!(matches!(rig.page().sheet, Sheet::Edit(_)), "{button:?}");
        press(&mut rig, button);
        rig.jobs();
        assert!(
            matches!(rig.page().sheet, Sheet::Places(s) if !s.is_empty()),
            "{button:?}"
        );
    }
}

#[test]
fn remove_place_asks_once_esc_means_no_and_the_box_stays_and_is_known_again() {
    let mut rig = two_places();
    let remove = FillClick::Row {
        row: 1,
        button: RowButton::Remove,
    };
    rig.click(remove);
    assert_eq!(asked(&rig).0, Some(BackupAsk::Remove(1)));
    rig.send(Msg::Key(PanelKey::Esc));
    assert_eq!((asked(&rig).0, rows(&rig).len()), (None, 2), "Esc is No");
    rig.click(remove);
    rig.click(FillClick::Answer(true));
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
    assert!(box_in(&rig, STICK).is_some(), "its box stays");
    press(&mut rig, FillButton::BackUpNow);
    rig.jobs();
    assert_ne!(
        box_in(&rig, CLOUD),
        box_in(&rig, STICK),
        "only the removed place knows its box"
    );
    add(&mut rig, STICK_ROW);
    assert_eq!(
        rows(&rig).len(),
        2,
        "the box this computer wrote asks nothing"
    );
}

#[test]
fn a_place_dropped_under_removes_question_ends_it_so_yes_takes_no_other_place() {
    let mut rig = two_places();
    press(&mut rig, FillButton::BackUpNow);
    rig.click(FillClick::Row {
        row: 0,
        button: RowButton::Remove,
    });
    assert_eq!(asked(&rig).0, Some(BackupAsk::Remove(0)));
    rig.fakes.places.fail(PlaceCall::Write, PlaceError::Refused);
    rig.jobs();
    let stick = [("E: USB".to_owned(), true)];
    assert_eq!(rows(&rig), stick, "the refused place went");
    assert_eq!(asked(&rig).0, None, "its question went too");
    rig.click(FillClick::Answer(true));
    assert_eq!(rows(&rig), stick, "the other place stays");
}

#[test]
fn a_place_dropped_above_the_highlight_leaves_it_within_the_list() {
    let mut rig = two_places();
    press(&mut rig, FillButton::BackUpNow);
    rig.send(Msg::Key(PanelKey::Down));
    rig.fakes.places.fail(PlaceCall::Write, PlaceError::Refused);
    rig.jobs();
    rig.send(Msg::Key(PanelKey::Enter));
    assert_eq!(
        asked(&rig).0,
        Some(BackupAsk::Remove(0)),
        "Enter presses the one row left"
    );
}

#[test]
fn change_password_rewrites_every_box_and_one_gone_keeps_the_old_until_it_returns() {
    let mut rig = two_places();
    rig.fakes.places.vanish(Path::new(STICK));
    press(&mut rig, FillButton::ChangePassword);
    type_password(&mut rig, NEW_KEY);
    rig.job();
    let behind = [("OneDrive".to_owned(), false), ("E: USB".to_owned(), false)];
    assert_eq!(rows(&rig), behind, "every box waits for its rewrite");
    rig.jobs();
    assert_eq!(
        rows(&rig),
        [("OneDrive".to_owned(), true), ("E: USB".to_owned(), false)]
    );
    let cloud = box_in(&rig, CLOUD).unwrap();
    assert!(opens(&cloud, NEW_KEY) && !opens(&cloud, KEY));
    assert!(
        opens(&box_in(&rig, STICK).unwrap(), KEY),
        "the old password until rewritten"
    );
    rig.fakes.places.come_back(Path::new(STICK));
    rig.advance(waits().1);
    rig.jobs();
    assert!(opens(&box_in(&rig, STICK).unwrap(), NEW_KEY));
}

#[test]
fn back_up_now_writes_every_place_at_once() {
    let mut rig = two_places();
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    press(&mut rig, FillButton::BackUpNow);
    assert_eq!(rig.fakes.queued(), 1, "no wait");
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), seals + 1);
}

#[test]
fn a_write_due_at_the_stop_goes_after_the_next_start() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    save_note(&mut rig, "Shed");
    rig.send(Msg::Quit);
    assert_eq!(rig.fakes.stops(), 1, "the job thread ends");
    let behind = kept(&rig);
    assert!(behind.behind);
    let mut next = Rig::kept(Vec::new(), &behind);
    next.advance(waits().0);
    assert_eq!(next.fakes.queued(), 1);
}

#[test]
fn the_memory_checks_switch_writes_its_setting() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    assert!(asked(&rig).1);
    rig.click(FillClick::Checks(false));
    assert!(!asked(&rig).1);
    let section = rig.sink.section();
    assert_eq!(
        section.get(MEMORY_CHECKS),
        Some(&toml::Value::Boolean(false))
    );
}

#[test]
fn a_second_yes_after_one_that_waited_on_an_edit_opens_setup_once() {
    let mut rig = Rig::kept(Vec::new(), &Kept::default());
    rig.open();
    save_note(&mut rig, "Shed");
    press(&mut rig, FillButton::New);
    let yes = || Msg::Answered {
        key: keys::VAULT_OFFER,
        yes: true,
    };
    rig.send(yes());
    rig.send(Msg::Open);
    rig.send(yes());
    rig.jobs();
    assert_eq!(rig.fakes.places.calls(PlaceCall::Suggested), 1);
}
