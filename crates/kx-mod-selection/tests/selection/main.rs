//! The selection module through the kernel's parts: its start and stop over the shared fake host, its thread with a
//! fake keyboard, and the repair of a bad `[selection]` value.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod module;
mod rig;
mod settings;
mod thread;
