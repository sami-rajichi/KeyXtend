//! The panel's pages on the hub: the rows a page holds, told at the start and again as a panel opens, since the ui
//! module starts before the voice module listens, and a pad page tap that turns the open panel.

use super::super::Hub;
use kx_module_api::ports::{VoiceAct, VoiceClick, VoicePage};

impl Hub {
    /// A pad page tap turns the open panel's page; true when it did.
    pub(in crate::hub) fn voice_pad(&self, back: bool) -> bool {
        let round = {
            let s = self.voice();
            match s.page.as_deref() {
                Some(VoicePage {
                    panel: Some(_),
                    round,
                    ..
                }) => *round,
                _ => return false,
            }
        };
        let click = VoiceClick::Page { back };
        self.voice_send(&VoiceAct { round, click });
        true
    }

    /// Tells the module the rows a page holds, `[ui] page_rows`; it pages by the last one heard.
    pub(super) fn voice_rows(&self) {
        let rows = self.lock().settings.page_rows;
        let rows = VoiceClick::Rows(u8::try_from(rows).unwrap_or(u8::MAX));
        let round = self.voice().page.as_ref().map_or(0, |p| p.round);
        self.voice_send(&VoiceAct { round, click: rows });
    }
}
