//! The worker's loop: one ask in, one reply out, until goodbye or the end of input. Every failed draw is a `Failed`
//! reply, a panic included; a frame that cannot be read or written ends the loop.

use crate::media::Picture;
use kx_ipc::{Ask, Codec, IpcError, Reply, Why};
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};

/// Answers each draw read from `r` on `w` with `draw`, one at a time, until `Bye` or the end of input.
///
/// # Errors
/// The first frame that cannot be read or written; the stream is then out of step, so the worker exits.
pub fn serve<R, W, D>(r: &mut R, w: &mut W, draw: D) -> Result<(), IpcError>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
    D: Fn(Codec, &[u8], u32) -> Result<Picture, Why>,
{
    loop {
        let (id, outcome) = match Ask::read(r) {
            Ok(Ask::Draw {
                id,
                codec,
                side,
                bytes,
            }) => (id, guard(|| draw(codec, &bytes, side))),
            Ok(Ask::Bye) | Err(IpcError::Closed) => return Ok(()),
            Err(e) => return Err(e),
        };
        // The image's bytes were dropped with the ask above, before the reply is written.
        reply(id, outcome).write(w)?;
    }
}

/// Keeps a draw's panic off stderr: its reply says `Internal`, and its message and place are never written.
pub fn hush() {
    panic::set_hook(Box::new(|_| {}));
}

/// Runs `draw`, taking a panic as an internal failure.
fn guard(draw: impl FnOnce() -> Result<Picture, Why>) -> Result<Picture, Why> {
    panic::catch_unwind(AssertUnwindSafe(draw)).unwrap_or(Err(Why::Internal))
}

/// The reply to draw `id`: its pixels, or why there are none.
fn reply(id: u32, outcome: Result<Picture, Why>) -> Reply {
    match outcome {
        Ok(p) => Reply::Pixels {
            id,
            width: p.width,
            height: p.height,
            rgba: p.rgba,
        },
        Err(why) => Reply::Failed { id, why },
    }
}

#[cfg(test)]
mod tests;
