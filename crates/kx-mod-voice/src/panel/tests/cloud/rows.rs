//! The Cloud page's rows, a panel's rows to a page: the tests find a row by its shown name, page by page, and a row
//! carries its card's warning fact id, so the row's mark names that warning, not a fixed one.

use kx_module_api::ports::{ServiceName, ServiceRow, VoiceClick};

use super::super::super::rig::Rig;
use super::{cloud_sheet, on_cloud};
use crate::services::tests::ROUTER;

/// A named row's shown name.
pub(super) fn named(name: &str) -> ServiceName {
    ServiceName::Named(name.into())
}

/// Turns the Cloud page to the one that lists `name`, from the first, and gives its row there.
pub(super) fn row_of(rig: &mut Rig, name: &ServiceName) -> u8 {
    while rig.view().unwrap().pages.at > 0 {
        rig.click(&VoiceClick::Page { back: true });
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
    row_of(rig, &named("Groq"));
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

#[test]
fn a_row_carries_its_cards_warning_fact_id_and_none_without_one() {
    let mut rig = on_cloud(&format!("{ROUTER}warn = {{ id = \"keeps-logs\" }}\n"));
    let rows = every_row(&mut rig);
    let router = named("Router Test");
    assert!(rows.iter().any(|r| r.name == router), "{rows:?}");
    for row in &rows {
        let want = (row.name == router).then(|| "keeps-logs".to_owned());
        assert_eq!(row.warn, want, "{:?}", row.name);
    }
}
