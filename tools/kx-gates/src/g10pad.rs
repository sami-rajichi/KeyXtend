//! G10 on the scroll pad (spec §5.2): each arrow and Stop must be found by UI Automation, named and pressable, and,
//! with the pointer resting on the pad, a press must light an arrow with its speed, a second go fast and Stop end it.
//! The element at each arrow's tap point, as Narrator's mouse mode reads it, is recorded only.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::{Ctx, Opened};
use crate::keymap::{KeyRead, Reader};
use crate::mouse;
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
/// changed that speed, and Stop left both silent.
fn answered(slow: &Said, fast: &Said, stopped: &Said) -> bool {
    lit_then_stopped(slow, stopped) && lit(fast) && fast.arrow != slow.arrow
}

/// True when the arrow reads a speed and Stop's dots read the same one.
fn lit(s: &Said) -> bool {
    !s.arrow.is_empty() && s.arrow == s.stop
}

/// True when a press lit the arrow with Stop's speed, and Stop then left both silent.
fn lit_then_stopped(once: &Said, stopped: &Said) -> bool {
    lit(once) && stopped.arrow.is_empty() && stopped.stop.is_empty()
}

/// Where G10 presses the pad: the keyboard and its reader.
struct Desk<'a> {
    ctx: &'a Ctx<'a>,
    kb: &'a Keyboard,
    reader: Reader,
}

impl Desk<'_> {
    /// Presses part `id` through UI Automation, then waits the pad's settle; what the arrow towards `dir` and Stop say.
    fn press(&self, id: &str, dir: Dir) -> Result<Said, String> {
        self.reader.invoke(self.kb.hwnd, id)?;
        sleep_ms(self.ctx.cfg.pad.settle_ms);
        self.said(dir)
    }

    /// What the arrow towards `dir` and Stop say now.
    fn said(&self, dir: Dir) -> Result<Said, String> {
        let pad = &self.ctx.cfg.pad;
        Ok(Said {
            arrow: self.reader.described(self.kb.hwnd, &pad.arrow(dir))?,
            stop: self.reader.described(self.kb.hwnd, &pad.stop_id())?,
        })
    }

    /// Presses down twice then Stop, and each other way once then Stop: whether every press answered, what Stop lacks
    /// while it shows, and what each press said, by way.
    fn presses(&self) -> Result<(bool, Vec<String>, Value), String> {
        let (pad, down) = (&self.ctx.cfg.pad, Dir::Down);
        let slow = self.press(&pad.arrow(down), down)?;
        let stop = faults(&self.reader.walk(self.kb.hwnd)?, &[pad.stop_id()]);
        let fast = self.press(&pad.arrow(down), down)?;
        let stopped = self.press(&pad.stop_id(), down)?;
        let mut ok = answered(&slow, &fast, &stopped);
        let mut said = vec![json!({ "dir": down, "slow": slow, "fast": fast, "stopped": stopped })];
        for dir in [Dir::Right, Dir::Left, Dir::Up] {
            let once = self.press(&pad.arrow(dir), dir)?;
            let stopped = self.press(&pad.stop_id(), dir)?;
            ok &= lit_then_stopped(&once, &stopped);
            said.push(json!({ "dir": dir, "once": once, "stopped": stopped }));
        }
        Ok((ok, stop, json!(said)))
    }
}

/// The UI Automation id at each arrow's tap point, by way; recorded only, as Qt hit-tests a turned part by its box.
fn by_point(reader: &Reader, pin: &Pinned) -> Value {
    let id_at = |p: POINT| reader.id_at(p).unwrap_or_else(|e| e);
    let found: Vec<Value> = Dir::ALL
        .into_iter()
        .map(|d| json!({ "dir": d, "found": id_at(pin.pad.arrow(d)) }))
        .collect();
    json!(found)
}

/// Runs G10's pad part on keyboard `kb`, whose buttons read as `read`, with `target` laid under it; the pointer leaves
/// the keyboard after, and Stop is pressed again if a press broke off.
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
    pin.glide_in(ctx, mouse::move_to)?;
    let points = by_point(&desk.reader, &pin);
    let pressed = desk.presses();
    if pressed.is_err() {
        let _ = desk.reader.restore(kb.hwnd, &pad.stop_id());
    }
    mouse::move_to(pin.start)?;
    let (answers, stop, said) = pressed?;
    found.extend(stop);
    Ok(json!({
        "ok": found.is_empty() && answers, "faults": found, "answered": answers, "said": said, "by_point": points,
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
    fn two_presses_go_slow_then_fast_and_stop_silences_both() {
        let (slow, fast, quiet) = (said("Slow", "Slow"), said("Fast", "Fast"), said("", ""));
        assert!(answered(&slow, &fast, &quiet));
        assert!(
            !answered(&slow, &slow, &quiet),
            "the second press kept the speed"
        );
        assert!(
            !answered(&slow, &said("Fast", "Slow"), &quiet),
            "the dots stayed slow"
        );
        assert!(
            !answered(&said("", ""), &fast, &quiet),
            "the first press lit nothing"
        );
        assert!(
            !answered(&slow, &fast, &said("Fast", "")),
            "Stop did not stop it"
        );
    }

    #[test]
    fn one_press_lights_its_arrow_and_stop_silences_it() {
        let quiet = said("", "");
        assert!(lit_then_stopped(&said("Slow", "Slow"), &quiet));
        assert!(!lit_then_stopped(&quiet, &quiet), "the press did nothing");
        assert!(!lit_then_stopped(
            &said("Slow", "Slow"),
            &said("Slow", "Slow")
        ));
    }
}
