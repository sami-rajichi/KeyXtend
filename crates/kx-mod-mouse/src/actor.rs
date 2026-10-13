//! The module's actor: it starts the hook only while the engine needs it, hands the keyboard's commands to the
//! engine, and turns the engine's notes into bus events. A plain struct driven by messages, plus a thread loop.

mod notes;

use crate::hook_engine::{Cmd, HookEngine, Moves};
use crate::{Engine, EngineCfg, ID, MouseSettings, Note};
use kx_module_api::ports::{
    AssistChanged, GrabState, HookError, HookRun, LatchesAsk, PointerHook, Scan, Toggle,
};
use kx_module_api::{Bus, Notice, NoticeArg, SettingsHandle, args, keys};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

/// A message for the actor, from the bus, the hook's engine or the module; it has no `Debug`, as latches are keys.
pub enum Msg {
    /// The keyboard flips a mode.
    Toggle(Toggle),
    /// The keyboard's latched modifiers changed.
    Latches(Vec<Scan>),
    /// A module asks for the state.
    Ask,
    /// The engine noted something; for a move, the latest point waits in the shared move slot.
    Note(Note),
    /// The engine no longer needs the hook, after applying this many queued commands.
    Idle(u64),
    /// The module stops.
    Stop,
}

/// A running hook, the queue to its engine, and how many commands went into that queue.
struct Live {
    run: Box<dyn HookRun>,
    queue: Sender<Cmd>,
    sent: u64,
}

/// The mouse module's runtime: one per start, on its own thread.
pub struct Actor {
    hook: Arc<dyn PointerHook>,
    bus: Bus,
    store: SettingsHandle,
    settings: MouseSettings,
    /// The actor's own inbox, which each hook engine writes to.
    inbox: Sender<Msg>,
    moves: Arc<Moves>,
    /// Right-click as saved: Grab turns it off only for a while.
    saved: bool,
    /// Right-click and Grab as the keyboard shows them.
    rclick: bool,
    grab: GrabState,
    /// The keyboard's latched modifiers.
    latches: Vec<Scan>,
    live: Option<Live>,
    /// The last start failed; until one works, a start no mouse key asked for fails without a word.
    failing: bool,
}

impl Actor {
    /// An actor with no hook yet; `inbox` sends to its own inbox, for the hook engines it starts.
    #[must_use]
    pub fn new(
        hook: Arc<dyn PointerHook>,
        bus: Bus,
        store: SettingsHandle,
        settings: MouseSettings,
        inbox: Sender<Msg>,
    ) -> Self {
        let saved = settings.rclick;
        Self {
            hook,
            bus,
            store,
            settings,
            inbox,
            moves: Arc::default(),
            saved,
            rclick: saved,
            grab: GrabState::Off,
            latches: Vec::new(),
            live: None,
            failing: false,
        }
    }

    /// The thread's loop: begins, then handles each message until `Stop`.
    pub fn run(mut self, inbox: &Receiver<Msg>) {
        self.begin();
        while let Ok(msg) = inbox.recv() {
            if !self.handle(msg) {
                return;
            }
        }
    }

    /// Starts the hook at once for a saved Right-click, then tells the state and asks the keyboard for its latches.
    pub fn begin(&mut self) {
        if self.saved {
            self.start(None);
            if self.live.is_none() {
                (self.saved, self.rclick) = (false, false);
            }
        }
        self.tell_state();
        self.publish(&LatchesAsk);
    }

    /// Handles one message; false after `Stop`, when the actor is done.
    pub fn handle(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Toggle(which) => self.command(Cmd::Toggle(which)),
            Msg::Latches(held) => self.latch(held),
            Msg::Ask => self.tell_state(),
            Msg::Note(note) => self.note(note),
            Msg::Idle(applied) => self.idle(applied),
            Msg::Stop => {
                self.stop();
                return false;
            }
        }
        true
    }

    /// Stops the hook and tells the keyboard the module is no longer live.
    fn stop(&mut self) {
        self.drop_run();
        let (rclick, grab) = (self.saved, GrabState::Off);
        self.publish(&AssistChanged {
            live: false,
            rclick,
            grab,
        });
    }

    /// Queues `cmd` for the running engine and wakes it, or starts a hook with it.
    fn command(&mut self, cmd: Cmd) {
        let Some(live) = &mut self.live else {
            return self.start(Some(cmd));
        };
        // A run holds its engine until it drops, so the queue stays open while `live` does.
        if live.queue.send(cmd).is_ok() {
            live.sent += 1;
        }
        live.run.wake();
    }

    /// Keeps the keyboard's latches; they start the hook when they hold a key.
    fn latch(&mut self, held: Vec<Scan>) {
        self.latches.clone_from(&held);
        if self.live.is_some() || !held.is_empty() {
            self.command(Cmd::Latches(held));
        }
    }

    /// Starts the hook with a fresh engine in the wanted state, given `first` at once.
    fn start(&mut self, first: Option<Cmd>) {
        let base = EngineCfg::from(&self.settings);
        let mut engine = Engine::new(EngineCfg {
            rclick: self.rclick,
            ..base
        });
        engine.set_latches(self.latches.clone());
        let by_key = matches!(first, Some(Cmd::Toggle(_)));
        let (queue, cmds) = mpsc::channel();
        let moves = Arc::clone(&self.moves);
        let engine = HookEngine::new(engine, first, cmds, self.inbox.clone(), moves);
        match self
            .hook
            .start(Box::new(engine), self.settings.hook_config())
        {
            Ok(run) => {
                run.wake();
                self.failing = false;
                self.live = Some(Live {
                    run,
                    queue,
                    sent: 0,
                });
            }
            Err(error) => self.refused(error, by_key),
        }
    }

    /// The hook did not start, so the modes stay off. Only a mouse key's start tells the user, who can press it
    /// again; other starts warn once per streak of failures.
    fn refused(&mut self, error: HookError, by_key: bool) {
        if by_key || !self.failing {
            tracing::warn!(%error, "mouse hook did not start");
        }
        self.failing = true;
        if by_key {
            self.publish(&Notice {
                key: keys::MOUSE_HOOK_FAILED,
                module: Some(ID),
                args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
            });
        }
    }

    /// Stops the hook once the engine reported idle after every command sent so far.
    fn idle(&mut self, applied: u64) {
        if self.live.as_ref().is_some_and(|live| live.sent == applied) {
            self.drop_run();
        }
    }

    /// Stops the hook, which lets go of anything held, and logs its numbers, never a point or a key.
    fn drop_run(&mut self) {
        let Some(live) = self.live.take() else {
            return;
        };
        let s = live.run.stats();
        drop(live);
        tracing::info!(
            events = s.events,
            p99_us = s.p99_us,
            max_us = s.max_us,
            max_wait_ms = s.max_wait_ms,
            reinstalls = s.reinstalls,
            send_errors = s.send_errors,
            lost_echoes = s.lost_echoes,
            "mouse hook stopped"
        );
    }
}
