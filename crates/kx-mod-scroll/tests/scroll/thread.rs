//! The module's actor thread over the bus: it says it is live, answers a late ask, opens the run on first need, ticks
//! while scrolling, and leaves no outline or run behind at a stop or a panic.

use crate::logs::{MESSAGE, Planned, lines_of};
use crate::rig::{APP, T0};
use kx_mod_scroll::{MANIFEST, ScrollModule, ScrollSettings};
use kx_module_api::ports::{
    Dir, Reach, ScrollAimed, ScrollAsk, ScrollChanged, ScrollConfig, ScrollError, ScrollPeek,
    ScrollRun, ScrollTap, ScrollTarget, ScrollTargetKey,
};
use kx_module_api::{Bus, Event, Module, ModuleCx, ServiceKey, Subscription, parse_as};
use kx_platform_fake::{FakeClock, FakeScrollTarget};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use tracing::Level;

/// The longest a threaded test waits for the actor; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);

/// A target whose open panics, as a platform bug would.
struct Broken;

impl ScrollTarget for Broken {
    fn open(&self, _: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError> {
        std::panic::panic_any(Planned)
    }
}

/// A host granting `Scroll`, holding `target`, on a clock the test moves.
fn host(target: Arc<dyn ScrollTarget>, clock: &FakeClock) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = MANIFEST.capabilities.to_vec();
    let stored: Stored = (ScrollTargetKey::CAPABILITY, Box::new(target));
    host.services.insert(ScrollTargetKey::ID, stored);
    host.clock = Arc::new(clock.clone());
    host
}

/// A running module on `target`, and an unlimited bus to talk to it.
struct Live {
    host: FakeHost,
    clock: FakeClock,
    module: ScrollModule,
    bus: Bus,
}

impl Live {
    /// Starts the module on `target` after `listen` subscribes on its bus.
    fn start<T>(target: Arc<dyn ScrollTarget>, listen: impl FnOnce(&Bus) -> T) -> (Self, T) {
        let clock = FakeClock::new(T0);
        let mut host = host(target, &clock);
        let bus = Bus::new(Arc::clone(&host.core));
        let heard = listen(&bus);
        let mut module = ScrollModule::default();
        module.start(&mut ModuleCx::new(&mut host)).unwrap();
        let live = Self {
            host,
            clock,
            module,
            bus,
        };
        (live, heard)
    }

    /// Publishes `event` as the keyboard would.
    fn send<E: Event>(&self, event: &E) {
        self.bus.publish(event).unwrap();
    }

    /// Stops the module as the kernel does: its handlers leave the bus first.
    fn stop(&mut self) {
        self.host.held.clear();
        self.module.stop();
    }
}

/// Sends every `E` published on `bus` to the returned channel.
fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let sub = bus
        .subscribe(move |e: &E| {
            let _ = tx.send(e.clone());
        })
        .unwrap();
    (sub, rx)
}

/// The live state: scrolling `dir` slow, or still.
fn live(dir: Option<Dir>) -> ScrollChanged {
    ScrollChanged {
        live: true,
        dir,
        fast: false,
    }
}

#[test]
fn the_thread_says_it_is_live_and_a_late_ask_hears_it_again() {
    let target = FakeScrollTarget::new();
    let (mut run, (_ear, told)) = Live::start(Arc::new(target), hear::<ScrollChanged>);
    assert_eq!(told.recv_timeout(WAIT), Ok(live(None)));
    let (_late, late) = hear::<ScrollChanged>(&run.bus);
    run.send(&ScrollAsk);
    assert_eq!(late.recv_timeout(WAIT), Ok(live(None)), "a late keyboard");
    run.stop();
    assert_eq!(late.try_recv(), Ok(ScrollChanged::OFF), "the stop joined");
}

#[test]
fn a_peek_opens_the_run_and_the_stop_closes_it_and_hides_the_outline() {
    let target = FakeScrollTarget::new();
    target.push_aim(Ok(APP));
    let (mut run, (_ear, aimed)) = Live::start(Arc::new(target.clone()), hear::<ScrollAimed>);
    run.send(&ScrollPeek { on: true });
    assert_eq!(aimed.recv_timeout(WAIT), Ok(ScrollAimed { area: APP.area }));
    assert_eq!((target.running(), target.opens()), (true, 1));
    run.stop();
    assert_eq!((target.running(), target.closes()), (false, 1));
    assert_eq!(aimed.try_recv(), Ok(ScrollAimed { area: None }));
}

#[test]
fn a_scroll_ticks_on_the_thread_until_the_end() {
    let target = FakeScrollTarget::new();
    target.push_aim(Ok(APP));
    let (mut run, (_ear, told)) = Live::start(Arc::new(target.clone()), hear::<ScrollChanged>);
    assert_eq!(told.recv_timeout(WAIT), Ok(live(None)));
    run.send(&ScrollTap { dir: Dir::Down });
    assert_eq!(told.recv_timeout(WAIT), Ok(live(Some(Dir::Down))));
    target.set_reach(Dir::Down, Reach::End);
    let settings: ScrollSettings = parse_as(&run.host.settings).unwrap();
    run.clock
        .advance(Duration::from_millis(settings.check_ms.into()));
    assert_eq!(told.recv_timeout(WAIT), Ok(live(None)), "a tick checked");
    run.stop();
    assert!(!target.running());
}

#[test]
fn an_actor_that_panics_says_at_once_the_module_is_off_and_is_logged_once() {
    let lines = lines_of(|| {
        let listen = |bus: &Bus| (hear::<ScrollChanged>(bus), hear::<ScrollAimed>(bus));
        let (mut run, ((_e1, told), (_e2, aimed))) = Live::start(Arc::new(Broken), listen);
        assert_eq!(told.recv_timeout(WAIT), Ok(live(None)));
        run.send(&ScrollPeek { on: true });
        let off = Ok(ScrollChanged::OFF);
        assert_eq!(told.recv_timeout(WAIT), off, "the pad dims now");
        assert_eq!(aimed.recv_timeout(WAIT), Ok(ScrollAimed { area: None }));
        run.stop();
        assert!(told.try_recv().is_err(), "said once");
    });
    let warned: Vec<_> = lines.iter().filter(|(l, _)| *l == Level::WARN).collect();
    assert_eq!(warned.len(), 1, "{lines:?}");
    let fields: Vec<_> = warned[0].1.keys().copied().collect();
    assert_eq!(fields, [MESSAGE], "the line holds no point or box");
}
