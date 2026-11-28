//! The snip module through the kernel's parts: its actor started over the shared fake host, and the repair of a bad
//! `[snip]` value with the settings notice.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod module;
mod settings;
