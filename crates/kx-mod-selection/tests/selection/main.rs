//! The selection module through the kernel's parts: the repair of a bad `[selection]` value.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod settings;
