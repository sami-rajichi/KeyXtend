//! Property: any run of Start, Stop, Cancel, Try again, the gear, Paste at once on and off, the key page's Paste and
//! Test, Compare's cloud trial and a service Off and On, the fakes answering in any order, types or pastes each text at
//! most once, asks for no speech before its run ended, tells the flow and Compare each upload's end, ends each paste.

mod act;
mod ports;

use std::collections::HashSet;

use kx_module_api::ports::RunStep;
use kx_platform_fake::{CloudLog, RunLog};
use proptest::prelude::*;

use super::*;
use crate::INSERTS;
use act::{Act, act};

/// The most acts in one case, as the cloud pages' property takes.
const MAX_STEPS: usize = 40;
/// Paste at once's key in `[voice]`.
const INSERT: &str = "insert";

/// The rig, and what the property gave it and saw.
struct Seen {
    rig: Rig,
    /// Each token a service wrote, with its run's place.
    tokens: Vec<(String, usize)>,
    /// For each cloud run, true when it was Compare's.
    trials: Vec<bool>,
    /// The speech runs Compare opened.
    compare: HashSet<usize>,
    /// The paste rounds answered.
    pasted: HashSet<u32>,
    /// The tokens a paste put in before any typing of them.
    first: HashSet<String>,
    /// How many of the oldest texts the keyboard answered.
    typed: usize,
    /// The key page's takes answered.
    taken: HashSet<u32>,
    /// The tokens of a text that rightly never goes in: its erase was refused, or it could not replace the text before.
    lost: HashSet<String>,
}

/// What stands before an act: each cloud run's state, the hand-overs, and the texts asked.
struct Before {
    states: Vec<CloudState>,
    handed: usize,
    asked: usize,
}

impl Seen {
    /// Groq on for English with its key and two pieces of speech ready, its texts pasted when `paste`.
    fn new(paste: bool) -> Self {
        let rig = Rig::build(|fake, host| {
            groq(fake, host);
            host.put(INSERT, INSERTS[usize::from(paste)].0.into());
        });
        rig.fake.speech.set_speech(vec![vec![0; PIECE]; 2]);
        Self {
            rig,
            tokens: Vec::new(),
            trials: Vec::new(),
            compare: HashSet::new(),
            pasted: HashSet::new(),
            first: HashSet::new(),
            typed: 0,
            taken: HashSet::new(),
            lost: HashSet::new(),
        }
    }

    /// Does `act`, then checks what holds after any act.
    fn apply(&mut self, act: Act) -> Result<(), TestCaseError> {
        let before = self.before();
        let trial = self.step(act);
        self.check(&before, trial)
    }

    /// What stands before an act.
    fn before(&self) -> Before {
        Before {
            states: self.runs().iter().map(|r| r.state).collect(),
            handed: self.handed(),
            asked: self.asked(),
        }
    }

    /// The cloud runs so far.
    fn runs(&self) -> Vec<CloudLog> {
        self.rig.fake.cloud.calls().runs
    }

    /// The hand-overs of speech so far, over every run.
    fn handed(&self) -> usize {
        self.rig.fake.speech.runs().iter().map(|r| r.pieces).sum()
    }

    /// The texts asked to be typed or pasted so far.
    fn asked(&self) -> usize {
        self.rig.asks().len() + self.rig.pastes().len()
    }

    /// After an act from `before`, its new runs Compare's when `trial`: no request without a hand-over, no speech
    /// asked for before its run ended, an upload or a trial held only while its run is out, the clipboard left to the
    /// history, and each text in once at most.
    fn check(&mut self, before: &Before, trial: bool) -> Result<(), TestCaseError> {
        let runs = self.runs();
        if runs.len() > before.states.len() {
            let handed = self.handed() > before.handed;
            prop_assert!(handed, "a request with no speech handed over");
        }
        let speech = self.rig.fake.speech.runs();
        let early = speech.iter().all(|r| r.early == 0);
        prop_assert!(early, "speech asked for before its run ended");
        self.trials.resize(runs.len(), trial);
        self.kept_out(before, &runs);
        let out = |of: bool| {
            let both = runs.iter().zip(&self.trials);
            both.filter(|(r, t)| r.state == CloudState::Sending && **t == of)
                .count()
        };
        let net = &self.rig.actor.net;
        let held = net.trials.len();
        prop_assert_eq!(held, out(true), "a trial lives while its run is out");
        let idle = out(true) + out(false) == 0;
        prop_assert_eq!(net.idle(), idle, "an upload is held while its run is out");
        let clip = self.rig.fake.clip.writes();
        prop_assert!(clip.is_empty(), "the clipboard is the history's");
        self.texts(&runs)
    }

    /// A dictation's run that ended in this act with nothing asked kept its text out, as it cannot replace a text
    /// whose typing is not confirmed (`flow/text.rs`); its tokens are spared the exactly-once check.
    fn kept_out(&mut self, before: &Before, runs: &[CloudLog]) {
        if self.asked() > before.asked {
            return;
        }
        let was = |at: usize| before.states.get(at) == Some(&CloudState::Done);
        let ended = |at: usize| whole(runs, &self.trials, at) && !was(at);
        for (token, run) in &self.tokens {
            if ended(*run) {
                self.lost.insert(token.clone());
            }
        }
    }

    /// Each token in one text typed and one pasted at most, never typed once pasted, and only from a dictation's run
    /// that ended with every text; each paste waits as set.
    fn texts(&self, runs: &[CloudLog]) -> Result<(), TestCaseError> {
        let (asks, pastes) = (self.rig.asks(), self.rig.pastes());
        let paste_ms = self.rig.actor.settings().paste_ms;
        let waits = pastes.iter().all(|p| p.wait_ms == paste_ms);
        prop_assert!(waits, "a paste waits as set");
        for (token, run) in &self.tokens {
            let typed = asks.iter().filter(|a| holds(&a.text, token)).count();
            let put = pastes.iter().filter(|p| holds(&p.text, token)).count();
            prop_assert!(typed <= 1 && put <= 1, "{token} went twice");
            let first = self.first.contains(token);
            prop_assert!(typed == 0 || !first, "{token} pasted, then typed");
            let none = typed + put == 0;
            let kept = whole(runs, &self.trials, *run);
            prop_assert!(kept || none, "{token} of a run cut short, or Compare's");
        }
        Ok(())
    }

    /// Answers everything out, waits past both waits, then answers again: every paste ended, the flow and Compare
    /// heard each upload's end, and each token of a dictation's whole run went in once, unless its erase was refused.
    fn settle(&mut self) -> Result<(), TestCaseError> {
        self.drain()?;
        self.apply(Act::Wait(false))?;
        self.apply(Act::Wait(true))?;
        self.drain()?;
        prop_assert!(self.rig.actor.paste_due().is_none(), "every paste ended");
        self.told()?;
        let (runs, asks) = (self.runs(), self.rig.asks());
        for (token, run) in &self.tokens {
            if whole(&runs, &self.trials, *run) && !self.lost.contains(token) {
                let typed = asks.iter().filter(|a| holds(&a.text, token)).count();
                let went = typed + usize::from(self.first.contains(token));
                prop_assert_eq!(went, 1, "{} went in once", token);
            }
        }
        Ok(())
    }

    /// With nothing out, the bar waits only on a stopped run's end, and no ticked cloud model of a running Compare
    /// is still writing.
    fn told(&self) -> Result<(), TestCaseError> {
        let due = self.rig.fake.speech.runs().iter().any(awaits);
        let waits = self.rig.bar() == VoiceBar::Transcribing;
        prop_assert!(due || !waits, "the flow never heard its upload end");
        let compare = &self.rig.actor.panel.compare;
        let (Some((_, rows)), Some(plan)) = (compare.running(), compare.plan()) else {
            return Ok(());
        };
        let clouds = &rows[rows.len().saturating_sub(plan.clouds.len())..];
        let writing = clouds.iter().any(|(_, step)| *step == RunStep::Writing);
        prop_assert!(!writing, "Compare never heard a cloud model's end");
        Ok(())
    }

    /// Answers what is out, one act at a time, until nothing is.
    fn drain(&mut self) -> Result<(), TestCaseError> {
        while let Some(act) = self.due() {
            self.apply(act)?;
        }
        Ok(())
    }
}

/// True when speech run `run` is live and stopped, its end still due: one `Done` for its recording and one per `again`.
fn awaits(run: &RunLog) -> bool {
    let live = !run.closed && !run.dead;
    live && run.stops > 0 && run.dones <= run.agains.len()
}

/// True when cloud run `run` of `runs` is a dictation's that ended with every text; `trials` marks Compare's.
fn whole(runs: &[CloudLog], trials: &[bool], run: usize) -> bool {
    runs[run].state == CloudState::Done && !trials[run]
}

/// What `text` says.
fn said(text: &Secret) -> &str {
    text.expose().as_str()
}

/// True when `text` holds `token`.
fn holds(text: &Secret, token: &str) -> bool {
    said(text).contains(token)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn each_text_goes_in_once_nothing_leaves_before_stop_and_an_upload_lives_while_its_run_is_out(
        paste in any::<bool>(),
        acts in prop::collection::vec(act(), 1..MAX_STEPS),
    ) {
        let mut seen = Seen::new(paste);
        for act in acts {
            seen.apply(act)?;
        }
        seen.settle()?;
    }
}
