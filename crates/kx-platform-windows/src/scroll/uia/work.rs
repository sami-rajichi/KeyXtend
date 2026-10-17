//! The UI Automation thread's body: the client, the walk up to what scrolls, its reads and its line steps. Every job
//! counts as done before it answers, so a caller that got its answer finds the thread free.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use kx_module_api::ports::Dir;
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement,
    IUIAutomationScrollPattern, ScrollAmount, ScrollAmount_NoAmount, ScrollAmount_SmallDecrement,
    ScrollAmount_SmallIncrement, UIA_ScrollPatternId,
};
use windows::core::{BOOL, Interface};

use super::{Job, Read, Seek};
use crate::com::Com;

/// The pattern of what scrolls at a place, held for the steps that follow a read there.
pub(super) type Held = Option<(Seek, IUIAutomationScrollPattern)>;

/// Makes the client, then runs each job in turn until the handle drops; a job past its time is dropped unstarted.
pub(super) fn run(inbox: &Receiver<Job>, pending: &AtomicUsize, read_ms: u32) {
    let Ok(_com) = Com::start() else {
        return tracing::warn!("no UI Automation: COM did not start");
    };
    let Some(auto) = client(read_ms) else {
        return tracing::warn!("no UI Automation: its client did not start");
    };
    let mut held: Held = None;
    for job in inbox {
        match job {
            Job::Read { seek, by, reply } => {
                let read = (Instant::now() <= by).then(|| look(&auto, seek, by, &mut held));
                pending.fetch_sub(1, Ordering::SeqCst);
                let _ = reply.send(read.flatten());
            }
            Job::Steps {
                seek,
                dir,
                count,
                by,
            } => {
                step(&auto, (seek, dir, count), by, &mut held);
                pending.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}

/// The system's client, with its connection and call timeouts at `read_ms`.
#[allow(unsafe_code, reason = "Makes and sets up the UI Automation client.")]
pub(super) fn client(read_ms: u32) -> Option<IUIAutomation> {
    // SAFETY: makes the system's UI Automation client on this thread, where COM runs.
    let auto: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }.ok()?;
    if let Ok(timed) = auto.cast::<IUIAutomation2>() {
        // SAFETY: plain settings on our own client; a refusal keeps Windows' defaults.
        unsafe {
            let _ = timed.SetConnectionTimeout(read_ms);
            let _ = timed.SetTransactionTimeout(read_ms);
        }
    }
    Some(auto)
}

/// The element `seek` names; none when UI Automation cannot say.
#[allow(unsafe_code, reason = "A UI Automation lookup.")]
fn element(auto: &IUIAutomation, seek: Seek) -> Option<IUIAutomationElement> {
    let found = match seek {
        // SAFETY: a plain lookup on our live client; a failure gives an error.
        Seek::At(p) => unsafe { auto.ElementFromPoint(POINT { x: p.x, y: p.y }) },
        // SAFETY: as above, for a test's own window.
        #[cfg(test)]
        Seek::Window(raw) => unsafe { auto.ElementFromHandle(crate::window::from_raw(raw)) },
    };
    found.ok()
}

/// A walk up the tree that could not finish: it ran past its time, or UI Automation failed a call.
struct Unfinished;

/// The first element from `el` up that scrolls either way, with its pattern; none when the walk reached the root.
#[allow(unsafe_code, reason = "Walks up the UI Automation tree.")]
fn scroller(
    auto: &IUIAutomation,
    mut el: IUIAutomationElement,
    by: Instant,
) -> Result<Option<(IUIAutomationElement, IUIAutomationScrollPattern)>, Unfinished> {
    // SAFETY: a plain call on our live client.
    let walker = unsafe { auto.ControlViewWalker() }.map_err(|_| Unfinished)?;
    while Instant::now() <= by {
        if let Some(pattern) = scrolls(&el) {
            return Ok(Some((el, pattern)));
        }
        // SAFETY: a plain call on a live element.
        el = match unsafe { walker.GetParentElement(&el) } {
            Ok(parent) => parent,
            // The root has no parent: the call succeeds and gives none.
            Err(e) if e.code().is_ok() => return Ok(None),
            Err(_) => return Err(Unfinished),
        };
    }
    Err(Unfinished)
}

/// The scroll pattern of `el` when it can scroll either way.
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
fn scrolls(el: &IUIAutomationElement) -> Option<IUIAutomationScrollPattern> {
    let yes = |b: windows::core::Result<BOOL>| b.is_ok_and(BOOL::as_bool);
    // SAFETY: plain pattern and property reads on a live element; a failure reads as no.
    unsafe {
        let pattern: IUIAutomationScrollPattern =
            el.GetCurrentPatternAs(UIA_ScrollPatternId).ok()?;
        let either = yes(pattern.CurrentHorizontallyScrollable())
            || yes(pattern.CurrentVerticallyScrollable());
        either.then_some(pattern)
    }
}

/// The box and percents of what scrolls at `seek`, whose pattern it holds for steps; an empty read when nothing does up
/// to the root, and none, holding what it held, when UI Automation could not look there or finish the walk in time.
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub(super) fn look(auto: &IUIAutomation, seek: Seek, by: Instant, held: &mut Held) -> Option<Read> {
    let found = scroller(auto, element(auto, seek)?, by).ok()?;
    *held = found.as_ref().map(|(_, pattern)| (seek, pattern.clone()));
    let Some((el, pattern)) = found else {
        return Some(Read {
            area: None,
            pair: None,
        });
    };
    // SAFETY: plain property reads on a live element and its pattern; a failure reads as none.
    unsafe {
        let edges = el.CurrentBoundingRectangle().ok();
        let h = pattern.CurrentHorizontalScrollPercent().ok();
        let v = pattern.CurrentVerticalScrollPercent().ok();
        Some(Read {
            area: edges.map(|r| [r.left, r.top, r.right, r.bottom]),
            pair: h.zip(v),
        })
    }
}

/// Takes `count` small steps towards `dir` on what scrolls at `seek`, by the held pattern there if any; past `by` it
/// looks nothing up, and a failed step stops them.
#[allow(unsafe_code, reason = "UI Automation scroll calls.")]
pub(super) fn step(
    auto: &IUIAutomation,
    (seek, dir, count): (Seek, Dir, u16),
    by: Instant,
    held: &mut Held,
) {
    if Instant::now() > by {
        return;
    }
    if held.as_ref().is_none_or(|(at, _)| *at != seek) {
        let found = element(auto, seek).and_then(|el| scroller(auto, el, by).ok().flatten());
        *held = found.map(|(_, pattern)| (seek, pattern));
    }
    let Some(pattern) = held.as_ref().map(|(_, p)| p.clone()) else {
        return;
    };
    let (h, v) = amounts(dir);
    for _ in 0..count {
        if Instant::now() > by {
            break;
        }
        // SAFETY: a plain call on a live pattern; a failure stops the steps and drops it.
        if unsafe { pattern.Scroll(h, v) }.is_err() {
            *held = None;
            break;
        }
    }
}

/// `ScrollPattern`'s small steps towards `dir`: sideways, then down.
pub(super) fn amounts(dir: Dir) -> (ScrollAmount, ScrollAmount) {
    let (none, more, less) = (
        ScrollAmount_NoAmount,
        ScrollAmount_SmallIncrement,
        ScrollAmount_SmallDecrement,
    );
    match dir {
        Dir::Down => (none, more),
        Dir::Up => (none, less),
        Dir::Right => (more, none),
        Dir::Left => (less, none),
    }
}
