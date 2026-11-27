//! The fake folder's copy marks: one per video of the unsaved part, made at the clock's time, which a test may also
//! place or refuse, and pruned once its video is gone.

use super::{FakeSnipFolder, lock};
use kx_module_api::Mono;
use kx_module_api::ports::{FileKind, SaveError, SnipFile};

impl FakeSnipFolder {
    /// Marks `video` copied at `at`, whether or not it is held, as a mark left by an earlier run.
    pub fn mark_at(&self, video: SnipFile, at: Mono) {
        let path = video.expose().to_path_buf();
        lock(&self.table).marks.insert(path, (video, at));
    }

    /// When `video` was marked copied, if it is.
    #[must_use]
    pub fn marked(&self, video: &SnipFile) -> Option<Mono> {
        lock(&self.table)
            .marks
            .get(video.expose())
            .map(|(_, at)| *at)
    }

    /// Every video marked copied, by path.
    #[must_use]
    pub fn marks(&self) -> Vec<SnipFile> {
        let table = lock(&self.table);
        table
            .marks
            .values()
            .map(|(video, _)| video.clone())
            .collect()
    }

    /// Refuses later marks with `error`, or makes them again with `None`.
    pub fn set_mark_error(&self, error: Option<SaveError>) {
        lock(&self.table).mark_error = error;
    }

    /// Marks `video` copied now, as `SnipFolder::mark_copied` says.
    pub(super) fn mark(&self, video: &SnipFile) -> Result<(), SaveError> {
        let now = self.clock.now();
        let mut table = lock(&self.table);
        let dirs = table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        if video.kind() != FileKind::Mp4 || !video.lies_in(&dirs.unsaved) {
            return Err(SaveError::BadPath);
        }
        table.mark_error.map_or(Ok(()), Err)?;
        if !table.files.contains_key(video.expose()) {
            return Err(SaveError::Missing);
        }
        let path = video.expose().to_path_buf();
        table.marks.insert(path, (video.clone(), now));
        Ok(())
    }

    /// Deletes each mark whose video is gone, as `SnipFolder::prune_marks` says.
    pub(super) fn prune(&self) -> Result<(), SaveError> {
        let mut table = lock(&self.table);
        table.dirs.as_ref().ok_or(SaveError::NoFolder)?;
        table.list_error.map_or(Ok(()), Err)?;
        let table = &mut *table;
        table.marks.retain(|path, _| table.files.contains_key(path));
        Ok(())
    }
}
