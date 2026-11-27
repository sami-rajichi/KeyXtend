//! Recordings: each run starts in a job and each blocking call goes to one in turn, Cancel and a switch-off going
//! first. A run goes once its last note came or it was cancelled, so a broken video goes (edge 13).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;

use kx_module_api::ports::{
    Gear, Part, RecordAsk, RecordError, RecordNote, Recorder, SnipFolder, Source,
};

use super::run::{Ended, Run};
use super::{Actor, Back, Job, Leftovers, Msg};
use crate::flow::{Call, In, Video};

/// One recording's run, its calls waiting, and whether a job holds it.
#[derive(Default)]
struct Lane {
    run: Option<Run>,
    /// A job holds the run, for its start or a call.
    out: bool,
    calls: VecDeque<Call>,
    /// Its last note came, so it goes once no job holds it.
    over: bool,
}

impl Lane {
    /// Queues `call`: Cancel drops the calls before it, and a switch-off drops its source's calls and goes first.
    fn queue(&mut self, call: Call) {
        match call {
            Call::Cancel => {
                self.calls.clear();
                self.calls.push_back(call);
            }
            Call::Mic(false) | Call::Sound(false) => {
                self.calls.retain(|queued| !same_source(*queued, call));
                self.calls.push_front(call);
            }
            Call::Mic(true) | Call::Sound(true) | Call::Stop => self.calls.push_back(call),
        }
    }
}

/// True when `a` and `b` switch the same sound.
fn same_source(a: Call, b: Call) -> bool {
    matches!(
        (a, b),
        (Call::Mic(_), Call::Mic(_)) | (Call::Sound(_), Call::Sound(_))
    )
}

/// The recordings whose runs the actor holds.
#[derive(Default)]
pub(super) struct Runs(HashMap<Video, Lane>);

impl Runs {
    /// Drops every run the actor holds: one still recording is cancelled and its video deleted.
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

/// The ear of `video`'s note stream: it marks the run ended at its last note, then hands the note to the actor.
fn teller(video: Video, back: &Sender<Msg>, ended: Ended) -> Box<dyn Fn(RecordNote) + Send> {
    let back = back.clone();
    Box::new(move |note: RecordNote| {
        if note.ends() {
            ended.store(true, Ordering::Release);
        }
        let _ = back.send(Msg::Note { video, note });
    })
}

impl Actor {
    /// Reads what the system can record, off the actor.
    pub(super) fn read_facts(&self) {
        let (recorder, back) = (self.ports.recorder.clone(), self.ports.back.clone());
        let job: Job = Box::new(move || {
            let _ = back.send(Msg::Back(Back::Facts(recorder.facts())));
        });
        if !(self.ports.spawn)(job) {
            tracing::warn!("the recorder's facts job did not start");
        }
    }

    /// Records `part` into `video` with the snip's held settings; a sound that cannot record starts off.
    pub(super) fn record(&mut self, video: Video, part: Part, mic: bool, sound: bool) {
        let ask = self.ask(part, mic, sound);
        let (recorder, back, folder, leftovers) = self.kit_run();
        let job: Job = Box::new(move || {
            let ended = Ended::default();
            let tell = teller(video, &back, Arc::clone(&ended));
            let run = recorder.start(ask, tell);
            let run = run.map(|run| Run::new(run, ended, folder, leftovers));
            let _ = back.send(Msg::Back(Back::Started { video, run }));
        });
        let lane = Lane {
            out: true,
            ..Lane::default()
        };
        self.runs.0.insert(video, lane);
        if !(self.ports.spawn)(job) {
            self.lost(video);
        }
    }

    /// Calls `video`'s run after the calls before it, as `Lane::queue` orders them.
    pub(super) fn call(&mut self, video: Video, call: Call) {
        let Some(lane) = self.runs.0.get_mut(&video) else {
            return;
        };
        lane.queue(call);
        self.pump(video);
    }

    /// Recording `video` started, or the flow hears why not.
    pub(super) fn started(&mut self, video: Video, run: Result<Run, RecordError>) {
        match run {
            Ok(run) => {
                // A run with no lane left drops here, cancelled and its video deleted.
                let Some(lane) = self.runs.0.get_mut(&video) else {
                    return;
                };
                let file = run.file().clone();
                (lane.run, lane.out) = (Some(run), false);
                self.made(video, file);
                self.pump(video);
            }
            Err(error) => {
                self.runs.0.remove(&video);
                self.step(In::Note {
                    video,
                    note: RecordNote::Fault(error),
                });
            }
        }
    }

    /// A call ended: the run is back for the next, and a sound that failed to switch on is heard as lost, unless the
    /// run ended first, whose last note then says why.
    pub(super) fn called(
        &mut self,
        video: Video,
        run: Run,
        call: Call,
        done: Result<(), RecordError>,
    ) {
        let mut live = false;
        if let Some(lane) = self.runs.0.get_mut(&video) {
            (lane.run, lane.out, live) = (Some(run), false, !lane.over);
        }
        if let Err(error) = done {
            tracing::warn!(%error, "a recording's call failed");
            let lost = match call {
                Call::Mic(true) => Some(Source::Mic),
                Call::Sound(true) => Some(Source::Sound),
                _ => None,
            };
            if let Some(source) = lost.filter(|_| live && error != RecordError::Gone) {
                self.step(In::Note {
                    video,
                    note: RecordNote::Lost(source),
                });
            }
        }
        self.pump(video);
    }

    /// A note of `video`'s run for the flow; its last one lets the run go.
    pub(super) fn heard(&mut self, video: Video, note: RecordNote) {
        if note.ends() {
            if let Some(lane) = self.runs.0.get_mut(&video) {
                lane.over = true;
            }
            self.pump(video);
        }
        self.step(In::Note { video, note });
    }

    /// Makes `video`'s next call while no job holds its run, then lets an ended run go; Cancel acts at hand.
    fn pump(&mut self, video: Video) {
        let Some(lane) = self.runs.0.get_mut(&video) else {
            return;
        };
        if lane.out || lane.run.is_none() {
            return;
        }
        match lane.calls.pop_front() {
            Some(Call::Cancel) => {
                let lane = self.runs.0.remove(&video);
                if let Some(run) = lane.and_then(|lane| lane.run) {
                    run.cancel();
                }
            }
            Some(call) => self.send_call(video, call),
            None if lane.over => {
                self.runs.0.remove(&video);
            }
            None => {}
        }
    }

    /// Makes `call` on `video`'s run in a job, which hands the run back.
    fn send_call(&mut self, video: Video, call: Call) {
        let pass = matches!(call, Call::Mic(true)).then(|| self.ports.mics.pass());
        let back = self.ports.back.clone();
        let Some(lane) = self.runs.0.get_mut(&video) else {
            return;
        };
        let Some(mut run) = lane.run.take() else {
            return;
        };
        lane.out = true;
        let job: Job = Box::new(move || {
            let done = match call {
                Call::Mic(_) => run.run().mic(pass),
                Call::Sound(on) => run.run().sound(on),
                Call::Stop => run.run().stop(),
                // `pump` cancels at hand, so no job is sent for it.
                Call::Cancel => Ok(()),
            };
            let _ = back.send(Msg::Back(Back::Called {
                video,
                run,
                call,
                done,
            }));
        });
        if !(self.ports.spawn)(job) {
            tracing::warn!("a recording's call job did not start");
            self.lost(video);
        }
    }

    /// `video`'s job did not start; a run it held was dropped with it, so cancelled and its video deleted. The lane
    /// goes and the flow hears a fault.
    fn lost(&mut self, video: Video) {
        self.runs.0.remove(&video);
        self.step(In::Note {
            video,
            note: RecordNote::Fault(RecordError::Gone),
        });
    }

    /// What `part`'s run asks, from the snip's held settings and what the system can record.
    fn ask(&self, part: Part, mic: bool, sound: bool) -> RecordAsk {
        let facts = self.seen.facts;
        let mic = (mic && facts.mic == Gear::Ready).then(|| self.ports.mics.pass());
        RecordAsk {
            part,
            fps: self.held.fps,
            mic,
            sound: sound && facts.sound == Gear::Ready,
            max_min: self.held.max_min,
            min_free_mb: self.held.min_free_mb,
            job_mb: self.held.record_mb,
            waits: self.held.record_waits(),
        }
    }

    /// What a recording's job holds: the recorder, the inbox, the folder and the missed deletes.
    fn kit_run(
        &self,
    ) -> (
        Arc<dyn Recorder>,
        Sender<Msg>,
        Arc<dyn SnipFolder>,
        Leftovers,
    ) {
        let ports = &self.ports;
        (
            ports.recorder.clone(),
            ports.back.clone(),
            ports.folder.clone(),
            Arc::clone(&ports.leftovers),
        )
    }
}
