//! A cloud dictation's count (ADR-0047): kept under its service and listed model, on no device of this PC; a typed
//! model's id is never kept, a pick of a service the table lacks counts as any other model, and the run's table names
//! its own rows.

use super::*;
use crate::cloud::pick_text;
use crate::services::tests::shipped::{first, model as listed, pick};
use crate::services::tests::{ROUTER, with_rows};

/// The counts' one bucket after one dictation by `pick` on day 14.
fn only(pick: &str) -> (Bucket, Counts) {
    let mut counts = Counts::default();
    counts.add(day(14), &count(pick, 4_000, 700, None));
    let bucket = counts.rows().keys().next().unwrap().clone();
    assert_eq!(counts.rows().len(), 1);
    (bucket, counts)
}

#[test]
fn a_cloud_dictation_counts_under_its_service_and_model() {
    let (bucket, counts) = only(&pick(first(), 0));
    let model = Model::Cloud {
        service: first().id.clone(),
        model: Some(listed(first(), 0).id.clone()),
    };
    assert_eq!(bucket, self::bucket(14, model, Device::Cloud));
    assert_eq!(counts.rows()[&bucket].wait_ms, 700);
}

#[test]
fn a_typed_models_id_is_never_kept() {
    let (bucket, counts) = only("custom:my-secret-model");
    let model = Model::Cloud {
        service: "custom".into(),
        model: None,
    };
    assert_eq!(bucket, self::bucket(14, model, Device::Cloud));
    assert!(!format!("{counts:?}").contains("secret"));
}

#[test]
fn a_service_the_table_lacks_counts_as_any_other_model() {
    let (bucket, _) = only("nowhere:whisper");
    assert_eq!(bucket, self::bucket(14, Model::Other, Device::Cpu));
}

#[test]
fn a_service_only_the_runs_table_knows_counts_under_it() {
    let held = with_rows(ROUTER);
    let picked = pick_text("router-test", "openai/whisper-large-v3-turbo");
    let mut counts = Counts::default();
    counts.add_in(day(14), &count(&picked, 1_000, 100, None), Some(&held));
    let bucket = counts.rows().keys().next().unwrap();
    let model = Model::Cloud {
        service: "router-test".into(),
        model: Some("openai/whisper-large-v3-turbo".into()),
    };
    assert_eq!(*bucket, self::bucket(14, model, Device::Cloud));
    assert_eq!(
        Model::of(&picked, None),
        Model::Other,
        "no table names no service"
    );
}
