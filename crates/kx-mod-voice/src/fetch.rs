//! Downloads (design "The models folder and downloads", ADR-0043 decision 8): one at a time, as the `fetch` job takes
//! them, each on its row with progress, Pause and Cancel. Room on the drive is checked first; a fault becomes the
//! note the panel shows. Pure: the actor starts the run and hands its notes in.

use kx_module_api::ports::{
    BarHint, Drive, FetchAsk, FetchButton, FetchNote, FetchState, ModelError,
};

use crate::VoiceSettings;
use crate::folder::Folder;
use crate::models::Row;

/// What a download's note did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetched {
    /// It moved on.
    Moved,
    /// The file was checked and took its name.
    Done,
    /// It was cancelled, so its part is gone and nothing is said.
    Stopped,
    /// It failed, with the note to show.
    Failed(BarHint),
    /// It names no live download, as a late note after Cancel.
    Stale,
}

/// The live download: its row and where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Live {
    id: String,
    state: FetchState,
}

/// The downloads, one at a time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fetches {
    live: Option<Live>,
}

impl Fetches {
    /// Row `id`'s download while it runs or waits paused.
    #[must_use]
    pub fn state(&self, id: &str) -> Option<FetchState> {
        let live = self.live.as_ref().filter(|l| l.id == id);
        live.map(|l| l.state)
    }

    /// True while a download runs or waits paused, so another waits.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.live.is_some()
    }

    /// Starts `row`'s download into `folder` once its drive among `drives` has the room; gives the ask to fetch.
    ///
    /// # Errors
    /// The note to show: `Folder` without a folder or its drive, `Full` under the file and `min_free_mb`, `Print` while
    /// another file holds its name, as the job never replaces a file.
    pub fn start(
        &mut self,
        row: &Row,
        folder: &Folder,
        drives: &[Drive],
        settings: &VoiceSettings,
    ) -> Result<FetchAsk, BarHint> {
        if folder.taken(row).is_some() {
            return Err(BarHint::Print);
        }
        let min_free_mb = settings.min_free_mb;
        let refused = |e| hint(e).unwrap_or(BarHint::Net);
        folder
            .room(drives, row.bytes, min_free_mb)
            .map_err(refused)?;
        let dir = folder.dir().ok_or(BarHint::Folder)?;
        let ask = row.fetch(dir, settings.job(), min_free_mb, settings.stall_s);
        let ask = ask.ok_or(BarHint::Net)?;
        ask.check().map_err(refused)?;
        let state = FetchState {
            done: 0,
            total: row.bytes,
            paused: false,
        };
        self.live = Some(Live {
            id: row.id.clone(),
            state,
        });
        Ok(ask)
    }

    /// A download's button on row `id`; gives what the run must do, or `None` when the button cannot act.
    pub fn press(&mut self, id: &str, button: FetchButton) -> Option<FetchButton> {
        let live = self.live.as_mut().filter(|l| l.id == id)?;
        match button {
            FetchButton::Pause if !live.state.paused => live.state.paused = true,
            FetchButton::Resume if live.state.paused => live.state.paused = false,
            FetchButton::Cancel => self.live = None,
            FetchButton::Pause | FetchButton::Resume => return None,
        }
        Some(button)
    }

    /// A note from row `id`'s download.
    pub fn note(&mut self, id: &str, note: FetchNote) -> Fetched {
        let Some(live) = self.live.as_mut().filter(|l| l.id == id) else {
            return Fetched::Stale;
        };
        let done = match note {
            FetchNote::Progress { done, total } => {
                (live.state.done, live.state.total) = (done, total);
                return Fetched::Moved;
            }
            FetchNote::Done => Fetched::Done,
            FetchNote::Fault(error) => hint(error).map_or(Fetched::Stopped, Fetched::Failed),
        };
        self.live = None;
        done
    }
}

/// The note a download's or the folder's error shows; `None` for a cancel, which needs none.
#[must_use]
pub fn hint(error: ModelError) -> Option<BarHint> {
    use ModelError as E;
    Some(match error {
        E::Print | E::Size => BarHint::Print,
        E::Full => BarHint::Full,
        E::Missing | E::ReadOnly | E::BadPath | E::Io => BarHint::Folder,
        E::Net | E::Stalled | E::Host | E::Server | E::Internal | E::Gone | E::Range(_) => {
            BarHint::Net
        }
        E::Cancelled => return None,
    })
}

#[cfg(test)]
mod tests;
