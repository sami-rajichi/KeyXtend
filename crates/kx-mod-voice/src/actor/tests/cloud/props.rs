//! Property: any run of Start, Stop, Cancel, Try again, the gear, Paste at once on and off, the key page's Paste and
//! Test, Compare's cloud trial and a service Off and On, the fakes answering in any order, types or pastes each text at
//! most once, sends nothing before Stop, holds an upload or a trial only while its run is out, and ends every paste.

mod act;
mod ports;

use std::collections::HashSet;

use kx_platform_fake::CloudLog;
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
    /// An erase was refused, so the text waiting on it may never go in.
    lost: bool,
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
            lost: false,
        }
    }

    /// Does `act`, then checks what holds after any act.
    fn apply(&mut self, act: Act) -> Result<(), TestCaseError> {
        let before = (self.runs().len(), self.handed());
        let trial = self.step(act);
        self.check(before, trial)
    }

    /// The cloud runs so far.
    fn runs(&self) -> Vec<CloudLog> {
        self.rig.fake.cloud.calls().runs
    }

    /// The hand-overs of speech so far, over every run.
    fn handed(&self) -> usize {
        self.rig.fake.speech.runs().iter().map(|r| r.pieces).sum()
    }

    /// After an act that found the cloud runs and hand-overs `before`, its new runs Compare's when `trial`: no request
    /// without speech handed over after Stop, an upload or a trial held only while its run is out, the clipboard left
    /// to the history, and each text in once at most.
    fn check(&mut self, before: (usize, usize), trial: bool) -> Result<(), TestCaseError> {
        let runs = self.runs();
        if runs.len() > before.0 {
            let handed = self.handed() > before.1;
            prop_assert!(handed, "a request with no speech handed over");
        }
        let speech = self.rig.fake.speech.runs();
        let stopped = speech.iter().all(|r| r.pieces == 0 || r.stops > 0);
        prop_assert!(stopped, "speech handed over before Stop");
        self.trials.resize(runs.len(), trial);
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
            let whole = runs[*run].state == CloudState::Done && !self.trials[*run];
            let none = typed + put == 0;
            prop_assert!(whole || none, "{token} of a run cut short, or Compare's");
        }
        Ok(())
    }

    /// Answers everything out, waits past both waits, then answers again: every paste ended, and each token of a
    /// dictation's run that ended with every text went in once, unless an erase was refused.
    fn settle(&mut self) -> Result<(), TestCaseError> {
        self.drain()?;
        self.apply(Act::Wait(false))?;
        self.apply(Act::Wait(true))?;
        self.drain()?;
        prop_assert!(self.rig.actor.paste_due().is_none(), "every paste ended");
        let (runs, asks) = (self.runs(), self.rig.asks());
        for (token, run) in &self.tokens {
            let whole = runs[*run].state == CloudState::Done && !self.trials[*run];
            if whole && !self.lost {
                let typed = asks.iter().filter(|a| holds(&a.text, token)).count();
                let went = typed + usize::from(self.first.contains(token));
                prop_assert_eq!(went, 1, "{} went in once", token);
            }
        }
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
