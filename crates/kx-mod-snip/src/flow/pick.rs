//! Picking and the strip: the capture bar's ways, a part taken as a picture or a video to record, and the strip's keys,
//! which copy, save, edit a picture, switch the sounds, record, play or speed up a recorded video, or close.

use kx_module_api::ports::{BarKey, Frame, Keep, Part, Point, StripKey};

use super::{After, Edit, Flow, Held, Out, Picking, Sounds, Stage, Strip};
use crate::part::{self, Pick};

/// A way that records.
const fn video(way: BarKey) -> bool {
    matches!(way, BarKey::VideoRegion | BarKey::VideoFull)
}

impl Flow {
    /// A capture bar key while picking: a way, which drops the corner, Close, or the settings key, which ends the snip
    /// and opens the Snip panel.
    pub(super) fn bar(&mut self, key: BarKey, out: &mut Vec<Out>) {
        let Stage::Picking(picking) = &mut self.stage else {
            return;
        };
        match key {
            BarKey::Close => self.end(out),
            BarKey::Settings => {
                self.end(out);
                out.push(Out::Panel);
            }
            way => *picking = Picking { way, corner: None },
        }
    }

    /// A click on the frozen screen: while picking, read by the part rules for the way picked, a region's corner, the
    /// same spot twice, which drops it, or a part taken; on a region's strip, by the frame rules.
    pub(super) fn spot(&mut self, at: Point) {
        if matches!(self.stage, Stage::Strip(_)) {
            return self.adjust(at);
        }
        let (Stage::Picking(picking), Some(frozen)) = (&mut self.stage, &self.frozen) else {
            return;
        };
        match part::read(picking.way, picking.corner, at, frozen) {
            Some(Pick::Corner(corner)) => picking.corner = Some(corner),
            Some(Pick::Unset) => picking.corner = None,
            Some(Pick::Part(part)) => self.take(part),
            None => {}
        }
    }

    /// Takes `part` with the screen holding it: a picture with its pixels cut from the freeze, or a video ready to
    /// record with the start switches, which holds none; a region's with its frame. A part on no screen is not taken.
    fn take(&mut self, part: Part) {
        let (Stage::Picking(picking), Some(frozen)) = (&self.stage, &self.frozen) else {
            return;
        };
        let Some(home) = part::home(part, &frozen.screens) else {
            return;
        };
        let sounds = Sounds {
            mic: self.settings.mic_on,
            sound: self.settings.sound_on,
        };
        let (held, min) = if video(picking.way) {
            // A video's sides are even, so its least side is too (edge 2).
            let min = self.settings.frame_min_px.next_multiple_of(part::PAIR);
            (Held::Ready(sounds), min)
        } else {
            let Some(shot) = part.cut(&frozen.screens) else {
                return;
            };
            (Held::Picture(shot), self.settings.frame_min_px)
        };
        let frame = part::within(picking.way, part, &frozen.screens).map(|within| Frame {
            within,
            min,
            grip: None,
        });
        self.stage = Stage::Strip(Strip {
            part,
            home,
            held,
            done: None,
            frame,
        });
        self.fresh();
    }

    /// A strip key; none acts while a handle is held, as the strip hides, and while a tick that ends the snip shows,
    /// only Close acts, and ends the kept snip.
    pub(super) fn strip(&mut self, key: StripKey, out: &mut Vec<Out>) {
        let ending = self.due.is_some_and(|due| due.ends);
        if !self.free() || ending && key != StripKey::Close {
            return;
        }
        let Stage::Strip(strip) = &mut self.stage else {
            return;
        };
        if strip.gripped() {
            return;
        }
        let taken = !matches!(strip.held, Held::Ready(_));
        match (key, &mut strip.held) {
            (StripKey::Close, _) => self.close(After::End, out),
            (StripKey::Mic, Held::Ready(sounds)) => sounds.mic = !sounds.mic,
            (StripKey::Sound, Held::Ready(sounds)) => sounds.sound = !sounds.sound,
            (StripKey::Record, Held::Ready(_)) => self.count(out),
            (StripKey::Play, Held::Video(reel)) => {
                let reel = *reel;
                self.play(reel, out);
            }
            (StripKey::Speed, Held::Video(reel)) => {
                reel.speed = reel.speed.next();
                self.change();
            }
            (StripKey::Copy, _) if taken => self.keep(Keep::Copy, None, out),
            (StripKey::Save, _) if taken => self.keep(Keep::Save, None, out),
            (StripKey::Edit, Held::Picture(_)) => self.open(out),
            _ => {}
        }
    }

    /// Edit: the editor opens on the picture, the frozen screen ends and the keyboard comes back.
    fn open(&mut self, out: &mut Vec<Out>) {
        let Stage::Strip(strip) = &self.stage else {
            return;
        };
        let Some(shot) = strip.picture() else {
            return;
        };
        out.push(Out::Edit(shot.clone()));
        self.stage = Stage::Edit(Edit { done: None });
        self.frozen = None;
        self.release(out);
    }
}
