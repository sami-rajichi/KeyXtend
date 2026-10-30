//! The frames between the keyboard and its worker processes (ADR-0004, ADR-0034 decision 5): a fixed header, a
//! payload capped by its kind, and every field checked before it is trusted or a buffer is allocated.

#![forbid(unsafe_code)]

mod ask;
mod code;
mod error;
mod frame;
mod limits;
mod reply;

pub use ask::Ask;
pub use code::{Codec, Why};
pub use error::IpcError;
pub use limits::{
    MAX_ASK_LEN, MAX_IMAGE_BYTES, MAX_REPLY_LEN, MAX_RGBA_BYTES, MAX_SIDE, RGBA_BYTES, VERSION,
};
pub use reply::Reply;

#[cfg(test)]
mod tests;
