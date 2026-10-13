//! The mouse module over the shared fake host: its start, and its actor over the fake hook.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod failures;
mod logs;
mod module;
mod props;
mod rig;
mod runtime;
