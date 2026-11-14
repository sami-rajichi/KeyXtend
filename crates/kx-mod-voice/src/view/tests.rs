//! The panel's view (plan task 19): pages hold `page_rows` rows, 5 by default, and turn within the list; the home
//! list names the model, the device, the last Compare and the counts; Try again acts as the flow allows.

use kx_module_api::ports::{
    DeviceLabel, DeviceName, Pages, PanelButton, VoiceClick, VoiceHome, VoiceSheet, VoiceTab,
};

use crate::counts::Board;
use crate::panel::rig::{ROWS, Rig, row};

/// The rig on `tab`.
fn on(tab: VoiceTab) -> Rig {
    let mut rig = Rig::foldered();
    rig.click(&VoiceClick::Open(tab));
    rig
}

#[test]
fn a_page_holds_five_rows_and_turns_within_the_list() {
    let mut rig = on(VoiceTab::Models);
    let view = rig.view().unwrap();
    assert_eq!(view.pages, Pages { at: 0, count: 2 }, "ten English models");
    let VoiceSheet::Models(sheet) = view.sheet else {
        panic!("not Models");
    };
    assert_eq!(sheet.rows.len(), usize::from(ROWS));
    for _ in 0..3 {
        rig.click(&VoiceClick::Page { back: false });
    }
    assert_eq!(
        rig.view().unwrap().pages,
        Pages { at: 1, count: 2 },
        "the last page holds"
    );
    rig.click(&VoiceClick::Page { back: true });
    assert_eq!(rig.view().unwrap().pages.at, 0);
}

#[test]
fn the_history_pages_by_five() {
    let mut rig = Rig::new();
    rig.texts(12);
    rig.click(&VoiceClick::Open(VoiceTab::History));
    assert_eq!(rig.view().unwrap().pages.count, 3);
    rig.click(&VoiceClick::Page { back: false });
    rig.click(&VoiceClick::Page { back: false });
    let VoiceSheet::History(sheet) = rig.view().unwrap().sheet else {
        panic!("not History");
    };
    assert_eq!(sheet.rows.len(), 2);
}

#[test]
fn a_row_on_another_page_ticks_what_that_page_shows() {
    let mut rig = on(VoiceTab::Models);
    rig.click(&VoiceClick::Page { back: false });
    rig.click(&VoiceClick::Row(0));
    let VoiceSheet::Models(sheet) = rig.view().unwrap().sheet else {
        panic!("not Models");
    };
    assert!(sheet.rows[0].tick);
}

#[test]
fn zero_rows_from_the_ui_count_as_one() {
    let mut rig = on(VoiceTab::Models);
    rig.click(&VoiceClick::Rows(0));
    assert_eq!(rig.view().unwrap().pages.count, 10);
}

#[test]
fn the_home_list_names_the_model_the_device_compare_and_the_counts() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = Rig::foldered();
    rig.holds(&[parakeet]);
    rig.texts(3);
    rig.settings.sounds = false;
    let day = rig.with(|_, f| f.now.date);
    assert!(rig.scores.put(Board {
        lang: "en".into(),
        day,
        auto: true,
        card: None,
        scores: Vec::new()
    }));
    let VoiceSheet::Home(home) = rig.view().unwrap().sheet else {
        panic!("not Home");
    };
    let want = VoiceHome {
        model: Some(kx_module_api::Redacted::new(parakeet.name.clone())),
        device: DeviceLabel::Auto(DeviceName::Cpu),
        compared: Some(day),
        history: 3,
        changed: 1,
    };
    assert_eq!(home, want);
}

#[test]
fn the_home_list_with_no_model_names_none() {
    let mut rig = Rig::new();
    let VoiceSheet::Home(home) = rig.view().unwrap().sheet else {
        panic!("not Home");
    };
    assert_eq!((home.model, home.compared, home.changed), (None, None, 0));
}

#[test]
fn try_again_acts_on_the_home_list_as_the_flow_allows() {
    let mut rig = Rig::new();
    assert_eq!(rig.view().unwrap().acts, [PanelButton::Close]);
    rig.again = true;
    assert_eq!(
        rig.view().unwrap().acts,
        [PanelButton::Close, PanelButton::Again]
    );
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    assert_eq!(
        rig.view().unwrap().acts,
        [PanelButton::Back, PanelButton::Close]
    );
}

#[test]
fn the_panel_draws_nothing_while_closed() {
    let mut rig = Rig::new();
    rig.click(&VoiceClick::Button(PanelButton::Close));
    assert!(rig.view().is_none());
}
