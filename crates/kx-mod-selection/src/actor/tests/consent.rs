//! Translate's one question: nothing is read before its own Yes within `consent_s`, the Yes is saved, and Later, a late
//! answer, another notice or a forged one opens nothing (plan edge case 11).

use std::time::Duration;

use kx_module_api::ports::{SelChip, SelKind, SelSpot};
use kx_module_api::{Notice, NoticeArg, NoticeAsk, args, keys};

use super::super::Msg;
use super::rig::Rig;
use super::spots::sel;
use crate::ID;
use crate::settings::SelSettings;

/// The smallest step of time.
const TICK: Duration = Duration::from_micros(1);

/// How long a question waits by default.
fn limit() -> Duration {
    SelSettings::from_defaults().unwrap().consent()
}

/// A rig with a read-only selection of some text, its Translate clicked once.
fn asked() -> Rig {
    let mut rig = Rig::new();
    rig.watch.set_text("bonjour");
    rig.focus(sel(SelKind::ReadOnly, 40));
    rig.act(SelChip::Translate);
    rig
}

/// True when nothing was read or opened.
fn untouched(rig: &Rig) -> bool {
    rig.reads().is_empty() && rig.web.opened().is_empty()
}

#[test]
fn the_first_translate_asks_by_the_sites_name_and_reads_nothing_before_the_yes() {
    let mut rig = asked();
    let ask = rig.question().unwrap();
    assert_eq!(
        (ask.module, ask.ask),
        (Some(ID), Some(&NoticeAsk::YES_LATER))
    );
    let site = (args::SITE, NoticeArg::Text("Google Translate".into()));
    assert!(ask.args.contains(&site), "{ask:?}");
    assert!(untouched(&rig));
    rig.act(SelChip::Translate);
    assert_eq!(
        rig.keys(),
        [keys::TRANSLATE_ASK],
        "asked once while it shows"
    );
    rig.reply(true);
    assert_eq!((rig.reads().len(), rig.web.opened().len()), (1, 1));
    assert!(rig.saved().translate_ok, "the yes is kept");
}

#[test]
fn after_the_yes_a_new_strip_opens_at_once() {
    let mut rig = asked();
    rig.reply(true);
    rig.focus(sel(SelKind::ReadOnly, 60));
    rig.act(SelChip::Translate);
    assert_eq!(rig.web.opened().len(), 2);
    assert_eq!(rig.keys(), [keys::TRANSLATE_ASK]);
}

#[test]
fn a_click_while_the_question_waits_points_its_yes_at_the_latest_strip() {
    let mut rig = asked();
    rig.focus(sel(SelKind::ReadOnly, 60));
    rig.act(SelChip::Translate);
    assert_eq!(rig.keys(), [keys::TRANSLATE_ASK], "one question");
    rig.reply(true);
    assert_eq!(rig.web.opened().len(), 1, "{:?}", rig.keys());
}

#[test]
fn later_opens_nothing_and_the_next_click_asks_again() {
    let mut rig = asked();
    rig.reply(false);
    assert!(untouched(&rig));
    assert!(!rig.saved().translate_ok);
    rig.act(SelChip::Translate);
    assert_eq!(rig.keys(), [keys::TRANSLATE_ASK; 2]);
}

#[test]
fn a_yes_within_consent_s_opens_and_a_later_one_does_not() {
    let mut rig = asked();
    rig.advance(limit());
    rig.reply(true);
    assert_eq!(rig.web.opened().len(), 1, "at the limit");
    let mut rig = asked();
    rig.advance(limit().saturating_add(TICK));
    rig.reply(true);
    assert!(untouched(&rig));
    assert!(!rig.saved().translate_ok);
    rig.act(SelChip::Translate);
    assert_eq!(
        rig.keys().len(),
        2,
        "a late answer leaves the next click to ask"
    );
}

/// Notices that differ from `ask` in one part each: the key, the answers, the module, the site's name.
fn others(ask: &Notice) -> [Notice; 4] {
    let mut evil = ask.clone();
    evil.args.last_mut().unwrap().1 = NoticeArg::Text("Evil".into());
    [
        Notice {
            key: keys::VAULT_OFFER,
            ..ask.clone()
        },
        Notice {
            ask: Some(&NoticeAsk::YES_NO),
            ..ask.clone()
        },
        Notice {
            module: None,
            ..ask.clone()
        },
        evil,
    ]
}

#[test]
fn a_forged_or_other_answer_opens_nothing_and_leaves_the_question() {
    let mut rig = Rig::new();
    rig.watch.set_text("bonjour");
    rig.focus(sel(SelKind::ReadOnly, 40));
    let forged = Notice {
        ask: Some(&NoticeAsk::YES_LATER),
        ..Notice::about(keys::TRANSLATE_ASK, ID)
    };
    rig.send(Msg::Answered {
        notice: forged,
        yes: true,
    });
    assert!(untouched(&rig), "no question was raised");
    rig.act(SelChip::Translate);
    for notice in others(&rig.question().unwrap()) {
        rig.send(Msg::Answered { notice, yes: true });
    }
    assert!(untouched(&rig));
    rig.reply(true);
    assert_eq!(
        rig.web.opened().len(),
        1,
        "the question waited for its own answer"
    );
}

#[test]
fn the_box_is_checked_again_after_the_wait_right_before_the_read() {
    let mut rig = asked();
    rig.watch.set_still(Some(false));
    rig.reply(true);
    assert!(untouched(&rig));
    assert_eq!(rig.keys(), [keys::TRANSLATE_ASK, keys::TRANSLATE_FAILED]);
}

#[test]
fn a_box_that_turned_closed_during_the_wait_is_never_read() {
    let mut rig = asked();
    rig.watch.change_unheard(SelSpot {
        kind: SelKind::Closed,
        ..sel(SelKind::ReadOnly, 40)
    });
    rig.reply(true);
    assert!(untouched(&rig), "{:?}", rig.reads());
    assert_eq!(rig.keys(), [keys::TRANSLATE_ASK, keys::TRANSLATE_FAILED]);
}
