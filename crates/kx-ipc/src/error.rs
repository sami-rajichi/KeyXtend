//! Why a frame was not read or written.

use std::io;
use thiserror::Error;

/// Why a frame was refused; after any error the stream is out of step, so the caller drops it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum IpcError {
    /// The stream ended cleanly, before a frame began.
    #[error("the stream ended")]
    Closed,
    /// The stream ended inside a frame.
    #[error("the frame was cut short")]
    Short,
    /// The frame does not start with the protocol's magic.
    #[error("the frame has a wrong magic")]
    BadMagic,
    /// The frame is from another protocol version.
    #[error("the frame is from another protocol version")]
    BadVersion,
    /// The frame's kind is unknown, or not one this side reads.
    #[error("the frame's kind is not one this side reads")]
    BadKind,
    /// The payload is longer than its kind allows.
    #[error("the frame is over its size limit")]
    TooLong,
    /// The payload's length disagrees with its kind or its fields, or a goodbye carries an id.
    #[error("the frame's length disagrees with its fields")]
    Inconsistent,
    /// A field holds a value outside its range: a codec, a reason or a side.
    #[error("a field of the frame is out of range")]
    BadField,
    /// The stream failed for another reason.
    #[error("the stream failed: {0:?}")]
    Io(io::ErrorKind),
}

impl From<io::Error> for IpcError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            Self::Short
        } else {
            Self::Io(e.kind())
        }
    }
}
