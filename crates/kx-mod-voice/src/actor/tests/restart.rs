//! A choice the panel makes survives a restart (ADR-0054): it goes through the settings handle, is saved to a real
//! settings file and read back by the store, and a fresh actor on the same PC, read and repaired as the module starts,
//! shows it. Options and the device are in `restart/rest.rs`.

mod rest;

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CloudButton, CloudSheet, LayoutChanged, LayoutId, ModelSide, ModelsSheet, PanelButton,
    ServiceMark, ServiceName, ServiceSheet, VoiceClick, VoiceSheet, VoiceTab,
};
use kx_settings::{Files, Store};
use kx_test_support::tempdir::TempDir;

use super::Msg;
use super::cloud::{key, set_key};
use super::rig::Rig;
use crate::panel::rig::ROWS;
use crate::{ID, SPEC};

/// The keyboard languages a choice is made and shown in, with their direction.
const LANGS: [(&str, bool); 3] = [("en", false), ("fr", false), ("ar", true)];
/// An address the Custom row takes.
const ADDRESS: &str = "https://stt.example.com/v1/audio/transcriptions";
/// An Other model id typed on Custom's line.
const MODEL: &str = "my-model";

/// The section `rig`'s own writes left, saved to a settings file and read back by the store with no notice.
fn saved_file(rig: &Rig) -> toml::Table {
    let dir = TempDir::new("voice-restart").unwrap();
    let (mut store, _) = Store::load(Files::new(dir.path()), &[(ID, &SPEC)]);
    store.set_all(ID, rig.host.sink.section()).unwrap();
    store.save().unwrap();
    let (store, report) = Store::load(Files::new(dir.path()), &[(ID, &SPEC)]);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    store.get(ID).unwrap().clone()
}

/// A fresh actor on the same PC, started from `rig`'s saved file.
fn restart(rig: &Rig) -> Rig {
    Rig::restarted(rig.fake.clone(), saved_file(rig))
}

/// The panel's page as last drawn.
fn sheet(rig: &Rig) -> VoiceSheet {
    let panel = rig.views().last().and_then(|v| v.panel.clone());
    panel.map(|p| p.sheet).expect("the panel is open")
}

/// Opens the panel anew on `tab`, with language `lang` of `LANGS` in front.
fn open(rig: &mut Rig, lang: usize, tab: VoiceTab) {
    let (iso, rtl) = LANGS[lang];
    let id = LayoutId::new(u64::try_from(lang).unwrap() + 1);
    let (iso, name) = (iso.into(), iso.into());
    rig.send(Msg::Layout(LayoutChanged { id, iso, name, rtl }));
    rig.send(Msg::Close);
    rig.send(Msg::Open);
    rig.act(VoiceClick::Rows(ROWS));
    rig.act(VoiceClick::Open(tab));
}

/// The Models page as last drawn.
fn models(rig: &Rig) -> ModelsSheet {
    match sheet(rig) {
        VoiceSheet::Models(page) => page,
        other => panic!("not the Models page: {other:?}"),
    }
}

/// The name of the model the Models page shows in use.
fn used(rig: &Rig) -> Option<String> {
    let rows = models(rig).rows;
    let row = rows.iter().find(|r| r.used);
    row.map(|r| r.name.expose().clone())
}

/// The Cloud page as last drawn.
fn cloud(rig: &Rig) -> CloudSheet {
    match sheet(rig) {
        VoiceSheet::Cloud(page) => page,
        other => panic!("not the Cloud page: {other:?}"),
    }
}

/// The key page as last drawn.
fn lines(rig: &Rig) -> ServiceSheet {
    match sheet(rig) {
        VoiceSheet::Service(page) => page,
        other => panic!("not a key page: {other:?}"),
    }
}

/// Ticks service `name` on the Cloud page, paging on until it shows; gives its mark.
fn tick(rig: &mut Rig, name: &ServiceName) -> ServiceMark {
    for _ in 0..u8::MAX {
        let page = cloud(rig);
        if let Some(at) = page.rows.iter().position(|r| r.name == *name) {
            rig.act(VoiceClick::Row(u8::try_from(at).unwrap()));
            return page.rows[at].mark;
        }
        rig.act(VoiceClick::Page { back: false });
    }
    panic!("{name:?} is not listed");
}

/// Confirms service `name`'s card from the Cloud page, which opens its key page.
fn confirm(rig: &mut Rig, name: &ServiceName) {
    open(rig, 0, VoiceTab::Cloud);
    tick(rig, name);
    rig.act(VoiceClick::Service(CloudButton::On));
    rig.act(VoiceClick::Confirm(true));
}

/// Cloud on, Groq confirmed with its key, and its first model used in each language; the names used.
fn groq_everywhere(rig: &mut Rig) -> Vec<String> {
    set_key(&rig.fake, &key());
    open(rig, 0, VoiceTab::Cloud);
    rig.act(VoiceClick::Cloud(true));
    confirm(rig, &named("Groq"));
    let pick = |rig: &mut Rig, lang| {
        open(rig, lang, VoiceTab::Models);
        rig.act(VoiceClick::Side(ModelSide::Cloud));
        rig.act(VoiceClick::Row(0));
        rig.act(VoiceClick::Button(PanelButton::Use));
        used(rig).expect("a cloud model in use")
    };
    (0..LANGS.len()).map(|lang| pick(rig, lang)).collect()
}

/// The cloud model each language uses, as its Models page's Cloud side shows.
fn cloud_used(rig: &mut Rig) -> Vec<Option<String>> {
    let show = |rig: &mut Rig, lang| {
        open(rig, lang, VoiceTab::Models);
        rig.act(VoiceClick::Side(ModelSide::Cloud));
        used(rig)
    };
    (0..LANGS.len()).map(|lang| show(rig, lang)).collect()
}

/// A service by its own name.
fn named(name: &str) -> ServiceName {
    ServiceName::Named(name.into())
}

#[test]
fn a_local_pick_in_each_language_survives_a_restart() {
    let mut rig = Rig::new();
    let mut picked = Vec::new();
    for lang in 0..LANGS.len() {
        open(&mut rig, lang, VoiceTab::Models);
        let at = models(&rig).rows.iter().position(|r| !r.used).unwrap();
        rig.act(VoiceClick::Row(u8::try_from(at).unwrap()));
        rig.act(VoiceClick::Button(PanelButton::Use));
        picked.push(used(&rig));
    }
    let mut fresh = restart(&rig);
    for (lang, name) in picked.iter().enumerate() {
        open(&mut fresh, lang, VoiceTab::Models);
        assert_eq!(&used(&fresh), name, "{}", LANGS[lang].0);
    }
}

#[test]
fn a_cloud_pick_in_each_language_and_its_service_on_survive_a_restart() {
    let mut rig = Rig::new();
    let picked = groq_everywhere(&mut rig);
    let mut fresh = restart(&rig);
    assert_eq!(cloud_used(&mut fresh), in_use(picked));
    open(&mut fresh, 0, VoiceTab::Cloud);
    assert!(cloud(&fresh).on, "cloud stays on");
    assert_eq!(tick(&mut fresh, &named("Groq")), ServiceMark::Ready);
}

#[test]
fn cloud_off_keeps_each_cloud_pick_through_a_restart_and_cloud_on_brings_it_back() {
    let mut rig = Rig::new();
    let picked = groq_everywhere(&mut rig);
    open(&mut rig, 0, VoiceTab::Cloud);
    rig.act(VoiceClick::Cloud(false));
    let mut fresh = restart(&rig);
    open(&mut fresh, 0, VoiceTab::Cloud);
    assert!(!cloud(&fresh).on, "cloud stays off");
    fresh.act(VoiceClick::Cloud(true));
    assert_eq!(cloud_used(&mut fresh), in_use(picked), "ADR-0054");
}

#[test]
fn a_service_off_the_keep_switch_and_the_typed_lines_survive_a_restart() {
    let mut rig = Rig::new();
    open(&mut rig, 0, VoiceTab::Cloud);
    rig.act(VoiceClick::Cloud(true));
    confirm(&mut rig, &ServiceName::Custom);
    rig.act(VoiceClick::SetAddress(Redacted::new(ADDRESS.into())));
    rig.act(VoiceClick::SetModel(Redacted::new(MODEL.into())));
    confirm(&mut rig, &named("OpenRouter"));
    rig.act(VoiceClick::KeepNothing(false));
    confirm(&mut rig, &named("Lemonfox"));
    open(&mut rig, 0, VoiceTab::Cloud);
    tick(&mut rig, &named("Lemonfox"));
    rig.act(VoiceClick::Service(CloudButton::Off));
    let mut fresh = restart(&rig);
    open(&mut fresh, 0, VoiceTab::Cloud);
    assert_eq!(tick(&mut fresh, &named("Lemonfox")), ServiceMark::Off);
    tick(&mut fresh, &named("OpenRouter"));
    fresh.act(VoiceClick::Service(CloudButton::Lines));
    assert_eq!(lines(&fresh).keep, Some(false));
    open(&mut fresh, 0, VoiceTab::Cloud);
    tick(&mut fresh, &ServiceName::Custom);
    fresh.act(VoiceClick::Service(CloudButton::Lines));
    let page = lines(&fresh);
    let address = page.address.map(|a| a.expose().clone());
    assert_eq!(address.as_deref(), Some(ADDRESS));
    assert_eq!(page.model.expose(), MODEL);
}

/// Each of `names` as a language's model in use.
fn in_use(names: Vec<String>) -> Vec<Option<String>> {
    names.into_iter().map(Some).collect()
}
