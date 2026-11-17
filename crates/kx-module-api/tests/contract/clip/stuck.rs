//! A platform whose every clipboard port fails: the clipboard is busy, the shelf locked and the worker gone.

use kx_module_api::ports::{
    BlobName, Calendar, ClipConfig, ClipError, ClipNote, ClipPart, ClipRun, ClipWait, Clipboard,
    ClipboardWriter, DibAsk, FileFact, LocalTime, PictureAsk, PictureConfig, PictureError,
    PictureRun, Pictures, Pixels, Room, SealedError, SealedShelf,
};
use kx_module_api::{Redacted, Secret};
use std::path::PathBuf;

/// A platform whose clipboard is always busy, whose shelf is locked and whose worker is gone.
pub(super) struct Stuck;

/// A watch that hears nothing.
struct Deaf;

impl ClipRun for Deaf {}

impl Clipboard for Stuck {
    fn watch(
        &self,
        _: ClipConfig,
        tell: Box<dyn Fn(ClipNote) + Send>,
    ) -> Result<Box<dyn ClipRun>, ClipError> {
        tell(ClipNote::Busy);
        Ok(Box::new(Deaf))
    }

    fn files(&self, paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
        paths.expose().iter().map(|_| FileFact::Gone).collect()
    }

    fn text(&self, _: ClipWait, _: usize) -> Result<Option<Secret>, ClipError> {
        Err(ClipError::Busy)
    }

    fn unmarked_text(&self, _: ClipWait, _: usize) -> Result<Option<Secret>, ClipError> {
        Err(ClipError::Busy)
    }
}

impl ClipboardWriter for Stuck {
    fn write(&self, _: &[ClipPart], _: ClipWait) -> Result<(), ClipError> {
        Err(ClipError::Busy)
    }
}

impl SealedShelf for Stuck {
    fn load(&self, _: &BlobName, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        Err(SealedError::Locked)
    }

    fn save(&self, _: &BlobName, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn remove(&self, _: &BlobName, _: &BlobName) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn drop_folder(&self, _: &BlobName) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        Err(SealedError::Locked)
    }

    fn names(&self, _: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        Err(SealedError::Locked)
    }

    fn room(&self) -> Result<Room, SealedError> {
        Err(SealedError::Locked)
    }
}

impl PictureRun for Stuck {
    fn draw(&mut self, _: PictureAsk) -> Result<Pixels, PictureError> {
        Err(PictureError::Gone)
    }

    fn dib(&mut self, _: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        Err(PictureError::Gone)
    }
}

impl Pictures for Stuck {
    fn open(&self, _: PictureConfig) -> Box<dyn PictureRun> {
        Box::new(Stuck)
    }
}

impl Calendar for Stuck {
    #[allow(clippy::unwrap_used, reason = "the epoch, at UTC, is a date")]
    fn now(&self) -> LocalTime {
        LocalTime::at(0, 0).unwrap()
    }
}
