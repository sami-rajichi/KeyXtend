//! One piece's request, or a key's test, built once and sent in turn with the run's pace: a broken link, a stall or a
//! service down is tried again up to its tries, and a 429 waits once within the call's wait before it is the rate fault.
//! Every other fault is told at once, a reply is read within `MAX_NET_REPLY_BYTES`, and a dropped run's call stops.

use super::Kit;
use super::pace::Pace;
use super::reply;
use super::shape::{Piece, Request, Shape, Verdict};
use kx_ipc::{MAX_NET_REPLY_BYTES, NetCall, NetFault};
use kx_platform_net::{Call, Field, Head, Target};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use zeroize::Zeroizing;

/// Milliseconds in a second.
const MS_PER_S: u64 = kx_ipc::MS_PER_S as u64;

/// What a run's or a test's threads send with.
pub struct Sending {
    /// The adapter for the call's shape.
    pub shape: &'static dyn Shape,
    /// Where the requests go and how the key goes with them.
    pub call: NetCall<'static>,
    /// The model's id, when the run names one.
    pub model: Option<Cow<'static, str>>,
    /// The model's code for the language; empty for a test.
    pub lang: Cow<'static, str>,
    /// Tries per request before its fault is told.
    pub tries: u8,
    /// The run's pace, which every try waits its turn in; none for a key's test.
    pub pace: Option<Mutex<Pace>>,
    /// Set once the run or test is dropped: nothing more is sent.
    pub dropped: Arc<AtomicBool>,
}

/// The text of `wav`, one piece as a WAV file.
///
/// # Errors
/// The piece's `NetFault`; `Gone` once the run is dropped.
pub fn piece(kit: &Kit, s: &Sending, wav: &[u8]) -> Result<Zeroizing<String>, NetFault> {
    let piece = Piece {
        call: &s.call,
        model: s.model.as_deref(),
        lang: &s.lang,
        wav,
    };
    let request = s.shape.build(&piece)?;
    tries(kit, s, &request, |head, body| {
        s.shape.read(&s.call, head, body)
    })
}

/// Whether the call's key works.
///
/// # Errors
/// The test's `NetFault`; `Gone` once the test is dropped.
pub fn test(kit: &Kit, s: &Sending) -> Result<(), NetFault> {
    let request = s.shape.test(&s.call)?;
    tries(kit, s, &request, |head, body| {
        s.shape.tested(&s.call, head, body)
    })
}

/// Sends `request` until `read` says what was asked, or a fault is final.
fn tries<T>(
    kit: &Kit,
    s: &Sending,
    request: &Request,
    read: impl Fn(&Head, &[u8]) -> Verdict<T>,
) -> Result<T, NetFault> {
    let target = Target::parse(&s.call.url).map_err(NetFault::from)?;
    let (mut left, mut waited) = (s.tries, false);
    loop {
        turn(kit, s)?;
        match once(kit, s, &target, request, &read) {
            Verdict::Said(said) => return Ok(said),
            Verdict::Busy(Some(wait)) if !waited && wait <= u32::from(s.call.wait_s) => {
                waited = true;
                kit.clock.wait(u64::from(wait) * MS_PER_S);
            }
            Verdict::Busy(_) => return Err(NetFault::Rate),
            Verdict::Fault(fault) if again(fault) && left > 1 => left -= 1,
            Verdict::Fault(fault) => return Err(fault),
        }
    }
}

/// Waits the run's turn to send within `per_min`, so every try counts.
///
/// # Errors
/// `Gone` once the run or test is dropped, before or after the wait.
fn turn(kit: &Kit, s: &Sending) -> Result<(), NetFault> {
    let gone = || s.dropped.load(Ordering::SeqCst);
    if gone() {
        return Err(NetFault::Gone);
    }
    if let Some(pace) = &s.pace {
        let now = kit.clock.now_ms();
        let delay = pace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .turn(now);
        if delay > 0 {
            kit.clock.wait(delay);
        }
    }
    if gone() { Err(NetFault::Gone) } else { Ok(()) }
}

/// One request and what its reply says.
fn once<T>(
    kit: &Kit,
    s: &Sending,
    target: &Target,
    request: &Request,
    read: &impl Fn(&Head, &[u8]) -> Verdict<T>,
) -> Verdict<T> {
    let fields: Vec<Field<'_>> = request
        .fields
        .iter()
        .map(|(name, value)| Field { name, value })
        .collect();
    let call = Call {
        target,
        verb: request.verb,
        fields: &fields,
        body: &request.body,
        keep: request.keep,
        stall_s: s.call.wait_s,
        stop: &s.dropped,
    };
    let mut answer = match kit.http.call(&call) {
        Ok(answer) => answer,
        Err(e) => return Verdict::Fault(e.into()),
    };
    match reply::drain(answer.body.as_mut(), MAX_NET_REPLY_BYTES, &s.dropped) {
        Ok(body) => read(&answer.head, &body),
        Err(fault) => Verdict::Fault(fault),
    }
}

/// True for a fault another try may clear: a broken link, a stall or a service down.
fn again(fault: NetFault) -> bool {
    matches!(fault, NetFault::Network | NetFault::Timeout)
}
