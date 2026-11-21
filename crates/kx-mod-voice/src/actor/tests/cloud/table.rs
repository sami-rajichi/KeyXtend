//! The run's services table reaches the counts (L364): a dictation through a row only that table knows is counted under
//! its service and listed model, never as another file.

use super::*;
use crate::cloud::pick_text;
use crate::services::Services;
use crate::services::tests::{ROUTER, with_rows};

/// The test row's id and its listed model, as `ROUTER` writes them.
const ROW: &str = "router-test";
const LISTED: &str = "openai/whisper-large-v3-turbo";

/// The shipped table with the test row, kept for the run.
fn held() -> &'static Services {
    Box::leak(Box::new(with_rows(ROUTER)))
}

/// The test row picked for English and switched on with its terms confirmed; its key waits for the table, as the
/// start sweeps the keys of rows the shipped table lacks.
fn router(_: &Fakes, host: &mut FakeHost) {
    let row = held().get(ROW).unwrap();
    host.put("cloud", true.into());
    host.put("services", table(&entry(row, "")));
    host.put(
        "picks",
        table(&format!("en = \"{}\"", pick_text(ROW, LISTED))),
    );
}

#[test]
fn a_dictation_through_a_row_only_the_runs_table_knows_is_counted_under_it() {
    let mut rig = Rig::build(router);
    rig.actor.held.table = Some(held());
    let keys = Keys::new(Arc::new(rig.fake.shelf.clone()), false);
    keys.set(ROW, secret(&key())).unwrap();
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    assert_eq!(
        rig.typed_texts(),
        ["hello world"],
        "it went through the row"
    );
    let opened = Store::open(Arc::new(rig.fake.shelf.clone()), MAX_BLOB_BYTES).unwrap();
    let bucket = opened.counts.rows().keys().next().unwrap().clone();
    let want = Model::Cloud {
        service: ROW.to_owned(),
        model: Some(LISTED.to_owned()),
    };
    assert_eq!((bucket.model, bucket.device), (want, Device::Cloud));
}
