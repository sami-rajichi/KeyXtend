use super::*;
use kx_module_api::{
    Event, EventError, ModuleId, Settings, SettingsError, SettingsSpec, WriteError, validate_as,
};
use kx_settings::{NOT_A_SETTING, VERSION_KEY};
use serde::{Deserialize, Serialize};

/// A span whose low end may not pass its high end.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Span {
    low: u32,
    high: u32,
}

impl Settings for Span {
    fn check(&self) -> Result<(), SettingsError> {
        if self.low > self.high {
            return Err(SettingsError::Invalid("low is above high".into()));
        }
        Ok(())
    }
}

static SPAN: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "low = 1\nhigh = 5\n",
    migrations: &[],
    validate: validate_as::<Span>,
};

static SPANNED: Manifest = Manifest {
    id: ModuleId::new("spanned"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[Capability::Microphone],
    settings: Some(&SPAN),
};

/// A spec whose validator takes anything, so only the defaults can turn a key away.
static LAX: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "low = 1
",
    migrations: &[],
    validate: |_| Ok(()),
};

/// A module whose settings use the lax spec.
static LAXED: Manifest = Manifest {
    id: ModuleId::new("laxed"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: Some(&LAX),
};

/// A module that declares no capability.
static PLAIN: Manifest = Manifest {
    id: ModuleId::new("plain"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: None,
};

/// An event only a module holding the microphone may hear.
struct Heard;

impl Event for Heard {
    const NAME: &'static str = "heard";
    const CAPABILITY: Option<Capability> = Some(Capability::Microphone);
}

fn values(text: &str) -> Table {
    text.parse().unwrap_or_default()
}

fn span(host: &FakeHost) -> Result<Span, SettingsError> {
    host.sink.read_as::<Span>()
}

#[test]
fn it_starts_on_the_defaults_and_put_changes_both_copies() {
    let mut host = FakeHost::new(&SPANNED);
    assert_eq!(host.settings, values(SPAN.defaults));
    host.put("high", Value::Integer(7));
    assert_eq!(host.settings["high"], Value::Integer(7));
    assert_eq!(span(&host), Ok(Span { low: 1, high: 7 }));
}

#[test]
fn writes_are_checked_together_and_kept_whole_or_not_at_all() {
    let mut host = FakeHost::new(&SPANNED);
    let handle = host.handle();
    let alone = handle.set("low", Value::Integer(8));
    assert!(matches!(alone, Err(WriteError::Invalid(_))), "above high");
    let both = values("low = 8\nhigh = 9");
    assert_eq!(handle.set_all(both.clone()), Ok(()));
    assert_eq!(span(&host), Ok(Span { low: 8, high: 9 }));
    assert_eq!(host.sink.writes(), [(SPANNED.id, both)]);
}

#[test]
fn an_unsaved_write_holds_in_the_section_but_is_never_saved() {
    let mut host = FakeHost::new(&SPANNED);
    let handle = host.handle();
    host.sink.refuse(Some(WriteError::Unsaved));
    let both = handle.set_all(values("low = 2\nhigh = 6"));
    assert_eq!(both, Err(WriteError::Unsaved));
    assert_eq!(span(&host), Ok(Span { low: 2, high: 6 }), "it holds");
    assert!(host.sink.writes().is_empty(), "the file never got it");
    let bad = handle.set("low", Value::Integer(9));
    assert!(matches!(bad, Err(WriteError::Invalid(_))), "checked first");
    assert_eq!(span(&host), Ok(Span { low: 2, high: 6 }));
}

#[test]
fn a_refusal_or_a_refused_key_stores_nothing() {
    let mut host = FakeHost::new(&SPANNED);
    let handle = host.handle();
    host.sink
        .refuse_key("high", WriteError::Refused(SPANNED.id));
    let both = handle.set_all(values("low = 2\nhigh = 6"));
    assert_eq!(both, Err(WriteError::Refused(SPANNED.id)));
    assert_eq!(handle.set("low", Value::Integer(2)), Ok(()), "another key");
    host.sink.refuse(Some(WriteError::Refused(SPANNED.id)));
    let refused = handle.set("low", Value::Integer(3));
    assert_eq!(refused, Err(WriteError::Refused(SPANNED.id)));
    assert_eq!(span(&host), Ok(Span { low: 2, high: 5 }));
}

#[test]
fn the_version_key_is_refused_before_any_check_as_the_store_does() {
    let mut host = FakeHost::new(&SPANNED);
    let handle = host.handle();
    let version = handle.set_all(values(&format!("{VERSION_KEY} = 2\nlow = 2")));
    assert_eq!(version, Err(WriteError::Refused(SPANNED.id)));
    assert_eq!(span(&host), Ok(Span { low: 1, high: 5 }));
    assert!(host.sink.writes().is_empty());
}

#[test]
fn a_key_the_defaults_lack_is_refused_after_the_check_as_the_store_does() {
    let mut host = FakeHost::new(&LAXED);
    let typo = host.handle().set_all(values(
        "low = 2
lwo = 3",
    ));
    let not_a_setting = SettingsError::Invalid(NOT_A_SETTING.to_owned());
    assert_eq!(typo, Err(WriteError::Invalid(not_a_setting)));
    assert_eq!(host.sink.section(), values(LAX.defaults), "nothing kept");
    assert!(host.sink.writes().is_empty());
}

#[test]
fn a_provider_needs_the_capability_of_its_service() {
    let mut host = FakeHost::new(&SPANNED);
    let id = ServiceId::new("mic");
    let cap = Some(Capability::Microphone);
    let refused = host.provide(id, cap, Box::new(1_u8));
    assert_eq!(
        refused,
        Err(ServiceError::NotGranted(id, Capability::Microphone))
    );
    host.granted.push(Capability::Microphone);
    assert_eq!(host.provide(id, cap, Box::new(1_u8)), Ok(()));
    let twice = host.provide(id, cap, Box::new(2_u8));
    assert_eq!(twice, Err(ServiceError::AlreadyProvided(id)));
    assert!(host.service(id, cap).is_ok());
    assert_eq!((host.lookups.get(), host.clock().now()), (1, NOW));
}

#[test]
fn a_grant_the_manifest_never_declared_is_refused_as_the_kernel_does() {
    let mut host = FakeHost::new(&PLAIN);
    host.granted.push(Capability::Microphone);
    let (id, cap) = (ServiceId::new("mic"), Some(Capability::Microphone));
    let refused = ServiceError::NotGranted(id, Capability::Microphone);
    assert_eq!(
        host.provide(id, cap, Box::new(1_u8)),
        Err(refused),
        "provide"
    );
    host.services.insert(id, (cap, Box::new(1_u8)));
    assert_eq!(host.service(id, cap).err(), Some(refused), "service");
    let heard = host.bus().subscribe(|_: &Heard| {}).map(drop);
    let not_granted = EventError::NotGranted(Heard::NAME, Capability::Microphone);
    assert_eq!(heard, Err(not_granted), "bus");
}
