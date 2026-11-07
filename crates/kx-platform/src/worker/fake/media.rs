//! The media job as the script fakes it: pixels, DIBs and failures for each ask's id.

use super::{Call, Faked};
use crate::worker::{Heard, MediaFrames};
use kx_ipc::{Ask, IpcError, RGBA_BYTES, Reply, Why};

/// What the worker does with the next ask; with none queued it draws one pixel.
#[derive(Clone, Debug)]
pub enum Act {
    /// Pixels `w` by `h` for the ask's id.
    Draw(u32, u32),
    /// A DIB of this many bytes for the ask's id.
    Dib(usize),
    /// A failure for the ask's id.
    Fail(Why),
    /// This reply, whatever the ask's id.
    Raw(Reply),
    /// Nothing within the wait.
    Silent,
    /// A frame that could not be read.
    Broken(IpcError),
}

impl Faked for MediaFrames {
    type Act = Act;

    fn usual() -> Act {
        Act::Draw(1, 1)
    }

    fn note(ask: Ask<'static>) -> Option<(u32, Call)> {
        match ask {
            Ask::Draw { id, side, .. } => Some((id, Call::Draw { id, side })),
            Ask::Dib { id, max_bytes, .. } => Some((id, Call::Dib { id, max_bytes })),
            Ask::Bye => None,
        }
    }

    fn answer(act: Act, id: u32) -> Heard<Reply> {
        match act {
            Act::Draw(width, height) => {
                let len = usize::try_from(width * height).unwrap_or_default() * RGBA_BYTES;
                let rgba = vec![0; len];
                Heard::Frame(Ok(Reply::Pixels {
                    id,
                    width,
                    height,
                    rgba,
                }))
            }
            Act::Dib(len) => Heard::Frame(Ok(Reply::Dib {
                id,
                bytes: vec![0; len],
            })),
            Act::Fail(why) => Heard::Frame(Ok(Reply::Failed { id, why })),
            Act::Raw(reply) => Heard::Frame(Ok(reply)),
            Act::Silent => Heard::Late,
            Act::Broken(e) => Heard::Frame(Err(e)),
        }
    }
}
