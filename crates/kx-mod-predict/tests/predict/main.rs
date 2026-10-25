//! The prediction module over the shared fake host: its start, and its actor over the fake box, packs, sealed store,
//! clock and day.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod boxes;
mod emails;
mod lane;
mod lang;
mod logs;
mod menu;
mod module;
mod packs;
mod props;
mod rig;
mod saves;
mod thread;
