//! Snip's jobs' frames (ADR-0056): the `record` job's asks and the notes it sends on its own, and the `pick` job's
//! dialogs and their answers.

use crate::worker::{Frames, Streams};
use kx_ipc::{
    IpcError, PICK_JOB, PickAsk, PickReply, RECORD_JOB, RecordAsk, RecordFault, RecordNote,
};
use std::io::{Read, Write};

/// The record job's frames (ADR-0056 decision 3): recordings, their switches and re-timed copies, and their notes.
#[derive(Debug)]
pub struct RecordFrames;

impl Frames for RecordFrames {
    const JOB: &'static str = RECORD_JOB;
    type Ask = RecordAsk<'static>;
    type Reply = RecordNote;

    fn bye() -> RecordAsk<'static> {
        RecordAsk::Bye
    }

    fn id(note: &RecordNote) -> u32 {
        note.id()
    }

    fn write<W: Write + ?Sized>(ask: &RecordAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<RecordNote, IpcError> {
        RecordNote::read(r)
    }
}

impl Streams for RecordFrames {
    fn ask_id(ask: &RecordAsk<'static>) -> u32 {
        ask.id()
    }

    /// A run's or a copy's end, or a fault that ends it. A sound's fault only answers a switch; when one ends a start
    /// instead, the recorder's `start` stops the stream itself.
    fn last(note: &RecordNote) -> bool {
        match note {
            RecordNote::End { .. } => true,
            RecordNote::Fault { fault, .. } => !sound_fault(*fault),
            RecordNote::Time { .. } | RecordNote::Switched { .. } | RecordNote::Lost { .. } => {
                false
            }
        }
    }
}

/// True for a sound's fault, which answers a switch mid-run and a start before it.
pub(crate) const fn sound_fault(fault: RecordFault) -> bool {
    matches!(
        fault,
        RecordFault::NoMic | RecordFault::MicRefused | RecordFault::NoSound
    )
}

/// The pick job's frames (ADR-0056 decision 6): one dialog at a time, each answered once.
#[derive(Debug)]
pub struct PickFrames;

impl Frames for PickFrames {
    const JOB: &'static str = PICK_JOB;
    type Ask = PickAsk<'static>;
    type Reply = PickReply<'static>;

    fn bye() -> PickAsk<'static> {
        PickAsk::Bye
    }

    fn id(reply: &PickReply<'static>) -> u32 {
        reply.id()
    }

    fn write<W: Write + ?Sized>(ask: &PickAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<PickReply<'static>, IpcError> {
        PickReply::read(r)
    }
}

#[cfg(test)]
mod tests;
