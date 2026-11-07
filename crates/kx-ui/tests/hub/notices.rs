//! Notices from the bus, Try again, OK, Yes and No, and the settings QML reads.

use crate::{KEYBOARD, Rig, failed};
use kx_module_api::{Bus, Notice, NoticeAnswered, NoticeAsk, Subscription, keys};
use kx_test_support::host::FakeCore;
use kx_ui::{Hub, Request, UiSettings, Wake};
use std::sync::{Arc, Mutex};

/// A notice that asks Yes or No.
fn asking() -> Notice {
    Notice {
        ask: Some(&NoticeAsk::YES_NO),
        ..Notice::plain(keys::SETTINGS_LOCKED)
    }
}

/// A bus the hub sends on, the answers it carried, oldest first, and the subscriptions that keep both.
fn answers(rig: &Rig) -> (Arc<Mutex<Vec<NoticeAnswered>>>, Vec<Subscription>) {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keep = Arc::clone(&heard);
    let mut subs = vec![
        bus.subscribe(move |a: &NoticeAnswered| keep.lock().unwrap().push(a.clone()))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    (heard, subs)
}

#[test]
fn notices_on_the_bus_reach_the_banner_until_the_subscription_drops() {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let heard = rig.hub.listen(&bus).unwrap();
    bus.publish(&failed()).unwrap();
    assert_eq!(rig.wakes(), [Wake::Notices]);
    drop(heard);
    bus.publish(&Notice::plain(keys::SETTINGS_LOCKED)).unwrap();
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
fn an_answer_closes_the_notice_once_and_reaches_its_raiser_and_the_app() {
    let rig = Rig::new();
    let (heard, _subs) = answers(&rig);
    rig.hub.notice(asking());
    let shown = rig.hub.take_notices();
    let ask = shown[0].ask.as_ref().unwrap();
    assert_eq!((ask.yes.as_str(), ask.no.as_str()), ("Yes", "No"));
    assert!(rig.hub.answer(shown[0].id, true));
    let yes = NoticeAnswered {
        notice: asking(),
        yes: true,
    };
    assert_eq!(*heard.lock().unwrap(), [yes]);
    let request = Request::Answered {
        notice: asking(),
        yes: true,
    };
    assert_eq!(rig.requests.try_recv(), Ok(request));
    assert!(!rig.hub.answer(shown[0].id, false), "closed already");
    assert_eq!(heard.lock().unwrap().len(), 1);
    assert!(rig.requests.try_recv().is_err());
    assert!(rig.hub.shown_notices().is_empty());
}

#[test]
fn ok_or_try_again_on_an_asking_notice_answers_no() {
    let rig = Rig::new();
    let (heard, _subs) = answers(&rig);
    let ok = |hub: &Hub, id| hub.dismiss(id);
    let again = |hub: &Hub, id| assert!(!hub.retry(id), "nothing to restart");
    for close in [&ok as &dyn Fn(&Hub, u64), &again] {
        rig.hub.notice(asking());
        close(&rig.hub, rig.hub.take_notices()[0].id);
        let no = NoticeAnswered {
            notice: asking(),
            yes: false,
        };
        assert_eq!(std::mem::take(&mut *heard.lock().unwrap()), [no]);
        let request = Request::Answered {
            notice: asking(),
            yes: false,
        };
        assert_eq!(rig.requests.try_recv(), Ok(request));
        assert!(rig.hub.shown_notices().is_empty(), "closed");
    }
}

#[test]
fn answering_an_unknown_or_plain_notice_sends_nothing() {
    let rig = Rig::new();
    let (heard, _subs) = answers(&rig);
    assert!(!rig.hub.answer(u64::MAX, true), "never shown");
    rig.hub.notice(failed());
    let shown = rig.hub.take_notices();
    assert_eq!(shown[0].ask, None, "a plain notice asks nothing");
    assert!(!rig.hub.answer(shown[0].id, true));
    assert!(heard.lock().unwrap().is_empty());
    assert!(rig.requests.try_recv().is_err(), "nor Try again");
    assert!(rig.hub.shown_notices().is_empty(), "it closes as OK does");
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
fn qml_reads_the_timings_and_paging_and_the_language_apart() {
    let rig = Rig::new();
    let v: serde_json::Value = serde_json::from_str(&rig.hub.settings_json()).unwrap();
    let s = UiSettings::from_defaults().unwrap();
    assert_eq!(v["tipMs"], s.tip_ms);
    assert_eq!(v["pollMs"], s.poll_ms);
    assert_eq!(v["guardDelayMs"], s.guard_delay_ms);
    assert_eq!(v["trayGraceMs"], s.tray_grace_ms);
    assert_eq!(v["pageRows"], s.page_rows);
    assert_eq!(
        v["pressFrame"], "now",
        "the press frame draws first (ADR-0025)"
    );
    assert_eq!(v["pressFrameMs"], s.press_frame_ms);
    assert_eq!(v["frameLog"], false, "the frame log is off unless set");
    assert!(v.get("lang").is_none() && v.get("sizes").is_none(), "{v}");
}
