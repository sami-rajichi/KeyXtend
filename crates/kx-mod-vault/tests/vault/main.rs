//! The vault module over the shared fake host: its start, its gates, the notebook it loads, the actor it runs and
//! stops, and the starts that fail.

#![allow(
    clippy::unwrap_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod module;
