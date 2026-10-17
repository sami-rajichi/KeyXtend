//! G10 on the scroll pad (spec §5.2): each arrow and Stop must be found by UI Automation, named and pressable, and,
//! with the pointer resting on the pad, a press must light an arrow with its speed, a second go fast and Stop end it.
//! The element at each arrow's tap point, as Narrator's mouse mode reads it, must be that arrow.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::RECT;

use crate::apps::{Ctx, Opened};
use crate::keymap::{KeyRead, Reader};
use crate::mouse;
use crate::padread::PadRead;
use crate::pin::Pinned;
use crate::product::Keyboard;
use crate::win::sleep_ms;

/// What a screen reader reads as the pressed arrow's and Stop's descriptions after one press.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Said {
    /// The arrow's: its speed while it scrolls.
    pub arrow: String,
    /// Stop's: the speed its dots show.
    pub stop: String,
}

/// The pad's parts by id that UI Automation lacks, leaves unnamed or cannot press, each with why.
fn faults(read: &[KeyRead], ids: &[String]) -> Vec<String> {
    let fault = |id: &String| {
        let why = match read.iter().find(|k| k.id == *id) {
            None => "missing",
            Some(k) if k.name.is_empty() => "unnamed",
            Some(k) if !k.invokable => "cannot be pressed",
            Some(_) => return None,
        };
        Some(format!("{id}: {why}"))
    };
    ids.iter().filter_map(fault).collect()
}

/// True when the presses answered: the first lit the arrow with Stop's dots reading the same speed, the second
/// changed that speed, and Stop left the pad idle.
fn answered(slow: &Said, fast: &Said, stopped: bool) -> bool {
    lit_then_stopped(slow, stopped) && lit(fast) && fast.arrow != slow.arrow
}

/// True when the arrow reads a speed and Stop's dots read the same one.
fn lit(s: &Said) -> bool {
    !s.arrow.is_empty() && s.arrow == s.stop
}

/// True when a press lit the arrow with Stop's speed, and Stop then left the pad idle.
fn lit_then_stopped(once: &Said, stopped: bool) -> bool {
    lit(once) && stopped
}

/// True when the element at each arrow's tap point is that arrow, by the ids `want` and `found` in turn, and each
/// arrow has a box of its own among `boxes`, as roadmap G10 asks of Narrator's mouse mode.
fn point_judged(want: &[String], found: &[String], boxes: &[RECT]) -> bool {
    want == found && boxes.len() == want.len() && own_boxes(boxes)
}

/// True when no two of `boxes` are the same box.
fn own_boxes(boxes: &[RECT]) -> bool {
    boxes
        .iter()
        .enumerate()
        .all(|(i, a)| boxes[i + 1..].iter().all(|b| b != a))
}

/// Where G10 presses the pad: the keyboard and its reader.
struct Desk<'a> {
    ctx: &'a Ctx<'a>,
    kb: &'a Keyboard,
    reader: Reader,
}

impl Desk<'_> {
    /// The pad as UI Automation reads it.
    fn read(&self) -> PadRead<'_> {
        PadRead {
            reader: &self.reader,
            kb: self.kb.hwnd,
            pad: &self.ctx.cfg.pad,
        }
    }

    /// Presses the arrow towards `dir` through UI Automation and holds it `hold_ms`; what it and Stop say then.
    fn press(&self, dir: Dir, hold_ms: u64) -> Result<Said, String> {
        self.reader
            .invoke(self.kb.hwnd, &self.ctx.cfg.pad.arrow(dir))?;
        sleep_ms(hold_ms);
        let arrow = self.read().said(dir)?;
        let stop = if arrow.is_empty() {
            String::new()
        } else {
            self.read().stop_said()?
        };
        Ok(Said { arrow, stop })
    }

    /// Presses Stop unless the pad is idle already; whether it is idle after the pad's settle.
    fn halt(&self) -> Result<bool, String> {
        if self
            .read()
            .stop(|id| self.reader.invoke(self.kb.hwnd, id))?
        {
            sleep_ms(self.ctx.cfg.pad.settle_ms);
        }
        self.read().idle()
    }

    /// Presses down twice then Stop, and each other way once then Stop, right held `[pad] undo_div` times as long as
    /// left so left never runs back to the start: whether every press answered, what Stop lacks while it shows, and
    /// what each press said, by way.
    fn presses(&self) -> Result<(bool, Vec<String>, Value), String> {
        let (pad, down) = (&self.ctx.cfg.pad, Dir::Down);
        let settle = pad.settle_ms;
        let slow = self.press(down, settle)?;
        let stop = faults(&self.reader.walk(self.kb.hwnd)?, &[pad.stop_id()]);
        let fast = self.press(down, settle)?;
        let stopped = self.halt()?;
        let mut ok = answered(&slow, &fast, stopped);
        let mut said = vec![json!({ "dir": down, "slow": slow, "fast": fast, "stopped": stopped })];
        let right = settle.saturating_mul(u64::from(pad.undo_div));
        for (dir, hold) in [(Dir::Right, right), (Dir::Left, settle), (Dir::Up, settle)] {
            let once = self.press(dir, hold)?;
            let stopped = self.halt()?;
            ok &= lit_then_stopped(&once, stopped);
            said.push(json!({ "dir": dir, "once": once, "stopped": stopped }));
        }
        Ok((ok, stop, json!(said)))
    }

    /// Whether the element at each arrow's tap point on `pin`'s pad is that arrow, in a box of its own, with the id
    /// found at each, by way; the pointer never moves.
    fn by_point(&self, pin: &Pinned) -> Value {
        let pad = &self.ctx.cfg.pad;
        let want: Vec<String> = Dir::ALL.into_iter().map(|d| pad.arrow(d)).collect();
        let id_at = |d: Dir| self.reader.id_at(pin.pad.arrow(d)).unwrap_or_else(|e| e);
        let found: Vec<String> = Dir::ALL.into_iter().map(id_at).collect();
        let boxes: Vec<RECT> = want
            .iter()
            .filter_map(|id| self.reader.rect(self.kb.hwnd, id).ok())
            .collect();
        let ok = point_judged(&want, &found, &boxes);
        json!({ "ok": ok, "found": found, "own_boxes": own_boxes(&boxes) })
    }
}

/// Runs G10's pad part on keyboard `kb`, whose buttons read as `read`, with `target` laid under it; the pointer leaves
/// the keyboard after, and Stop is pressed again if a press broke off while the pad scrolls.
pub fn run(ctx: &Ctx, kb: &Keyboard, target: &Opened, read: &[KeyRead]) -> Result<Value, String> {
    let pad = &ctx.cfg.pad;
    let arrows: Vec<String> = Dir::ALL.into_iter().map(|d| pad.arrow(d)).collect();
    let mut found = faults(read, &arrows);
    let pin = Pinned::new(ctx, kb.hwnd, target)?;
    let desk = Desk {
        ctx,
        kb,
        reader: Reader::new()?,
    };
    pin.glide_in(ctx, target, mouse::move_to)?;
    let points = desk.by_point(&pin);
    let pressed = desk.presses();
    if pressed.is_err() {
        let _ = desk.read().stop(|id| desk.reader.restore(kb.hwnd, id));
    }
    mouse::move_to(pin.start)?;
    let (answers, stop, said) = pressed?;
    found.extend(stop);
    let pointed = points["ok"] == true;
    Ok(json!({
        "ok": found.is_empty() && answers && pointed, "faults": found, "answered": answers, "said": said,
        "by_point": points,
    }))
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::RECT;

    use super::*;

    fn key(id: &str, name: &str, invokable: bool) -> KeyRead {
        KeyRead {
            id: id.into(),
            name: name.into(),
            rect: RECT::default(),
            invokable,
        }
    }

    fn said(arrow: &str, stop: &str) -> Said {
        Said {
            arrow: arrow.into(),
            stop: stop.into(),
        }
    }

    #[test]
    fn a_part_must_be_found_named_and_pressable() {
        let read = [
            key("dpad.up", "Scroll up", true),
            key("dpad.left", "", true),
            key("dpad.right", "Scroll right", false),
        ];
        let ids: Vec<String> = ["dpad.up", "dpad.left", "dpad.right", "dpad.down"]
            .map(String::from)
            .to_vec();
        let got = faults(&read, &ids);
        assert_eq!(got.len(), 3, "{got:?}");
        assert!(got[0].starts_with("dpad.left") && got[0].contains("unnamed"));
        assert!(got[1].starts_with("dpad.right") && got[1].contains("press"));
        assert!(got[2].starts_with("dpad.down") && got[2].contains("missing"));
        assert!(faults(&read[..1], &ids[..1]).is_empty());
    }

    #[test]
    fn two_presses_go_slow_then_fast_and_stop_leaves_the_pad_idle() {
        let (slow, fast) = (said("Slow", "Slow"), said("Fast", "Fast"));
        assert!(answered(&slow, &fast, true));
        assert!(
            !answered(&slow, &slow, true),
            "the second press kept the speed"
        );
        assert!(
            !answered(&slow, &said("Fast", "Slow"), true),
            "the dots stayed slow"
        );
        assert!(
            !answered(&said("", ""), &fast, true),
            "the first press lit nothing"
        );
        assert!(!answered(&slow, &fast, false), "Stop did not stop it");
    }

    #[test]
    fn one_press_lights_its_arrow_and_stop_leaves_the_pad_idle() {
        assert!(lit_then_stopped(&said("Slow", "Slow"), true));
        assert!(
            !lit_then_stopped(&said("", ""), true),
            "the press did nothing"
        );
        assert!(!lit_then_stopped(&said("Slow", "Slow"), false));
    }

    #[test]
    fn each_arrows_tap_point_names_that_arrow_in_a_box_of_its_own() {
        let want: Vec<String> = ["dpad.up", "dpad.right", "dpad.down", "dpad.left"]
            .map(String::from)
            .to_vec();
        let at = |left, top| RECT {
            left,
            top,
            right: left + 10,
            bottom: top + 10,
        };
        let boxes = [at(0, 0), at(10, 0), at(10, 10), at(0, 10)];
        assert!(point_judged(&want, &want, &boxes));
        let mut swapped = want.clone();
        swapped.swap(0, 1);
        assert!(
            !point_judged(&want, &swapped, &boxes),
            "a tap point names its neighbour"
        );
        assert!(
            !point_judged(&want, &want, &[boxes[0]; 4]),
            "all four share one turned box"
        );
        assert!(
            !point_judged(&want, &want[..3], &boxes),
            "a point read nothing"
        );
    }
}
