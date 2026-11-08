//! `[g19_product]` as kx-gates-p10.toml writes it.
#![cfg(windows)]

use serde::Deserialize;

/// The classic console the gate opens on its own script.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Console {
    /// Its program.
    pub exe: String,
    /// Its arguments; `{input}` becomes the script.
    pub args: Vec<String>,
    /// Its window class.
    pub class: String,
}

/// `[g19_product]`: the gate's own texts, the strip's names, the keys and the waits; times in ms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G19Product {
    /// Notepad's line that Copy copies.
    pub copy_line: String,
    /// Notepad's line that Cut cuts.
    pub cut_line: String,
    /// Notepad's line that Paste replaces.
    pub paste_line: String,
    /// One Arabic sentence.
    pub arabic: String,
    /// How often the long Translate line holds `arabic`.
    pub arabic_repeat: usize,
    /// The long line's least length, in characters.
    pub long_min_chars: usize,
    /// The page's read-only paragraph.
    pub page_text: String,
    /// The page's text box's line.
    pub box_text: String,
    /// The text box's name.
    pub box_name: String,
    /// The password box's name.
    pub pass_name: String,
    /// The password box's test value, never read.
    pub pass_value: String,
    /// The unknown box's name.
    pub unknown_name: String,
    /// The unknown box's role.
    pub unknown_role: String,
    /// The page's style.
    pub css: String,
    /// The DOM code of the copy key.
    pub copy_code: String,
    /// The title mark before the Ctrl+C count.
    pub copy_mark: String,
    /// The title mark before the Ctrl+Shift+C count.
    pub shift_mark: String,
    /// The terminals' line.
    pub term_line: String,
    /// The host `ping -t` pings.
    pub ping_host: String,
    /// The strip window title's text key.
    pub strip_title: String,
    /// The keycaps' UI Automation group.
    pub group: String,
    /// The keycaps' ids, in `SelChip::ALL`'s order.
    pub chips: Vec<String>,
    /// An editable box's keycaps.
    pub editable_chips: Vec<String>,
    /// A page's keycaps.
    pub page_chips: Vec<String>,
    /// A terminal's keycaps.
    pub terminal_chips: Vec<String>,
    /// A bare caret's keycaps.
    pub caret_chips: Vec<String>,
    /// The keyboard's close-to-tray button, by UI Automation id.
    pub tray: String,
    /// Browser programs.
    pub browsers: Vec<String>,
    /// Shift+Home.
    pub select_home: Vec<u16>,
    /// One caret step left.
    pub caret_left: Vec<u16>,
    /// Num Lock.
    pub numlock: Vec<u16>,
    /// How far the strip may sit from its expected place, in px.
    pub place_slack_px: i32,
    /// Wait after the keyboard shows.
    pub ready_ms: u64,
    /// Longest wait for the strip to show.
    pub strip_wait_ms: u64,
    /// Longest wait for the strip to go.
    pub gone_wait_ms: u64,
    /// How long no strip must show.
    pub quiet_ms: u64,
    /// Pause after each act.
    pub settle_ms: u64,
    /// How long target-window hangs.
    pub hang_ms: u64,
    /// How long after the hang message the gate acts.
    pub hang_lead_ms: u64,
    /// How long the ping must keep writing.
    pub ping_ms: u64,
    /// The caret Paste's slack against `caret_s`.
    pub caret_slack_ms: u64,
    /// Longest wait for the owner's translate page.
    pub owner_wait_ms: u64,
    /// The classic console.
    pub console: Console,
}
