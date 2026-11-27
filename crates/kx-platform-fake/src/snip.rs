//! Fakes of Snip's ports (ADR-0056): the frozen desktop, the recorder, the Save and folder dialogs, the player and the
//! Snips folder, which keeps its files in a table in memory. Tests spell paths with `/` or `Path::join`, never `\`, as
//! the table matches paths as each system parses them, and CI runs on Linux too.

mod dialog;
mod folder;
mod freeze;
mod play;
mod record;

pub use dialog::{FakeFileAsk, Pick};
pub use folder::{FakeSnipFolder, Stored};
pub use freeze::{FakeScreenFreeze, Seen, painted, pixel_at};
pub use play::FakeMediaOpen;
pub use record::{FakeRecorder, Heard, RecLog, RecordCall};

#[cfg(test)]
mod tests;
