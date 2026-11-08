//! One G19 run on the test build: the strip's reader, the gate's acts, and each selection case judged as the P10 design
//! says: seen with the clipboard untouched, the window in front kept, placed where `geom::anchor` puts it, and the
//! keycaps of the box's kind. The selected text is never kept: only its boxes are read.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use kx_mod_keyboard::KeyboardSettings;
use kx_mod_selection::SelSettings;
use kx_module_api::ports::{Rect, SelChip};
use kx_platform_windows::clip::{self, open};
use kx_platform_windows::screen;
use kx_platform_windows::uia::{self, Uia};
use kx_platform_windows::window::foreground;
use kx_theme::{Shape, Themes};
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, RECT};

use crate::apps::{Ctx, Opened};
use crate::clipkeep;
use crate::g19chips::act::Acts;
use crate::g19chips::clear::{self, Board, Clear};
use crate::g19chips::place;
use crate::g19chips::strip::{Shown, Strip, ids};
use crate::keymap::{Reader, centre};
use crate::p10cfg::{G19Product, selection_defaults};
use crate::simuser::keys_ours;

/// The characters of a selection read to tell it from a bare caret; they are never kept.
const PEEK_CHARS: usize = 1;
/// What a hung app's case says of its box, which is never asked.
const UNREAD: &str = "not read: the app hangs";

/// What held just before an act: the window in front, the clipboard's sequence number and the time.
pub struct Before {
    front: HWND,
    seq: u32,
}

/// The ms from the act's last input at `sent` to the strip's first sight at `seen`; none before the input.
pub fn wait_ms(sent: Instant, seen: Instant) -> u64 {
    u64::try_from(seen.saturating_duration_since(sent).as_millis()).unwrap_or(u64::MAX)
}

/// A case that must show no strip: `quiet` when none showed, and the box's kind unless it was not read.
fn quiet_json(name: &str, quiet: bool, focus: bool, kind: Option<String>) -> Value {
    let kind = kind.unwrap_or_else(|| UNREAD.to_string());
    json!({ "case": name, "chips": quiet, "focus": focus, "kind": kind })
}

/// A hung app's case: `quiet` when no strip showed, and its focus held when the hung `target` stayed in front.
fn hung_json(name: &str, quiet: bool, front: HWND, target: HWND) -> Value {
    quiet_json(name, quiet, front == target, None)
}

/// The run's readers and acts.
pub struct Gate<'a> {
    /// The run's settings.
    pub ctx: &'a Ctx<'a>,
    /// The test build's program, whose own files hold its settings.
    pub program: PathBuf,
    /// The strip of the test build's process.
    pub strip: Strip<'a>,
    /// The gate's acts.
    pub acts: Acts<'a>,
    /// A reader of the product's buttons.
    pub reader: Reader,
    /// The selection module's defaults, as the fresh data folder holds them.
    pub sel: SelSettings,
    uia: Uia,
    /// The shared sizes, whose `[chips]` gap sets where the strip should sit.
    shape: Shape,
    /// The keyboard's plate, bubble and size.
    board: Board<'a>,
}

impl<'a> Gate<'a> {
    /// The readers of the copy at `program` with process id `pid`.
    pub fn new(ctx: &'a Ctx<'a>, (pid, program): (u32, &Path)) -> Result<Self, String> {
        let themes = Themes::shipped().map_err(|e| format!("theme {e}"))?;
        let keyboard = KeyboardSettings::from_defaults()
            .map_err(|e| format!("the keyboard's defaults: {e}"))?;
        let corner_px = themes.shape.extra.corner_px[0];
        Ok(Self {
            ctx,
            program: program.to_path_buf(),
            strip: Strip::new(ctx, pid)?,
            acts: Acts::new(ctx),
            reader: Reader::new()?,
            sel: selection_defaults()?,
            uia: Uia::new().map_err(|e| e.to_string())?,
            shape: themes.shape,
            board: Board::new(ctx, pid, corner_px, keyboard.size),
        })
    }

    /// The settings.
    pub fn g(&self) -> &G19Product {
        &self.ctx.cfg.g19_product
    }

    /// What holds now, before an act.
    pub fn before() -> Before {
        Before {
            front: foreground(),
            seq: open::sequence(),
        }
    }

    /// The focused box's class and control type, for the notes; never its name or text.
    pub fn kind(&self) -> String {
        self.uia
            .focused()
            .map_or_else(|e| e.to_string(), |el| uia::kind(&el))
    }

    /// The boxes of the focused box's selection.
    pub fn boxes(&self) -> Result<Vec<RECT>, String> {
        let el = self.uia.focused().map_err(|e| e.to_string())?;
        let tp = self
            .uia
            .text_of(&el)
            .ok_or("the focused box has no text pattern")?;
        let sel = uia::selection(&tp, PEEK_CHARS).map_err(|e| e.to_string())?;
        Ok(sel.ok_or("the focused box holds no selection")?.boxes)
    }

    /// The UI Automation reader of text boxes.
    pub fn uia(&self) -> &Uia {
        &self.uia
    }

    /// What the strip keeps clear of now.
    pub fn clear(&self) -> Clear {
        self.board.clear(&self.reader)
    }

    /// The verdict on where `s` sits for the focused box's selection, clear of the plate or the bubble, landed where
    /// Qt puts a window at 125 % and every other scale.
    fn place(&self, s: &Shown) -> Result<Value, String> {
        let raw = self.boxes()?;
        let last = *raw.last().ok_or("the selection has no box")?;
        let rects: Vec<Rect> = raw.iter().map(place::rect).collect();
        let area = place::rect(&screen::work_area(&last).map_err(|e| e.to_string())?);
        let clear = self.clear();
        let gap = place::gap(&self.shape, clear.size, screen::dpi_of(&last));
        let got = place::rect(&s.rect);
        let want =
            place::want(&rects, (got.w, got.h), gap, area, &clear.boxes).ok_or("no line box")?;
        let origin = clear::origin(want).unwrap_or((area.x, area.y));
        let want = place::snap(want, origin, screen::dpi_of(&s.rect));
        let first = place::line(&rects, 0).unwrap_or_default();
        let end = place::line(&rects, rects.len().saturating_sub(1));
        let slack = self.g().place_slack_px;
        let mut v = place::judge(want, got, slack, &clear.boxes, first);
        v["clear"] = clear.note;
        v["boxes"] = json!(rects.len());
        v["one_line"] = json!(Some(first) == end);
        Ok(v)
    }

    /// Case `name` after an act from `before`: the strip shows within `strip_wait_ms` with the keycaps `want` names.
    pub fn case(
        &self,
        name: &str,
        before: &Before,
        want: &[String],
    ) -> Result<(Value, Option<Shown>), String> {
        self.case_within(name, before, want, self.g().strip_wait_ms)
    }

    /// `case`, waiting up to `most_ms` for the strip; `ms` runs from the act's last input to the strip's first sight.
    pub fn case_within(
        &self,
        name: &str,
        before: &Before,
        want: &[String],
        most_ms: u64,
    ) -> Result<(Value, Option<Shown>), String> {
        let shown = self.strip.within_at(most_ms)?;
        let untouched = open::sequence() == before.seq;
        let mut v =
            json!({ "case": name, "focus": foreground() == before.front, "kind": self.kind() });
        let Some((s, seen)) = shown else {
            v["seen"] = json!(false);
            v["chips"] = json!(false);
            return Ok((v, None));
        };
        let chips = ids(&s.chips(), self.g());
        let placed = self
            .place(&s)
            .unwrap_or_else(|e| json!({ "ok": false, "error": e }));
        v["seen"] = json!(untouched);
        v["chips"] = json!(chips == want);
        v["shown"] = json!(chips);
        v["ms"] = json!(wait_ms(self.acts.sent(), seen));
        v["place"] = placed["ok"].clone();
        v["placed"] = placed;
        Ok((v, Some(s)))
    }

    /// Case `name` after an act from `before`: no strip shows for `quiet_ms`.
    pub fn quiet(&self, name: &str, before: &Before) -> Result<Value, String> {
        let quiet = self.strip.quiet(self.g().quiet_ms)?;
        let focus = foreground() == before.front;
        Ok(quiet_json(name, quiet, focus, Some(self.kind())))
    }

    /// Case `name` on the hung app `target`, in front: no strip shows for `hung_quiet_ms`, it stays in front, and its
    /// box is never asked.
    pub fn quiet_hung(&self, name: &str, target: HWND) -> Result<Value, String> {
        let quiet = self.strip.quiet(self.g().hung_quiet_ms)?;
        Ok(hung_json(name, quiet, foreground(), target))
    }

    /// Clicks keycap `chip` on `s` while `app`'s tab is in front: whether the window in front stayed, and the
    /// clipboard's number before the click.
    pub fn tap(&self, app: &Opened, s: &Shown, chip: SelChip) -> Result<(bool, u32), String> {
        let (_, r) = (s.caps.iter())
            .find(|(c, _)| *c == chip)
            .ok_or_else(|| format!("the strip shows no {chip:?}"))?;
        keys_ours(app)?;
        let (front, seq) = (foreground(), open::sequence());
        self.acts.press_on(s.hwnd, centre(r))?;
        Ok((foreground() == front, seq))
    }

    /// True once the clipboard changed from `seq` and holds `want`, line ends aside; its text is never kept.
    pub fn clip_holds(&self, seq: u32, want: &str) -> bool {
        let t = &self.ctx.cfg.timing;
        let max = self.ctx.cfg.g1.max_read_chars;
        let read = || clip::read_text(clipkeep::wait(t), max).ok().flatten();
        open::wait_change(seq, clipkeep::wait(t)) && read().is_some_and(|c| c.trim_end() == want)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_wait_runs_from_the_last_input_and_never_below_none() {
        let sent = Instant::now();
        let seen = sent + Duration::from_millis(37);
        assert_eq!(
            wait_ms(sent, seen),
            37,
            "the settle after the act counts not"
        );
        assert_eq!(wait_ms(seen, sent), 0);
    }

    #[test]
    fn a_hung_apps_case_never_reads_its_box() {
        let hung = quiet_json("hung", true, true, None);
        assert_eq!(hung["kind"], UNREAD);
        assert_eq!(hung["chips"], true);
        let read = quiet_json("tray", false, true, Some("Edit".into()));
        assert_eq!(
            (read["kind"].as_str(), read["chips"].as_bool()),
            (Some("Edit"), Some(false))
        );
    }

    #[test]
    fn a_hung_case_whose_target_stayed_in_front_passes_the_focus_group() {
        use crate::g19chips::plan::{FOCUS, groups};
        let (target, notepad) = (
            HWND(std::ptr::without_provenance_mut(1)),
            HWND(std::ptr::without_provenance_mut(2)),
        );
        let good = json!({ "seen": true, "focus": true, "place": true, "chips": true });
        let stayed = hung_json("hung", true, target, target);
        assert_eq!(groups(&[good.clone(), stayed])[FOCUS]["ok"], true);
        let lost = hung_json("hung", true, notepad, target);
        assert_eq!(
            groups(&[good, lost])[FOCUS]["ok"],
            false,
            "Notepad came in front"
        );
    }

    #[test]
    fn a_selection_is_peeked_at_one_character_at_most() {
        assert_eq!(
            PEEK_CHARS, 1,
            "enough to tell text from a caret, and no more"
        );
    }
}
