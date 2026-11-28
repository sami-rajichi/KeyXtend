//! The actor over the fakes, one file per area: Copy, Save, recordings, the editor, a maximised editor (`full`), every
//! end, Quit, the start's sweep, the panel, the keyboard's keys, the work area (`place`) and the debug times (`logs`);
//! `rig` drives it without a thread. Here: the start, the stop, clicks on an older face, a slow freeze, and the
//! keyboard's ask.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod aside;
mod bare;
mod copy;
mod crop;
mod editor;
mod ends;
mod esc;
mod fakes;
mod frame;
mod full;
mod gear;
mod keys;
mod logs;
mod panel;
mod place;
mod play;
mod quit;
mod record;
mod reel;
mod rig;
mod save;
mod sweep;

use std::time::Duration;

use kx_module_api::ports::{
    BarKey, FolderKey, Keep, PanelLine, SnipAct, SnipAside, SnipClick, SnipLit, SnipNote,
    SnipStage, StripKey,
};

use super::{Job, Msg};
use rig::{Heard, Rig};

/// Drops a job unrun, as a thread that died before it ran.
fn lost(job: Job) -> bool {
    drop(job);
    true
}

#[test]
fn a_job_that_dies_before_it_answers_still_answers() {
    let mut rig = Rig::new();
    rig.snap();
    rig.actor.ports.spawn = lost;
    rig.keep(Keep::Copy);
    assert_eq!(rig.page().note, Some(SnipNote::CopyFailed));
    rig.act(SnipClick::Strip(StripKey::Close));
    assert!(rig.page().ask.is_some(), "the strip acts again");
    let mut rig = Rig::new();
    rig.act(SnipClick::Open);
    rig.actor.ports.spawn = lost;
    rig.act(SnipClick::Folder(FolderKey::Change));
    assert_eq!(rig.page().note, Some(SnipNote::FolderFailed));
    assert!(!rig.page().dialog, "no dialog waits");
}

#[test]
fn the_start_tells_a_dark_key_an_empty_view_and_no_line() {
    let rig = Rig::new();
    let heard = rig.heard();
    assert!(heard.contains(&Heard::Lit(SnipLit::Off)));
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert!(heard.contains(&Heard::Line(None)));
    assert_eq!(rig.views().len(), 1, "one view until something changes");
}

#[test]
fn the_stop_ends_a_snip_and_nothing_is_handled_after_it() {
    let mut rig = Rig::new();
    rig.snap();
    rig.send(Msg::Quit);
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(rig.told_line(), None);
    let told = rig.heard().len();
    rig.key();
    assert_eq!(rig.heard().len(), told, "a stopped actor tells nothing");
    assert_eq!(rig.fake.freeze.caps().len(), 1, "and freezes nothing");
}

#[test]
fn a_click_on_windows_of_another_face_is_dropped() {
    let mut rig = Rig::new();
    rig.key();
    let picking = rig.page().round;
    rig.act(SnipClick::Bar(BarKey::Close));
    rig.key();
    rig.send(Msg::Act(SnipAct {
        round: picking.wrapping_sub(1),
        click: SnipClick::Bar(BarKey::Close),
    }));
    assert!(
        matches!(rig.stage(), SnipStage::Pick(_)),
        "a click from before the freeze"
    );
}

#[test]
fn a_slow_freeze_never_holds_the_actor_and_a_late_one_is_dropped() {
    let mut rig = Rig::new();
    rig.actor.ports.spawn = rig::held;
    rig.key();
    rig.key();
    assert_eq!(
        rig.sides().last(),
        Some(&Heard::Aside(SnipAside::Back)),
        "ended while freezing"
    );
    rig::release();
    rig.pump();
    assert_eq!(rig.fake.freeze.caps().len(), 1);
    assert_eq!(rig.stage(), SnipStage::Idle, "the late screens are dropped");
}

#[test]
fn the_freeze_waits_for_the_keyboards_step_aside_or_aside_ms() {
    let mut rig = Rig::new();
    rig.send(Msg::Key);
    assert!(
        rig.fake.freeze.caps().is_empty(),
        "the keyboard steps aside first"
    );
    rig.act(SnipClick::Aside);
    assert_eq!(rig.fake.freeze.caps().len(), 1, "the ui answered");
    let mut rig = Rig::new();
    rig.send(Msg::Key);
    let aside = Duration::from_millis(u64::from(rig.actor.settings.aside_ms));
    assert_eq!(rig.actor.wait(), Some(aside));
    rig.advance(aside);
    assert_eq!(rig.fake.freeze.caps().len(), 1, "no answer came");
    assert!(matches!(rig.stage(), SnipStage::Pick(_)));
}

#[test]
fn an_ask_tells_the_light_and_the_line_again_even_unchanged() {
    let mut rig = Rig::new();
    let at = rig.heard().len();
    rig.send(Msg::Ask);
    let idle = [Heard::Lit(SnipLit::Off), Heard::Line(None)];
    assert_eq!(rig.heard()[at..], idle, "dark, and no line");
    rig.snap();
    let at = rig.heard().len();
    rig.send(Msg::Ask);
    let strip = [Heard::Lit(SnipLit::On), Heard::Line(Some(PanelLine::None))];
    assert_eq!(rig.heard()[at..], strip, "no new view either");
}
