//! The view reaches QML as JSON: every window's state by id, the frozen screens as provider URLs that change only with
//! their pixels, the editor's picture, the question, a note and the panel, one wake per read.

use super::{editing, heard, idle, json, picking, screens, strip};
use crate::Rig;
use kx_module_api::ports::{
    CaptureSheet, CloseAsk, Dim, Ends, FolderPath, Keep, Keycap, RecordAsk, SnipKeyboard, SnipNote,
    SnipSheet, SnipStage, SnipView, Speed, StripKey, VideoSheet,
};
use kx_ui::SnipWake;
use serde_json::json as j;

#[test]
fn before_any_view_qml_reads_null() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    assert_eq!(*rig.hub.snip_json(), "null");
}

#[test]
fn picking_reaches_qml_with_each_key_by_id_and_the_frozen_screens_by_url() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard
        .bus
        .publish(&SnipView::new(picking(4, screens())))
        .unwrap();
    let v = json(&rig);
    assert_eq!(v["round"], j!(4));
    assert_eq!(v["stage"], j!("pick"));
    let first = &v["frozen"][0];
    assert_eq!(first["rect"], j!({"x": 0, "y": 0, "w": 8, "h": 6}));
    assert_eq!(first["dpi"], j!(96));
    let src = first["src"].as_str().unwrap();
    assert!(
        src.starts_with("image://snip/") && src.ends_with("/0"),
        "{src}"
    );
    assert!(v["frozen"][1]["src"].as_str().unwrap().ends_with("/1"));
    let keys = &v["pick"]["keys"];
    assert_eq!(keys[0], j!({"key": "region", "on": true, "dim": null}));
    assert_eq!(
        keys[4],
        j!({"key": "video-full", "on": false, "dim": "no-video"})
    );
    assert_eq!(v["pick"]["corner"], j!({"x": 2, "y": 3}));
    assert_eq!(v["pick"]["windows"], j!([{"x": 1, "y": 1, "w": 4, "h": 3}]));
    assert_eq!((v["hide"].clone(), v["low"].clone()), (j!([]), j!(false)));
}

#[test]
fn a_run_of_views_wakes_once_until_qml_reads() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard
        .bus
        .publish(&SnipView::new(picking(1, screens())))
        .unwrap();
    heard
        .bus
        .publish(&SnipView::new(picking(2, screens())))
        .unwrap();
    assert_eq!(heard.sent.woken(), [SnipWake::View]);
    assert_eq!(json(&rig)["round"], j!(2), "the newest view");
    heard.bus.publish(&SnipView::new(strip(3))).unwrap();
    assert_eq!(
        heard.sent.woken(),
        [SnipWake::View],
        "the read lets it wake again"
    );
}

#[test]
fn the_frozen_urls_change_only_with_new_pixels() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let frozen = screens();
    heard.shown(&rig, picking(1, frozen.clone()));
    let first = json(&rig)["frozen"][0]["src"].clone();
    heard.shown(&rig, picking(2, frozen));
    assert_eq!(json(&rig)["frozen"][0]["src"], first, "the same pixels");
    heard.shown(&rig, picking(3, screens()));
    assert_ne!(json(&rig)["frozen"][0]["src"], first, "a new freeze");
    heard.shown(&rig, idle(4));
    assert_eq!(json(&rig)["frozen"], j!([]));
}

#[test]
fn the_strip_question_and_note_name_their_ids() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut page = strip(5);
    if let SnipStage::Strip(taken) = &mut page.stage {
        taken.done = Some(Keep::Copy);
        taken.length_ms = Some(2500);
        taken.keys[4].dim = Some(Dim::Bare);
    }
    page.ask = Some(CloseAsk {
        save_dim: Some(Dim::Hidden),
    });
    page.note = Some(SnipNote::ClipBusy);
    heard.shown(&rig, page);
    let v = json(&rig);
    let s = &v["strip"];
    assert_eq!(
        (s["spot"].clone(), s["done"].clone()),
        (j!("under"), j!("copy"))
    );
    assert_eq!(
        (s["length_ms"].clone(), s["keys"][4]["key"].clone()),
        (j!(2500), j!("record"))
    );
    assert_eq!(s["keys"][4]["dim"], j!("bare"));
    assert_eq!(s["speed"], j!(null), "a picture has no speed");
    assert_eq!(s["part"], j!({"x": 1, "y": 1, "w": 4, "h": 3}));
    assert_eq!(v["ask"], j!({"save_dim": "hidden"}));
    assert_eq!(v["note"], j!("clip-busy"));
}

#[test]
fn a_recorded_videos_strip_names_its_keys_its_speed_and_its_length_at_it() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut page = strip(6);
    if let SnipStage::Strip(taken) = &mut page.stage {
        let keys = [StripKey::Play, StripKey::Speed, StripKey::Copy];
        let keys = keys.into_iter().chain([StripKey::Save, StripKey::Close]);
        let cap = |key| Keycap {
            key,
            on: false,
            dim: None,
        };
        taken.keys = keys.map(cap).collect();
        (taken.length_ms, taken.speed) = (Some(1500), Speed::new(2));
    }
    heard.shown(&rig, page);
    let s = &json(&rig)["strip"];
    let ids: Vec<_> = s["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["key"].clone())
        .collect();
    assert_eq!(
        ids,
        [j!("play"), j!("speed"), j!("copy"), j!("save"), j!("close")]
    );
    assert_eq!(
        (s["speed"].clone(), s["length_ms"].clone()),
        (j!(2), j!(1500))
    );
}

#[test]
fn a_picture_editor_shows_its_picture_by_url_and_size() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(6, None));
    let e = &json(&rig)["edit"];
    assert_eq!((e["width"].clone(), e["height"].clone()), (j!(4), j!(3)));
    assert!(e["src"].as_str().unwrap().starts_with("image://snip/"));
    assert_eq!(e["picture"]["crop"], j!({"x": 0, "y": 0, "w": 4, "h": 3}));
}

#[test]
fn the_panel_shows_its_page_its_folder_and_its_steps() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let folder = FolderPath::new(std::env::temp_dir()).unwrap();
    let shown = folder.expose().display().to_string();
    let mut capture = idle(8);
    capture.panel = Some(SnipSheet::Capture(CaptureSheet {
        keyboard: SnipKeyboard::Hidden,
        folder: Some(folder),
    }));
    heard.shown(&rig, capture);
    assert_eq!(
        json(&rig)["panel"],
        j!({"tab": "capture", "keyboard": "hidden", "folder": shown})
    );
    let mut video = idle(9);
    let ends = |low, high| Ends { low, high };
    video.panel = Some(SnipSheet::Video(VideoSheet {
        countdown_s: 0,
        countdown: ends(true, false),
        fps: 30,
        max_min: 120,
        longest: ends(false, true),
        mic_on: true,
        sound_on: false,
    }));
    heard.shown(&rig, video);
    let want = j!({"tab": "video", "countdown_s": 0, "countdown": {"low": true, "high": false}, "fps": 30,
        "rates": RecordAsk::FPS, "max_min": 120, "longest": {"low": false, "high": true}, "mic_on": true,
        "sound_on": false});
    assert_eq!(json(&rig)["panel"], want);
}
