//! Tests of `workertok`.

use windows::Win32::System::SystemServices::{
    SECURITY_MANDATORY_HIGH_RID, SECURITY_MANDATORY_LOW_RID, SECURITY_MANDATORY_SYSTEM_RID,
};

use super::*;

const HIGH: u32 = SECURITY_MANDATORY_HIGH_RID.cast_unsigned();
const LOW: u32 = SECURITY_MANDATORY_LOW_RID.cast_unsigned();
const SYSTEM: u32 = SECURITY_MANDATORY_SYSTEM_RID.cast_unsigned();

fn rights(ui_access: bool, level: u32) -> Rights {
    Rights { ui_access, level }
}

#[test]
fn a_product_without_ui_access_skips_the_check() {
    let worker = [rights(true, HIGH)];
    assert_eq!(verdict(rights(false, MEDIUM), &worker), Verdict::Skipped);
    assert_eq!(verdict(rights(false, MEDIUM), &[]), Verdict::Skipped);
}

#[test]
fn with_ui_access_every_worker_drops_it_and_stays_at_most_medium() {
    let ours = rights(true, MEDIUM);
    assert_eq!(verdict(ours, &[rights(false, MEDIUM)]), Verdict::Reduced);
    assert_eq!(verdict(ours, &[rights(false, LOW)]), Verdict::Reduced);
    assert_eq!(verdict(ours, &[rights(true, MEDIUM)]), Verdict::Broad);
    assert_eq!(verdict(ours, &[rights(false, HIGH)]), Verdict::Broad);
    assert_eq!(
        verdict(ours, &[rights(false, SYSTEM)]),
        Verdict::Broad,
        "System fails"
    );
    let mixed = [rights(false, MEDIUM), rights(true, MEDIUM)];
    assert_eq!(verdict(ours, &mixed), Verdict::Broad, "one is enough");
    assert_eq!(verdict(ours, &[]), Verdict::NoWorker);
}

#[test]
fn only_a_passing_verdict_passes_only_a_broad_one_fails_and_each_says_why() {
    assert!(Verdict::Reduced.ok() && !Verdict::Reduced.fails());
    assert!(Verdict::Broad.fails());
    for v in [Verdict::Skipped, Verdict::Broad, Verdict::NoWorker] {
        assert!(!v.ok(), "{v:?}");
        assert_eq!(v.fails(), v == Verdict::Broad, "{v:?}");
        assert!(!v.why().is_empty());
    }
    assert!(Verdict::Skipped.why().contains("uiAccess"));
}

#[test]
fn workers_are_the_ones_beside_the_product_in_any_letter_case() {
    let dir = Path::new(r"C:\Program Files\KeyXtend-dev\keyxtend");
    let found = [
        (
            4,
            r"C:\Program Files\KeyXtend-dev\keyxtend\keyxtend-worker.exe".to_string(),
        ),
        (
            5,
            r"c:\program files\keyxtend-dev\KEYXTEND\Keyxtend-Worker.EXE".to_string(),
        ),
        (6, r"D:\elsewhere\keyxtend-worker.exe".to_string()),
        (
            7,
            r"C:\Program Files\KeyXtend-dev\keyxtend\keyxtend.exe".to_string(),
        ),
    ];
    assert_eq!(beside(&found, dir), [4, 5]);
}

#[test]
fn each_workers_rights_are_logged_as_level_and_ui_access() {
    let line = json!({ "workers": [rights(false, MEDIUM)] });
    assert_eq!(line["workers"][0]["level"], MEDIUM);
    assert_eq!(line["workers"][0]["ui_access"], false);
}

/// Reads worker 4, and fails on worker 5 with `why`.
fn fails_on_five(why: &str) -> impl Fn(u32) -> Result<Rights, String> + '_ {
    move |pid| match pid {
        5 => Err(why.to_string()),
        _ => Ok(rights(false, MEDIUM)),
    }
}

#[test]
fn a_worker_that_ended_before_its_token_was_read_is_skipped() {
    let got = read_each(vec![4, 5], fails_on_five("OpenProcess 5: gone"), |_| false);
    assert_eq!(got, Ok(vec![rights(false, MEDIUM)]));
    let none = read_each(vec![5], fails_on_five("gone"), |_| false);
    assert_eq!(none, Ok(vec![]), "no worker left, as when none started");
}

#[test]
fn a_running_worker_that_cannot_be_read_fails_the_check() {
    let runs = |pid: u32| {
        assert_eq!(pid, 5, "only a failed read is looked up again");
        true
    };
    let got = read_each(vec![4, 5], fails_on_five("OpenProcess 5: denied"), runs);
    assert_eq!(got, Err("OpenProcess 5: denied".to_string()));
}

#[test]
fn the_products_rights_are_read_back_from_the_worker_part() {
    let part =
        json!({ "product": { "level": 8192, "ui_access": false }, "workers": [], "ok": false });
    assert_eq!(product_of(&part), Some(rights(false, MEDIUM)));
    let ours = json!({ PRODUCT: rights(true, MEDIUM), "ok": true });
    assert_eq!(product_of(&ours), Some(rights(true, MEDIUM)));
    let unread = json!({ "error": "OpenProcess 4: denied", "ok": false, "fails": true });
    assert_eq!(product_of(&unread), None, "an unread token");
}
