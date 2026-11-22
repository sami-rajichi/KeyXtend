//! Remove key in the Cloud page's footer (owner, 2026-11-21) acts on the ticked service whenever its key is sealed,
//! On, Off or with newer terms declined, and the row keeps its mark. With cloud off the page lists nothing and
//! switching it on sends nothing, so a key stays removable, and Off alone never deletes one.

use kx_module_api::ports::{CloudButton, CloudSheet, ServiceMark, ServiceName, VoiceClick};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::rows::{named, row_of, tick};
use super::{act, apply, card, cloud_sheet, lines_sheet, on_cloud, put};
use crate::services::tests::ROUTER;
use crate::services::tests::shipped::{entry, first, pick};
use crate::settings::{PARKED, SERVICES};

/// Groq, the first offered row, on with its key sealed, on the Cloud page.
fn keyed_groq() -> Rig {
    let mut rig = on_cloud("");
    put(&mut rig, "services", &entry(first(), ""));
    rig.keyed.insert(first().id.clone());
    rig
}

/// Ticks row `name`, then gives its mark, whether the footer offers Remove key, and what clicking it does.
fn remove(rig: &mut Rig, name: &ServiceName) -> (ServiceMark, bool, Vec<Do>) {
    tick(rig, name);
    let at = usize::from(row_of(rig, name));
    let sheet = cloud_sheet(rig);
    let offers = sheet.acts.contains(&CloudButton::Remove);
    let out = rig.click(&VoiceClick::Service(CloudButton::Remove));
    (sheet.rows[at].mark, offers, out)
}

/// Unticks row `name` if it is ticked, so `remove` ticks it once.
fn untick(rig: &mut Rig, name: &ServiceName) {
    let at = row_of(rig, name);
    if cloud_sheet(rig).rows[usize::from(at)].tick {
        rig.click(&VoiceClick::Row(at));
    }
}

#[test]
fn a_ticked_service_on_with_a_sealed_key_offers_remove_key_in_the_footer() {
    let mut rig = keyed_groq();
    let removed = vec![Do::RemoveKey(first().id.clone())];
    let want = (ServiceMark::Ready, true, removed);
    assert_eq!(remove(&mut rig, &first().shown()), want);
}

#[test]
fn switching_a_service_off_keeps_its_off_mark_and_remove_key_in_the_footer() {
    let mut rig = keyed_groq();
    let out = act(&mut rig, &first().shown(), CloudButton::Off);
    apply(&mut rig, &out);
    untick(&mut rig, &first().shown());
    let removed = vec![Do::RemoveKey(first().id.clone())];
    let want = (ServiceMark::Off, true, removed);
    assert_eq!(remove(&mut rig, &first().shown()), want);
}

#[test]
fn declining_newer_terms_leaves_remove_key_in_the_footer_without_agreeing_to_them() {
    let mut rig = on_cloud(&ROUTER.replace("terms = 1", "terms = 2"));
    let on = r#""router-test" = { on = 2026-11-01, terms = 1 }"#;
    put(&mut rig, "services", on);
    rig.keyed.insert("router-test".into());
    tick(&mut rig, &named("Router Test"));
    rig.click(&VoiceClick::Service(CloudButton::On));
    assert_eq!(card(&mut rig).on, None, "it asks again");
    let out = rig.click(&VoiceClick::Confirm(false));
    apply(&mut rig, &out);
    untick(&mut rig, &named("Router Test"));
    let removed = vec![Do::RemoveKey("router-test".into())];
    let want = (ServiceMark::Off, true, removed);
    assert_eq!(remove(&mut rig, &named("Router Test")), want);
    let services = &rig.settings.services;
    assert!(services.is_empty(), "the newer terms stay unconfirmed");
}

#[test]
fn a_ticked_service_with_no_key_offers_no_remove_key() {
    let mut rig = keyed_groq();
    rig.keyed.clear();
    assert_eq!(
        remove(&mut rig, &first().shown()),
        (ServiceMark::NeedsKey, false, Vec::new())
    );
}

#[test]
fn cloud_off_lists_nothing_and_switching_it_on_sends_nothing_so_a_key_stays_removable() {
    let mut rig = on_cloud("");
    rig.keyed.insert(first().id.clone());
    rig.settings.cloud = false;
    let off = CloudSheet {
        on: false,
        rows: Vec::new(),
        acts: Vec::new(),
    };
    assert_eq!(cloud_sheet(&mut rig), off);
    let none = rig.click(&VoiceClick::Service(CloudButton::Remove));
    assert_eq!(none, Vec::new(), "nothing to remove from");
    let out = rig.click(&VoiceClick::Cloud(true));
    assert!(out.iter().all(|d| matches!(d, Do::Set(..))), "{out:?}");
    apply(&mut rig, &out);
    let removed = vec![Do::RemoveKey(first().id.clone())];
    let want = (ServiceMark::Off, true, removed);
    assert_eq!(remove(&mut rig, &first().shown()), want);
}

#[test]
fn a_hidden_run_lists_no_row_and_removes_nothing() {
    let mut rig = keyed_groq();
    rig.hidden = true;
    assert_eq!(cloud_sheet(&mut rig).rows, Vec::new());
    let out = rig.click(&VoiceClick::Service(CloudButton::Remove));
    assert_eq!(out, Vec::new());
}

#[test]
fn the_key_page_keeps_its_own_remove_key() {
    let mut rig = keyed_groq();
    let out = act(&mut rig, &first().shown(), CloudButton::Lines);
    assert!(
        out.iter().all(|d| !matches!(d, Do::RemoveKey(_))),
        "{out:?}"
    );
    let out = rig.click(&VoiceClick::Service(CloudButton::Remove));
    assert_eq!(out, [Do::RemoveKey(first().id.clone())]);
}

#[test]
fn switching_a_service_off_keeps_its_key_so_on_again_finds_it_ready() {
    let mut rig = on_cloud("");
    put(&mut rig, "services", &entry(first(), ""));
    put(
        &mut rig,
        "picks",
        &format!(r#"en = "{}""#, pick(first(), 0)),
    );
    rig.keyed.insert(first().id.clone());
    let out = act(&mut rig, &first().shown(), CloudButton::Off);
    let parks = matches!(&out[..], [Do::Set(PARKED, _), Do::Set(SERVICES, _)]);
    assert!(parks, "parked first, so a stop between keeps it: {out:?}");
    apply(&mut rig, &out);
    assert!(rig.settings.services.is_empty(), "Groq is Off");
    assert!(rig.settings.parked.contains_key("groq"), "its entry waits");
    assert!(rig.settings.picks.contains_key("en"), "its pick waits");
    rig.click(&VoiceClick::Service(CloudButton::On)); // Groq stays ticked.
    let out = rig.click(&VoiceClick::Confirm(true));
    let back = matches!(&out[..], [Do::Set(SERVICES, _), Do::Set(PARKED, _), ..]);
    assert!(back, "On first, so a stop between keeps it: {out:?}");
    apply(&mut rig, &out);
    assert!(rig.settings.parked.is_empty());
    assert_eq!(lines_sheet(&mut rig).mark, ServiceMark::Ready);
}
