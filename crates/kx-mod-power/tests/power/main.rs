//! The power module over the shared fake host: its start, its actor over the fake dialog, and its thread.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod asks;
mod module;
mod rig;
mod thread;
