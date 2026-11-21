//! G22 on the product (`[g22_product]`): the voice gate build beside the bench's gate worker, the layout each clip's
//! language takes, the device its copy runs on, its waits and the bar's gate ids.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::env::consts::EXE_SUFFIX;
use std::path::Path;

use kx_module_api::ports::DevicePick;
use serde::Deserialize;

use super::G22Bench;
use crate::langkey::lang_id;
use crate::p5cfg::SEP;
use crate::product::NAME;

/// G22 on the product (`g22 keyxtend`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G22Product {
    /// The app built with `gate-voice`, relative to the settings folder, in the gate worker's own folder.
    pub program: String,
    /// The keyboard layout (LANGID, 4 hex digits) target-window takes for each clip's language.
    pub layouts: BTreeMap<String, String>,
    /// Where the copy's models run, as `[voice] device` takes it.
    pub device: String,
    /// Wait after the copy's keyboard shows before the first click, so its models folder is read, in ms.
    pub ready_ms: u64,
    /// With a card in `device`, the wait after a clip's layout is in front before Dictation, so the card's check of
    /// that language's model can end, in ms.
    pub card_wait_ms: u64,
    /// Longest wait for the bar to show after the Dictation click, in ms.
    pub bar_wait_ms: u64,
    /// Longest wait for target-window to type in a clip's layout, in ms.
    pub layout_wait_ms: u64,
    /// Longest wait after the bar shows for the copy's worker to load the model and start the clip, in ms.
    pub open_ms: u64,
    /// How long the typed text must stay the same after Stop to count as done, in ms.
    pub settle_ms: u64,
    /// The line the voice gate build logs at start with its clip; without it the copy may hear the microphone.
    pub warn_line: String,
    /// The bar's gate-id group (kx-ui `VoiceNames.group`).
    pub group: String,
    /// Stop's id on the bar (kx-ui `bar_button_id`).
    pub stop: String,
    /// The copy's `[kernel] log_level`; it must keep `LINES_LEVEL`'s lines.
    pub log_level: String,
    /// What marks the copy's log lines a failed language prints, each found as written.
    pub why_marks: Vec<String>,
    /// The voice module's line as a text goes to the keyboard, which ends Stop to text.
    pub typed_line: String,
}

impl G22Product {
    /// The level of the stage and typed lines, which the copy's log must keep.
    pub const LINES_LEVEL: &str = "debug";

    /// Stop's gate id.
    #[cfg_attr(
        not(feature = "voice"),
        allow(dead_code, reason = "Only the voice gates call it.")
    )]
    pub fn stop_id(&self) -> String {
        format!("{}{SEP}{}", self.group, self.stop)
    }

    /// The LANGID of `lang`'s layout; none for a language without one.
    pub fn layout(&self, lang: &str) -> Option<u16> {
        self.layouts.get(lang).and_then(|l| lang_id(l))
    }

    /// The device pick `device` names; none for a name `[voice] device` refuses.
    pub fn pick(&self) -> Option<DevicePick> {
        DevicePick::parse(&self.device)
    }

    /// Puts `pick`, the run's `--device`, in place of `device` for this run; none keeps the file's.
    pub fn set_pick(&mut self, pick: Option<&DevicePick>) {
        if let Some(pick) = pick {
            pick.name().clone_into(&mut self.device);
        }
    }

    /// The wait before Dictation once a clip's layout is in front: `card_wait_ms` with a card in `device`, else none.
    #[cfg_attr(
        not(feature = "voice"),
        allow(dead_code, reason = "Only the voice gates call it.")
    )]
    pub fn dictate_wait_ms(&self) -> u64 {
        match self.pick() {
            Some(DevicePick::Card(_)) => self.card_wait_ms,
            _ => 0,
        }
    }

    /// Refuses settings the run could not judge by, naming the first bad key; `bench` holds the clips and worker.
    pub fn check(&self, bench: &G22Bench) -> Result<(), String> {
        self.check_program(bench)?;
        let langs: Vec<&String> = bench.clips.keys().collect();
        let mine: Vec<&String> = self.layouts.keys().collect();
        if langs != mine || self.layouts.keys().any(|l| self.layout(l).is_none()) {
            return Err(format!(
                "g22_product.layouts must give {langs:?} each a 4-digit hex LANGID"
            ));
        }
        if self.pick().is_none() {
            return Err("g22_product.device must be one [voice] device takes".into());
        }
        self.check_lines()?;
        let waits = [
            self.ready_ms,
            self.card_wait_ms,
            self.bar_wait_ms,
            self.layout_wait_ms,
            self.open_ms,
        ];
        if waits.contains(&0) || self.settle_ms == 0 {
            return Err("g22_product's waits must be above 0".into());
        }
        let named = [&self.warn_line, &self.group, &self.stop];
        if named.iter().any(|t| t.trim().is_empty()) || self.group.contains(SEP) {
            return Err(format!(
                "g22_product needs a warn_line, a group without {SEP:?} and stop"
            ));
        }
        self.check_marks()
    }

    /// Refuses a log level that drops the timing lines, or a blank typed line.
    fn check_lines(&self) -> Result<(), String> {
        let level = kx_kernel::log::level(&self.log_level);
        if level.is_none() || level < kx_kernel::log::level(Self::LINES_LEVEL) {
            return Err(format!(
                "g22_product.log_level must be a log level name that keeps {} lines",
                Self::LINES_LEVEL
            ));
        }
        if self.typed_line.trim().is_empty() {
            return Err("g22_product needs a typed_line".into());
        }
        Ok(())
    }

    /// Refuses an empty `why_marks` or a blank mark in it.
    fn check_marks(&self) -> Result<(), String> {
        let blank = self.why_marks.iter().any(|m| m.trim().is_empty());
        if self.why_marks.is_empty() || blank {
            return Err("g22_product.why_marks must list marks that are not blank".into());
        }
        Ok(())
    }

    /// The program names the product and sits in the bench's gate-worker folder, so the copy starts the gate worker.
    fn check_program(&self, bench: &G22Bench) -> Result<(), String> {
        let (program, worker) = (Path::new(&self.program), Path::new(&bench.worker));
        let exe = format!("{NAME}{EXE_SUFFIX}");
        let named = program
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case(&exe));
        if !named || program.parent() != worker.parent() {
            return Err(format!(
                "g22_product.program must be {exe} beside g22_bench.worker"
            ));
        }
        Ok(())
    }
}
