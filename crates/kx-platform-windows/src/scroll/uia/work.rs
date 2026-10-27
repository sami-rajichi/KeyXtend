//! The UI Automation thread's body: the client, the walk down a window to a point, the walk up to what scrolls along a
//! way's axis, its reads and its line steps. Every job counts as done before it answers, so a caller that got its
//! answer finds the thread free.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use kx_module_api::ports::Dir;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationScrollPattern, ScrollAmount,
    ScrollAmount_NoAmount, ScrollAmount_SmallDecrement, ScrollAmount_SmallIncrement,
    UIA_ScrollPatternId,
};
use windows::core::BOOL;

use super::down::{View, deepest};
use super::{DOWN_SHARE, Job, Read, Seek};
use crate::com::Com;
use crate::scroll::win::handle;
use crate::uia::client;

/// The pattern of what scrolls at a place along one axis (true for sideways), held for the steps that follow a read
/// there.
pub(super) type Held = Option<(Seek, bool, IUIAutomationScrollPattern)>;

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
            Job::Read {
                seek,
                way,
                by,
                reply,
            } => {
                let read = (Instant::now() <= by).then(|| look(&auto, (seek, way), by, &mut held));
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

/// The element `seek` names; none when UI Automation cannot say. In a window, it goes down from the window's element
/// to the deepest one holding the point within its share of the time left before `by` (`DOWN_SHARE`).
#[allow(unsafe_code, reason = "A UI Automation lookup.")]
fn element(auto: &IUIAutomation, seek: Seek, by: Instant) -> Option<IUIAutomationElement> {
    match seek {
        // SAFETY: a plain lookup on our live client; a failure gives an error.
        Seek::At(p) => unsafe { auto.ElementFromPoint(POINT { x: p.x, y: p.y }) }.ok(),
        Seek::Window(window, p) => {
            // SAFETY: as above, for the target's own window, which may lie under ours or be hidden in a test.
            let top = unsafe { auto.ElementFromHandle(handle(window)) }.ok()?;
            // SAFETY: a plain call on our live client; without a walker the window's own element stands.
            let Ok(walker) = (unsafe { auto.ControlViewWalker() }) else {
                return Some(top);
            };
            let now = Instant::now();
            let down_by = now.checked_add(by.saturating_duration_since(now) / DOWN_SHARE);
            let late = || Instant::now() > down_by.unwrap_or(by);
            Some(deepest(&View(&walker), top, p, late))
        }
    }
}

/// A walk up the tree that could not finish: it ran past its time, or UI Automation failed a call.
struct Unfinished;

/// The first element from `el` up that scrolls along the axis (`sideways` or not), with its pattern; none when the walk
/// reached the root.
#[allow(unsafe_code, reason = "Walks up the UI Automation tree.")]
fn scroller(
    auto: &IUIAutomation,
    mut el: IUIAutomationElement,
    sideways: bool,
    by: Instant,
) -> Result<Option<(IUIAutomationElement, IUIAutomationScrollPattern)>, Unfinished> {
    // SAFETY: a plain call on our live client.
    let walker = unsafe { auto.ControlViewWalker() }.map_err(|_| Unfinished)?;
    while Instant::now() <= by {
        if let Some(pattern) = scrolls(&el, sideways) {
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

/// The scroll pattern of `el` when it can scroll along the axis (`sideways` or not).
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
fn scrolls(el: &IUIAutomationElement, sideways: bool) -> Option<IUIAutomationScrollPattern> {
    let yes = |b: windows::core::Result<BOOL>| b.is_ok_and(BOOL::as_bool);
    // SAFETY: plain pattern and property reads on a live element; a failure reads as no.
    unsafe {
        let pattern: IUIAutomationScrollPattern =
            el.GetCurrentPatternAs(UIA_ScrollPatternId).ok()?;
        let along = if sideways {
            yes(pattern.CurrentHorizontallyScrollable())
        } else {
            yes(pattern.CurrentVerticallyScrollable())
        };
        along.then_some(pattern)
    }
}

/// The box and percents of the first that scrolls along `way`'s axis at `seek`, whose pattern it holds for steps; an
/// empty read when nothing does up to the root, and none, holding what it held, when UI Automation could not look there
/// or finish the walk in time.
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub(super) fn look(
    auto: &IUIAutomation,
    (seek, way): (Seek, Dir),
    by: Instant,
    held: &mut Held,
) -> Option<Read> {
    let sideways = way.sideways();
    let found = scroller(auto, element(auto, seek, by)?, sideways, by).ok()?;
    *held = found
        .as_ref()
        .map(|(_, pattern)| (seek, sideways, pattern.clone()));
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

/// Takes `count` small steps towards `dir` on the first that scrolls along its axis at `seek`, by the held pattern
/// there if any; past `by` it looks nothing up, and a failed step stops them.
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
    let sideways = dir.sideways();
    if held.as_ref().is_none_or(|h| (h.0, h.1) != (seek, sideways)) {
        let found = element(auto, seek, by).and_then(|el| scroller(auto, el, sideways, by).ok());
        *held = found
            .flatten()
            .map(|(_, pattern)| (seek, sideways, pattern));
    }
    let Some(pattern) = held.as_ref().map(|h| h.2.clone()) else {
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
