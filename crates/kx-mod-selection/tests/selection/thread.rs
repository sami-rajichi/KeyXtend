//! The module on its own thread: a selection shows and Copy goes through a keyboard, a held browser open holds only
//! its own thread and never the stop, and a panic hides the strip, lets presses go and tells the user.

use std::any::Any;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Once};
use std::thread;

use kx_mod_selection::{MANIFEST, TRANSLATE_OK_KEY};
use kx_module_api::ports::{
    ChipAct, ChipsView, PressesAsk, SelChip, SelConfig, SelError, SelKind, SelRun, SelSpot,
    SelectionWatch,
};
use kx_module_api::{Bus, Redacted, keys};
use kx_platform_fake::FakeSelectionWatch;
use toml::Value;

use crate::rig::{Fakes, Heard, Said, WAIT, host_of, keyboard, module, sel, start, stop};

#[test]
fn a_selection_shows_and_copy_goes_through_the_keyboard_on_the_actors_thread() {
    let (fakes, mut module) = (Fakes::new(), module());
    let mut host = fakes.host(MANIFEST.capabilities);
    let (heard, _keys) = (Heard::listen(&host), keyboard(&host));
    let bus = Bus::new(Arc::clone(&host.core));
    start(&mut module, &mut host).unwrap();
    fakes.watch.focus(sel(SelKind::Editable));
    let round = heard.shown();
    bus.publish(&ChipAct {
        round,
        chip: SelChip::Copy,
    })
    .unwrap();
    let ticked = |s| match s {
        Said::View(ChipsView::Shown { acted, .. }) => acted,
        _ => None,
    };
    assert_eq!(heard.wait(ticked), SelChip::Copy);
    stop(&mut module, &mut host);
}

#[test]
fn a_held_open_holds_only_its_own_thread_and_the_stop_never_waits_on_it() {
    let (fakes, mut module) = (Fakes::new(), module());
    let mut host = fakes.host(MANIFEST.capabilities);
    host.put(TRANSLATE_OK_KEY, Value::Boolean(true));
    let heard = Heard::listen(&host);
    let bus = Bus::new(Arc::clone(&host.core));
    fakes.web.hold(true);
    fakes.watch.set_text("hello");
    start(&mut module, &mut host).unwrap();
    fakes.watch.focus(sel(SelKind::ReadOnly));
    let round = heard.shown();
    let chip = SelChip::Translate;
    bus.publish(&ChipAct { round, chip }).unwrap();
    assert!(
        fakes.web.wait_opened(1, WAIT),
        "the open waits in the shell"
    );
    bus.publish(&PressesAsk).unwrap();
    heard.wait(|s| (s == Said::Presses(true)).then_some(()));
    let stopped = thread::scope(|s| {
        let (done, stopping) = mpsc::channel();
        s.spawn(move || {
            stop(&mut module, &mut host);
            let _ = done.send(());
        });
        let stopped = stopping.recv_timeout(WAIT);
        fakes.web.hold(false);
        stopped
    });
    assert_eq!(stopped, Ok(()), "the stop waited on the open");
    assert!(fakes.web.wait_left(1, WAIT));
}

/// The payload of a panic the test plans; the panic hook prints nothing for it.
struct Planned;

/// Hushes planned panics for the whole test binary, once.
fn hush() {
    static HUSH: Once = Once::new();
    HUSH.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !<dyn Any + Send>::is::<Planned>(info.payload()) {
                usual(info);
            }
        }));
    });
}

/// A watch whose looks panic once armed, as a platform bug would.
struct Armed {
    real: FakeSelectionWatch,
    armed: Arc<AtomicBool>,
}

/// The armed watch's run.
struct ArmedRun {
    real: Box<dyn SelRun>,
    armed: Arc<AtomicBool>,
}

impl SelectionWatch for Armed {
    fn watch(
        &self,
        cfg: SelConfig,
        tell: Box<dyn Fn() + Send>,
    ) -> Result<Box<dyn SelRun>, SelError> {
        let real = self.real.watch(cfg, tell)?;
        let armed = Arc::clone(&self.armed);
        Ok(Box::new(ArmedRun { real, armed }))
    }
}

impl SelRun for ArmedRun {
    fn look(&mut self) -> Result<SelSpot, SelError> {
        if self.armed.load(Ordering::SeqCst) {
            panic::panic_any(Planned);
        }
        self.real.look()
    }

    fn still(&mut self, box_id: u64) -> bool {
        self.real.still(box_id)
    }

    fn read(&mut self, max_chars: u16) -> Result<Redacted<String>, SelError> {
        self.real.read(max_chars)
    }

    fn clip_holds(&self) -> bool {
        self.real.clip_holds()
    }
}

#[test]
fn a_panic_on_the_actors_thread_hides_the_strip_lets_presses_go_and_tells_the_user() {
    hush();
    let (fakes, armed) = (Fakes::new(), Arc::new(AtomicBool::new(false)));
    let watch = Armed {
        real: fakes.watch.clone(),
        armed: Arc::clone(&armed),
    };
    let mut host = host_of(Arc::new(watch), fakes.web.clone(), MANIFEST.capabilities);
    let heard = Heard::listen(&host);
    let mut module = module();
    start(&mut module, &mut host).unwrap();
    fakes.watch.focus(sel(SelKind::Editable));
    heard.shown();
    armed.store(true, Ordering::SeqCst);
    fakes.watch.change(sel(SelKind::Editable));
    heard.wait(|s| (s == Said::View(ChipsView::Hidden)).then_some(()));
    heard.wait(|s| (s == Said::Presses(false)).then_some(()));
    let failed = heard.wait(|s| match s {
        Said::Notice(key) => Some(key),
        _ => None,
    });
    assert_eq!(failed, keys::MODULE_FAILED);
    stop(&mut module, &mut host);
    assert!(!fakes.watch.watching(), "the run went with the thread");
}
