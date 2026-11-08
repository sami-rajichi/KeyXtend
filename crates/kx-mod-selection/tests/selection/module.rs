//! The start and the stop: the watch follows them with the settings' config and presses are wanted only while it runs;
//! the helper off or a refused watch leaves the module idle (plan edge case 16), and a missing gate fails the start.

use kx_mod_selection::{MANIFEST, SelSettings};
use kx_module_api::ports::{ChipAct, ChipsView, SelError, SelectionWatchKey, WebLinkKey};
use kx_module_api::{Capability, Event, EventError, ModuleError, ServiceError, ServiceKey};
use toml::Value;

use crate::rig::{Fakes, Heard, Said, module, start, stop};

#[test]
fn the_start_watches_by_the_settings_and_wants_presses_and_the_stop_lets_go() {
    let (fakes, mut module) = (Fakes::new(), module());
    let mut host = fakes.host(MANIFEST.capabilities);
    let heard = Heard::listen(&host);
    start(&mut module, &mut host).unwrap();
    let settings = SelSettings::from_defaults().unwrap();
    assert_eq!(fakes.watch.config(), Some(settings.sel_config()));
    assert!(fakes.watch.watching());
    heard.wait(|s| (s == Said::Presses(true)).then_some(()));
    stop(&mut module, &mut host);
    assert!(!fakes.watch.watching());
    let end = heard.drain();
    let tail = &end[end.len() - 2..];
    assert_eq!(tail, [Said::View(ChipsView::Hidden), Said::Presses(false)]);
}

#[test]
fn a_restart_lets_go_of_the_first_watch() {
    let (fakes, mut module) = (Fakes::new(), module());
    let mut host = fakes.host(MANIFEST.capabilities);
    start(&mut module, &mut host).unwrap();
    start(&mut module, &mut host).unwrap();
    let calls = fakes.watch.calls();
    assert_eq!((calls.watches, calls.stops), (2, 1));
    stop(&mut module, &mut host);
    assert!(!fakes.watch.watching());
}

#[test]
fn with_the_helper_off_nothing_is_watched_and_no_presses_are_wanted() {
    let (fakes, mut module) = (Fakes::new(), module());
    let mut host = fakes.host(MANIFEST.capabilities);
    host.put("on", Value::Boolean(false));
    let heard = Heard::listen(&host);
    start(&mut module, &mut host).unwrap();
    assert_eq!(fakes.watch.calls().watches, 0);
    assert_eq!(heard.drain(), [Said::View(ChipsView::Hidden)]);
    assert_eq!(host.lookups.get(), 0, "no service taken");
    stop(&mut module, &mut host);
    assert!(
        heard.drain().is_empty(),
        "the hook stays as the other askers want it"
    );
}

#[test]
fn a_refused_watch_leaves_the_module_idle() {
    let (fakes, mut module) = (Fakes::new(), module());
    fakes.watch.fail_watch(SelError::Unavailable);
    let mut host = fakes.host(MANIFEST.capabilities);
    let heard = Heard::listen(&host);
    start(&mut module, &mut host).unwrap();
    assert_eq!(heard.drain(), [Said::View(ChipsView::Hidden)]);
    assert!(!fakes.watch.watching());
    stop(&mut module, &mut host);
}

#[test]
fn each_missing_gate_fails_the_start_and_watches_nothing() {
    let want = |cap| match cap {
        Capability::ReadFocusedText => {
            ModuleError::Service(ServiceError::NotGranted(SelectionWatchKey::ID, cap))
        }
        Capability::OpenLink => ModuleError::Service(ServiceError::NotGranted(WebLinkKey::ID, cap)),
        _ => ModuleError::Event(EventError::NotGranted(ChipAct::NAME, cap)),
    };
    for &cap in MANIFEST.capabilities {
        let fakes = Fakes::new();
        let left: Vec<_> = MANIFEST
            .capabilities
            .iter()
            .copied()
            .filter(|c| *c != cap)
            .collect();
        let mut host = fakes.host(&left);
        let refused = start(&mut module(), &mut host);
        assert_eq!(refused, Err(want(cap)), "{cap:?}");
        assert_eq!(fakes.watch.calls().watches, 0, "{cap:?}");
    }
}
