//! The steps a cloud run meets, their odds, the picks they move between, and how each is fed to the rig.

use super::*;
use kx_module_api::ports::{CloudFault, CloudNote, ServiceName};

/// Each language's pick: none, Groq, Custom, Groq without its key, and Groq with newer terms.
const PICKS: usize = 5;
/// How often a note goes to an older upload.
const STALE: f64 = 0.1;
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
pub(super) enum Wire {
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
    /// The open take's run writes a word, then ends, as a local model does.
    Write,
    /// The last upload tells a word, then its end.
    Reply,
    /// The panel's Try again.
    Retry,
    /// The box in front turns into a password box when true, else into a text box.
    Box(bool),
}

/// What an upload may tell.
#[derive(Clone, Debug)]
pub(super) enum Told {
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

pub(super) fn wire() -> impl Strategy<Value = Wire> {
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
        3 => Just(Wire::Local),
        3 => Just(Wire::Write),
        2 => Just(Wire::Reply),
        3 => Just(Wire::Retry),
        3 => any::<bool>().prop_map(Wire::Box),
    ]
}

/// The pick at `i`.
pub(super) fn pick(i: usize) -> Option<Net> {
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
pub(super) fn may_send(step: &Wire) -> bool {
    let acts = |b: &BarButton| matches!(b, BarButton::Anyway | BarButton::Again);
    match step {
        Wire::Base(Step::Said(said, _)) => matches!(said, Said::Done),
        Wire::Base(Step::Bar(b) | Step::Late(b, _)) => acts(b),
        Wire::Base(Step::Again) | Wire::Over | Wire::Write | Wire::Retry => true,
        _ => false,
    }
}

/// True for a step that tells the last upload's end.
pub(super) fn ends_upload(step: &Wire) -> bool {
    match step {
        Wire::Sent(told, false) => !matches!(told, Told::Piece(_)),
        Wire::Reply => true,
        _ => false,
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

/// Feeds `step`; `ask` is the last text ask, and `ended` gains each take a run's end was told to.
pub(super) fn feed(
    rig: &mut Rig,
    step: &Wire,
    ask: Option<&TextAsk>,
    ended: &mut BTreeSet<u32>,
    s: &VoiceSettings,
) -> Vec<Out> {
    match step {
        Wire::Base(base) => {
            if let Step::Said(Said::Done, old) = base {
                ended.insert(rig.take.wrapping_sub(u32::from(*old)));
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
        Wire::Over | Wire::Write => {
            ended.insert(rig.take);
            run_end(rig, step)
        }
        _ => feed_more(rig, step),
    }
}

/// The open take's run ends, after writing a word for `Write`.
fn run_end(rig: &mut Rig, step: &Wire) -> Vec<Out> {
    match step {
        Wire::Write => rig.write(&["mot"]),
        _ => rig.tell(SpeechNote::Done(done(0))),
    }
}

/// Feeds the steps `feed` leaves: the microphone, a whole reply, Try again and the box in front.
fn feed_more(rig: &mut Rig, step: &Wire) -> Vec<Out> {
    match step {
        Wire::Talk => {
            let mut out = rig.tell(SpeechNote::Stage(SpeechStage::Open));
            out.extend(rig.tell(SpeechNote::Heard));
            out
        }
        Wire::Reply => {
            let mut out = tell_sent(rig, &Told::Piece(0), false);
            out.extend(tell_sent(rig, &Told::Done, false));
            out
        }
        Wire::Retry => rig.again(),
        Wire::Box(password) => {
            let kind = [FieldKind::Text, FieldKind::Password][usize::from(*password)];
            rig.field(kind, BOX)
        }
        _ => Vec::new(),
    }
}
