//! The voice module through the kernel's parts: its start over the shared fake host, the repair of a bad `[voice]`
//! value with the settings notice, and the module on its own threads.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod module;
mod rig;
mod settings;
mod thread;
