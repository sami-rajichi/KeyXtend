//! Windows' disk dialogs stay off on the thread while a call touches a place, and the thread gets its own mode back.

use super::*;
use crate::places::quiet::{Quiet, mode, set_mode, touched};
use kx_module_api::ports::BOX_EXT;
use tree::boxed;
use windows::Win32::System::Diagnostics::Debug::{SEM_FAILCRITICALERRORS, SEM_NOOPENFILEERRORBOX};

/// Whether `mode` keeps both disk dialogs off.
fn off(mode: u32) -> bool {
    let both = SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX;
    mode & both.0 == both.0
}

#[test]
fn the_guard_turns_the_dialogs_off_and_gives_the_thread_its_mode_back() {
    let start = mode();
    set_mode(0);
    {
        let _outer = Quiet::new();
        assert!(off(mode()), "on inside");
        {
            let _inner = Quiet::new();
            assert!(off(mode()), "on inside a second guard");
        }
        assert!(
            off(mode()),
            "the inner guard gives back the outer one's mode"
        );
    }
    assert_eq!(mode(), 0, "back as it was");
    set_mode(start);
}

#[test]
fn each_call_that_reaches_a_drive_runs_with_the_dialogs_off() {
    let start = mode();
    set_mode(0);
    let mut tree = Tree::new("places-quiet");
    let stick = tree.add_drive(PlaceKind::Removable, "usb", "");
    let places = tree.places();
    let calls: [(&str, &dyn Fn()); 5] = [
        ("suggested", &|| _ = places.suggested()),
        ("folders", &|| _ = places.folders(None)),
        ("find", &|| _ = places.find()),
        ("state", &|| _ = places.state(&stick, 1)),
        ("write", &|| _ = places.write(&stick, b"box")),
    ];
    for (name, call) in calls {
        call();
        assert!(tree.seen.take().is_some_and(off), "{name} runs quiet");
        assert_eq!(mode(), 0, "{name} gives the mode back");
    }
    set_mode(start);
}

#[test]
fn each_call_that_lists_or_reads_a_place_touches_it_with_the_dialogs_off() {
    let start = mode();
    set_mode(0);
    let mut tree = Tree::new("places-touch");
    let stick = tree.add_drive(PlaceKind::Removable, "usb", "");
    let file = stick.join(box_file());
    fs::write(&file, boxed(b"box")).unwrap();
    let places = tree.places();
    let calls: [(&str, &dyn Fn()); 5] = [
        ("files", &|| _ = places.files(&stick, BOX_EXT)),
        ("read", &|| _ = places.read(&file, 1)),
        ("modified", &|| _ = places.modified(&file)),
        ("folders", &|| _ = places.folders(Some(&stick))),
        // The find's own touch comes after its suggestions, whose guard has gone by then.
        ("find", &|| _ = places.find()),
    ];
    for (name, call) in calls {
        call();
        assert!(touched().is_some_and(off), "{name} touches it quiet");
        assert_eq!(mode(), 0, "{name} gives the mode back");
    }
    set_mode(start);
}
