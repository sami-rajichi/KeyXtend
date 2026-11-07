//! A panic on the actor's thread: no `quit` runs, so `run_actor` alone says the module stopped, dims the fill key and
//! tells the user.

use super::*;
use kx_module_api::ports::LocalTime;
use std::sync::atomic::{AtomicBool, Ordering};

/// The payload of a panic the test plans.
struct Planned;

/// A calendar that panics once armed, as a platform bug would; before that it tells the time, so the start goes well.
struct Armed {
    armed: Arc<AtomicBool>,
    real: FakeCalendar,
}

impl Calendar for Armed {
    fn now(&self) -> LocalTime {
        if self.armed.load(Ordering::SeqCst) {
            std::panic::panic_any(Planned);
        }
        self.real.now()
    }
}

#[test]
fn a_panic_on_the_actors_thread_says_it_stopped_and_tells_the_user() {
    let sealed = FakeSealedStore::new();
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let armed = Arc::new(AtomicBool::new(false));
    let calendar: Arc<dyn Calendar> = Arc::new(Armed {
        armed: Arc::clone(&armed),
        real: FakeCalendar::new(Date::new(2026, 11, 2).unwrap(), 9, 30, PARIS),
    });
    put::<CalendarKey>(&mut host, calendar);
    let (to_lives, lives) = mpsc::channel();
    let ear = move |e: &FillLive| {
        let _ = to_lives.send(e.live);
    };
    let _ear = host.bus().subscribe(ear).unwrap();
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(lives.recv_timeout(WITHIN), Ok(true));
    armed.store(true, Ordering::SeqCst);
    // The ask wakes its loop, which reads the calendar before it waits again.
    host.bus().publish(&FillAsk).unwrap();
    let mut told = std::iter::from_fn(|| lives.recv_timeout(WITHIN).ok());
    assert!(told.any(|live| !live), "the fill key dims");
    module.stop();
    let keys_heard: Vec<&str> = ears.notices.lock().unwrap().iter().map(|n| n.key).collect();
    assert!(keys_heard.contains(&keys::MODULE_FAILED), "{keys_heard:?}");
}
