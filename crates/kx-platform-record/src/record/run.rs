//! A whole run: its start checks (Media Foundation, the room, the file, the sounds), then its take, then its last note;
//! a fault or Cancel after the file is made deletes it, so only a whole video is ever kept.

use std::any::Any;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use super::mix::Mixer;
use super::take::{Halt, Take};
use super::{Order, Plan, RecordFault, RecordNote, Studio, Switch, scrap};
use crate::table::{Table, table};

/// Records `plan` with `studio`'s parts, obeying `orders` between frames and telling each note on `tell` but the last,
/// which it hands back once every part is closed: the end, the fault that ended the run, nothing after Cancel, or
/// `Failed` when the video could not be deleted. Call it on a thread of its own, which it readies for the system.
#[must_use]
pub fn record<S: Studio>(
    studio: &S,
    plan: &Plan,
    orders: &Receiver<Order>,
    tell: &mut dyn FnMut(RecordNote),
) -> Option<RecordNote> {
    let id = plan.id;
    let fault = |fault| Some(RecordNote::Fault { id, fault });
    let (ready, table, mix) = match begin(studio, plan) {
        Ok(begun) => begun,
        Err(f) => return fault(f),
    };
    let filmed = film(studio, plan, table, mix, orders, tell);
    // Every part that held the file is closed by now; Media Foundation ends too before the file is deleted.
    drop(ready);
    match filmed {
        Ok(end) => Some(end),
        // A halted run deletes its file; one still there is left to the host, which `Failed` tells to delete it.
        Err(_) if !scrap(&plan.file) => fault(RecordFault::Failed),
        Err(Halt::Fault(f)) => fault(f),
        Err(Halt::Cancel) => None,
    }
}

/// Readies the thread and checks the start: an even part, no file there yet, room above the floor, and each sound
/// switched on opened.
fn begin<S: Studio>(
    studio: &S,
    plan: &Plan,
) -> Result<(Box<dyn Any>, &'static Table, Mixer), RecordFault> {
    let table = table().ok_or(RecordFault::Failed)?;
    let ready = studio.ready()?;
    let shape = plan.shape();
    if shape.width == 0 || shape.height == 0 || plan.file.exists() {
        return Err(RecordFault::Failed);
    }
    let folder = plan.file.parent().ok_or(RecordFault::Failed)?;
    let room = studio.room_mb(folder).ok_or(RecordFault::Failed)?;
    if room < u64::from(plan.min_free_mb) {
        return Err(RecordFault::Full);
    }
    let mut mix = Mixer::new(table);
    for (which, on) in [(Switch::Mic, plan.mic), (Switch::Sound, plan.sound)] {
        if on {
            mix.open(which, studio.ear(which)?);
        }
    }
    Ok((ready, table, mix))
}

/// Opens the capture and the file, then runs the take to its end, and hands back its end note.
fn film<S: Studio>(
    studio: &S,
    plan: &Plan,
    table: &Table,
    mix: Mixer,
    orders: &Receiver<Order>,
    tell: &mut dyn FnMut(RecordNote),
) -> Result<RecordNote, Halt> {
    let film = studio.film(plan)?;
    let mut take = Take::new(studio, plan, film, mix);
    take.first(Duration::from_millis(u64::from(table.video.first_frame_ms)))?;
    let end = take.slots(orders, tell)?;
    Ok(take.finish(end)?)
}
