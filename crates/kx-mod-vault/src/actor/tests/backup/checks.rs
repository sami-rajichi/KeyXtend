//! Memory checks over the actor: each falls due a step of `check_days` after setup or the last right answer, the last
//! step repeating; a right answer steps on, a wrong one counts down, and the last wrong one makes a new password.
//! Later, the switch, a wall clock that jumps either way, and no banner while a fill waits or an entry is typed.

use std::time::Duration;

use kx_module_api::ports::{
    Date, FillButton, FillClick, FillNote, FillTyped, PasswordAsk, RowButton, Sheet,
};
use kx_module_api::{NoticeAsk, keys};

use super::super::boxes::{CLOUD, STICK};
use super::super::logs;
use super::super::rig::Rig;
use super::{
    CLOUD_ROW, KEY, MINUTE, NEW_KEY, box_in, kept, opens, press, set_up, two_places, type_password,
};
use crate::actor::Msg;
use crate::settings::VaultSettings;

/// A wrong password.
const WRONG: &str = "not-the-password";

/// The waits before each check, and after Later.
fn waits() -> (Vec<Duration>, Duration) {
    let settings = VaultSettings::from_defaults().unwrap();
    (settings.check_waits(), settings.later())
}

/// How many memory check banners asked, each with Check and Later.
fn banners(rig: &Rig) -> usize {
    let checks = rig
        .notices()
        .into_iter()
        .filter(|n| n.key == keys::VAULT_CHECK);
    let asks: Vec<_> = checks.map(|n| n.ask).collect();
    assert!(asks.iter().all(|a| *a == Some(&NoticeAsk::CHECK_LATER)));
    asks.len()
}

/// Answers the banner: Check, or Later.
fn answer(rig: &mut Rig, check: bool) {
    rig.send(Msg::Answered {
        key: keys::VAULT_CHECK,
        yes: check,
    });
}

/// What the password page on the last view asks, if one shows.
fn asks(rig: &Rig) -> Option<PasswordAsk> {
    match rig.page().sheet {
        Sheet::Password(page) => Some(page.ask),
        _ => None,
    }
}

/// A rig set up in the cloud folder, its first check due.
fn due() -> Rig {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.pass(waits().0[0]);
    rig
}

#[test]
fn a_checks_password_needs_a_character_and_loses_its_marks_as_the_new_one_did() {
    let mut rig = due();
    answer(&mut rig, true);
    press(&mut rig, FillButton::Done);
    assert_eq!(rig.note(), Some(FillNote::TooShort));
    assert_eq!(rig.fakes.queued(), 0, "nothing goes to the worker");
    type_password(&mut rig, &format!("\u{200f}{KEY}\u{200e}"));
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Remembered));
}

#[test]
fn a_check_falls_due_after_each_step_and_the_last_step_repeats() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let steps = waits().0;
    let last = *steps.last().unwrap();
    for (n, wait) in steps.iter().chain([&last, &last]).enumerate() {
        rig.pass(wait.saturating_sub(MINUTE));
        assert_eq!(banners(&rig), n, "not yet before {wait:?}");
        rig.pass(MINUTE);
        assert_eq!(banners(&rig), n + 1, "due after {wait:?}");
        answer(&mut rig, true);
        type_password(&mut rig, KEY);
        rig.jobs();
        assert_eq!(rig.note(), Some(FillNote::Remembered));
    }
}

#[test]
fn a_wrong_answer_counts_down_and_the_last_makes_a_new_password_for_every_box() {
    let mut rig = two_places();
    rig.pass(waits().0[0]);
    answer(&mut rig, true);
    let tries = VaultSettings::from_defaults().unwrap().check_tries;
    assert_eq!(asks(&rig), Some(PasswordAsk::Check { tries }));
    for left in (1..tries).rev() {
        type_password(&mut rig, WRONG);
        rig.jobs();
        assert_eq!(rig.note(), Some(FillNote::WrongPassword));
        assert_eq!(asks(&rig), Some(PasswordAsk::Check { tries: left }));
    }
    type_password(&mut rig, WRONG);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::TriesUsed));
    assert!(matches!(asks(&rig), Some(PasswordAsk::New { .. })));
    type_password(&mut rig, NEW_KEY);
    rig.jobs();
    for folder in [CLOUD, STICK] {
        let rewritten = box_in(&rig, folder).unwrap();
        assert!(
            opens(&rewritten, NEW_KEY) && !opens(&rewritten, KEY),
            "{folder}"
        );
    }
    assert_eq!(
        kept(&rig).checks.missed,
        0,
        "a new password starts the checks again"
    );
}

#[test]
fn a_right_answer_after_a_wrong_one_gives_every_try_back() {
    let mut rig = due();
    answer(&mut rig, true);
    type_password(&mut rig, WRONG);
    rig.jobs();
    type_password(&mut rig, KEY);
    rig.jobs();
    assert_eq!(kept(&rig).checks.missed, 0);
    rig.pass(waits().0[1]);
    answer(&mut rig, true);
    let tries = VaultSettings::from_defaults().unwrap().check_tries;
    assert_eq!(asks(&rig), Some(PasswordAsk::Check { tries }));
}

#[test]
fn later_asks_again_after_later_h_and_so_does_leaving_the_check() {
    let mut rig = due();
    let later = waits().1;
    answer(&mut rig, false);
    rig.pass(later.saturating_sub(MINUTE));
    assert_eq!(banners(&rig), 1);
    rig.pass(MINUTE);
    assert_eq!(banners(&rig), 2, "Later asks again");
    answer(&mut rig, true);
    press(&mut rig, FillButton::Back);
    rig.pass(later);
    assert_eq!(banners(&rig), 3, "leaving the check is Later");
}

#[test]
fn turned_off_no_check_asks_and_the_loop_waits_for_none() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.click(FillClick::Checks(false));
    rig.pass(waits().0[0] * 2);
    assert_eq!(banners(&rig), 0);
    assert_eq!(rig.actor.wait(), None);
    rig.click(FillClick::Checks(true));
    rig.pass(MINUTE);
    assert_eq!(banners(&rig), 1, "on again, the overdue check asks once");
}

#[test]
fn turning_checks_off_under_a_banner_lets_them_ask_again_when_on() {
    let mut rig = due();
    assert_eq!(banners(&rig), 1);
    rig.click(FillClick::Checks(false));
    rig.click(FillClick::Checks(true));
    assert_eq!(banners(&rig), 2, "the check is still overdue");
}

#[test]
fn a_clock_set_ahead_asks_once_and_one_set_back_still_asks() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let steps = waits().0;
    rig.pass(steps[0] * 100);
    (0..5).for_each(|_| rig.actor.tick());
    assert_eq!(banners(&rig), 1, "one check, never a storm");
    answer(&mut rig, true);
    type_password(&mut rig, KEY);
    rig.jobs();
    rig.calendar.set(Date::new(2025, 1, 1).unwrap(), 9, 30);
    rig.actor.tick();
    rig.pass(steps[1].saturating_sub(MINUTE));
    assert_eq!(banners(&rig), 1);
    rig.pass(MINUTE);
    assert_eq!(banners(&rig), 2, "a clock set back never stops the checks");
}

#[test]
fn no_banner_shows_while_a_fill_waits_or_an_entry_is_typed() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    press(&mut rig, FillButton::Back);
    let bank = rig.names().iter().position(|n| n == "Bank").unwrap();
    rig.click(FillClick::Row {
        row: u8::try_from(bank).unwrap(),
        button: RowButton::User,
    });
    rig.pass(waits().0[0]);
    assert_eq!(banners(&rig), 0, "a fill waits");
    let round = rig.types().last().unwrap().round;
    rig.send(Msg::Typed(FillTyped { round, ok: true }));
    assert_eq!(banners(&rig), 1);
    answer(&mut rig, false);
    press(&mut rig, FillButton::Back);
    press(&mut rig, FillButton::New);
    rig.pass(waits().1);
    assert_eq!(banners(&rig), 1, "an entry is typed");
    press(&mut rig, FillButton::Cancel);
    assert_eq!(banners(&rig), 2);
}

#[test]
fn the_checks_password_never_shows_in_debug_or_a_log() {
    let mut rig = due();
    answer(&mut rig, true);
    rig.text(KEY);
    assert!(!format!("{:?}", rig.page()).contains(KEY));
    rig.fakes.boxes.fail(
        kx_platform_fake::BoxCall::Check,
        kx_module_api::ports::BoxError::TimedOut,
    );
    let lines = logs::lines_of(|| {
        press(&mut rig, FillButton::Done);
        rig.jobs();
    });
    assert_eq!(rig.note(), Some(FillNote::BoxFailed));
    assert!(lines.iter().all(|l| !l.contains(KEY)), "{lines:?}");
}
