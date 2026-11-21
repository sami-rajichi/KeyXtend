//! A Cloud page row's own Remove key (owner, 2026-11-20): it shows while a key is sealed and its key page is out of
//! reach, as the service or cloud services are off, whose rows show dimmed and take no tick; a hidden run lists none.

use kx_module_api::ports::{CloudButton, ServiceMark, ServiceName, VoiceClick};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::rows::{every_row, named, row_of, tick};
use super::{act, apply, card, cloud_sheet, on_cloud, put};
use crate::services::{self, tests::ROUTER};

/// Groq on with its key sealed, on the Cloud page.
fn keyed_groq() -> Rig {
    let mut rig = on_cloud("");
    put(
        &mut rig,
        "services",
        "groq = { on = 2026-11-01, terms = 1 }",
    );
    rig.keyed.insert("groq".into());
    rig
}

/// The Remove key click of row `name`, on the page that lists it, and whether the row showed one.
fn remove(rig: &mut Rig, name: &ServiceName) -> (bool, Vec<Do>) {
    let at = row_of(rig, name);
    let shown = cloud_sheet(rig).rows[usize::from(at)].remove;
    (shown, rig.click(&VoiceClick::RemoveKey(at)))
}

#[test]
fn switching_a_service_off_leaves_its_row_a_remove_key_that_deletes_it() {
    let mut rig = keyed_groq();
    let out = act(&mut rig, &named("Groq"), CloudButton::Off);
    apply(&mut rig, &out);
    let at = usize::from(row_of(&mut rig, &named("Groq")));
    assert_eq!(cloud_sheet(&mut rig).rows[at].mark, ServiceMark::Off);
    let removed = vec![Do::RemoveKey("groq".into())];
    assert_eq!(remove(&mut rig, &named("Groq")), (true, removed));
}

#[test]
fn declining_newer_terms_leaves_the_remove_key_without_agreeing_to_them() {
    let mut rig = on_cloud(&ROUTER.replace("terms = 1", "terms = 2"));
    let on = r#""router-test" = { on = 2026-11-01, terms = 1 }"#;
    put(&mut rig, "services", on);
    rig.keyed.insert("router-test".into());
    tick(&mut rig, &named("Router Test"));
    rig.click(&VoiceClick::Service(CloudButton::On));
    assert_eq!(card(&mut rig).on, None, "it asks again");
    let out = rig.click(&VoiceClick::Confirm(false));
    apply(&mut rig, &out);
    let removed = vec![Do::RemoveKey("router-test".into())];
    assert_eq!(remove(&mut rig, &named("Router Test")), (true, removed));
    assert_eq!(
        rig.settings.services.len(),
        0,
        "the newer terms stay unconfirmed"
    );
}

#[test]
fn cloud_off_lists_the_rows_dimmed_untickable_each_sealed_one_with_its_remove_key() {
    let mut rig = keyed_groq();
    rig.settings.cloud = false;
    let rows = every_row(&mut rig);
    assert!(rows.len() > 1, "{rows:?}");
    for row in &rows {
        let groq = row.name == named("Groq");
        assert_eq!(
            (row.mark, row.remove, row.tick),
            (ServiceMark::Off, groq, false)
        );
    }
    let at = row_of(&mut rig, &named("Groq"));
    rig.click(&VoiceClick::Row(at));
    let sheet = cloud_sheet(&mut rig);
    assert!(
        !sheet.rows[usize::from(at)].tick && sheet.acts.is_empty(),
        "no tick"
    );
    let other = rows.iter().find(|r| r.name != named("Groq")).unwrap();
    assert_eq!(remove(&mut rig, &other.name), (false, Vec::new()));
    let removed = vec![Do::RemoveKey("groq".into())];
    assert_eq!(remove(&mut rig, &named("Groq")), (true, removed));
}

#[test]
fn a_hidden_run_lists_no_row_and_removes_nothing() {
    let mut rig = keyed_groq();
    rig.hidden = true;
    assert_eq!(cloud_sheet(&mut rig).rows, Vec::new());
    assert_eq!(rig.click(&VoiceClick::RemoveKey(0)), Vec::new());
}

#[test]
fn a_remove_key_on_rows_that_moved_since_is_dropped() {
    let mut rig = keyed_groq();
    rig.keyed
        .extend(services::table().unwrap().offered().map(|s| s.id.clone()));
    rig.settings.cloud = false;
    let at = row_of(&mut rig, &named("Groq"));
    let seen = rig.round;
    rig.click(&VoiceClick::Page { back: false });
    rig.view();
    assert_eq!(rig.click_at(seen, &VoiceClick::RemoveKey(at)), Vec::new());
}

#[test]
fn a_service_on_keeps_remove_key_on_its_key_page_not_on_its_row() {
    let mut rig = keyed_groq();
    assert_eq!(remove(&mut rig, &named("Groq")), (false, Vec::new()));
    let out = act(&mut rig, &named("Groq"), CloudButton::Lines);
    assert!(
        out.iter().all(|d| !matches!(d, Do::RemoveKey(_))),
        "{out:?}"
    );
    let out = rig.click(&VoiceClick::Service(CloudButton::Remove));
    assert_eq!(out, [Do::RemoveKey("groq".into())]);
}
