//! The configs the parts take from `[clipboard]`, each in its own units: bytes, days, ms and pixels.

use std::time::Duration;

use kx_module_api::ports::{ClipConfig, ClipWait, MAX_BLOB_BYTES, MB, PictureConfig};

use super::ClipSettings;
use crate::entry::EntryConfig;
use crate::follow::Follow;
use crate::insert::JoinConfig;
use crate::keep::Limits;
use crate::media::Sides;
use crate::panel::PanelConfig;
use crate::store::StoreConfig;

impl ClipSettings {
    /// The history's limits, in bytes and days.
    #[must_use]
    pub fn limits(&self) -> Limits {
        Limits {
            keep_days: self.keep_days,
            media_days: self.media_days,
            max_entries: self.max_entries,
            max_bytes: mb(self.max_mb),
            min_free_bytes: mb(self.min_free_mb),
            max_entry_bytes: mb(self.max_entry_mb),
        }
    }

    /// What building an entry needs.
    #[must_use]
    pub fn entry_config(&self) -> EntryConfig {
        EntryConfig {
            title_chars: self.title_chars,
            search_chars: self.search_chars,
        }
    }

    /// What the store needs; a blob holds the port's most.
    #[must_use]
    pub fn store_config(&self) -> StoreConfig {
        StoreConfig {
            thumb_px: self.thumb_px,
            blob_bytes: MAX_BLOB_BYTES,
        }
    }

    /// What the panel needs; its rows per page come from the ui (`ClipAct::Rows`), so they start at none.
    #[must_use]
    pub fn panel_config(&self) -> PanelConfig {
        PanelConfig {
            rows: 0,
            query_chars: self.search_chars,
        }
    }

    /// How the platform waits for the clipboard.
    #[must_use]
    pub fn wait(&self) -> ClipWait {
        ClipWait {
            open_ms: self.open_ms,
            poll_ms: self.poll_ms,
        }
    }

    /// How the platform watches the clipboard: copies up to one entry's cap are read.
    #[must_use]
    pub fn clip_config(&self) -> ClipConfig {
        ClipConfig {
            max_bytes: mb(self.max_entry_mb),
            wait: self.wait(),
        }
    }

    /// How the platform runs the media worker.
    #[must_use]
    pub fn picture_config(&self) -> PictureConfig {
        PictureConfig {
            deadline_ms: self.media_ms,
            idle_s: self.worker_idle_s,
            memory_mb: self.worker_mb,
            max_pixels: self.max_pixels,
            restart_ms: self.restart_ms,
            restart_max_ms: self.restart_max_ms,
        }
    }

    /// What an insert pastes between two texts.
    #[must_use]
    pub fn join_config(&self) -> JoinConfig {
        JoinConfig {
            breaks: self.join_breaks,
            rule: self.join_rule.clone(),
        }
    }

    /// The sides pictures are drawn at.
    #[must_use]
    pub fn sides(&self) -> Sides {
        Sides {
            thumb: self.thumb_px,
            preview: self.preview_px,
        }
    }

    /// Follow-the-clear and the skipped apps.
    #[must_use]
    pub fn follow(&self) -> Follow {
        Follow::new(self.clear_window(), self.skip_apps.clone())
    }

    /// How long a clear follows a copy.
    #[must_use]
    pub fn clear_window(&self) -> Duration {
        Duration::from_secs(self.clear_follow_s.into())
    }

    /// The save's wait after a first change.
    #[must_use]
    pub fn save(&self) -> Duration {
        Duration::from_millis(self.save_ms.into())
    }

    /// The wait after each pasted step.
    #[must_use]
    pub fn paste_gap(&self) -> Duration {
        Duration::from_millis(self.paste_gap_ms.into())
    }

    /// The wait for the keyboard's paste.
    #[must_use]
    pub fn paste_wait(&self) -> Duration {
        Duration::from_millis(self.paste_wait_ms.into())
    }

    /// The wait for an insert's file check.
    #[must_use]
    pub fn check_wait(&self) -> Duration {
        Duration::from_millis(self.check_ms.into())
    }
}

/// `n` megabytes in bytes.
fn mb(n: u32) -> u64 {
    u64::from(n) * MB
}
