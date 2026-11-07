//! The boxes found offer themselves only over the list of an empty notebook, and count as offered only once they show.

use kx_module_api::ports::{FillButton, Sheet};

use super::super::rig::Rig;
use super::bring::{found, new_computer, plant_sample};
use super::press;
use crate::actor::Msg;

/// Closes the open panel and opens it again, every job run.
fn open_again(rig: &mut Rig) {
    rig.send(Msg::Open);
    rig.open();
    rig.jobs();
}

#[test]
fn an_offer_that_came_back_over_another_page_shows_at_the_next_open() {
    let mut rig = new_computer();
    plant_sample(&rig, 1);
    rig.open();
    press(&mut rig, FillButton::New);
    rig.jobs();
    assert!(
        matches!(rig.page().sheet, Sheet::Edit(_)),
        "an edit is never dropped"
    );
    press(&mut rig, FillButton::Cancel);
    open_again(&mut rig);
    assert_eq!(found(&rig), [1]);
}

#[test]
fn an_offer_that_showed_never_shows_again_this_run() {
    let mut rig = new_computer();
    plant_sample(&rig, 1);
    rig.open();
    rig.jobs();
    assert_eq!(found(&rig), [1]);
    press(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
    open_again(&mut rig);
    assert!(matches!(rig.page().sheet, Sheet::List(_)), "once a run");
}
