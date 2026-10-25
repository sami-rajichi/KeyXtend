//! The layouts module: reads the OS keyboard layouts through `LayoutSource` and provides `Layouts`,
//! the table of the layout shown, the language list in the OS order, and the switch requests.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod logic;
mod manifest;
mod scans;
mod service;
mod settings;

pub use logic::follow::Follow;
pub use logic::label::label;
pub use manifest::{ID, MANIFEST};
pub use scans::{ScanTableError, scans};
pub use service::LayoutsService;
pub use settings::{LayoutsSettings, SPEC};

use kx_module_api::ports::{LayoutAsk, LayoutSourceKey, Layouts, LayoutsKey};
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};
use std::sync::Arc;

/// Why the module could not start: its compiled-in scan table is broken.
const SCANS_BROKEN: &str = "the scan table is broken";

/// The layouts module; the app registers it behind the `mod-layouts` feature.
#[derive(Debug, Default)]
pub struct LayoutsModule;

impl Module for LayoutsModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let source = cx.service::<LayoutSourceKey>()?;
        let settings = cx.settings::<LayoutsSettings>()?;
        let scans = scans().map_err(|_| ModuleError::Start(SCANS_BROKEN))?;
        let service = Arc::new(LayoutsService::new(source, settings, scans, cx.bus()));
        let teller = Arc::clone(&service);
        cx.subscribe(move |_: &LayoutAsk| teller.tell())?;
        let service: Arc<dyn Layouts> = service;
        cx.provide::<LayoutsKey>(service)?;
        Ok(())
    }
}
