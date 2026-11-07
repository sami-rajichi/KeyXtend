//! The jobs that read the places: a folder list with its files, the look at a picked folder, and a box or a `KeePass`
//! file read within its cap and opened in the worker.

use std::path::{Path, PathBuf};

use kx_module_api::ports::{
    KeepassImport, MAX_KEEPASS_BYTES, Place, PlaceError, PlaceState, box_file,
};
use kx_module_api::{Redacted, Secret};

use super::{Look, Runner, Shut, Unboxed};
use crate::backup::mark::mark;

impl Runner {
    /// The folders in `folder`, or the drives, then with `files` the files of that extension in `folder`.
    pub(super) fn listed(
        &self,
        folder: Option<&Redacted<PathBuf>>,
        files: Option<&str>,
    ) -> Result<Vec<Place>, PlaceError> {
        let folder = folder.map(|f| f.expose().as_path());
        let mut rows = self.places.folders(folder)?;
        if let (Some(folder), Some(ext)) = (folder, files) {
            rows.extend(self.places.files(folder, ext)?);
        }
        Ok(rows)
    }

    /// Reads the `KeePass` `file` within `MAX_KEEPASS_BYTES` and has the worker open it with `password`, as the module
    /// never parses it; a file past the cap never reaches the worker.
    pub(super) fn keepass(
        &mut self,
        file: &Path,
        password: &Secret,
    ) -> Result<KeepassImport, Shut> {
        let cap = u64::try_from(MAX_KEEPASS_BYTES).unwrap_or(u64::MAX);
        let bytes = self.places.read(file, cap).map_err(Shut::Read)?;
        self.run
            .keepass(bytes.expose(), password)
            .map_err(Shut::Open)
    }

    /// What `folder` holds as the box; the data folder is refused before anything is read.
    pub(super) fn look(&self, folder: &Path, cap: u64) -> Look {
        if self.places.state(folder, 0) == PlaceState::Refused {
            return Look::Unread(PlaceError::Refused);
        }
        let file = folder.join(box_file());
        match self.places.read(&file, cap) {
            Ok(bytes) => Look::Box {
                mark: Some(mark(bytes.expose())),
                modified_ms: self.places.modified(&file),
            },
            Err(PlaceError::TooLarge) => Look::Box {
                mark: None,
                modified_ms: self.places.modified(&file),
            },
            Err(PlaceError::Missing) => Look::Empty,
            Err(error) => Look::Unread(error),
        }
    }

    /// Reads the box `file` within `cap` bytes and opens it with `password`; the bytes are dropped once opened.
    pub(super) fn open(
        &mut self,
        file: &Path,
        cap: u64,
        password: &Secret,
    ) -> Result<Unboxed, Shut> {
        let bytes = self.places.read(file, cap).map_err(Shut::Read)?;
        let bytes = bytes.expose();
        let opened = self.run.open(bytes, password).map_err(Shut::Open)?;
        Ok(Unboxed {
            opened,
            mark: mark(bytes),
            len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            beside: self.beside(file),
        })
    }

    /// True when `file` is a copy and the box's own file sits beside it, where a write to the folder goes.
    fn beside(&self, file: &Path) -> bool {
        let own = file.with_file_name(box_file());
        own != file && self.places.modified(&own).is_some()
    }
}
