//! The property's acts: the user's clicks and keys, the runs' notes, and each fake's answers in `ports`.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, BarText, CloudButton, CloudFault, CloudLine, PanelButton, SpeechNote, SpeechStage,
    VoiceBar, VoiceClick, VoiceTab,
};
use kx_platform_fake::SpeechCall;
use proptest::prelude::*;
use proptest::sample::{select, subsequence};

use super::super::done;
use super::{INSERT, Seen, awaits};
use crate::INSERTS;
use crate::actor::Msg;
use crate::panel::Do;
use crate::panel::rig::ROWS;

/// Compare's first two cloud rows: the first service's models.
const TICKS: [u8; 2] = [0, 1];
/// The faults a service tells on a piece.
const FAULTS: [CloudFault; 3] = [CloudFault::Rate, CloudFault::Credit, CloudFault::Key];
/// The bar's buttons the user clicks: Cancel, Stop, Try again, Transcribe anyway, Undo and Close.
const BAR: [BarButton; 6] = [
    BarButton::Cancel,
    BarButton::Stop,
    BarButton::Again,
    BarButton::Anyway,
    BarButton::Undo,
    BarButton::Close,
];
/// The panel's buttons: Cancel and Back, which stop Compare, and Try again, which replaces a text put in.
const PANEL: [PanelButton; 3] = [PanelButton::Cancel, PanelButton::Back, PanelButton::Again];
/// The set text Compare reads out.
const SET_TEXT: &str = "hello world";

/// One thing the user or a port does.
#[derive(Clone, Debug)]
pub(super) enum Act {
    /// The Dictation key: Start, or Stop while it listens.
    Key,
    /// The newest live run hears speech.
    Heard,
    /// The newest live run, once stopped, ends.
    Done,
    /// A whole dictation: Start, speech, Stop and its run's end.
    Dictate,
    /// A whole Compare with these rows ticked: Start, speech, Stop and its run's end.
    Trial(Vec<u8>),
    /// A bar button.
    Bar(BarButton),
    /// The gear opens the panel.
    Gear,
    /// The panel closes.
    Close,
    /// Paste at once on, or off.
    Paste(bool),
    /// The first service's key page opens.
    Lines,
    /// The key page's Paste on the key line.
    KeyPaste,
    /// The key page's Test.
    Test,
    /// Compare starts with these rows ticked.
    Compare(Vec<u8>),
    /// A panel button.
    Panel(PanelButton),
    /// The first service on, or off.
    Service(bool),
    /// The service answers the sending run at this place, wrapped: its next text, else this fault.
    Answer(usize, Option<CloudFault>),
    /// The service writes every piece left of the sending run at this place, wrapped.
    Write(usize),
    /// The uploads are written and the texts go in, then the panel's Try again replaces the last.
    Redo,
    /// The connection to the sending run at this place, wrapped, drops.
    Cut(usize),
    /// The clipboard history answers the open paste at this place, wrapped: pasted, or refused.
    Pasted(usize, bool),
    /// The keyboard answers the oldest text: typed, or refused.
    Typed(bool),
    /// The clipboard history answers the key page's Paste with the key.
    Taken,
    /// The clock moves on by the key page's wait, else Paste at once's.
    Wait(bool),
}

/// A random act, whole dictations and the answers most often so runs and pastes end.
pub(super) fn act() -> impl Strategy<Value = Act> {
    let at = any::<usize>;
    let fault = prop::option::of(select(FAULTS.to_vec()));
    let ticks = || subsequence(TICKS.to_vec(), 0..=TICKS.len());
    prop_oneof![
        3 => Just(Act::Key),
        2 => Just(Act::Heard),
        3 => Just(Act::Done),
        3 => Just(Act::Dictate),
        1 => ticks().prop_map(Act::Trial),
        2 => select(BAR.to_vec()).prop_map(Act::Bar),
        1 => Just(Act::Gear),
        1 => Just(Act::Close),
        1 => any::<bool>().prop_map(Act::Paste),
        1 => Just(Act::Lines),
        1 => Just(Act::KeyPaste),
        1 => Just(Act::Test),
        1 => ticks().prop_map(Act::Compare),
        3 => select(PANEL.to_vec()).prop_map(Act::Panel),
        1 => any::<bool>().prop_map(Act::Service),
        4 => (at(), fault).prop_map(|(at, fault)| Act::Answer(at, fault)),
        2 => at().prop_map(Act::Write),
        2 => Just(Act::Redo),
        1 => at().prop_map(Act::Cut),
        2 => (at(), any::<bool>()).prop_map(|(at, ok)| Act::Pasted(at, ok)),
        2 => any::<bool>().prop_map(Act::Typed),
        1 => Just(Act::Taken),
        1 => any::<bool>().prop_map(Act::Wait),
    ]
}

impl Seen {
    /// Does `act` on the rig; true when the cloud runs it opened are Compare's.
    pub(super) fn step(&mut self, act: Act) -> bool {
        match act {
            Act::Key => {
                self.cue();
                self.rig.key();
            }
            Act::Heard => self.tell(SpeechNote::Heard),
            Act::Done => return self.done(),
            Act::Dictate => {
                self.cue();
                self.rig.key();
                return self.ended();
            }
            Act::Trial(ticks) => {
                self.compare(&ticks);
                return self.ended();
            }
            Act::Bar(button) => self.rig.act(VoiceClick::Bar(button)),
            Act::Gear => self.rig.send(Msg::Open),
            Act::Close => self.rig.send(Msg::Close),
            Act::Paste(on) => self.insert(on),
            Act::Lines => self.service(CloudButton::Lines),
            Act::KeyPaste => self.rig.act(VoiceClick::Paste(CloudLine::Key)),
            Act::Test => self.rig.act(VoiceClick::Service(CloudButton::Test)),
            Act::Compare(ticks) => self.compare(&ticks),
            Act::Panel(button) => self.panel(button),
            Act::Service(on) => self.switch(on),
            Act::Answer(at, fault) => self.answer(at, fault),
            Act::Write(at) => self.write(at),
            Act::Redo => self.redo(),
            Act::Cut(at) => self.cut(at),
            Act::Pasted(at, ok) => self.paste_answer(at, ok),
            Act::Typed(ok) => self.type_answer(ok),
            Act::Taken => self.take_answer(),
            Act::Wait(take) => self.wait(take),
        }
        false
    }

    /// The microphone opens on the next run.
    fn cue(&self) {
        let open = vec![SpeechNote::Stage(SpeechStage::Open)];
        self.rig.fake.speech.on(SpeechCall::Open, open);
    }

    /// The newest live run tells `note`, when one hears it.
    fn tell(&mut self, note: SpeechNote) {
        let _ = self.rig.fake.speech.tell(note);
        self.rig.pump();
    }

    /// The newest live run, once stopped and its end still due.
    pub(super) fn stopped(&self) -> Option<usize> {
        let runs = self.rig.fake.speech.runs();
        let at = runs.iter().rposition(|r| !r.closed && !r.dead)?;
        awaits(&runs[at]).then_some(at)
    }

    /// The stopped run ends, its recording or an `again`'s writing; true when it is Compare's.
    fn done(&mut self) -> bool {
        let Some(at) = self.stopped() else {
            return false;
        };
        self.tell(SpeechNote::Done(done(0)));
        self.compare.contains(&at)
    }

    /// The run just started hears speech, the key stops it and it ends; true when it is Compare's.
    fn ended(&mut self) -> bool {
        self.tell(SpeechNote::Heard);
        self.rig.key();
        self.done()
    }

    /// A panel button; Try again goes through `try_again`, which first notes whether Undo is off.
    fn panel(&mut self, button: PanelButton) {
        if matches!(button, PanelButton::Again) {
            return self.try_again();
        }
        self.rig.act(VoiceClick::Button(button));
    }

    /// The panel's Try again, after noting whether Undo is off: only then may the text it brings stay out.
    pub(super) fn try_again(&mut self) {
        let undo = matches!(self.rig.bar(), VoiceBar::Text(BarText { undo: true, .. }));
        self.unsure = Some(!undo);
        self.rig.act(VoiceClick::Button(PanelButton::Again));
    }

    /// Paste at once on, or off, as the options page writes it.
    fn insert(&mut self, on: bool) {
        let (name, _) = INSERTS[usize::from(on)];
        self.rig.actor.dos(vec![Do::Set(INSERT, name.into())]);
        self.rig.pump();
    }

    /// The gear, then `button` on the Cloud page's first service.
    fn service(&mut self, button: CloudButton) {
        self.rig.send(Msg::Open);
        self.rig.act(VoiceClick::Open(VoiceTab::Cloud));
        self.rig.act(VoiceClick::Row(0));
        self.rig.act(VoiceClick::Service(button));
    }

    /// The first service on with its card confirmed, or off.
    fn switch(&mut self, on: bool) {
        if !on {
            return self.service(CloudButton::Off);
        }
        self.service(CloudButton::On);
        self.rig.act(VoiceClick::Confirm(true));
    }

    /// The gear, then Compare with the rows at `ticks` ticked, started on its set text; the run it opens is Compare's.
    fn compare(&mut self, ticks: &[u8]) {
        self.rig.send(Msg::Open);
        self.rig.act(VoiceClick::Rows(ROWS));
        self.rig.act(VoiceClick::Open(VoiceTab::Compare));
        for at in ticks {
            self.rig.act(VoiceClick::Row(*at));
        }
        self.cue();
        let opened = self.rig.fake.speech.runs().len();
        let text = Redacted::new(SET_TEXT.into());
        self.rig.act(VoiceClick::Compare(text));
        let now = self.rig.fake.speech.runs().len();
        self.compare.extend(opened..now);
    }
}
