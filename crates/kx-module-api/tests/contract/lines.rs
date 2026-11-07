//! Which line a panel shows travels only with that panel's gate (ADR-0040), its `Debug` never shows a search, and the
//! keyboard's wish for presses carries nothing, so it passes with no grants.

use kx_module_api::Capability::{InjectInput, ReadClipboard, Vault};
use kx_module_api::ports::{ClipLine, FillLine, PanelLine, PanelPressesWanted};
use kx_module_api::{Bus, Capability, Event, EventError, Host, Manifest, ModuleId, Redacted};
use kx_test_support::host::FakeHost;

/// A search that appears nowhere else ("hello").
const SEARCH: &str = "مرحبا";

/// A module declaring both gates, and `InjectInput` to show that another grant opens neither.
static PANEL: Manifest = Manifest {
    id: ModuleId::new("panel"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[Vault, ReadClipboard, InjectInput],
    settings: None,
};

/// A bus granted `granted`.
fn bus(granted: &[Capability]) -> Bus {
    let mut host = FakeHost::new(&PANEL);
    host.granted.extend(granted);
    host.bus()
}

/// A search line holding `SEARCH`.
fn search() -> PanelLine {
    PanelLine::Search(Redacted::new(SEARCH.to_owned()))
}

/// The refusal of `E` for want of `cap`.
fn refused<E: Event>(cap: Capability) -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, cap))
}

#[test]
fn the_clipboards_line_needs_read_clipboard() {
    let closed = bus(&[Vault, InjectInput]);
    let want = refused::<ClipLine>(ReadClipboard);
    assert_eq!(closed.publish(&ClipLine(search())), want);
    assert_eq!(closed.subscribe(|_: &ClipLine| {}).map(drop), want);
    let open = bus(&[ReadClipboard]);
    assert_eq!(open.publish(&ClipLine(search())), Ok(()));
    assert_eq!(open.subscribe(|_: &ClipLine| {}).map(drop), Ok(()));
}

#[test]
fn quick_fills_line_needs_vault() {
    let closed = bus(&[ReadClipboard, InjectInput]);
    let want = refused::<FillLine>(Vault);
    assert_eq!(closed.publish(&FillLine(search())), want);
    assert_eq!(closed.subscribe(|_: &FillLine| {}).map(drop), want);
    let open = bus(&[Vault]);
    assert_eq!(open.publish(&FillLine(search())), Ok(()));
    assert_eq!(open.subscribe(|_: &FillLine| {}).map(drop), Ok(()));
}

#[test]
fn a_line_reaches_its_listener_with_its_text() {
    let open = bus(&[Vault]);
    let (tx, rx) = std::sync::mpsc::channel();
    let _sub = open
        .subscribe(move |e: &FillLine| tx.send(e.clone()).unwrap())
        .unwrap();
    open.publish(&FillLine(search())).unwrap();
    assert_eq!(rx.try_recv(), Ok(FillLine(search())));
}

#[test]
fn the_panels_wish_for_presses_passes_with_no_grants() {
    assert_eq!(bus(&[]).publish(&PanelPressesWanted { on: true }), Ok(()));
}
