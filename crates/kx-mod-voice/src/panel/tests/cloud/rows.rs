//! The Cloud page's rows, a panel's rows to a page: the tests find a row by its shown name, page by page, and a row
//! carries its card's warning fact id, so the row's mark names that warning, not a fixed one. An untested row shows its
//! mark there and on its card.

use kx_module_api::ports::{CloudButton, ServiceName, ServiceRow, VoiceClick};

use super::super::super::rig::Rig;
use super::{act, card, cloud_sheet, on_cloud};
use crate::services;
use crate::services::tests::ROUTER;
use crate::services::tests::shipped::warn_id;

/// A named row's shown name.
pub(super) fn named(name: &str) -> ServiceName {
    ServiceName::Named(name.into())
}

/// Shipped row `id`'s shown name, as the table gives it.
pub(super) fn shown(id: &str) -> ServiceName {
    services::table().unwrap().get(id).unwrap().shown()
}

/// Turns the Cloud page to the one that lists `name`, from the first, and gives its row there.
pub(super) fn row_of(rig: &mut Rig, name: &ServiceName) -> u8 {
    while rig.view().unwrap().pages.at > 0 {
        let before = rig.view().unwrap().pages;
        rig.click(&VoiceClick::Page { back: true });
        assert_ne!(rig.view().unwrap().pages, before, "no page back");
    }
    loop {
        let rows = cloud_sheet(rig).rows;
        if let Some(at) = rows.iter().position(|r| r.name == *name) {
            return u8::try_from(at).unwrap();
        }
        let before = rig.view().unwrap().pages;
        rig.click(&VoiceClick::Page { back: false });
        assert_ne!(rig.view().unwrap().pages, before, "no row {name:?}");
    }
}

/// Ticks the Cloud page's row `name`, on the page that lists it.
pub(super) fn tick(rig: &mut Rig, name: &ServiceName) {
    let at = row_of(rig, name);
    rig.click(&VoiceClick::Row(at));
}

/// Every page's rows, from the first page to the last.
pub(super) fn every_row(rig: &mut Rig) -> Vec<ServiceRow> {
    row_of(rig, &shown("groq"));
    let mut rows = cloud_sheet(rig).rows;
    loop {
        let before = rig.view().unwrap().pages;
        rig.click(&VoiceClick::Page { back: false });
        if rig.view().unwrap().pages == before {
            return rows;
        }
        rows.extend(cloud_sheet(rig).rows);
    }
}

/// The warning fact id of the shipped row shown as `name`; `None` without one.
fn own_warn(name: &ServiceName) -> Option<String> {
    let rows = services::table().unwrap().rows();
    let row = rows.iter().find(|s| s.shown() == *name)?;
    row.facts.warn.as_ref().map(|f| f.id.clone())
}

#[test]
fn a_row_carries_its_cards_warning_fact_id_and_none_without_one() {
    let mut rig = on_cloud(&format!("{ROUTER}warn = {{ id = \"keeps-logs\" }}\n"));
    let rows = every_row(&mut rig);
    let router = named("Router Test");
    assert!(rows.iter().any(|r| r.name == router), "{rows:?}");
    let eleven = rows.iter().find(|r| r.name == shown("elevenlabs"));
    let trains = eleven.and_then(|r| r.warn.as_deref());
    assert_eq!(trains, Some(warn_id("elevenlabs")), "its card's warning");
    for row in &rows {
        let want = if row.name == router {
            Some("keeps-logs".to_owned())
        } else {
            own_warn(&row.name)
        };
        assert_eq!(row.warn, want, "{:?}", row.name);
    }
}

/// The untested mark (edge 18): `OpenRouter`, untested in the table, shows it on its row and its card; Groq does not.
#[test]
fn an_untested_row_shows_its_mark_on_the_cloud_page_and_its_card() {
    let table = services::table().unwrap();
    assert!(table.get("openrouter").unwrap().untested, "still untested");
    let mut rig = on_cloud("");
    let rows = every_row(&mut rig);
    let marks = ["groq", "openrouter"].map(|id| {
        let row = rows.iter().find(|r| r.name == shown(id)).unwrap();
        (id, row.untested)
    });
    assert_eq!(marks, [("groq", false), ("openrouter", true)]);
    act(&mut rig, &shown("openrouter"), CloudButton::Facts);
    assert!(card(&mut rig).untested, "its card says so too");
}
