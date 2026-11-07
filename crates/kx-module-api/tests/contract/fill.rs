//! Quick-fill's events and ports (ADR-0038 decision 3, ADR-0039): the panel's state, keys, clicks, fills and emails
//! travel only with `Vault`, Paste's hand-over only with `ReadClipboard`, each port needs `VaultData`, and the open
//! toggle is open.

use kx_module_api::Capability::{InjectInput, ReadClipboard, SecretStore, Vault, VaultData};
use kx_module_api::ports::{
    BlobName, BoxConfig, BoxError, BoxPart, BoxRun, Boxes, BoxesKey, ClipTake, ClipTaken, ClipText,
    FillAct, FillAsk, FillButton, FillClick, FillClose, FillKey, FillOpen, FillPanel, FillType,
    FillTyped, FillView, Found, KeepassImport, Opened, Padlock, PanelKey, Place, PlaceError,
    PlaceState, Places, PlacesKey, SealedError, SealedStore, VaultEmails, VaultStoreKey,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Redacted, Secret,
    ServiceError, ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// A vault module declaring its gates, and `SecretStore` and `InjectInput` to show that other grants open none.
static VAULT: Manifest = Manifest {
    id: ModuleId::new("vault"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[VaultStoreKey::ID, BoxesKey::ID, PlacesKey::ID],
    provides: &[],
    capabilities: &[Vault, VaultData, ReadClipboard, SecretStore, InjectInput],
    settings: None,
};

/// A platform whose store is locked, whose worker fails and whose places are all gone.
struct Shut;

impl SealedStore for Shut {
    fn load(&self, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        Err(SealedError::Locked)
    }

    fn save(&self, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }
}

impl BoxRun for Shut {
    fn padlock(&mut self, _: &Secret, _: u8) -> Result<Padlock, BoxError> {
        Err(BoxError::Failed)
    }

    fn seal(&mut self, _: &Padlock, _: &[BoxPart]) -> Result<Redacted<Vec<u8>>, BoxError> {
        Err(BoxError::Failed)
    }

    fn open(&mut self, _: &[u8], _: &Secret) -> Result<Opened, BoxError> {
        Err(BoxError::Failed)
    }

    fn check(&mut self, _: &Padlock, _: &Secret) -> Result<bool, BoxError> {
        Err(BoxError::Failed)
    }

    fn keepass(&mut self, _: &[u8], _: &Secret) -> Result<KeepassImport, BoxError> {
        Err(BoxError::Failed)
    }
}

impl Boxes for Shut {
    fn start(&self, _: BoxConfig) -> Box<dyn BoxRun> {
        Box::new(Shut)
    }
}

impl Places for Shut {
    fn suggested(&self) -> Vec<Place> {
        Vec::new()
    }

    fn folders(&self, _: Option<&Path>) -> Result<Vec<Place>, PlaceError> {
        Err(PlaceError::Missing)
    }

    fn files(&self, _: &Path, _: &str) -> Result<Vec<Place>, PlaceError> {
        Err(PlaceError::Missing)
    }

    fn find(&self) -> Vec<Found> {
        Vec::new()
    }

    fn modified(&self, _: &Path) -> Option<i64> {
        None
    }

    fn read(&self, _: &Path, _: u64) -> Result<Redacted<Vec<u8>>, PlaceError> {
        Err(PlaceError::Missing)
    }

    fn write(&self, _: &Path, _: &[u8]) -> Result<(), PlaceError> {
        Err(PlaceError::Missing)
    }

    fn state(&self, _: &Path, _: u64) -> PlaceState {
        PlaceState::Missing
    }
}

/// Puts `api` into `host` under `K`'s id and gate.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    host.services.insert(K::ID, (K::CAPABILITY, Box::new(api)));
}

/// A host granted `granted`, holding the three ports as the platform would provide them.
fn host(granted: &[Capability]) -> FakeHost {
    let mut host = FakeHost::new(&VAULT);
    host.granted.extend(granted);
    put::<VaultStoreKey>(&mut host, Arc::new(Shut));
    put::<BoxesKey>(&mut host, Arc::new(Shut));
    put::<PlacesKey>(&mut host, Arc::new(Shut));
    host
}

/// Sends each gated event but the view, subscribes to each, then sends the open `FillOpen`, `FillClose` and `FillAsk`.
fn gated(bus: &Bus) -> [Result<(), EventError>; 16] {
    let click = FillClick::Button(FillButton::Close);
    [
        bus.publish(&FillPanel { open: true }),
        bus.publish(&FillKey(PanelKey::Up)),
        bus.publish(&FillAct { round: 1, click }),
        bus.publish(&FillType {
            round: 1,
            parts: Vec::new(),
            tab_gap: Duration::ZERO,
        }),
        bus.publish(&FillTyped { round: 1, ok: true }),
        bus.publish(&VaultEmails { emails: Vec::new() }),
        bus.subscribe(|_: &FillPanel| {}).map(drop),
        bus.subscribe(|_: &FillKey| {}).map(drop),
        bus.subscribe(|_: &FillView| {}).map(drop),
        bus.subscribe(|_: &FillAct| {}).map(drop),
        bus.subscribe(|_: &FillType| {}).map(drop),
        bus.subscribe(|_: &FillTyped| {}).map(drop),
        bus.subscribe(|_: &VaultEmails| {}).map(drop),
        bus.publish(&FillOpen),
        bus.publish(&FillClose),
        bus.publish(&FillAsk),
    ]
}

/// The refusal of `E` for want of `cap`.
fn refused<E: Event>(cap: Capability) -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, cap))
}

#[test]
fn without_vault_no_value_travels_but_the_open_toggle_the_close_and_the_ask_do() {
    let bus = host(&[ReadClipboard, SecretStore, InjectInput]).bus();
    let want = [
        refused::<FillPanel>(Vault),
        refused::<FillKey>(Vault),
        refused::<FillAct>(Vault),
        refused::<FillType>(Vault),
        refused::<FillTyped>(Vault),
        refused::<VaultEmails>(Vault),
        refused::<FillPanel>(Vault),
        refused::<FillKey>(Vault),
        refused::<FillView>(Vault),
        refused::<FillAct>(Vault),
        refused::<FillType>(Vault),
        refused::<FillTyped>(Vault),
        refused::<VaultEmails>(Vault),
        Ok(()),
        Ok(()),
        Ok(()),
    ];
    assert_eq!(gated(&bus), want);
}

#[test]
fn vault_opens_every_quick_fill_event() {
    let bus = host(&[Vault]).bus();
    assert!(gated(&bus).iter().all(Result::is_ok));
}

#[test]
fn the_delete_key_and_a_placed_caret_need_vault() {
    let closed = host(&[ReadClipboard, SecretStore]).bus();
    let key = FillKey(PanelKey::Delete);
    let caret = FillAct {
        round: 1,
        click: FillClick::Caret { at: 1.into() },
    };
    assert_eq!(closed.publish(&key), refused::<FillKey>(Vault));
    assert_eq!(closed.publish(&caret), refused::<FillAct>(Vault));
    let open = host(&[Vault]).bus();
    assert_eq!(open.publish(&key), Ok(()));
    assert_eq!(open.publish(&caret), Ok(()));
}

#[test]
fn the_paste_hand_over_needs_read_clipboard_not_vault() {
    let taken = ClipTaken {
        round: 1,
        text: ClipText::Empty,
    };
    let take = ClipTake::new(1, 9);
    let closed = host(&[Vault]).bus();
    assert_eq!(closed.publish(&take), refused::<ClipTake>(ReadClipboard));
    assert_eq!(closed.publish(&taken), refused::<ClipTaken>(ReadClipboard));
    let open = host(&[ReadClipboard]).bus();
    assert_eq!(open.publish(&take), Ok(()));
    assert_eq!(open.subscribe(|_: &ClipTaken| {}).map(drop), Ok(()));
}

#[test]
fn each_port_needs_vault_data_and_vault_or_secret_store_opens_none() {
    let mut host = host(&[Vault, SecretStore, ReadClipboard]);
    let cx = ModuleCx::new(&mut host);
    let store = ServiceError::NotGranted(VaultStoreKey::ID, VaultData);
    assert_eq!(cx.service::<VaultStoreKey>().err(), Some(store));
    let boxes = ServiceError::NotGranted(BoxesKey::ID, VaultData);
    assert_eq!(cx.service::<BoxesKey>().err(), Some(boxes));
    let places = ServiceError::NotGranted(PlacesKey::ID, VaultData);
    assert_eq!(cx.service::<PlacesKey>().err(), Some(places));
}

#[test]
fn ports_reached_through_vault_data_answer() {
    let mut host = host(&[VaultData]);
    let cx = ModuleCx::new(&mut host);
    let name = BlobName::new("notebook").unwrap();
    let store = cx.service::<VaultStoreKey>().unwrap();
    assert_eq!(store.load(&name), Err(SealedError::Locked));
    let cfg = BoxConfig {
        deadline_ms: 1,
        idle_s: 1,
        restart_ms: 1,
        restart_max_ms: 1,
    };
    let mut run = cx.service::<BoxesKey>().unwrap().start(cfg);
    let password = Secret::default();
    assert_eq!(run.padlock(&password, 1), Err(BoxError::Failed));
    let places = cx.service::<PlacesKey>().unwrap();
    assert_eq!(places.state(Path::new("backup"), 1), PlaceState::Missing);
}
