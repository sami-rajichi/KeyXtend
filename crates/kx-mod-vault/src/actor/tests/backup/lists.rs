//! The lists a place is picked from, read on the job thread: a fill is answered while one is read, and a list that
//! comes after the page moved on is dropped.

use kx_module_api::ports::{FillButton, FillClick, FillTyped, RowButton, Sheet};
use kx_platform_fake::PlaceCall;

use super::super::rig::Rig;
use super::{pick, press};
use crate::actor::Msg;

#[test]
fn a_listing_runs_off_the_actor_so_a_fill_is_still_answered_meanwhile() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(FillClick::Row {
        row: 1,
        button: RowButton::Password,
    });
    press(&mut rig, FillButton::Backup);
    let listed = rig.fakes.places.calls(PlaceCall::Suggested);
    assert_eq!(listed, 0, "the listing waits for the job thread");
    let (round, saves) = (rig.types().last().unwrap().round, rig.sealed.saves());
    rig.send(Msg::Typed(FillTyped { round, ok: true }));
    assert_eq!(rig.sealed.saves(), saves + 1, "the fill was answered");
    rig.jobs();
    assert!(matches!(rig.page().sheet, Sheet::Places(_)));
}

#[test]
fn a_folder_list_that_comes_after_back_is_dropped_and_the_page_stays() {
    let mut rig = Rig::new();
    rig.open();
    for button in [FillButton::Backup, FillButton::OtherFolder] {
        press(&mut rig, button);
        rig.jobs();
    }
    pick(&mut rig, 1);
    press(&mut rig, FillButton::Back);
    let (shown, views) = (rig.page().sheet, rig.views().len());
    assert!(rig.job(), "the opened folder's list comes late");
    let after = (rig.page().sheet, rig.views().len());
    assert_eq!(after, (shown, views), "the page stays as Back left it");
    rig.jobs();
    let Sheet::Folders(drives) = rig.page().sheet else {
        panic!("not the folder list: {:?}", rig.page().sheet);
    };
    assert_eq!(
        (drives.folder, drives.rows.len()),
        (None, 3),
        "Back's own list shows"
    );
}
