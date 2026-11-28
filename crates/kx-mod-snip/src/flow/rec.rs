//! A video: Record counts down `countdown_s` on the live screen, then records; the record bar switches the sounds,
//! stops or cancels; and the run's notes tell the time, its end with the video kept, a lost sound or a fault.

use kx_module_api::Mono;
use kx_module_api::ports::{
    RecKey, RecordDone, RecordEnd, RecordError, RecordNote, SnipNote, Source, Speed,
};

use super::{
    Call, Count, Film, Flow, Held, Out, Rec, Recorded, Reel, SECOND, Sounds, Stage, Strip, Video,
};

/// The note a fault shows: before the run started, why it could not; after, that it failed.
const fn why(error: RecordError, live: bool) -> SnipNote {
    match (error, live) {
        (RecordError::Full, false) => SnipNote::DiskLow,
        (RecordError::NoVideo, false) => SnipNote::NoVideo,
        _ => SnipNote::RecordFailed,
    }
}

/// The sound whose device a fault names: a microphone missing or refused, or no sound device (edge 11).
const fn device(error: RecordError) -> Option<Source> {
    match error {
        RecordError::NoMic | RecordError::MicRefused => Some(Source::Mic),
        RecordError::NoSound => Some(Source::Sound),
        _ => None,
    }
}

/// Turns `source`'s switch off in `sounds`; the note saying it could not record.
const fn mute(sounds: &mut Sounds, source: Source) -> SnipNote {
    match source {
        Source::Mic => {
            sounds.mic = false;
            SnipNote::MicFailed
        }
        Source::Sound => {
            sounds.sound = false;
            SnipNote::SoundFailed
        }
    }
}

impl Flow {
    /// Record: the frozen screen lifts and the countdown starts, or the video records at once without one.
    pub(super) fn count(&mut self, out: &mut Vec<Out>) {
        let Stage::Strip(strip) = &self.stage else {
            return;
        };
        let Held::Ready(sounds) = strip.held else {
            return;
        };
        let film = Film {
            part: strip.part,
            home: strip.home,
            sounds,
        };
        self.frozen = None;
        let left_s = self.settings.countdown_s;
        if left_s == 0 {
            return self.record(film, out);
        }
        let next = self.now().saturating_add(SECOND);
        self.stage = Stage::Count(Count { film, left_s, next });
    }

    /// Each second passed by `now` goes; the last one starts the recording.
    pub(super) fn count_down(&mut self, now: Mono, out: &mut Vec<Out>) {
        let Stage::Count(count) = &mut self.stage else {
            return;
        };
        while count.left_s > 0 && now >= count.next {
            count.left_s -= 1;
            count.next = count.next.saturating_add(SECOND);
        }
        if count.left_s == 0 {
            let film = count.film.clone();
            self.record(film, out);
        }
    }

    /// Records `film` into a new video.
    fn record(&mut self, film: Film, out: &mut Vec<Out>) {
        let video = self.video();
        let (part, mic, sound) = (film.part, film.sounds.mic, film.sounds.sound);
        out.push(Out::Record {
            video,
            part,
            mic,
            sound,
        });
        self.stage = Stage::Record(Rec {
            film,
            video,
            ms: 0,
            live: false,
            stopping: false,
        });
    }

    /// A record bar key: a sound switch, Stop, or Cancel, which also ends the countdown with no file (edge 10).
    pub(super) fn rec_key(&mut self, key: RecKey, out: &mut Vec<Out>) {
        let filming = matches!(self.stage, Stage::Count(_) | Stage::Record(_));
        match key {
            RecKey::Cancel if filming => self.end(out),
            RecKey::Stop => self.stop(out),
            RecKey::Mic | RecKey::Sound => self.switch(key == RecKey::Mic, out),
            RecKey::Cancel => {}
        }
    }

    /// Switches the microphone, or else the computer's sound, of a recording not stopping.
    fn switch(&mut self, mic: bool, out: &mut Vec<Out>) {
        let Stage::Record(rec) = &mut self.stage else {
            return;
        };
        if rec.stopping {
            return;
        }
        let sounds = &mut rec.film.sounds;
        let call = if mic {
            sounds.mic = !sounds.mic;
            Call::Mic(sounds.mic)
        } else {
            sounds.sound = !sounds.sound;
            Call::Sound(sounds.sound)
        };
        out.push(Out::Run {
            video: rec.video,
            call,
        });
    }

    /// A capture window could not be kept out of captures, so its record bar may hide: a countdown ends with no file,
    /// and a recording stops as Stop does, keeping its video; a note says why, once.
    pub(super) fn bare(&mut self, out: &mut Vec<Out>) {
        let stop: fn(&mut Self, &mut Vec<Out>) = match &self.stage {
            Stage::Count(_) => Self::end,
            Stage::Record(rec) if !rec.stopping => Self::stop,
            _ => return,
        };
        out.push(Out::Note(SnipNote::Bare));
        stop(self, out);
    }

    /// Stop, once: the run ends with its video kept, and its end shows the strip.
    pub(super) fn stop(&mut self, out: &mut Vec<Out>) {
        let Stage::Record(rec) = &mut self.stage else {
            return;
        };
        if !rec.stopping {
            rec.stopping = true;
            out.push(Out::Run {
                video: rec.video,
                call: Call::Stop,
            });
        }
    }

    /// A note of recording `video`; one of a run the flow left is `stale`.
    pub(super) fn note(&mut self, video: Video, note: RecordNote, out: &mut Vec<Out>) {
        let Stage::Record(rec) = &mut self.stage else {
            return stale(video, note, out);
        };
        if rec.video != video {
            return stale(video, note, out);
        }
        match note {
            RecordNote::Time { ms } => (rec.ms, rec.live) = (ms, true),
            RecordNote::Lost(source) => out.push(Out::Note(mute(&mut rec.film.sounds, source))),
            RecordNote::Done(done) => self.recorded(done, out),
            RecordNote::Fault(error) => self.fault(error, out),
        }
    }

    /// A run's fault: a listed device that failed before the run went live turns its switch off and shows the strip
    /// again; any other fault notes why and ends the snip.
    fn fault(&mut self, error: RecordError, out: &mut Vec<Out>) {
        let Stage::Record(rec) = &self.stage else {
            return;
        };
        let Some(source) = device(error).filter(|_| !rec.live) else {
            out.push(Out::Note(why(error, rec.live)));
            return self.wrap(out);
        };
        let mut film = rec.film.clone();
        out.push(Out::Note(mute(&mut film.sounds, source)));
        self.stage = Stage::Strip(Strip {
            part: film.part,
            home: film.home,
            held: Held::Ready(film.sounds),
            done: None,
            frame: None,
        });
    }

    /// The run ended with its video kept: the strip shows its length, with a note when it stopped by itself.
    fn recorded(&mut self, done: RecordDone, out: &mut Vec<Out>) {
        let Stage::Record(rec) = &self.stage else {
            return;
        };
        let (film, video) = (rec.film.clone(), rec.video);
        self.files.add(video);
        match done.end {
            RecordEnd::Longest => out.push(Out::Note(SnipNote::Longest)),
            RecordEnd::Full => out.push(Out::Note(SnipNote::DiskLow)),
            RecordEnd::Stopped => {}
        }
        let clip = Recorded {
            video,
            length_ms: done.length_ms,
        };
        let (part, home) = (film.part, film.home);
        let held = Held::Video(Reel {
            clip,
            speed: Speed::PLAIN,
        });
        self.stage = Stage::Strip(Strip {
            part,
            home,
            held,
            done: None,
            frame: None,
        });
        self.fresh();
    }
}

/// A note of a run the flow left: a kept video is deleted, a run still telling is cancelled, a fault's host deleted it.
fn stale(video: Video, note: RecordNote, out: &mut Vec<Out>) {
    match note {
        RecordNote::Done(_) => out.push(Out::Delete(video)),
        RecordNote::Time { .. } | RecordNote::Lost(_) => out.push(Out::Run {
            video,
            call: Call::Cancel,
        }),
        RecordNote::Fault(_) => {}
    }
}
