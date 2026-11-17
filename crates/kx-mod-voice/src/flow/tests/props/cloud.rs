//! Properties over any cloud run: an upload goes only from a run still listening alone, never from a password box or
//! a take that went to one, only once its run was told to end, only to the service the language picks now with its
//! key set and its terms confirmed, and only at that end, Transcribe anyway or Try again; a dropped or ended upload's
//! notes do nothing; a hand-over counts nothing, even after the local model took over; and a take's text goes in
//! fresh at most once. The steps are in `steps`, and the cases the property once shrank to in `cases`.

mod cases;
mod steps;

use super::*;
use steps::{Wire, ends_run, ends_upload, feed, may_send, pick, wire};

/// Random runs: more than the other properties, as an upload needs a whole dictation first.
const RUNS: u32 = 1_024;

/// What the cloud run saw so far.
#[derive(Default)]
struct Track {
    /// The takes opened listen-only, until a model writes them.
    alone: BTreeSet<u32>,
    /// The takes opened listen-only, kept once a model writes them.
    born: BTreeSet<u32>,
    /// The takes whose hand-over was told.
    handed: BTreeSet<u32>,
    /// The takes a run's end was told to.
    ended: BTreeSet<u32>,
    /// Uploads dropped or ended, by take and round.
    dead: BTreeSet<(u32, u32)>,
    /// The takes whose text went in fresh.
    fresh: BTreeSet<u32>,
}

impl Track {
    /// Checks one step's effects against what was seen before it; `masked` holds the takes shown as dots.
    fn check(
        &mut self,
        rig: &Rig,
        (step, out): (&Wire, &[Out]),
        before: Option<(u32, u32)>,
        masked: &BTreeSet<u32>,
    ) -> Result<(), TestCaseError> {
        for effect in out {
            match effect {
                Out::Open { take, ask } if ask.model.is_none() => {
                    self.alone.insert(*take);
                    self.born.insert(*take);
                }
                // A run that writes with a model no longer listens alone, so it can never hand speech over again.
                Out::Run {
                    take,
                    call: RunCall::Again(_),
                } => {
                    self.alone.remove(take);
                }
                Out::Send(up) => self.sent(rig, step, up, masked)?,
                Out::Halt { take } => {
                    if let Some(up) = before.filter(|u| u.0 == *take) {
                        self.dead.insert(up);
                    }
                }
                Out::Type(ask) | Out::Paste(ask)
                    if ask.erase == 0 && !ask.text.expose().is_empty() =>
                {
                    prop_assert!(self.fresh.insert(rig.take), "take {} typed twice", rig.take);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Checks an upload: from a listen-only run that ended, out of no password box and of no take that ever went to
    /// one, to the service picked now.
    fn sent(
        &self,
        rig: &Rig,
        step: &Wire,
        up: &Upload,
        masked: &BTreeSet<u32>,
    ) -> Result<(), TestCaseError> {
        prop_assert!(may_send(step), "{step:?} sent by itself");
        prop_assert_eq!(up.take, rig.take);
        prop_assert!(self.alone.contains(&up.take), "a run with a model sent");
        prop_assert!(
            self.ended.contains(&up.take),
            "sent before the speech was handed over"
        );
        prop_assert!(!rig.flow.password(), "a password box sent");
        prop_assert!(!masked.contains(&up.take), "a password box's take sent");
        let now = rig.book.1.get(&up.lang);
        let fits = now.is_some_and(|n| {
            n.service == up.service && n.key && !n.terms && n.unavailable.is_none()
        });
        prop_assert!(fits, "sent to {} while the pick is {now:?}", up.service);
        Ok(())
    }
}

impl Track {
    /// Feeds `step` to `rig` and checks what it did, with `seen`'s checks too.
    fn step(
        &mut self,
        rig: &mut Rig,
        seen: &mut Seen,
        step: &Wire,
        (settings, hidden): (&VoiceSettings, bool),
    ) -> Result<(), TestCaseError> {
        let (lent, before) = (rig.flow.lent.is_some(), rig.upload);
        let dead = match step {
            Wire::Sent(_, old) => *old || before.is_some_and(|u| self.dead.contains(&u)),
            Wire::Reply => before.is_some_and(|u| self.dead.contains(&u)),
            _ => false,
        };
        let hand_over = ends_run(step) && self.born.contains(&rig.take);
        let hand_over = hand_over && self.handed.insert(rig.take);
        let out = feed(rig, step, seen.ask.as_ref(), &mut self.ended, settings);
        if dead {
            let acted = out.iter().any(|o| !matches!(o, Out::View(_) | Out::Lit(_)));
            prop_assert!(!acted, "a dropped upload's note did {out:?}");
        }
        let counted = out.iter().any(|o| matches!(o, Out::Count(_)));
        prop_assert!(!(hand_over && counted), "a hand-over counted: {out:?}");
        if let (true, Some(up)) = (ends_upload(step), rig.upload) {
            self.dead.insert(up);
        }
        seen.effects(&out, hidden, lent)?;
        self.check(rig, (step, &out), before, &seen.masked)?;
        seen.bar(rig)
    }
}

/// Runs `steps` on a fresh rig with Groq picked for every language, checking each step.
fn run(steps: &[Wire], hidden: bool, keep_voice: bool) -> Result<(), TestCaseError> {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.keep_voice = keep_voice;
    let mut rig = Rig::with(settings.clone(), hidden);
    rig.book
        .1
        .extend(LANGS.map(|lang| (lang.to_owned(), pick(1).unwrap())));
    let (mut seen, mut track) = (Seen::default(), Track::default());
    for step in steps {
        track.step(&mut rig, &mut seen, step, (&settings, hidden))?;
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(RUNS))]

    #[test]
    fn a_cloud_run_sends_only_what_the_user_picked_and_never_twice(
        steps in vec(wire(), 1..MAX_STEPS),
        hidden in any::<bool>(),
        keep_voice in any::<bool>(),
    ) {
        run(&steps, hidden, keep_voice)?;
    }
}
