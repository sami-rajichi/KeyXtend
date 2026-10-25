//! The module's thread over the bus: it asks for the layout, follows the keyboard's pieces, types a pick through the
//! keyboard, answers an ask for presses, saves at stop and says when it fails; with the pack builder, it closes one
//! pack before it opens the next.

use crate::rig::{blob, learner};
use kx_mod_predict::{Days, MANIFEST, PredictModule, SystemDays};
use kx_module_api::ports::{
    ChipPicked, FieldKind, FieldWatch, FieldWatchKey, Lane, LayoutAsk, LayoutChanged, LayoutId,
    PackError, PackView, Piece, PressesAsk, PressesWanted, SealedStore, SealedStoreKey, Suggested,
    TypeWord, Typed, WordPacks, WordPacksKey,
};
use kx_module_api::{
    Bus, Event, Module, ModuleCx, Notice, NoticeArg, Redacted, ServiceKey, Subscription, args, keys,
};
use kx_platform_fake::{FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// The longest a threaded test waits for the module; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);

/// A running module and a bus to talk to it.
struct Live {
    host: FakeHost,
    module: PredictModule,
    bus: Bus,
}

impl Live {
    /// Starts the module on the fakes after `listen` subscribes.
    fn start<T>(
        field: &FakeFieldWatch,
        packs: Arc<dyn WordPacks>,
        store: &FakeSealedStore,
        listen: impl FnOnce(&Bus) -> T,
    ) -> (Self, T) {
        let mut host = FakeHost::new(&MANIFEST);
        host.granted = MANIFEST.capabilities.to_vec();
        let field: Arc<dyn FieldWatch> = Arc::new(field.clone());
        let store: Arc<dyn SealedStore> = Arc::new(store.clone());
        let stored: Stored = (FieldWatchKey::CAPABILITY, Box::new(field));
        host.services.insert(FieldWatchKey::ID, stored);
        let stored: Stored = (WordPacksKey::CAPABILITY, Box::new(packs));
        host.services.insert(WordPacksKey::ID, stored);
        let stored: Stored = (SealedStoreKey::CAPABILITY, Box::new(store));
        host.services.insert(SealedStoreKey::ID, stored);
        let bus = Bus::new(Arc::clone(&host.core));
        let heard = listen(&bus);
        let mut module = PredictModule::default();
        module.start(&mut ModuleCx::new(&mut host)).unwrap();
        (Self { host, module, bus }, heard)
    }

    /// Publishes `event` as the keyboard, the layouts or the UI would.
    fn send<E: Event>(&self, event: &E) {
        self.bus.publish(event).unwrap();
    }

    /// Stops the module as the kernel does: its handlers leave the bus first.
    fn stop(&mut self) {
        self.host.held.clear();
        self.module.stop();
    }
}

/// Sends every `E` published on `bus` to the returned channel.
fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let sub = bus.subscribe(move |e: &E| {
        let _ = tx.send(e.clone());
    });
    (sub.unwrap(), rx)
}

/// Waits for a lane whose words are `want`.
fn lane_of(lanes: &Receiver<Suggested>, want: &[&str]) -> Suggested {
    loop {
        let lane = lanes.recv_timeout(WAIT).unwrap();
        if let Lane::Chips(chips) = &lane.lane {
            let words: Vec<&str> = chips.iter().map(|c| c.text.expose().as_str()).collect();
            if words == want {
                return lane;
            }
        }
    }
}

/// The language in front is `iso`, which the OS names `name`.
fn named(iso: &str, name: &str) -> LayoutChanged {
    let (id, rtl) = (LayoutId::default(), false);
    LayoutChanged {
        id,
        iso: iso.to_owned(),
        name: name.to_owned(),
        rtl,
    }
}

/// The language in front is `iso`, named by its code.
fn lang(iso: &str) -> LayoutChanged {
    named(iso, iso)
}

#[test]
fn the_thread_asks_the_layout_follows_typing_types_a_pick_and_saves_at_stop() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    // Learned today, so the real day's purge keeps them; "world" keeps the first lane from matching the typed one.
    let known = learner("en", &["hello", "help", "world"], SystemDays.today());
    store.plant(&blob("en"), known.to_bytes());
    field.focus(FieldKind::Text);
    field.set_ends(Some(true));
    let listen = |bus: &Bus| {
        (
            hear::<LayoutAsk>(bus),
            hear::<Suggested>(bus),
            hear::<TypeWord>(bus),
            hear::<PressesWanted>(bus),
        )
    };
    let (mut run, ((_a, asks), (_l, lanes), (_w, words), (_p, presses))) =
        Live::start(&field, Arc::new(FakeWordPacks::new()), &store, listen);
    assert_eq!(asks.recv_timeout(WAIT), Ok(LayoutAsk));
    run.send(&lang("en"));
    run.send(&Typed {
        piece: Piece::Text(Redacted::new("hel".to_owned())),
    });
    let shown = lane_of(&lanes, &["hello", "help"]);
    run.send(&ChipPicked {
        round: shown.round,
        index: 0,
    });
    let word = words.recv_timeout(WAIT).unwrap();
    assert_eq!(
        (word.round, word.erase, word.text.expose().as_str()),
        (shown.round, 3, "hello ")
    );
    run.send(&Typed {
        piece: Piece::Text(Redacted::new("lo ".to_owned())),
    });
    assert_eq!(presses.recv_timeout(WAIT).map(|p| p.on), Ok(true));
    run.stop();
    let last = presses.try_iter().last().map(|p| p.on);
    assert_eq!(last, Some(false), "the stop joined and let presses go");
    assert_eq!(store.saves(), 1, "the learned word saved at stop");
}

#[test]
fn a_layouts_module_that_starts_later_still_brings_the_language() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let known = learner("en", &["hello", "help"], SystemDays.today());
    store.plant(&blob("en"), known.to_bytes());
    let listen = |bus: &Bus| (hear::<LayoutAsk>(bus), hear::<Suggested>(bus));
    let (mut run, ((_a, asks), (_l, lanes))) =
        Live::start(&field, Arc::new(FakeWordPacks::new()), &store, listen);
    assert_eq!(asks.recv_timeout(WAIT), Ok(LayoutAsk), "nobody answers yet");
    run.send(&lang("en"));
    // The later start's own `LayoutChanged` loads the learner, whose words show.
    lane_of(&lanes, &["hello", "help"]);
    run.stop();
}

#[test]
fn an_ask_for_presses_gets_the_current_flag_again() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    field.focus(FieldKind::Text);
    let listen = |bus: &Bus| (hear::<PressesWanted>(bus), hear::<Suggested>(bus));
    let (mut run, ((_p, presses), (_l, lanes))) =
        Live::start(&field, Arc::new(FakeWordPacks::new()), &store, listen);
    let on = |rx: &Receiver<PressesWanted>| rx.recv_timeout(WAIT).map(|p| p.on);
    run.send(&PressesAsk);
    assert_eq!(on(&presses), Ok(false), "no word yet");
    run.send(&lang("en"));
    run.send(&Typed {
        piece: Piece::Text(Redacted::new("hel".to_owned())),
    });
    assert_eq!(on(&presses), Ok(true), "a word is held");
    // The word's lane went out just before its flag, so the lanes so far are all in.
    let _ = lanes.try_iter().count();
    for _ in 0..2 {
        run.send(&PressesAsk);
        assert_eq!(on(&presses), Ok(true), "told again, with no new word");
    }
    // The second answer comes after the first ask was fully handled.
    assert!(lanes.try_recv().is_err(), "an ask tells no new lane");
    run.stop();
}

/// Packs that have none, and panic for French, as a bug on the actor's thread would.
struct Broken;

impl WordPacks for Broken {
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        assert_ne!(iso, "fr", "a broken pack port");
        Ok(None)
    }

    fn ships(&self, _: &str) -> bool {
        false
    }
}

#[test]
fn a_panic_on_the_thread_empties_the_lane_lets_presses_go_and_tells_the_user() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let known = learner("en", &["hello", "help"], SystemDays.today());
    store.plant(&blob("en"), known.to_bytes());
    let listen = |bus: &Bus| {
        (
            hear::<Notice>(bus),
            hear::<PressesWanted>(bus),
            hear::<Suggested>(bus),
        )
    };
    let (mut run, ((_n, notices), (_p, presses), (_l, lanes))) =
        Live::start(&field, Arc::new(Broken), &store, listen);
    run.send(&lang("en"));
    while !matches!(lanes.recv_timeout(WAIT).unwrap().lane, Lane::Chips(_)) {}
    run.send(&lang("fr"));
    while notices.recv_timeout(WAIT).unwrap().key != keys::MODULE_FAILED {}
    let wanted: Vec<bool> = presses.try_iter().map(|p| p.on).collect();
    assert_eq!(wanted, [false], "presses let go");
    let last = lanes.try_iter().last().map(|l| l.lane);
    assert_eq!(last, Some(Lane::Empty), "the chips went");
    run.stop();
}

#[test]
fn a_missing_pack_is_told_by_the_name_the_layouts_gave() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let packs = FakeWordPacks::new();
    packs.ship("ar");
    let listen = |bus: &Bus| hear::<Notice>(bus);
    let (mut run, (_n, notices)) = Live::start(&field, Arc::new(packs), &store, listen);
    run.send(&named("ar", "العربية"));
    let told = notices.recv_timeout(WAIT).unwrap();
    assert_eq!(told.key, keys::WORDS_MISSING);
    let lang = told.args.iter().find(|a| a.0 == args::LANG);
    assert_eq!(lang.map(|a| &a.1), Some(&NoticeArg::Text("العربية".into())));
    run.stop();
}

#[cfg(feature = "build")]
mod closes;
