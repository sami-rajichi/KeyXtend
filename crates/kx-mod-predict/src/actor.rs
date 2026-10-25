//! The module's actor: it follows the keyboard's pieces and the focused box, tells the lane, types picks through the
//! keyboard, learns, and saves the learned words and emails. A plain struct driven by messages, plus a thread loop
//! that keeps the language in front's pack parsed once per switch.

mod asks;
mod lang;
mod notes;
mod picks;
mod stats;
mod store;
mod typing;

use crate::days::Days;
use crate::email::{EmailConfig, Emails};
use crate::lane::Chips;
use crate::learn::{LearnConfig, Learner};
use crate::menu::Menu;
use crate::pack::Pack;
use crate::score::ScoreConfig;
use crate::settings::PredictSettings;
use crate::track::{Show, Tracker};
use kx_module_api::ports::{
    ChipPicked, FieldChange, FieldConfig, FieldKind, FieldRun, FieldWatch, Piece, SealedStore,
    Suggested, TypingMode, WordPacks, WordsAsk,
};
use kx_module_api::{Bus, Clock, SettingsHandle};
use stats::Stats;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use store::Saves;

use notes::Notes;
pub(crate) use notes::notice;
pub use stats::STOPPED_LINE;
pub use store::{EMAILS_BLOB, emails_blob, learned_blob};

/// A message for the actor, from the bus, the box or the module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    /// The keyboard typed a piece.
    Typed(Piece),
    /// The keyboard's latches changed.
    Mode(TypingMode),
    /// The keyboard shows a layout in language `iso`.
    Lang {
        /// The language, as `LangLabel::iso` names it.
        iso: String,
        /// It is written right to left.
        rtl: bool,
    },
    /// A press landed on our windows (true) or elsewhere.
    Pressed(bool),
    /// A chip was tapped.
    Picked(ChipPicked),
    /// The menu, the list or a control chip was tapped.
    Ask(WordsAsk),
    /// The mouse module asks whether presses are wanted.
    PressesAsk,
    /// Focus moved.
    Focus(FieldChange),
    /// The module stops.
    Quit,
}

/// What the loop does after a message.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Go on with the same pack.
    Go,
    /// The language changed: close the pack, then open the new one.
    Switch,
    /// The actor is done.
    Quit,
}

/// The actor's ports, from the module's start or a test.
pub struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for the save timers.
    pub clock: Arc<dyn Clock>,
    /// The day number, for the learner.
    pub days: Arc<dyn Days>,
    /// The word packs.
    pub packs: Arc<dyn WordPacks>,
    /// The sealed store for the learned words and emails.
    pub store: Arc<dyn SealedStore>,
    /// The module's own settings, for Pause and Off.
    pub handle: SettingsHandle,
    /// The focused box's watch; none when it did not start.
    pub field: Option<Box<dyn FieldRun>>,
}

/// Starts a watch of the focused box with `cfg` that tells `tx`; `None`, logged, when the platform refuses.
pub fn watch(
    field: &dyn FieldWatch,
    cfg: FieldConfig,
    tx: &Sender<Msg>,
) -> Option<Box<dyn FieldRun>> {
    let tx = tx.clone();
    let tell = Box::new(move |change| {
        // Only a stopped actor misses this, and its watch goes with it.
        let _ = tx.send(Msg::Focus(change));
    });
    field
        .watch(cfg, tell)
        .inspect_err(|error| tracing::warn!(%error, "the focused box cannot be watched"))
        .ok()
}

/// The prediction module's runtime: one per start, on its own thread.
pub struct Actor {
    ports: Ports,
    /// The settings, with `on` and `learn` as the menu last set them.
    settings: PredictSettings,
    score: ScoreConfig,
    learn_cfg: LearnConfig,
    email_cfg: EmailConfig,
    /// The language in front, and whether it reads right to left.
    iso: String,
    rtl: bool,
    kind: FieldKind,
    mode: TypingMode,
    tracker: Tracker,
    learner: Option<Learner>,
    emails: Emails,
    menu: Menu,
    /// The lane's round, its chips and what the tracker showed for them.
    round: u32,
    chips: Chips,
    show: Option<Show>,
    /// A pick was sent and the keyboard has not told it yet.
    pending: bool,
    /// The last lane told, for `Again`.
    last: Option<Suggested>,
    /// The last `PressesWanted` told.
    pressing: bool,
    saves: Saves,
    notes: Notes,
    stats: Stats,
}

impl Actor {
    /// An actor with no language yet; it reads the time and the day from its ports.
    #[must_use]
    pub fn new(ports: Ports, settings: &PredictSettings) -> Self {
        Self {
            ports,
            score: settings.score_config(),
            learn_cfg: settings.learn_config(),
            email_cfg: settings.email_config(),
            iso: String::new(),
            rtl: false,
            kind: FieldKind::None,
            mode: TypingMode::default(),
            tracker: Tracker::new(settings.track_config()),
            learner: None,
            emails: Emails::default(),
            menu: Menu::default(),
            round: 0,
            chips: Chips::default(),
            show: None,
            pending: false,
            last: None,
            pressing: false,
            saves: Saves::new(Duration::from_secs(settings.save_s.into())),
            notes: Notes::default(),
            stats: Stats::default(),
            settings: settings.clone(),
        }
    }

    /// The thread's loop: begins, then for each language opens its pack once and serves messages until a switch or
    /// `Quit`; the old pack closes before the next opens.
    pub fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        loop {
            let view = self.open();
            let pack = self.parse(view.as_deref());
            self.arrive(pack.as_ref());
            if self.serve(pack.as_ref(), inbox) == Flow::Quit {
                return;
            }
        }
    }

    /// Handles messages with `seed` until one switches the language or quits, ticking after each wake.
    fn serve(&mut self, seed: Option<&Pack<'_>>, inbox: &Receiver<Msg>) -> Flow {
        loop {
            let next = match self.wait() {
                Some(wait) => inbox.recv_timeout(wait),
                None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            let flow = match next {
                Ok(msg) => self.handle(seed, msg),
                Err(RecvTimeoutError::Timeout) => Flow::Go,
                Err(RecvTimeoutError::Disconnected) => self.handle(seed, Msg::Quit),
            };
            if flow != Flow::Go {
                return flow;
            }
            self.tick();
        }
    }

    /// Handles one message with the open `seed`; the loop opens the new language on `Switch`.
    pub fn handle(&mut self, seed: Option<&Pack<'_>>, msg: Msg) -> Flow {
        let told = match msg {
            Msg::Typed(piece) => self.typed(seed, &piece),
            Msg::Mode(mode) => {
                self.mode = mode;
                true
            }
            Msg::Lang { iso, rtl } => return self.lang(seed, iso, rtl),
            Msg::Pressed(ours) => !ours && self.clicked(seed),
            Msg::Picked(pick) => self.picked(seed, pick),
            Msg::Ask(ask) => self.ask(seed, ask),
            Msg::PressesAsk => {
                self.presses_again();
                false
            }
            Msg::Focus(change) => self.focus(seed, change),
            Msg::Quit => {
                self.quit();
                return Flow::Quit;
            }
        };
        if told {
            self.tell(seed);
        }
        Flow::Go
    }

    /// How long the loop waits for a message before it ticks: until the next save is due, else no limit.
    #[must_use]
    pub fn wait(&self) -> Option<Duration> {
        self.saves.next(self.ports.clock.now())
    }
}
