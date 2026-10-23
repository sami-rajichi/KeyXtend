//! The module's actor thread over the bus: it says it is live, opens the dialog off its own thread, never waits on an
//! open at a stop, tells nothing after it, and says at once that it is off when an open panics.

use crate::rig::{LIVE, failed, stopped};
use kx_mod_power::{MANIFEST, PowerModule};
use kx_module_api::ports::{
    PowerAsk, PowerAsked, PowerChanged, PowerDialog, PowerDialogKey, PowerError,
};
use kx_module_api::{Bus, Event, Module, ModuleCx, Notice, ServiceKey, Subscription};
use kx_platform_fake::FakePowerDialog;
use kx_test_support::host::{FakeHost, Stored};
use std::any::Any;
use std::panic;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, Once};
use std::thread;
use std::time::Duration;

/// The longest a threaded test waits for the module; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);
/// How long a held open waits for the test's answer: well past `WAIT`, so a test under load still answers first.
const GIVE_UP: Duration = WAIT.saturating_mul(6);

/// The payload of a panic a test plans; the panic hook prints nothing for it.
struct Planned;

/// A dialog whose open panics, as a platform bug would.
struct Broken;

impl PowerDialog for Broken {
    fn open(&self) -> Result<(), PowerError> {
        panic::panic_any(Planned)
    }
}

/// A dialog whose open says it began, then waits for the answer the test gives; it says when it is dropped. An open
/// gives up after `GIVE_UP`, so a module that waits on it fails its test instead of hanging it.
struct Held {
    began: Sender<()>,
    answer: Mutex<Receiver<Result<(), PowerError>>>,
    gone: Sender<()>,
}

impl PowerDialog for Held {
    fn open(&self) -> Result<(), PowerError> {
        let _ = self.began.send(());
        let answer = self.answer.lock().unwrap().recv_timeout(GIVE_UP);
        answer.unwrap_or(Err(PowerError::Failed))
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        let _ = self.gone.send(());
    }
}

/// The test's side of a `Held` dialog.
struct Hold {
    began: Receiver<()>,
    answer: Sender<Result<(), PowerError>>,
    gone: Receiver<()>,
}

/// A held dialog for the module, and the test's side of it.
fn held() -> (Arc<dyn PowerDialog>, Hold) {
    let (began, began_rx) = mpsc::channel();
    let (answer_tx, answer) = mpsc::channel();
    let (gone, gone_rx) = mpsc::channel();
    let answer = Mutex::new(answer);
    let dialog = Held {
        began,
        answer,
        gone,
    };
    let hold = Hold {
        began: began_rx,
        answer: answer_tx,
        gone: gone_rx,
    };
    (Arc::new(dialog), hold)
}

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

/// A running module on `dialog`, and an unlimited bus to talk to it.
struct Live {
    host: FakeHost,
    module: PowerModule,
    bus: Bus,
}

impl Live {
    /// Starts the module on `dialog` after `listen` subscribes on its bus.
    fn start<T>(dialog: Arc<dyn PowerDialog>, listen: impl FnOnce(&Bus) -> T) -> (Self, T) {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        let stored: Stored = (PowerDialogKey::CAPABILITY, Box::new(dialog));
        host.services.insert(PowerDialogKey::ID, stored);
        let bus = Bus::new(Arc::clone(&host.core));
        let heard = listen(&bus);
        let mut module = PowerModule::default();
        module.start(&mut ModuleCx::new(&mut host)).unwrap();
        (Self { host, module, bus }, heard)
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

/// Hears the module's state and its notices.
type Ears = (
    (Subscription, Receiver<PowerChanged>),
    (Subscription, Receiver<Notice>),
);

/// Hears the module's state and its notices on `bus`.
fn both(bus: &Bus) -> Ears {
    (hear::<PowerChanged>(bus), hear::<Notice>(bus))
}

#[test]
fn the_thread_says_it_is_live_and_a_late_ask_hears_it_again() {
    let dialog = Arc::new(FakePowerDialog::new());
    let (mut run, (_ear, told)) = Live::start(dialog, hear::<PowerChanged>);
    assert_eq!(told.recv_timeout(WAIT), Ok(LIVE));
    let (_late, late) = hear::<PowerChanged>(&run.bus);
    run.send(&PowerAsk);
    assert_eq!(late.recv_timeout(WAIT), Ok(LIVE), "a late keyboard");
    run.stop();
    assert_eq!(late.try_recv(), Ok(PowerChanged::OFF), "the stop joined");
}

#[test]
fn a_click_opens_the_dialog_off_the_actor_and_its_failure_reaches_the_user() {
    let (dialog, hold) = held();
    let (mut run, ((_e1, told), (_e2, notes))) = Live::start(dialog, both);
    assert_eq!(told.recv_timeout(WAIT), Ok(LIVE));
    run.send(&PowerAsked);
    assert_eq!(hold.began.recv_timeout(WAIT), Ok(()), "the dialog opens");
    run.send(&PowerAsk);
    assert_eq!(told.recv_timeout(WAIT), Ok(LIVE), "the actor still answers");
    run.send(&PowerAsked);
    assert_eq!(
        hold.began.recv_timeout(WAIT),
        Ok(()),
        "a second click opens too"
    );
    hold.answer.send(Ok(())).unwrap();
    hold.answer.send(Err(PowerError::Unavailable)).unwrap();
    assert_eq!(notes.recv_timeout(WAIT), Ok(failed()));
    run.stop();
    assert_eq!(told.try_recv(), Ok(PowerChanged::OFF));
    assert!(notes.try_recv().is_err(), "told once");
}

#[test]
fn a_stop_never_waits_on_an_open_and_a_late_answer_tells_nothing() {
    let (dialog, hold) = held();
    let (mut run, ((_e1, told), (_e2, notes))) = Live::start(dialog, both);
    assert_eq!(told.recv_timeout(WAIT), Ok(LIVE));
    run.send(&PowerAsked);
    assert_eq!(hold.began.recv_timeout(WAIT), Ok(()), "the open waits");
    let stopped = thread::scope(|s| {
        let (done, stop) = mpsc::channel();
        s.spawn(move || {
            run.stop();
            drop(run);
            let _ = done.send(());
        });
        let stopped = stop.recv_timeout(WAIT);
        hold.answer.send(Err(PowerError::Failed)).unwrap();
        stopped
    });
    assert_eq!(stopped, Ok(()), "the stop waited on the open");
    assert_eq!(told.try_recv(), Ok(PowerChanged::OFF));
    assert_eq!(hold.gone.recv_timeout(WAIT), Ok(()), "the open ended");
    assert!(told.try_recv().is_err(), "no live after off");
    assert!(notes.try_recv().is_err(), "no notice after the stop");
}

#[test]
fn an_open_that_panics_says_at_once_the_module_is_off() {
    hush();
    let (mut run, ((_e1, told), (_e2, notes))) = Live::start(Arc::new(Broken), both);
    assert_eq!(told.recv_timeout(WAIT), Ok(LIVE));
    run.send(&PowerAsked);
    assert_eq!(
        told.recv_timeout(WAIT),
        Ok(PowerChanged::OFF),
        "the key dims now"
    );
    assert_eq!(
        notes.recv_timeout(WAIT),
        Ok(stopped()),
        "the user hears why"
    );
    run.stop();
    assert!(told.try_recv().is_err(), "said once");
    assert!(notes.try_recv().is_err(), "told once");
}
