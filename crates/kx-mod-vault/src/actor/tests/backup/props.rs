//! Property test of the backup pages (`backup::flow`): any run of clicks, keys, job answers, waits, failed saves and
//! whole walks through a flow never panics, and while held no box is sealed or written and no write is queued.

mod steps;

use kx_module_api::keys;
use kx_module_api::ports::{FillButton, FillClick, SealedError, Sheet, box_file};
use kx_platform_fake::{BoxCall, PlaceCall};
use proptest::prelude::*;

use super::super::boxes::STICK;
use super::super::rig::{Rig, sample};
use super::bring::{new_computer, plant_sample};
use super::held::set_up_before;
use super::import::{file, login, plant_file};
use super::{
    NEW_KEY, OLD_KEY, STICK_ROW, add, pick, plant, press, their_box, type_password, waits,
};
use crate::actor::Msg;
use crate::backup::Done;
use crate::settings::VaultSettings;
use steps::{Step, Walk, answers, step};

/// Random runs tried.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 48;
/// How far a stale answer's round is from the job's.
const STALE: u32 = 1000;
/// The drive that holds the `KeePass` file, as the folder list names it.
const DRIVE_NAME: &str = "D:";

/// What a run starts from.
#[derive(Copy, Clone, Debug)]
enum Start {
    /// A notebook that did not read, over a backup set up, so the boxes are held.
    Held,
    /// A notebook and a backup set up.
    SetUp,
    /// No entry and no backup, as on a new computer.
    New,
}

/// An open rig from `start`, with a box another computer sealed and a `KeePass` file to find, and such a box on the
/// stick when `foreign`.
fn rig(start: Start, foreign: bool) -> Rig {
    let mut rig = match start {
        Start::Held => Rig::held(&set_up_before()),
        Start::SetUp => Rig::kept(sample(), &set_up_before()),
        Start::New => new_computer(),
    };
    plant_sample(&rig, 1);
    plant_file(&rig, vec![login("Site", "sam")]);
    if foreign {
        plant(&rig, STICK, &box_file(), their_box(sample(), OLD_KEY).0, 2);
    }
    rig.open();
    rig
}

/// Leaves any page for the Backup page, past the offer an empty notebook makes.
fn to_home(rig: &mut Rig) {
    rig.send(Msg::Close);
    rig.open();
    rig.jobs();
    if !matches!(rig.page().sheet, Sheet::List(_)) {
        press(rig, FillButton::Back);
    }
    press(rig, FillButton::Backup);
    rig.jobs();
}

/// Clicks the shown folder list's row named `name`, if it shows.
fn open_row(rig: &mut Rig, name: &str) {
    if let Sheet::Folders(list) = rig.page().sheet {
        let at = list.rows.iter().position(|r| r.name.expose() == name);
        if let Some(row) = at.and_then(|at| u8::try_from(at).ok()) {
            pick(rig, row);
        }
    }
}

/// Goes the whole way through a flow from the Backup page, every job run.
fn walk(rig: &mut Rig, way: Walk) {
    to_home(rig);
    match way {
        Walk::AddPlace => {
            add(rig, STICK_ROW);
            if matches!(rig.page().sheet, Sheet::Foreign(_)) {
                rig.click(FillClick::Answer(true));
                type_password(rig, OLD_KEY);
            }
        }
        Walk::NewPassword => {
            press(rig, FillButton::ChangePassword);
            type_password(rig, NEW_KEY);
        }
        Walk::BringBack => {
            press(rig, FillButton::BringBack);
            rig.jobs();
            pick(rig, 0);
            rig.jobs();
            type_password(rig, OLD_KEY);
        }
        Walk::Import => {
            press(rig, FillButton::Import);
            rig.jobs();
            open_row(rig, DRIVE_NAME);
            rig.jobs();
            open_row(rig, &file());
            type_password(rig, OLD_KEY);
        }
    }
    rig.jobs();
}

/// Gives the oldest job the answer at `at`, of its own round or a stale one.
fn answer(rig: &mut Rig, at: prop::sample::Index, stale: bool) {
    let Some(ask) = rig.fakes.take() else {
        return;
    };
    let mut all = answers();
    let answer = all.swap_remove(at.index(all.len()));
    let round = if stale {
        ask.round.wrapping_add(STALE)
    } else {
        ask.round
    };
    rig.send(Msg::Done(Done { round, answer }));
}

/// Applies `step`.
fn apply(rig: &mut Rig, step: Step) {
    match step {
        Step::Walk(way) => walk(rig, way),
        Step::Click(click) => rig.click(click),
        Step::Key(key) => rig.send(Msg::Key(key)),
        Step::Run => {
            rig.job();
        }
        Step::Answer { at, stale } => answer(rig, at, stale),
        Step::Wait => rig.pass(waits().0 + waits().1),
        Step::Days => rig.pass(VaultSettings::from_defaults().unwrap().check_waits()[0]),
        Step::Notice { check, yes } => {
            let key = if check {
                keys::VAULT_CHECK
            } else {
                keys::VAULT_OFFER
            };
            rig.send(Msg::Answered { key, yes });
        }
        Step::Toggle => rig.send(Msg::Open),
        Step::FailSave => rig.sealed.fail_save(SealedError::Io),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_run_never_panics_and_writes_nothing_while_held(
        start in prop::sample::select(vec![Start::Held, Start::Held, Start::SetUp, Start::New]),
        foreign: bool,
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut rig = rig(start, foreign);
        for step in steps {
            apply(&mut rig, step);
            if rig.actor.backup.paused() {
                prop_assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 0);
                prop_assert_eq!(rig.fakes.places.calls(PlaceCall::Write), 0);
                prop_assert!(!rig.fakes.writes_queued());
            }
        }
        rig.send(Msg::Quit);
    }
}
