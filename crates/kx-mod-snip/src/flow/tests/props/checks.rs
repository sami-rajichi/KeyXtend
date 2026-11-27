//! What each step must leave: a free close asks exactly when nothing was kept, no strip key acts while a handle is
//! held, a region's part keeps to its frame, and over frozen screens three Escs at most end the snip (ADR-0059).

use super::super::*;
use super::Step;
use super::world::World;
use kx_module_api::ports::Handle;
use proptest::prelude::*;

/// The most Escs a snip over frozen screens with no job asked takes to end: a held handle, the question, Don't save.
const ESCS: usize = 3;

/// `step` closes the snip held in `before`: a picture or a recorded video on the strip, or the editor; while a frame's
/// handle is held, the strip hides and an Esc only drops it (edge 4).
fn closes(step: &Step, before: &Stage) -> bool {
    let gripped = matches!(before, Stage::Strip(s) if s.gripped());
    let close = match before {
        Stage::Strip(strip) => {
            let taken = !matches!(strip.held, Held::Ready(_));
            taken && !gripped && matches!(step, Step::Strip(StripKey::Close))
        }
        Stage::Edit(_) => matches!(step, Step::Edit(EditKey::Close)),
        _ => false,
    };
    let holds = matches!(before, Stage::Strip(s) if !matches!(s.held, Held::Ready(_)))
        || matches!(before, Stage::Edit(_));
    let ends = matches!(step, Step::Key) || matches!(step, Step::Esc) && !gripped;
    close || (holds && ends)
}

/// No strip key acts while a frame's handle is held, as the strip hides.
pub(super) fn check_hidden(
    world: &World,
    step: &Step,
    before: &Stage,
    outs: &[Out],
) -> Result<(), TestCaseError> {
    let gripped = matches!(before, Stage::Strip(s) if s.gripped());
    if gripped && matches!(step, Step::Strip(_)) {
        let still = outs.is_empty() && world.rig.stage() == before;
        prop_assert!(still, "{step:?} acted while a handle was held");
    }
    Ok(())
}

/// Checks that a free close asked exactly when nothing was kept.
pub(super) fn check_close(
    world: &World,
    step: &Step,
    before: &Stage,
    kept: bool,
    outs: &[Out],
) -> Result<(), TestCaseError> {
    if !closes(step, before) {
        return Ok(());
    }
    let asking = world.rig.flow.asking();
    if kept {
        prop_assert!(
            outs.contains(&Out::End) && !asking,
            "kept, yet {step:?} asked"
        );
    } else {
        prop_assert!(
            asking && !outs.contains(&Out::End),
            "unkept, yet {step:?} ended"
        );
    }
    Ok(())
}

/// A region's part stays within its frame's box, and a set leaves each side it moved at least the frame's least, or
/// the box's side when shorter.
pub(super) fn check_frame(step: &Step, before: &Stage, after: &Stage) -> Result<(), TestCaseError> {
    let Stage::Strip(Strip {
        part,
        frame: Some(frame),
        ..
    }) = after
    else {
        return Ok(());
    };
    let (r, w) = (part.rect(), frame.within);
    let inside = r.x >= w.x && r.y >= w.y && r.x + r.w <= w.x + w.w && r.y + r.h <= w.y + w.h;
    prop_assert!(inside, "{:?} left its frame's box {:?}", r, w);
    let (Stage::Strip(was), Step::Spot(to)) = (before, step) else {
        return Ok(());
    };
    let Some(grip) = was.frame.and_then(|f| f.grip).filter(|_| was.part != *part) else {
        return Ok(());
    };
    let least = |side: i32| i32::from(frame.min).min(side);
    let across = !matches!(grip.handle, Handle::Top | Handle::Bottom) && to.x != grip.at.x;
    let down = !matches!(grip.handle, Handle::Left | Handle::Right) && to.y != grip.at.y;
    prop_assert!(
        !across || r.w >= least(w.w),
        "{:?} set narrower than its least",
        r
    );
    prop_assert!(
        !down || r.h >= least(w.h),
        "{:?} set shorter than its least",
        r
    );
    Ok(())
}

/// Over frozen screens with no job asked, `ESCS` Escs at most end the snip, a held handle and the question included.
pub(super) fn check_escs(world: &mut World) -> Result<(), TestCaseError> {
    if world.rig.flow.frozen().is_none() || world.rig.flow.busy.is_some() {
        return Ok(());
    }
    for _ in 0..ESCS {
        world.act(&Step::Esc)?;
    }
    prop_assert_eq!(
        world.rig.stage(),
        &Stage::Idle,
        "{} Escs left the snip open",
        ESCS
    );
    Ok(())
}
