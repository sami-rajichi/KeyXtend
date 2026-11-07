//! The module's start through the shared fake host, which gates services as the kernel does; its actor runs on its
//! own thread, so each test waits on a channel for what it publishes, never sleeping.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_mod_vault::book::Notebook;
use kx_mod_vault::entry::{Draft, Stamp};
use kx_mod_vault::store::Store;
use kx_mod_vault::{MANIFEST, VaultModule, VaultSettings};
use kx_module_api::ports::{
    BlobName, Boxes, BoxesKey, Calendar, CalendarKey, Date, EntryKind, FillAsk, FillLive,
    FillPanel, Places, PlacesKey, Random, RandomKey, SealedError, SealedStore, VaultEmails,
    VaultStoreKey,
};
use kx_module_api::{
    Capability, Host, Module, ModuleCx, ModuleError, Notice, Redacted, ServiceError, ServiceKey,
    Subscription, Zeroizing, keys,
};
use kx_platform_fake::{FakeBoxes, FakeCalendar, FakePlaces, FakeRandom, FakeSealedStore, PARIS};
use kx_test_support::host::{FakeHost, Stored};

mod panic;

/// Everything the manifest declares.
const ALL: [Capability; 3] = [
    Capability::Vault,
    Capability::VaultData,
    Capability::ReadClipboard,
];
/// How long a test waits for the actor's thread.
const WITHIN: Duration = Duration::from_secs(10);
/// The saved person's email.
const EMAIL: &str = "alex@example.com";

/// A host granting `granted`, holding `sealed` as the notebook store, a calendar, random bytes, the boxes and the places.
fn host(granted: &[Capability], sealed: &FakeSealedStore) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    let store: Arc<dyn SealedStore> = Arc::new(sealed.clone());
    put::<VaultStoreKey>(&mut host, store);
    let day = Date::new(2026, 11, 2).unwrap();
    let calendar: Arc<dyn Calendar> = Arc::new(FakeCalendar::new(day, 9, 30, PARIS));
    put::<CalendarKey>(&mut host, calendar);
    let random: Arc<dyn Random> = Arc::new(FakeRandom::new(3));
    put::<RandomKey>(&mut host, random);
    let boxes: Arc<dyn Boxes> = Arc::new(FakeBoxes::new());
    put::<BoxesKey>(&mut host, boxes);
    let places: Arc<dyn Places> = Arc::new(FakePlaces::new());
    put::<PlacesKey>(&mut host, places);
    host
}

/// Stores `api` under `K` on `host`, gated as `K` says.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    let stored: Stored = (K::CAPABILITY, Box::new(api));
    host.services.insert(K::ID, stored);
}

/// Starts `module` on `host`.
fn start(module: &mut VaultModule, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// What the module publishes from now on, and the guards that keep it coming.
struct Ears {
    emails: Receiver<Vec<String>>,
    panels: Receiver<bool>,
    notices: Arc<Mutex<Vec<Notice>>>,
    _guards: Vec<Subscription>,
}

/// Listens on `host`'s bus.
fn ears(host: &FakeHost) -> Ears {
    let bus = host.bus();
    let (to_emails, emails) = mpsc::channel();
    let (to_panels, panels) = mpsc::channel();
    let notices = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&notices);
    let text = |e: &VaultEmails| e.emails.iter().map(|s| s.expose().to_string()).collect();
    let guards = vec![
        bus.subscribe(move |e: &VaultEmails| drop(to_emails.send(text(e))))
            .unwrap(),
        bus.subscribe(move |p: &FillPanel| {
            let _ = to_panels.send(p.open);
        })
        .unwrap(),
        bus.subscribe(move |n: &Notice| sink.lock().unwrap().push(n.clone()))
            .unwrap(),
    ];
    Ears {
        emails,
        panels,
        notices,
        _guards: guards,
    }
}

/// A notebook of one person with an email, saved into `sealed`.
fn saved_person(sealed: &FakeSealedStore) -> Notebook {
    let mut book = Notebook::default();
    let mut person = Draft::new(EntryKind::Person);
    person.name = Redacted::new("Alex".to_owned());
    person.fields[1].value = Redacted::new(Zeroizing::new(EMAIL.to_owned()));
    let limits = VaultSettings::from_defaults().unwrap().limits();
    book.add(person, Stamp::from_unix_ms(1), &FakeRandom::new(1), &limits)
        .unwrap();
    Store::new(Arc::new(sealed.clone())).save(&book).unwrap();
    book
}

#[test]
fn it_starts_with_the_grants_and_tells_the_saved_emails() {
    let sealed = FakeSealedStore::new();
    saved_person(&sealed);
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let mut module = VaultModule::default();
    assert_eq!(start(&mut module, &mut host), Ok(()));
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(vec![EMAIL.to_owned()]));
    assert!(module.running());
    assert_eq!(sealed.loads(), 2, "the notebook and the backup state");
}

#[test]
fn fill_ask_on_the_bus_reaches_the_actor_which_answers_the_panel_and_the_emails() {
    let sealed = FakeSealedStore::new();
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(
        ears.emails.recv_timeout(WITHIN),
        Ok(Vec::new()),
        "an empty notebook"
    );
    host.bus().publish(&FillAsk).unwrap();
    assert_eq!(ears.panels.recv_timeout(WITHIN), Ok(false));
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(Vec::new()));
}

#[test]
fn a_damaged_notebook_starts_empty_and_says_so_once() {
    let sealed = FakeSealedStore::new();
    saved_person(&sealed);
    sealed.fail_load(SealedError::Damaged);
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(Vec::new()));
    let heard = ears.notices.lock().unwrap();
    assert_eq!(heard.len(), 1);
    assert_eq!(
        (heard[0].key, heard[0].module),
        (keys::VAULT_DAMAGED, Some(MANIFEST.id))
    );
}

#[test]
fn a_locked_or_failing_store_fails_the_start_with_nothing_written() {
    for error in [SealedError::Locked, SealedError::Io] {
        let sealed = FakeSealedStore::new();
        sealed.fail_load(error);
        let mut module = VaultModule::default();
        let got = start(&mut module, &mut host(&ALL, &sealed));
        assert!(matches!(got, Err(ModuleError::Start(_))), "{error:?}");
        assert_eq!((module.running(), sealed.saves()), (false, 0));
    }
}

#[test]
fn the_event_gates_alone_cannot_reach_the_store() {
    let refused = ServiceError::NotGranted(VaultStoreKey::ID, Capability::VaultData);
    let sealed = FakeSealedStore::new();
    let mut module = VaultModule::default();
    let gates = [Capability::Vault, Capability::ReadClipboard];
    let got = start(&mut module, &mut host(&gates, &sealed));
    assert_eq!(got, Err(ModuleError::Service(refused)));
    assert_eq!(sealed.loads(), 0);
}

#[test]
fn without_the_calendar_it_does_not_start() {
    let sealed = FakeSealedStore::new();
    let mut host = host(&ALL, &sealed);
    host.services.remove(&CalendarKey::ID);
    let mut module = VaultModule::default();
    let got = start(&mut module, &mut host);
    let missing = ServiceError::Missing(CalendarKey::ID);
    assert_eq!(got, Err(ModuleError::Service(missing)));
    assert!(!module.running());
}

#[test]
fn a_stop_shuts_the_panel_empties_the_emails_and_it_starts_again() {
    let sealed = FakeSealedStore::new();
    saved_person(&sealed);
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(vec![EMAIL.to_owned()]));
    module.stop();
    assert!(!module.running());
    assert_eq!(ears.panels.recv_timeout(WITHIN), Ok(false));
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(Vec::new()));
    start(&mut module, &mut host).unwrap();
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(vec![EMAIL.to_owned()]));
    assert_eq!(sealed.loads(), 4);
}

#[test]
fn it_says_it_runs_from_its_start_to_its_stop() {
    let sealed = FakeSealedStore::new();
    let mut host = host(&ALL, &sealed);
    let (to_lives, lives) = mpsc::channel();
    let ear = move |e: &FillLive| {
        let _ = to_lives.send(e.live);
    };
    let _ear = host.bus().subscribe(ear).unwrap();
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(lives.recv_timeout(WITHIN), Ok(true));
    module.stop();
    assert_eq!(lives.recv_timeout(WITHIN), Ok(false));
}

#[test]
fn a_newer_versions_notebook_fails_the_start_and_stays_as_it_was() {
    let sealed = FakeSealedStore::new();
    saved_person(&sealed);
    // The blob's name and its version byte, after the four-byte mark, as the format pins them.
    let name = BlobName::new("notebook").unwrap();
    let mut newer = sealed.blob(&name).unwrap();
    newer[4] += 1;
    sealed.plant(&name, newer.clone());
    let mut module = VaultModule::default();
    let got = start(&mut module, &mut host(&ALL, &sealed));
    assert!(matches!(got, Err(ModuleError::Start(_))));
    assert_eq!((sealed.blob(&name), sealed.saves()), (Some(newer), 1));
}

#[test]
fn a_damaged_backup_state_starts_clean_is_set_aside_and_says_so_once() {
    let (sealed, name) = (FakeSealedStore::new(), BlobName::new("backup").unwrap());
    sealed.plant(&name, b"not a backup".to_vec());
    let mut host = host(&ALL, &sealed);
    let ears = ears(&host);
    let mut module = VaultModule::default();
    start(&mut module, &mut host).unwrap();
    assert_eq!(ears.emails.recv_timeout(WITHIN), Ok(Vec::new()));
    let keys: Vec<_> = ears.notices.lock().unwrap().iter().map(|n| n.key).collect();
    assert_eq!(keys, [keys::VAULT_BACKUP_DAMAGED]);
    let aside = BlobName::new("backup-aside").unwrap();
    assert_eq!(sealed.blob(&aside), Some(b"not a backup".to_vec()));
}

#[test]
fn a_newer_versions_backup_state_fails_the_start_and_stays_as_it_was() {
    let (sealed, name) = (FakeSealedStore::new(), BlobName::new("backup").unwrap());
    sealed.plant(&name, b"KXBK\x02".to_vec());
    let mut module = VaultModule::default();
    let got = start(&mut module, &mut host(&ALL, &sealed));
    assert!(matches!(got, Err(ModuleError::Start(_))));
    assert_eq!(
        (sealed.blob(&name), sealed.saves()),
        (Some(b"KXBK\x02".to_vec()), 0)
    );
}
