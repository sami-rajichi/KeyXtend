//! The Voice panel's pages (design "The gear and the Voice panel"): the home list, Models, Cloud with its cards and key
//! pages, Device, Compare, History, Options and the folder list, each list paged as the ui's `page_rows` says, tick
//! then act. Texts, folder names, model names and typed lines, which may be the user's own, stay redacted.

mod cloud;
mod compare;
mod device;
mod history;
mod models;
mod options;

pub use cloud::{
    AddressFault, CardPart, CloudCard, CloudLine, CloudSheet, Fact, FactArg, FactValue, KeyCheck,
    ServiceMark, ServiceName, ServiceRow, ServiceSheet, UsePrice,
};
pub use compare::{CompareCloud, CompareMic, CompareRow, CompareRun, CompareSheet, RunStep};
pub use device::{CardCheck, DeviceItem, DeviceRow, DeviceSheet, FailedCheck};
pub use history::{HistoryRow, HistorySheet};
pub use models::{
    CloudBand, CloudRow, FetchState, FolderLine, ModelRow, ModelSide, ModelState, ModelsSheet,
    Unavailable,
};
pub use options::{Ends, OptionRow, OptionUnit, OptionValue};

use super::act::PanelButton;
use super::bar::{BarHint, VoiceLang};
use crate::Redacted;
use crate::ports::{Date, Pages};

listed! {
    /// A page the home list opens, in its order.
    pub enum VoiceTab ("Every page, in the home list's order.") {
        /// The models that know the language in front.
        Models,
        /// The cloud services, each with its card and key page (ADR-0047).
        Cloud,
        /// Automatic, the processor and each card.
        Device,
        /// Compare models on the user's own voice.
        Compare,
        /// The texts dictated.
        History,
        /// The switches and steppers of the settings.
        Options,
    }
}

/// A device that runs models, as a line names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceName {
    /// The processor.
    Cpu,
    /// A card, by the name the system gives it.
    Card(String),
}

/// The device picked, as the home list and Compare name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceLabel {
    /// Automatic, with the device it runs on now, as in Automatic (processor).
    Auto(DeviceName),
    /// The processor.
    Cpu,
    /// A card, by the name the system gives it.
    Card(String),
}

/// The home list's present values, one per page, and the pages its shown page lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceHome {
    /// The model used for the language in front; `None` when none is ready.
    pub model: Option<Redacted<String>>,
    /// The device picked.
    pub device: DeviceLabel,
    /// The day Compare last ran for the language, as in Last run: 12 Nov; `None` before its first run.
    pub compared: Option<Date>,
    /// How many texts History holds.
    pub history: u32,
    /// How many options differ from their defaults; 0 reads All at defaults.
    pub changed: u16,
    /// The Cloud row's value: how many services are ready while cloud services are on; `None` while off.
    pub cloud: Option<u16>,
    /// The pages the shown page of the list holds, `page_rows` to a page, in `VoiceTab::ALL`'s order.
    pub tabs: Vec<VoiceTab>,
}

/// The folder list, to pick the models folder: the drives, then a folder's subfolders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderList {
    /// The shown folder, or `None` at the drives.
    pub folder: Option<Redacted<String>>,
    /// The shown page's drives or subfolders.
    pub rows: Vec<Redacted<String>>,
}

/// The page the panel shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoiceSheet {
    /// The home list.
    Home(VoiceHome),
    /// The models.
    Models(ModelsSheet),
    /// The cloud services.
    Cloud(CloudSheet),
    /// A cloud service's card, boxed as it holds many facts.
    Card(Box<CloudCard>),
    /// A cloud service's key page.
    Service(ServiceSheet),
    /// The devices.
    Device(DeviceSheet),
    /// Compare models.
    Compare(CompareSheet),
    /// History.
    History(HistorySheet),
    /// The shown page of the options.
    Options(Vec<OptionRow>),
    /// The folder list.
    Folders(FolderList),
}

/// Everything the panel draws, above the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoicePanel {
    /// The page shown.
    pub sheet: VoiceSheet,
    /// The language in front, which the home list, Models and Compare name.
    pub lang: VoiceLang,
    /// Where the shown list is.
    pub pages: Pages,
    /// How many rows are ticked, on every page of the list, as in Insert (2).
    pub ticked: u16,
    /// The page's header and footer buttons that act now; the others stay in their place dimmed.
    pub acts: Vec<PanelButton>,
    /// A note band, if any.
    pub note: Option<BarHint>,
}
