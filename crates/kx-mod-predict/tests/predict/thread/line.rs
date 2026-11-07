//! The line over the bus: `AimedLine` reaches the thread, the lane shows its chips, a pick names the line, and a stop
//! leaves nothing to save.

use super::{Live, WAIT, hear, lane_of, lang};
use crate::rig::{DAY, blob, learner};
use kx_module_api::Redacted;
use kx_module_api::ports::{AimedLine, ChipPicked, FieldKind, PanelLine, TypeWord};
use kx_platform_fake::{FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use std::sync::Arc;

#[test]
fn the_thread_follows_an_aimed_search_and_a_pick_names_the_line() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let known = learner("en", &["hello", "help"], DAY);
    store.plant(&blob("en"), known.to_bytes());
    field.focus(FieldKind::Password);
    let listen = |bus: &kx_module_api::Bus| (hear(bus), hear::<TypeWord>(bus));
    let (mut run, ((_l, lanes), (_w, words))) =
        Live::start(&field, Arc::new(FakeWordPacks::new()), &store, listen);
    run.send(&lang("en"));
    run.send(&AimedLine {
        line: PanelLine::Search(Redacted::new("hel".to_owned())),
    });
    let shown = lane_of(&lanes, &["hello", "help"]);
    run.send(&ChipPicked {
        round: shown.round,
        index: 0,
    });
    let word = words.recv_timeout(WAIT).unwrap();
    assert!(word.line);
    assert_eq!((word.erase, word.text.expose().as_str()), (0, "lo "));
    let saves = store.saves();
    run.stop();
    assert_eq!(store.saves(), saves, "nothing on a line saves");
}
