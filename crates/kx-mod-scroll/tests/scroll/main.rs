//! The scroll module over the shared fake host: its start, and its actor over the fake target and clock.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod logs;
mod module;
mod props;
mod quits;
mod rig;
mod stops;
mod taps;
mod thread;
mod ticks;
