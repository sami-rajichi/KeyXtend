//! Quick-fill over the whole app (ADR-0038): it starts on its defaults and the fill key opens its panel, an entry saved
//! there is read again after a restart over the same store, and a hidden run keeps it off with its note and a dimmed
//! fill key, and writes nothing. `fill` types a saved login through the keyboard.

use crate::hidden::tree;
use crate::rig::{Rig, WAIT, char_key, hear};
use keyxtend::boot::Boot;
use keyxtend::drain;
use keyxtend::hidden::Run;
use kx_kernel::KernelError;
use kx_mod_keyboard::view::FILL_OFF_TIP;
use kx_mod_vault::store::Store;
use kx_module_api::ModuleState::{Active, Stopped};
use kx_module_api::ports::{
    EntryKind, FaceAction, FillAsk, FillButton, FillFilter, FillLive, FillOpen, FillPanel,
    FillView, KeyId,
};
use kx_settings::peek;
use kx_ui::I18n;
use kx_ui::asks;
use kx_ui::fill_acts::{BUTTON, FILTER, KIND};
use kx_ui::i18n::keys::{entry_kind_id, fill_button_id, fill_filter_id};
use serde_json::Value;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

mod fill;

/// The board's Quick-fill key, as `board.toml` names it.
const FILL_KEY: &str = "fill";
/// The keys that type the new entry's name on the English layout in front, and what they type.
const NAME_KEYS: [u16; 4] = [0x30, 0x1E, 0x31, 0x25];
const NAME: &str = "bank";
/// The New and Edit page's tag in the panel's JSON.
const EDIT_SHEET: &str = "edit";
/// The app a hidden run was started from.
const APP: &str = "Claude";

/// Clicks key `id` on the board.
fn tap(boot: &Boot, id: &str) {
    let _ = boot.hub.act(FaceAction::Tap {
        id: KeyId::new(id),
        back: false,
    });
}

/// The panel the hub keeps, as JSON; `null` while none is open.
fn kept(boot: &Boot) -> Value {
    serde_json::from_str(&boot.hub.fill_json()).unwrap()
}

/// Waits until the keyboard heard that the vault runs, so a tap on the fill key reaches it; the ear goes on after the
/// start, so the keyboard hears each answer first.
fn ready(boot: &Boot) {
    let bus = boot.kernel.bus();
    let (_live, live) = hear::<FillLive>(&bus);
    bus.publish(&FillAsk).unwrap();
    let mut told = std::iter::from_fn(|| live.recv_timeout(WAIT).ok());
    assert!(told.any(|l| l.live), "the vault runs");
}

/// The fill key's state as the keyboard draws it.
fn fill_key(boot: &Boot) -> Value {
    let state: Value = serde_json::from_str(&boot.hub.state_json()).unwrap();
    state["keys"][FILL_KEY].clone()
}

/// Whether the word lane offers the password box's Quick-fill chip.
fn chip_shows(boot: &Boot) -> bool {
    let lane: Value = serde_json::from_str(&boot.hub.words_json()).unwrap();
    lane["fill"] == true
}

/// The newest view the hub keeps once `ok` takes it, waiting on `views` for each newer one.
fn until(boot: &Boot, views: &Receiver<FillView>, ok: impl Fn(&Value) -> bool) -> Value {
    loop {
        let shown = kept(boot);
        if ok(&shown) {
            return shown;
        }
        views.recv_timeout(WAIT).unwrap();
    }
}

/// Sends the click `kind` names with `id` on the view the hub keeps.
fn click(boot: &Boot, kind: &str, id: &str, arg: i32) {
    let round = kept(boot)["round"].as_i64().unwrap();
    boot.hub.fill_act(kind, id, arg, round);
}

/// The names the list in `shown` holds.
fn names(shown: &Value) -> Vec<String> {
    let rows = shown["sheet"]["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let name = |r: &Value| r["name"]["text"].as_str().map(str::to_owned);
    rows.iter().filter_map(name).collect()
}

/// True when the filter chip `filter` is on in `shown`.
fn filtered(shown: &Value, filter: FillFilter) -> bool {
    let chips = shown["filters"].as_array().cloned().unwrap_or_default();
    let on = |c: &Value| c["id"] == fill_filter_id(filter) && c["on"] == true;
    chips.iter().any(on)
}

/// Opens the panel and shows the logins, then the list once every view the open brought is drawn: the filter click
/// queues behind them, as it goes after the first view.
fn logins(boot: &Boot, views: &Receiver<FillView>) -> Value {
    ready(boot);
    tap(boot, FILL_KEY);
    until(boot, views, |v| !v.is_null());
    click(boot, FILTER, fill_filter_id(FillFilter::Logins), 0);
    until(boot, views, |v| filtered(v, FillFilter::Logins))
}

/// Stops every module, as Quit does, so the vault's actor saved what waits.
fn quit(boot: Boot) {
    let Boot {
        kernel,
        hub,
        requests,
        heard,
        ..
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests, None).unwrap();
    assert!(drain::stop(&hub, thread));
    drop(heard);
}

#[test]
fn quick_fill_starts_on_its_defaults_and_the_fill_key_opens_its_panel() {
    let rig = Rig::new();
    let mut boot = rig.load();
    let (_p, panels) = hear::<FillPanel>(&boot.kernel.bus());
    boot.start().unwrap();
    assert_eq!(boot.kernel.state(kx_mod_vault::ID), Some(Active));
    // The file keeps only what differs from the defaults, so no section means the defaults rule.
    let file = peek(&rig.files()).unwrap_or_default();
    assert!(!file.contains_key(kx_mod_vault::ID.as_str()), "{file:?}");
    ready(&boot);
    assert!(chip_shows(&boot), "the password chip shows");
    tap(&boot, FILL_KEY);
    let mut heard = std::iter::from_fn(|| panels.recv_timeout(WAIT).ok());
    assert!(heard.any(|p| p.open), "the panel opened");
}

#[test]
fn an_entry_saved_in_the_panel_is_there_after_a_restart() {
    let rig = Rig::new();
    let mut boot = rig.load();
    let (_v, views) = hear::<FillView>(&boot.kernel.bus());
    boot.start().unwrap();
    ready(&boot);
    tap(&boot, FILL_KEY);
    until(&boot, &views, |v| !v.is_null());
    click(&boot, BUTTON, fill_button_id(FillButton::New), 0);
    click(&boot, KIND, entry_kind_id(EntryKind::Login), 0);
    until(&boot, &views, |v| {
        v["sheet"]["kind"] == entry_kind_id(EntryKind::Login)
    });
    for sc in NAME_KEYS {
        tap(&boot, &char_key(sc));
    }
    until(&boot, &views, |v| v["sheet"]["name"]["text"] == NAME);
    click(&boot, BUTTON, fill_button_id(FillButton::Save), 0);
    until(&boot, &views, |v| v["sheet"]["is"] != EDIT_SHEET);
    quit(boot);
    let mut again = rig.load();
    let (_v, views) = hear::<FillView>(&again.kernel.bus());
    again.start().unwrap();
    let shown = logins(&again, &views);
    assert_eq!(names(&shown), [NAME], "the store kept it");
}

#[test]
fn a_hidden_run_keeps_quick_fill_off_with_its_note_and_writes_nothing() {
    let rig = Rig::new();
    let files = rig.files();
    let before = tree(&files.dir);
    let run = Run::hidden(APP.into());
    let mut boot = rig.load_as(&run);
    boot.start().unwrap();
    let id = kx_mod_vault::ID;
    assert_eq!(boot.kernel.state(id), Some(Stopped));
    let on = boot.kernel.set_enabled(id, true);
    assert!(
        matches!(on, Err(KernelError::KeptOff(off)) if off == id),
        "{on:?}"
    );
    let en = I18n::new(kx_ui::find("en").unwrap());
    let banner = en.notice(&run.banner().unwrap());
    let shown = boot.hub.take_notices();
    assert!(shown.iter().any(|n| n.text == banner), "{shown:?}");
    let (_opens, opens) = hear::<FillOpen>(&boot.kernel.bus());
    let k = fill_key(&boot);
    let dim = (Value::Bool(true), Value::from(FILL_OFF_TIP));
    assert_eq!(
        (k["dim"].clone(), k["tip"].clone()),
        dim,
        "dimmed, saying why"
    );
    tap(&boot, FILL_KEY);
    assert_eq!(opens.try_iter().count(), 0, "the keyboard asks nobody");
    assert!(!chip_shows(&boot), "no password chip");
    boot.hub.words_ask(asks::FILL, 0);
    assert_eq!(opens.try_iter().count(), 0, "the lane asks nobody");
    assert!(kept(&boot).is_null(), "no panel opens");
    quit(boot);
    let loaded = Store::new(rig.vault.clone()).load().unwrap();
    assert!(
        loaded.book.is_empty() && !loaded.damaged,
        "the notebook was never written"
    );
    assert_eq!(tree(&files.dir), before, "nothing reached the data folder");
}
