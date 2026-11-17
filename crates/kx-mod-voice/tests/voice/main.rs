//! The voice module through the kernel's parts: its start over the shared fake host, the repair of a bad `[voice]`
//! value with the settings notice, the module on its own threads, a card's link through its own lookup, and Paste's
//! answer taken only from its own take.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod clip;
mod link;
mod module;
mod rig;
mod settings;
mod thread;
