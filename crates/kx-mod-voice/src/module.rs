//! The module: it reads `[voice]` at start, drops a bad pick alone with the settings notice, and stays idle; the
//! dictation flow and its actor arrive with plan tasks 17 to 20.

use kx_module_api::{Manifest, Module, ModuleCx, ModuleError, Notice, NoticeArg, args, keys};

use crate::settings::{PICKS, VoiceSettings};
use crate::{ID, MANIFEST};

/// The voice typing module.
#[derive(Debug, Default)]
pub struct VoiceModule;

impl VoiceModule {
    /// A module that has not started.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for VoiceModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        read(cx)?;
        Ok(())
    }
}

/// `[voice]` with each bad pick dropped alone (edge case 19): the settings notice names it, and the rest is written
/// back so the next start finds it gone.
fn read(cx: &ModuleCx<'_>) -> Result<VoiceSettings, ModuleError> {
    let mut settings = cx.settings::<VoiceSettings>()?;
    let dropped = settings.drop_bad_picks();
    if dropped.is_empty() {
        return Ok(settings);
    }
    if let Ok(picks) = toml::Value::try_from(&settings.picks) {
        // A failed save keeps the repair for this run, and the kernel tells the user about the file.
        let _ = cx.settings_handle().set(PICKS, picks);
    }
    let mut notice = Notice::about(keys::SETTINGS_RESET, ID);
    notice.args.push((args::KEYS, NoticeArg::List(dropped)));
    // A notice needs no gate, so none is refused.
    let _ = cx.bus().publish(&notice);
    Ok(settings)
}
