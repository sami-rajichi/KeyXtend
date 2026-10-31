//! The module on its own thread: a copy goes in through a keyboard that answers from inside its handler, and every
//! stop or restart closes the panel and frees the chord.

use kx_mod_clipboard::{ClipboardModule, MANIFEST};
use kx_module_api::ports::{ClipAct, ClipNote, ClipOpen, ClipPart, ClipView};
use kx_module_api::{Bus, Module, Redacted};
use toml::Value;

use crate::rig::{Fakes, Heard, Said, WAIT, keyboard, start, wall};

/// The ui's rows per page in these tests.
const ROWS: u8 = 5;

/// The first view `heard` shows with `rows` rows.
fn view(heard: &Heard, rows: usize) -> ClipView {
    heard.wait(|s| match s {
        Said::View(v) if v.0.rows.len() == rows => Some(v),
        _ => None,
    })
}

/// Waits until `heard` says the panel is `open`.
fn panel(heard: &Heard, open: bool) {
    heard.wait(|s| matches!(s, Said::Panel(o) if o == open).then_some(()));
}

#[test]
fn a_ticked_copy_goes_in_through_the_keyboard_and_the_panel_closes() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    let mut host = fakes.host(MANIFEST.capabilities);
    wall(&mut host);
    host.put("paste_gap_ms", Value::Integer(20));
    let (heard, _keys) = (Heard::listen(&host), keyboard(&host));
    let bus = Bus::new(std::sync::Arc::clone(&host.core));
    start(&mut module, &mut host).unwrap();
    assert!(fakes.clip.wait_watching(WAIT));
    let hello = ClipPart::Text(Redacted::new("hello".to_owned()));
    fakes.clip.push(ClipNote::Copied {
        parts: vec![hello.clone()],
        app: None,
    });
    bus.publish(&ClipOpen).unwrap();
    panel(&heard, true);
    bus.publish(&ClipAct::Rows(ROWS)).unwrap();
    let round = view(&heard, 1).0.round;
    bus.publish(&ClipAct::Tick { round, row: 0 }).unwrap();
    bus.publish(&ClipAct::Insert).unwrap();
    panel(&heard, false);
    assert_eq!(fakes.writer.writes(), [vec![hello]]);
    module.stop();
}

#[test]
fn a_stop_or_a_restart_closes_the_panel_and_frees_the_chord() {
    let (fakes, mut module) = (Fakes::new(), ClipboardModule::default());
    let mut host = fakes.host(MANIFEST.capabilities);
    let heard = Heard::listen(&host);
    let bus = Bus::new(std::sync::Arc::clone(&host.core));
    start(&mut module, &mut host).unwrap();
    bus.publish(&ClipOpen).unwrap();
    panel(&heard, true);
    start(&mut module, &mut host).unwrap();
    let restart = heard.drain();
    let first = restart.get(..2);
    assert!(
        matches!(first, Some([Said::Panel(false), Said::Wanted(false)])),
        "{restart:?}"
    );
    bus.publish(&ClipOpen).unwrap();
    panel(&heard, true);
    module.stop();
    let stop = heard.drain();
    assert!(
        matches!(stop[..], [Said::Panel(false), Said::Wanted(false)]),
        "{stop:?}"
    );
}
