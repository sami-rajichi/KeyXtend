//! The Models page: each model that knows the language in front with its size and state, a download's progress, the
//! best model's note, the slow-PC line and the models folder with its drive's room; its Cloud side lists the switched-on
//! services' models, each with its service and mark (ADR-0047).

use super::cloud::{ServiceMark, ServiceName};
use crate::Redacted;

listed! {
    /// Which models the Models page lists; each side pages alone.
    pub enum ModelSide ("Both sides.") {
        /// The models on this PC.
        Local,
        /// The models of the cloud services switched on.
        Cloud,
    }
}

listed! {
    /// What the Cloud side's band says, beside its button to the Cloud page.
    pub enum CloudBand ("Every band.") {
        /// Cloud services are off, so none is listed.
        Off,
        /// No service is switched on yet.
        NoneOn,
        /// The models of the services switched on; nothing leaves before Stop.
        Listed,
    }
}

listed! {
    /// Why a cloud model cannot be used now, which its row says.
    pub enum Unavailable ("Every reason.") {
        /// The keep-nothing switch is on, and no host of the model keeps nothing.
        KeepNothing,
    }
}

/// A cloud model's parts of its row: its service, which the cloud mark names, and where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudRow {
    /// Its service.
    pub service: ServiceName,
    /// Where its service stands; Use acts only while it is Ready.
    pub mark: ServiceMark,
    /// Why it cannot be used now; `None` when it can.
    pub unavailable: Option<Unavailable>,
}

/// Where a model's file stands on this PC.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModelState {
    /// Its file is in the folder, with the fingerprint its row names.
    Ready,
    /// Its file is not there: Download.
    Missing,
    /// It needs more memory than this PC has: Not for this PC, with the reason.
    NotHere {
        /// What it needs, in MB.
        need_mb: u32,
        /// What this PC has in all, in MB, which the fit goes by.
        have_mb: u32,
        /// What is free on this PC now, in MB, which the reason shows.
        free_mb: u32,
    },
    /// A file no row names: Other file, not checked.
    Other,
}

/// A download in progress on a model's row, with Pause or Resume and Cancel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FetchState {
    /// Bytes written so far.
    pub done: u64,
    /// The file's size.
    pub total: u64,
    /// It is paused, keeping its part.
    pub paused: bool,
}

/// One model's row on the Models page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRow {
    /// Its shown name, or the user's own file name for an other file.
    pub name: Redacted<String>,
    /// Its file's size in bytes, from its row or, for an other file, from the folder.
    pub bytes: u64,
    /// It is the language's default, the first of its tested order.
    pub default: bool,
    /// Where its file stands.
    pub state: ModelState,
    /// It is used for the language: the user's pick, else the default.
    pub used: bool,
    /// It is ticked.
    pub tick: bool,
    /// Its download, while one runs or waits paused.
    pub fetch: Option<FetchState>,
    /// Its service and mark, for a cloud model; `None` for a model on this PC.
    pub cloud: Option<CloudRow>,
}

/// A folder line: the folder and its drive's free room, as in Models folder: D:\KeyXtend\models · 212 GB free.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderLine {
    /// The folder.
    pub path: Redacted<String>,
    /// Bytes free on its drive.
    pub free: u64,
}

/// The Models page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelsSheet {
    /// The shown page's rows.
    pub rows: Vec<ModelRow>,
    /// The best model for the language, which the note names; `None` for no note.
    pub best: Option<String>,
    /// The language has no tested order of its own, so the general order is used.
    pub untested: bool,
    /// The language's default is Not for this PC, so the page says local models will be very slow here.
    pub slow: bool,
    /// The models folder, with Change and Open; `None` until the first download asks for it.
    pub folder: Option<FolderLine>,
    /// The folder offered at the first download, while that question shows.
    pub offer: Option<FolderLine>,
    /// The side shown.
    pub side: ModelSide,
    /// The Cloud side's band; `None` on the Local side.
    pub band: Option<CloudBand>,
}
