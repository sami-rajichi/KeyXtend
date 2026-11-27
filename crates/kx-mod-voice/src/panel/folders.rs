//! The folder list, to pick the models folder with one finger: the drives, then a folder's subfolders, Back up one
//! level and Here to use the folder shown. A folder that is no plain absolute path is refused (edge 38).

use std::path::{Path, PathBuf};

use kx_module_api::ports::{BarHint, Drive, FolderScan, ModelError, VoiceTab, path_ok};

use super::{Do, Page, Panel};

impl Panel {
    /// The folder list's rows, each a folder and its name: the drives, or the shown folder's subfolders.
    pub(crate) fn folder_rows(&self, drives: &[Drive]) -> Vec<(PathBuf, String)> {
        match &self.browse {
            None => (drives.iter())
                .map(|d| (d.root.expose().clone(), d.name.expose().clone()))
                .collect(),
            Some(dir) => (self.dirs.iter())
                .map(|n| (dir.join(n), n.clone()))
                .collect(),
        }
    }

    /// Opens folder `dir` in the folder list, once it passes `path_ok`.
    pub(super) fn open_dir(&mut self, dir: PathBuf) -> Vec<Do> {
        if !path_ok(&dir) {
            self.note = Some(BarHint::Folder);
            return Vec::new();
        }
        self.show_dir(Some(dir.clone()));
        vec![Do::Browse(dir)]
    }

    /// Up one folder; from the drives, back to Models.
    pub(super) fn up(&mut self) -> Vec<Do> {
        let Some(dir) = self.browse.clone() else {
            return self.go(Page::Tab(VoiceTab::Models));
        };
        let parent = dir.parent().map(Path::to_path_buf);
        self.show_dir(parent.clone());
        parent.map(Do::Browse).into_iter().collect()
    }

    /// A folder's subfolders as read; a folder that cannot be read shows its note and the drives.
    pub fn browsed(&mut self, dir: &Path, scan: Result<FolderScan, ModelError>) {
        if self.browse.as_deref() != Some(dir) {
            return;
        }
        if let Ok(scan) = scan {
            self.dirs = scan.folders.iter().map(|f| f.expose().clone()).collect();
        } else {
            self.note = Some(BarHint::Folder);
            self.show_dir(None);
        }
    }

    /// Shows `dir`, or the drives for `None`, from its first page.
    fn show_dir(&mut self, dir: Option<PathBuf>) {
        self.browse = dir;
        self.dirs.clear();
        self.at = 0;
    }
}
