//! Properties over any cloud run (carries 7 and 8): an upload goes only from a run opened listen-only, never from a
//! password box, only once its run was told to end, only to the service the language picks now with its key set and
//! its terms confirmed, and only at that end, Transcribe anyway or Try again; a dropped or ended upload's notes do
//! nothing; and a take's text goes in fresh at most once.

use super::*;
use kx_module_api::ports::{CloudFault, CloudNote, ServiceName};

/// Each language's pick: none, Groq, Custom, Groq without its key, and Groq with newer terms.
const PICKS: usize = 5;
/// How often a note goes to an older upload.
const STALE: f64 = 0.1;
/// Random runs: more than the other properties, as an upload needs a whole dictation first.
const RUNS: u32 = 1_024;
/// The faults a service's run may tell.
const FAULTS: [CloudFault; 6] = [
    CloudFault::Network,
    CloudFault::Key,
    CloudFault::Rate,
    CloudFault::Language,
    CloudFault::Empty,
    CloudFault::Gone,
];

/// One thing a cloud run meets.
#[derive(Clone, Debug)]
enum Wire {
    Base(Step),
    /// The last upload tells this, or an older one when true.
    Sent(Told, bool),
    /// Every language's cloud pick moves, by index.
    Pick(usize),
    Keys,
    Local,
    /// The Dictation key, past the last click's debounce.
    Key,
    /// The open take's microphone opens and hears speech.
    Talk,
    /// The open take's run ends: a listen-only one hands its speech over.
    Over,
}

/// What an upload may tell.
#[derive(Clone, Debug)]
enum Told {
    Piece(u16),
    Done,
    Fault(usize),
    Silent,
    Lost,
    Broken,
}

impl Told {
    /// What it stands for.
    fn sent(&self) -> Sent {
        match self {
            Self::Piece(index) => Sent::Note(CloudNote::Piece {
                index: *index,
                text: secret("mot"),
            }),
            Self::Done => Sent::Note(CloudNote::Done),
            Self::Fault(i) => Sent::Note(CloudNote::Fault(FAULTS[*i])),
            Self::Silent => Sent::Silent,
            Self::Lost => Sent::Speech(SpeechError::NoSound),
            Self::Broken => Sent::Speech(SpeechError::Internal),
        }
    }
}

fn wire() -> impl Strategy<Value = Wire> {
    let told = prop_oneof![
        3 => (0..3_u16).prop_map(Told::Piece),
        2 => Just(Told::Done),
        1 => (0..FAULTS.len()).prop_map(Told::Fault),
        1 => Just(Told::Silent),
        1 => Just(Told::Lost),
        1 => Just(Told::Broken),
    ];
    prop_oneof![
        4 => step().prop_map(Wire::Base),
        4 => (told, prop::bool::weighted(STALE)).prop_map(|(t, old)| Wire::Sent(t, old)),
        2 => (0..PICKS).prop_map(Wire::Pick),
        3 => Just(Wire::Key),
        3 => Just(Wire::Talk),
        3 => Just(Wire::Over),
        1 => Just(Wire::Keys),
        1 => Just(Wire::Local),
    ]
}

/// The pick at `i`.
fn pick(i: usize) -> Option<Net> {
    let groq = Net {
        service: "groq".into(),
        name: ServiceName::Named("Groq".into()),
        model: Redacted::new("groq:whisper-large-v3-turbo".into()),
        piece_s: *SpeechAsk::PIECE_S.end(),
        key: true,
        terms: false,
    };
    match i {
        1 => Some(groq),
        2 => Some(Net {
            service: "custom".into(),
            name: ServiceName::Custom,
            model: Redacted::new("custom:mine".into()),
            ..groq
        }),
        3 => Some(Net { key: false, ..groq }),
        4 => Some(Net {
            terms: true,
            ..groq
        }),
        _ => None,
    }
}

/// True for a step a user's act or a run's end makes, the only ones that may send.
fn may_send(step: &Wire) -> bool {
    let acts = |b: &BarButton| matches!(b, BarButton::Anyway | BarButton::Again);
    match step {
        Wire::Base(Step::Said(said, _)) => matches!(said, Said::Done),
        Wire::Base(Step::Bar(b) | Step::Late(b, _)) => acts(b),
        Wire::Base(Step::Again) | Wire::Over => true,
        _ => false,
    }
}

/// What the cloud run saw so far.
#[derive(Default)]
struct Track {
    /// The takes opened listen-only.
    alone: BTreeSet<u32>,
    /// The takes a run's end was told to.
    ended: BTreeSet<u32>,
    /// Uploads dropped or ended, by take and round.
    dead: BTreeSet<(u32, u32)>,
    /// The takes whose text went in fresh.
    fresh: BTreeSet<u32>,
}

impl Track {
    /// Checks one step's effects against what was seen before it.
    fn check(
        &mut self,
        rig: &Rig,
        step: &Wire,
        out: &[Out],
        before: Option<(u32, u32)>,
    ) -> Result<(), TestCaseError> {
        for effect in out {
            match effect {
                Out::Open { take, ask } if ask.model.is_none() => {
                    self.alone.insert(*take);
                }
                Out::Send(up) => self.sent(rig, step, up)?,
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

    /// Checks an upload: from a listen-only run that ended, out of no password box, to the service picked now.
    fn sent(&self, rig: &Rig, step: &Wire, up: &Upload) -> Result<(), TestCaseError> {
        prop_assert!(may_send(step), "{step:?} sent by itself");
        prop_assert_eq!(up.take, rig.take);
        prop_assert!(self.alone.contains(&up.take), "a run with a model sent");
        prop_assert!(
            self.ended.contains(&up.take),
            "sent before the speech was handed over"
        );
        prop_assert!(!rig.flow.password(), "a password box sent");
        let now = rig.book.1.get(&up.lang);
        let fits = now.is_some_and(|n| n.service == up.service && n.key && !n.terms);
        prop_assert!(fits, "sent to {} while the pick is {now:?}", up.service);
        Ok(())
    }
}

/// Gives every language cloud pick `i`, or none, then tells the settings.
fn repick(rig: &mut Rig, i: usize, s: &VoiceSettings) -> Vec<Out> {
    for lang in LANGS {
        match pick(i) {
            Some(net) => rig.book.1.insert(lang.into(), net),
            None => rig.book.1.remove(lang),
        };
    }
    rig.on(In::Settings(Box::new(s.clone())))
}

/// The last upload tells `told`, or the one before it when `old`.
fn tell_sent(rig: &mut Rig, told: &Told, old: bool) -> Vec<Out> {
    let Some((take, round)) = rig.upload else {
        return Vec::new();
    };
    let round = round.wrapping_sub(u32::from(old));
    rig.on(In::Sent {
        take,
        round,
        sent: told.sent(),
    })
}

/// Feeds `step`; `ask` is the last text ask.
fn feed(
    rig: &mut Rig,
    step: &Wire,
    ask: Option<&TextAsk>,
    track: &mut Track,
    s: &VoiceSettings,
) -> Vec<Out> {
    match step {
        Wire::Base(base) => {
            if let Step::Said(Said::Done, old) = base {
                track.ended.insert(rig.take.wrapping_sub(u32::from(*old)));
            }
            act(rig, base, ask)
        }
        Wire::Sent(told, old) => tell_sent(rig, told, *old),
        Wire::Pick(i) => repick(rig, *i, s),
        Wire::Keys => rig.on(In::Keys),
        Wire::Local => rig.click(VoiceClick::Local),
        Wire::Key => {
            rig.wait(u64::try_from(s.debounce().as_millis()).unwrap());
            rig.key()
        }
        Wire::Talk => {
            let mut out = rig.tell(SpeechNote::Stage(SpeechStage::Open));
            out.extend(rig.tell(SpeechNote::Heard));
            out
        }
        Wire::Over => {
            track.ended.insert(rig.take);
            rig.tell(SpeechNote::Done(done(0)))
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(RUNS))]

    #[test]
    fn a_cloud_run_sends_only_what_the_user_picked_and_never_twice(
        steps in vec(wire(), 1..MAX_STEPS),
        hidden in any::<bool>(),
        keep_voice in any::<bool>(),
    ) {
        let mut settings = VoiceSettings::from_defaults().unwrap();
        settings.keep_voice = keep_voice;
        let mut rig = Rig::with(settings.clone(), hidden);
        rig.book.1.extend(LANGS.map(|lang| (lang.to_owned(), pick(1).unwrap())));
        let (mut seen, mut track) = (Seen::default(), Track::default());
        for step in &steps {
            let lent = rig.flow.lent.is_some();
            let before = rig.upload;
            let dead = match step {
                Wire::Sent(_, old) => *old || before.is_some_and(|u| track.dead.contains(&u)),
                _ => false,
            };
            let out = feed(&mut rig, step, seen.ask.as_ref(), &mut track, &settings);
            if dead {
                let acted = out.iter().any(|o| !matches!(o, Out::View(_) | Out::Lit(_)));
                prop_assert!(!acted, "a dropped upload's note did {out:?}");
            }
            if let (Wire::Sent(told, false), Some(up)) = (step, rig.upload) {
                let ends = !matches!(told, Told::Piece(_));
                if ends { track.dead.insert(up); }
            }
            seen.effects(&out, hidden, lent)?;
            track.check(&rig, step, &out, before)?;
            seen.bar(&rig)?;
        }
    }
}
