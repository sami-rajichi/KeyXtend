//! A late, dead or lying vault worker: a key derivation past the deadline is killed and `TimedOut`, a dead job is
//! `Failed` and started again after the backoff, a stale or wrong reply ends the worker, and no value reaches `Debug`
//! or the log.

use super::*;
use kx_ipc::IpcError;
use std::cell::RefCell;
use std::fmt::{self, Write as _};
use std::sync::Once;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// The exit code of a job the system ended, such as one that overflowed its stack.
const DIED: u32 = 0xC000_00FD;

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

#[test]
fn a_call_past_the_deadline_is_killed_and_timed_out() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(VaultAct::Silent);
    assert_eq!(call(&mut host, Kind::Keepass), Err(BoxError::TimedOut));
    let within = ms(CFG.deadline_ms);
    assert!(
        script
            .calls()
            .ends_with(&[Call::Reply { within }, Call::End])
    );
}

#[test]
fn a_job_that_dies_mid_call_fails_and_the_next_call_after_the_backoff_starts_another() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    // A stack overflow or the memory cap ends the job, which closes its output.
    script.act(VaultAct::Broken(IpcError::Closed));
    assert_eq!(call(&mut host, Kind::Keepass), Err(BoxError::Failed));
    let within = ms(CFG.deadline_ms);
    let tail = [Call::Reply { within }, Call::Code { within }, Call::End];
    assert!(script.calls().ends_with(&tail), "its exit is read");
    assert_eq!(
        call(&mut host, Kind::Check),
        Err(BoxError::Failed),
        "it rests"
    );
    assert_eq!(script.tries(), 1);
    clock.advance(ms(CFG.restart_ms));
    script.act(VaultAct::Answer(checked));
    assert_eq!(call(&mut host, Kind::Check), Ok(()));
    assert_eq!(script.spawns(), 2);
}

#[test]
fn a_job_that_died_between_calls_is_replaced_at_once() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(VaultAct::Answer(checked));
    call(&mut host, Kind::Check).unwrap();
    script.quit(DIED);
    script.act(VaultAct::Answer(checked));
    assert_eq!(call(&mut host, Kind::Check), Ok(()));
    assert_eq!(script.spawns(), 2);
}

#[test]
fn a_reply_to_the_ask_that_timed_out_is_dropped_with_the_worker() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    script.act(VaultAct::Silent);
    assert_eq!(call(&mut host, Kind::Open), Err(BoxError::TimedOut));
    clock.advance(ms(CFG.restart_ms));
    script.act(VaultAct::Raw(opened(1)));
    assert_eq!(
        call(&mut host, Kind::Open),
        Err(BoxError::Failed),
        "ask 1's"
    );
    assert!(script.calls().ends_with(&[Call::End]));
}

#[test]
fn a_reply_of_another_kind_ends_the_worker() {
    for answer in [sealed, padlocked, entries, opened, checked] {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        script.act(VaultAct::Answer(answer));
        let wrong = if answer(1) == checked(1) {
            Kind::Padlock
        } else {
            Kind::Check
        };
        assert_eq!(call(&mut host, wrong), Err(BoxError::Failed), "{wrong:?}");
        assert!(script.calls().ends_with(&[Call::End]), "{wrong:?}");
    }
}

#[test]
fn no_password_part_or_value_reaches_debug_or_the_log() {
    let (shown, logged) = logged(|| {
        let (script, clock) = (Script::default(), Hands::default());
        let mut host = host(&script, &clock);
        script.act(VaultAct::Answer(entries));
        let read = host.keepass(KDBX.to_vec(), secret(PASSWORD));
        script.act(VaultAct::Answer(opened));
        let open = host.open(BOX.to_vec(), secret(PASSWORD));
        script.act(VaultAct::Answer(padlocked));
        let _ = call(&mut host, Kind::Seal);
        // The wrong reply ended the worker; past its backoff the padlock ask goes out and is late.
        clock.advance(ms(CFG.restart_ms));
        script.act(VaultAct::Silent);
        let late = call(&mut host, Kind::Padlock);
        assert_eq!(late, Err(BoxError::TimedOut), "a padlock was asked");
        drop(host);
        format!("{read:?} {open:?} {:?}", script.calls())
    });
    assert!(
        logged.contains(VAULT_JOB) && logged.contains("Late"),
        "the log names the job and the late worker: {logged}"
    );
    for text in [shown, logged] {
        assert!(!text.contains(PASSWORD) && !text.contains(NOTE), "{text}");
        assert!(!text.contains("بنك") && !text.contains("PIN"), "{text}");
    }
}

thread_local! {
    /// The text logged on this thread while `logged` runs here, else nothing.
    static SEEN: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Installs the capture for the whole test binary, once.
static INSTALL: Once = Once::new();

/// What `f` gives and the text it logs on this thread. One capture serves the whole binary, so `tracing` never
/// caches a log line as unheard while another test thread starts its own.
fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    INSTALL.call_once(|| {
        // Only this file sets a capture in this test binary, so the first call installs it.
        let _ = tracing::subscriber::set_global_default(Lines);
        tracing::callsite::rebuild_interest_cache();
    });
    SEEN.with(|seen| *seen.borrow_mut() = Some(String::new()));
    let made = f();
    let text = SEEN.with(|seen| seen.borrow_mut().take());
    (made, text.unwrap_or_default())
}

/// Hands each event's fields, as text, to the thread that logs it.
struct Lines;

impl Subscriber for Lines {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        SEEN.with(|seen| {
            if let Some(text) = seen.borrow_mut().as_mut() {
                event.record(&mut Line(text));
                text.push('\n');
            }
        });
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

/// One event's fields, written as `name=value`.
struct Line<'a>(&'a mut String);

impl Visit for Line<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let _ = write!(self.0, "{}={value:?} ", field.name());
    }
}
