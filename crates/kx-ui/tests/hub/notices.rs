//! Notices from the bus, Try again and OK, settings and texts.

use crate::{KEYBOARD, Rig, failed};
use kx_module_api::{Bus, Notice, keys};
use kx_test_support::host::FakeCore;
use kx_ui::i18n::keys::{SIZE, SIZE_NAME, SIZES};
use kx_ui::{Request, UiSettings, Wake};
use std::sync::Arc;

#[test]
fn notices_on_the_bus_reach_the_banner_until_the_subscription_drops() {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let heard = rig.hub.listen(&bus).unwrap();
    bus.publish(&failed()).unwrap();
    assert_eq!(rig.wakes(), [Wake::Notices]);
    drop(heard);
    bus.publish(&Notice {
        key: keys::SETTINGS_LOCKED,
        module: None,
        args: Vec::new(),
    })
    .unwrap();
    assert!(rig.wakes().is_empty());
    assert_eq!(rig.hub.take_notices().len(), 1);
}

#[test]
fn try_again_asks_the_app_to_restart_the_failed_module() {
    let rig = Rig::new();
    rig.hub.notice(failed());
    let shown = rig.hub.take_notices();
    assert!(shown[0].retry);
    assert!(rig.hub.retry(shown[0].id));
    assert_eq!(rig.requests.try_recv(), Ok(Request::Retry(KEYBOARD)));
    assert!(!rig.hub.retry(shown[0].id), "closed already");
}

#[test]
fn ok_closes_a_notice_without_a_request() {
    let rig = Rig::new();
    rig.hub.notice(failed());
    let id = rig.hub.take_notices()[0].id;
    rig.hub.dismiss(id);
    assert!(!rig.hub.retry(id));
    assert!(rig.requests.try_recv().is_err());
}

#[test]
fn attach_and_detach_swap_the_face_and_keep_the_settings() {
    let (rig, _face, _host) = Rig::with_face();
    rig.hub.detach();
    assert_eq!(rig.wakes(), [Wake::Settings]);
    assert_eq!(rig.hub.board_json(), "null");
    assert_eq!(rig.hub.settings(), UiSettings::from_defaults().unwrap());
}

#[test]
fn the_texts_follow_the_language_setting_else_windows() {
    let rig = Rig::new();
    assert_eq!(rig.hub.i18n().lang().tag, "en");
    rig.look.set_language("ar-TN");
    let mut host = kx_test_support::host::FakeHost::new(&kx_ui::MANIFEST);
    let face = Arc::new(crate::FakeFace::default());
    rig.attach(Arc::clone(&face), &mut host);
    assert_eq!(rig.hub.i18n().lang().tag, "ar", "auto follows Windows");
    let mut fr = UiSettings::from_defaults().unwrap();
    fr.language = "fr".into();
    rig.hub.attach(fr, face, host.handle());
    assert_eq!(rig.hub.i18n().lang().tag, "fr", "a set language wins");
}

#[test]
fn qml_reads_the_size_buttons_in_the_ui_language() {
    let rig = Rig::new();
    rig.look.set_language("ar");
    let mut host = kx_test_support::host::FakeHost::new(&kx_ui::MANIFEST);
    rig.attach(Arc::new(crate::FakeFace::default()), &mut host);
    let v: serde_json::Value = serde_json::from_str(&rig.hub.settings_json()).unwrap();
    let ar = rig.hub.i18n();
    let sizes = v["sizes"].as_array().expect("a list of sizes");
    assert_eq!(sizes.len(), SIZES.len());
    for (s, id) in sizes.iter().zip(SIZES) {
        assert_eq!(s["label"].as_str(), Some(ar.name_of(SIZE, id).as_str()));
        assert_eq!(s["name"].as_str(), Some(ar.name_of(SIZE_NAME, id).as_str()));
        assert_ne!(s["label"].as_str(), Some(id), "{id} has an Arabic letter");
    }
}

#[test]
fn qml_reads_the_timings_paging_and_direction() {
    let rig = Rig::new();
    let v: serde_json::Value = serde_json::from_str(&rig.hub.settings_json()).unwrap();
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!(v["tipMs"], s.tip_ms);
    assert_eq!(v["pollMs"], s.poll_ms);
    assert_eq!(v["guardDelayMs"], s.guard_delay_ms);
    assert_eq!(v["pageRows"], s.page_rows);
    assert_eq!(
        (v["lang"].as_str(), v["rtl"].as_bool()),
        (Some("en"), Some(false))
    );
}
