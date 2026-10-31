//! The worker's loop: one ask in, one reply out, until goodbye or the end of input. Every failed draw or DIB is a
//! `Failed` reply, a panic included; a frame that cannot be read or written ends the loop.

use crate::media::Picture;
use kx_ipc::{Ask, Codec, IpcError, Reply, Why};
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};

/// The media job's loop: `serve` with every ask's panic kept off stderr.
///
/// # Errors
/// As `serve`.
pub fn answer<R, W, D, M>(r: &mut R, w: &mut W, draw: D, make: M) -> Result<(), IpcError>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
    D: Fn(Codec, &[u8], u32) -> Result<Picture, Why>,
    M: Fn(&[u8], usize) -> Result<Vec<u8>, Why>,
{
    hush();
    serve(r, w, draw, make)
}

/// Answers each ask read from `r` on `w`, a draw with `draw` and a DIB with `make`, one at a time, until `Bye` or the
/// end of input.
///
/// # Errors
/// The first frame that cannot be read or written; the stream is then out of step, so the worker exits.
fn serve<R, W, D, M>(r: &mut R, w: &mut W, draw: D, make: M) -> Result<(), IpcError>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
    D: Fn(Codec, &[u8], u32) -> Result<Picture, Why>,
    M: Fn(&[u8], usize) -> Result<Vec<u8>, Why>,
{
    loop {
        // The image's bytes drop with the ask in its arm, before the reply is written.
        let reply = match Ask::read(r) {
            Ok(Ask::Draw {
                id,
                codec,
                side,
                bytes,
            }) => pixels(id, guard(|| draw(codec, &bytes, side))),
            Ok(Ask::Dib {
                id,
                max_bytes,
                bytes,
            }) => made(id, guard(|| make(&bytes, cap(max_bytes)))),
            Ok(Ask::Bye) | Err(IpcError::Closed) => return Ok(()),
            Err(e) => return Err(e),
        };
        reply.write(w)?;
    }
}

/// Keeps a draw's panic off stderr: its reply says `Internal`, and its message and place are never written.
fn hush() {
    panic::set_hook(Box::new(|_| {}));
}

/// Runs `work`, taking a panic as an internal failure.
fn guard<T>(work: impl FnOnce() -> Result<T, Why>) -> Result<T, Why> {
    panic::catch_unwind(AssertUnwindSafe(work)).unwrap_or(Err(Why::Internal))
}

/// The most bytes a DIB may hold, as asked; the frame keeps it within `MAX_IMAGE_BYTES`.
fn cap(max_bytes: u32) -> usize {
    usize::try_from(max_bytes).unwrap_or(usize::MAX)
}

/// The reply to draw `id`: its pixels, or why there are none.
fn pixels(id: u32, outcome: Result<Picture, Why>) -> Reply {
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

/// The reply to DIB ask `id`: its bytes, or why there are none.
fn made(id: u32, outcome: Result<Vec<u8>, Why>) -> Reply {
    match outcome {
        Ok(bytes) => Reply::Dib { id, bytes },
        Err(why) => Reply::Failed { id, why },
    }
}

#[cfg(test)]
mod tests;
