//! The clipboard module over the shared fake host: its start, its gates, the starts that fail, its thread with a fake
//! keyboard, and the kernel's repair of a bad `[clipboard]` value.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod module;
mod rig;
mod settings;
mod thread;
