//! The Cloud page and a service's card (ADR-0047): nothing offered while cloud is off (edge 21), Groq first (edge 24),
//! the untested mark (edge 18), the facts' day turning old (edge 15), newer terms asked again (edge 16), the Home's
//! count, the Models page's Cloud side with Use (edge 29), and Compare's cloud ticks with the send warning (edge 12).

use kx_module_api::ports::{
    CloudBand, CloudButton, CloudCard, CloudRow, CloudSheet, Date, ModelSide, PanelButton,
    ServiceMark, ServiceName, ServiceRow, ServiceSheet, Unavailable, VoiceClick, VoiceHome,
    VoiceSheet, VoiceTab,
};

use super::super::rig::Rig;
use super::super::{Do, Page};
use super::{models, on_models};
use crate::services::tests::{ROUTER, with_rows};
use crate::settings::{PICKS, SERVICES};
use rows::{named, row_of, tick};

mod answers;
mod lines;
mod offered;
mod parts;
mod props;
mod remove;
mod rows;
mod settle;
mod trial;
mod typing;
mod used;

/// Applies the `[voice]` writes in `dos`, as the settings would after the actor wrote them.
pub(super) fn apply(rig: &mut Rig, dos: &[Do]) {
    for done in dos {
        if let Do::Set(key, value) = done {
            rig.settings = rig.settings.with(key, value.clone()).unwrap();
        }
    }
}

/// Writes `text` as `[voice]` table `key`.
pub(super) fn put(rig: &mut Rig, key: &str, text: &str) {
    let value = toml::Value::Table(text.parse().unwrap());
    rig.settings = rig.settings.with(key, value).unwrap();
}

/// Cloud on, with the shipped rows and `rows` after them, on the Cloud page.
pub(super) fn on_cloud(rows: &str) -> Rig {
    let mut rig = Rig::new();
    rig.services = Some(with_rows(rows));
    rig.settings.cloud = true;
    rig.click(&VoiceClick::Open(VoiceTab::Cloud));
    rig
}

/// The Cloud page as shown.
pub(super) fn cloud_sheet(rig: &mut Rig) -> CloudSheet {
    match rig.view().unwrap().sheet {
        VoiceSheet::Cloud(sheet) => sheet,
        other => panic!("not Cloud: {other:?}"),
    }
}

/// The card as shown.
pub(super) fn card(rig: &mut Rig) -> CloudCard {
    match rig.view().unwrap().sheet {
        VoiceSheet::Card(card) => *card,
        other => panic!("not a card: {other:?}"),
    }
}

/// The key page as shown.
pub(super) fn lines_sheet(rig: &mut Rig) -> ServiceSheet {
    match rig.view().unwrap().sheet {
        VoiceSheet::Service(sheet) => sheet,
        other => panic!("not a key page: {other:?}"),
    }
}

/// The Home's values, after Back from the shown page.
fn home(rig: &mut Rig) -> VoiceHome {
    while rig.panel.page != Some(Page::Home) {
        rig.click(&VoiceClick::Button(PanelButton::Back));
    }
    match rig.view().unwrap().sheet {
        VoiceSheet::Home(home) => home,
        other => panic!("not Home: {other:?}"),
    }
}

/// Ticks the Cloud page's row `name`, then clicks the service's `button`.
pub(super) fn act(rig: &mut Rig, name: &ServiceName, button: CloudButton) -> Vec<Do> {
    tick(rig, name);
    rig.click(&VoiceClick::Service(button))
}

#[test]
fn cloud_off_offers_no_service_on_the_page_on_models_or_at_home() {
    let mut rig = Rig::new();
    rig.click(&VoiceClick::Open(VoiceTab::Cloud));
    let sheet = cloud_sheet(&mut rig);
    assert!(!sheet.on && sheet.acts.is_empty(), "{sheet:?}");
    let dimmed = |r: &ServiceRow| r.mark == ServiceMark::Off && !r.tick;
    assert!(
        sheet.rows.iter().all(dimmed),
        "only listed, dimmed: {sheet:?}"
    );
    assert_eq!(home(&mut rig).cloud, None);
    rig.click(&VoiceClick::Open(VoiceTab::Models));
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    let view = rig.view().unwrap();
    let sheet = models(&view);
    assert_eq!(
        (sheet.side, sheet.band, sheet.rows.len()),
        (ModelSide::Cloud, Some(CloudBand::Off), 0)
    );
}

#[test]
fn groq_is_listed_first_off_and_untested_and_custom_names_no_free_allowance() {
    let mut rig = on_cloud("");
    let sheet = cloud_sheet(&mut rig);
    assert!(sheet.on);
    let groq = &sheet.rows[0];
    assert_eq!(groq.name, named("Groq"), "edge 24; `offered` pins the rest");
    assert_eq!(
        (groq.mark, groq.untested, groq.free.is_some()),
        (ServiceMark::Off, true, true)
    );
    assert!(sheet.acts.is_empty(), "nothing acts before a tick");
    let custom = usize::from(row_of(&mut rig, &ServiceName::Custom));
    assert_eq!(cloud_sheet(&mut rig).rows[custom].free, None);
}

#[test]
fn a_service_goes_on_through_its_card_to_its_key_page_and_the_home_counts_it_once_ready() {
    let mut rig = on_cloud("");
    rig.click(&VoiceClick::Row(0));
    assert_eq!(
        cloud_sheet(&mut rig).acts,
        [CloudButton::Facts, CloudButton::On]
    );
    rig.click(&VoiceClick::Service(CloudButton::On));
    let card = card(&mut rig);
    assert_eq!((card.on, card.untested, card.old), (None, true, false));
    let out = rig.click(&VoiceClick::Confirm(true));
    assert!(matches!(&out[..], [Do::Set(SERVICES, _), Do::Dots(id)] if id == "groq"));
    apply(&mut rig, &out);
    assert_eq!(lines_sheet(&mut rig).mark, ServiceMark::NeedsKey);
    assert_eq!(home(&mut rig).cloud, Some(0));
    rig.keyed.insert("groq".into());
    assert_eq!(home(&mut rig).cloud, Some(1));
}

#[test]
fn a_card_shows_the_day_its_facts_were_read_and_turns_old_past_facts_days() {
    let read = ROUTER.replace("read = 2026-11-17", "read = 2026-01-01");
    let mut rig = on_cloud(&read);
    act(&mut rig, &named("Router Test"), CloudButton::Facts);
    let card = card(&mut rig);
    assert_eq!(
        (card.read, card.old),
        (Date::new(2026, 1, 1).unwrap(), true)
    );
    rig.settings.facts_days = 400;
    let card = rig.view().unwrap().sheet;
    assert!(matches!(card, VoiceSheet::Card(c) if !c.old));
}

#[test]
fn newer_terms_ask_again_and_a_no_switches_the_service_off_with_its_picks_but_keeps_its_key() {
    let newer = ROUTER.replace("terms = 1", "terms = 2");
    let mut rig = on_cloud(&newer);
    let on = r#""router-test" = { on = 2026-11-01, terms = 1 }"#;
    put(&mut rig, "services", on);
    put(
        &mut rig,
        "picks",
        r#"en = "router-test:openai/whisper-large-v3-turbo""#,
    );
    rig.keyed.insert("router-test".into());
    let router = named("Router Test");
    tick(&mut rig, &router);
    let acts = [
        CloudButton::Facts,
        CloudButton::Lines,
        CloudButton::Off,
        CloudButton::On,
    ];
    assert_eq!(cloud_sheet(&mut rig).acts, acts);
    rig.click(&VoiceClick::Service(CloudButton::On));
    assert_eq!(card(&mut rig).on, None, "it asks again");
    let out = rig.click(&VoiceClick::Confirm(false));
    assert!(
        !out.iter().any(|d| matches!(d, Do::RemoveKey(_))),
        "only Remove key deletes it"
    );
    apply(&mut rig, &out);
    assert!(rig.settings.services.is_empty() && !rig.settings.picks.contains_key("en"));
    let at = usize::from(row_of(&mut rig, &router));
    assert_eq!(cloud_sheet(&mut rig).rows[at].mark, ServiceMark::Off);
    let out = act(&mut rig, &router, CloudButton::On);
    assert!(out.is_empty());
    let out = rig.click(&VoiceClick::Confirm(true));
    apply(&mut rig, &out);
    assert_eq!(
        lines_sheet(&mut rig).mark,
        ServiceMark::Ready,
        "the key is still there"
    );
}

#[test]
fn the_models_page_has_a_cloud_side_that_pages_alone_and_leads_to_the_cloud_page() {
    let mut rig = on_models(&[]);
    rig.settings.cloud = true;
    rig.click(&VoiceClick::Page { back: false });
    let local = rig.view().unwrap().pages;
    assert!(
        local.count > 1 && local.at == 1,
        "the Local side has a second page"
    );
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    let view = rig.view().unwrap();
    let sheet = models(&view);
    assert_eq!(
        (sheet.band, sheet.folder.is_none(), view.pages.at),
        (Some(CloudBand::NoneOn), true, 0)
    );
    assert_eq!(view.acts, [PanelButton::Back, PanelButton::Close]);
    rig.click(&VoiceClick::Side(ModelSide::Local));
    assert_eq!(rig.view().unwrap().pages, local);
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    rig.click(&VoiceClick::Open(VoiceTab::Cloud));
    assert!(matches!(rig.view().unwrap().sheet, VoiceSheet::Cloud(_)));
}

#[test]
fn use_on_a_ready_cloud_model_picks_it_and_a_kept_model_is_unavailable_with_its_reason() {
    let mut rig = on_models(&[]);
    rig.services = Some(with_rows(ROUTER));
    rig.settings.cloud = true;
    put(
        &mut rig,
        "services",
        r#"groq = { on = 2026-11-01, terms = 1 }
"router-test" = { on = 2026-11-01, terms = 1, keep = true }"#,
    );
    rig.keyed.extend(["groq".into(), "router-test".into()]);
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    let view = rig.view().unwrap();
    let sheet = models(&view);
    assert_eq!(sheet.band, Some(CloudBand::Listed));
    let groq = CloudRow {
        service: ServiceName::Named("Groq".into()),
        mark: ServiceMark::Ready,
        unavailable: None,
    };
    assert_eq!(sheet.rows[0].cloud, Some(groq));
    let kept = sheet
        .rows
        .iter()
        .position(|r| r.name.expose() == "Kept")
        .unwrap();
    let reason = sheet.rows[kept].cloud.as_ref().and_then(|c| c.unavailable);
    assert_eq!(reason, Some(Unavailable::KeepNothing));
    rig.click(&VoiceClick::Row(u8::try_from(kept).unwrap()));
    assert!(!rig.view().unwrap().acts.contains(&PanelButton::Use));
    rig.click(&VoiceClick::Row(0));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    assert!(matches!(&out[..], [Do::Set(PICKS, _)]));
    apply(&mut rig, &out);
    assert_eq!(
        rig.settings.picks.get("en").map(String::as_str),
        Some("groq:whisper-large-v3-turbo")
    );
    let view = rig.view().unwrap();
    assert!(models(&view).rows[0].used);
}

#[test]
fn switching_a_service_off_keeps_its_key_so_on_again_finds_it_ready() {
    let mut rig = on_cloud("");
    put(
        &mut rig,
        "services",
        "groq = { on = 2026-11-01, terms = 1 }",
    );
    put(&mut rig, "picks", r#"en = "groq:whisper-large-v3-turbo""#);
    rig.keyed.insert("groq".into());
    let out = act(&mut rig, &named("Groq"), CloudButton::Off);
    assert!(
        !out.iter().any(|d| matches!(d, Do::RemoveKey(_))),
        "only Remove key deletes it"
    );
    apply(&mut rig, &out);
    assert!(rig.settings.services.is_empty() && !rig.settings.picks.contains_key("en"));
    rig.click(&VoiceClick::Service(CloudButton::On)); // Groq stays ticked.
    let out = rig.click(&VoiceClick::Confirm(true));
    apply(&mut rig, &out);
    assert_eq!(lines_sheet(&mut rig).mark, ServiceMark::Ready);
}
