//! A Cancel against an end on its way to the owner: an end the owner heard keeps the video, and one it never heard
//! loses it to the Cancel, whichever reaches the route first.

use super::super::route::Route;
use super::*;
use crate::worker::Told;

/// The video a test run records.
fn video() -> SnipFile {
    SnipFile::new(unsaved_at("1.mp4"), FileKind::Mp4).unwrap()
}

/// A route recording `file` into the rig's folder, its start heard.
fn live(rig: &Rig, file: &SnipFile) -> Route {
    let (answer, _) = mpsc::channel();
    let folder: Arc<dyn SnipFolder> = Arc::new(rig.folder.clone());
    let clock = Arc::new(rig.clock.clone());
    let switch = ms(ask().waits.switch_ms);
    let mut route = Route::new(file.clone(), folder, clock, answer, switch);
    assert!(route.heard(Told::Note(started(1))).is_some());
    route
}

/// `route` takes the job's end on Stop, which keeps the video.
fn end(route: &mut Route) {
    let done = route.heard(Told::Note(stopped(1)));
    assert!(matches!(done, Some(RecordNote::Done(_))), "{done:?}");
}

#[test]
fn a_cancel_after_an_end_its_owner_heard_keeps_the_video() {
    let rig = Rig::new();
    let mut route = live(&rig, &video());
    end(&mut route);
    assert!(!route.cancel(), "no job left to tell");
    route.leave();
    assert!(rig.folder.removes().is_empty());
}

#[test]
fn an_end_its_owner_never_heard_loses_its_video_to_a_cancel_either_way_round() {
    let rig = Rig::new();
    let file = video();
    let mut refused_first = live(&rig, &file);
    end(&mut refused_first);
    refused_first.unheard();
    assert!(rig.folder.removes().is_empty(), "kept until a Cancel");
    assert!(!refused_first.cancel());
    assert_eq!(rig.folder.removes(), std::slice::from_ref(&file));
    let mut cancelled_first = live(&rig, &file);
    end(&mut cancelled_first);
    assert!(!cancelled_first.cancel());
    cancelled_first.unheard();
    assert_eq!(rig.folder.removes(), [file.clone(), file]);
}

#[test]
fn a_dropped_runs_unheard_end_keeps_its_video() {
    let rig = Rig::new();
    let mut route = live(&rig, &video());
    end(&mut route);
    route.unheard();
    route.leave();
    assert!(rig.folder.removes().is_empty());
}
