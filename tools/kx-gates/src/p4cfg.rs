//! `[gate_input]` in `kx-gates-p4.toml`: the product's test build, which P4's gates start and drive.
#![cfg(windows)]

use serde::Deserialize;

/// The product's test build: built with its gate-input feature and started with `flag`, its engine hears only
/// kx-gates' simulated input.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateInput {
    /// The test build's program; a relative path starts at the settings folder.
    pub program: String,
    /// The flag that makes its engine act on kx-gates' simulated input only; kx-gates' code never holds this text.
    pub flag: String,
    /// The warning it logs at start with the flag; without it, its engine might act on the hand.
    pub warn_line: String,
    /// The line its mouse module logs with the hook's numbers at each hook stop.
    pub hook_line: String,
    /// Longest wait for its keyboard and its warning line after its start, in ms.
    pub start_ms: u64,
    /// Longest wait for a mode or latch key to read on or off after its click, in ms.
    pub lit_ms: u64,
}

impl GateInput {
    /// Refuses an empty text or a zero wait.
    pub fn check(&self) -> Result<(), String> {
        let texts = [
            ("program", &self.program),
            ("flag", &self.flag),
            ("warn_line", &self.warn_line),
            ("hook_line", &self.hook_line),
        ];
        if let Some((key, _)) = texts.iter().find(|(_, text)| text.is_empty()) {
            return Err(format!("gate_input.{key} must not be empty"));
        }
        if self.start_ms == 0 || self.lit_ms == 0 {
            return Err("gate_input.start_ms and lit_ms must be above 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
