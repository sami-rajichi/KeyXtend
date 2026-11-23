//! A skipped app's copy is never put back (ADR-0048 decision 5): a password manager's clear timer could fire during
//! the gap and leave its password there. A note from a skipped app, or a change the watch could not read, refuses
//! until another app's copy or a clear, even with our echoes still due; so does a mark that is not our echo, and a
//! change not ours drops the echoes still due.

use kx_module_api::ports::ClipNote;
use toml::Value;

use super::super::rig::Rig;
use super::{lend, lent, round_trip};
use crate::fixture::{app, text};

/// The skipped app.
pub(super) const VAULT: &str = "vault.exe";
/// A password manager that marks its copies by number markers alone, and is not skipped.
const KEEPER: &str = "keeper.exe";

/// A rig that skips `VAULT`'s copies.
pub(super) fn skipping() -> Rig {
    let skip = Value::Array(vec![VAULT.into()]);
    Rig::build(|_, host| host.put("skip_apps", skip))
}

/// Lends once and checks it was refused with nothing read or written.
fn refused(rig: &mut Rig, why: &str) {
    let (takes, tries) = (rig.fakes.clip.takes(), rig.fakes.writer.tries());
    lend(rig);
    assert_eq!(lent(rig).last(), Some(&false), "{why}");
    let after = (rig.fakes.clip.takes(), rig.fakes.writer.tries());
    assert_eq!(after, (takes, tries), "{why}: never read or written");
}

#[test]
fn a_skipped_apps_copy_is_typed_and_the_clipboard_untouched_until_another_copy() {
    let mut rig = skipping();
    rig.fakes.clip.set_text(Some("from the vault"));
    rig.copy(vec![text("from the vault")], Some(app(VAULT)));
    refused(&mut rig, "a copy");
    rig.fakes.clip.set_text(Some("mine"));
    rig.text("mine");
    assert_eq!(round_trip(&mut rig), [text("mine")], "another app's copy");
}

#[test]
fn a_skipped_apps_mark_or_a_missed_change_refuses_even_with_echoes_due() {
    let mut rig = skipping();
    rig.fakes.clip.set_text(Some("mine"));
    rig.text("mine");
    assert_eq!(round_trip(&mut rig), [text("mine")], "two echoes still due");
    rig.fakes.clip.set_text(Some("from the vault"));
    rig.push(ClipNote::Marked {
        app: Some(app(VAULT)),
    });
    refused(&mut rig, "its mark is never our echo");
    rig.push(ClipNote::Cleared { app: None });
    rig.push(ClipNote::Busy);
    refused(&mut rig, "a change the watch missed");
}

#[test]
fn a_mark_that_is_not_our_echo_refuses_until_another_copy_or_a_clear() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_text(Some("hunter2"));
    let keeper = || ClipNote::Marked {
        app: Some(app(KEEPER)),
    };
    rig.push(keeper());
    refused(&mut rig, "marked by number markers alone");
    rig.fakes.clip.set_text(Some("mine"));
    rig.text("mine");
    assert_eq!(round_trip(&mut rig), [text("mine")], "another app's copy");
    rig.push(ClipNote::Marked { app: None });
    rig.push(ClipNote::Marked { app: None });
    rig.fakes.clip.set_text(Some("hunter2"));
    rig.push(keeper());
    refused(&mut rig, "with no echo left due");
    rig.fakes.clip.set_text(None);
    rig.push(ClipNote::Cleared { app: None });
    assert!(round_trip(&mut rig).is_empty(), "a clear");
}

#[test]
fn a_change_not_ours_drops_the_echoes_due_so_a_later_mark_still_refuses() {
    let mut rig = Rig::new();
    let changes = [
        ClipNote::Copied {
            parts: vec![text("other")],
            app: None,
        },
        ClipNote::TooBig {
            app: None,
            bytes: u64::MAX,
        },
        ClipNote::Cleared { app: None },
    ];
    for change in changes {
        let why = format!("{change:?}");
        rig.fakes.clip.set_text(Some("mine"));
        rig.text("mine");
        assert_eq!(round_trip(&mut rig), [text("mine")], "two echoes still due");
        rig.push(change);
        rig.fakes.clip.set_text(Some("hunter2"));
        rig.push(ClipNote::Marked {
            app: Some(app(KEEPER)),
        });
        refused(&mut rig, &why);
    }
}
