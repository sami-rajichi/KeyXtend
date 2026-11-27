//! The Windows parts that need no screen or device: a missing Media Foundation library answers `NoVideo` before any of
//! its calls (edge 14), the libraries load exactly when the system folder holds them, a full drive reads as `Full`, a
//! part's box stays on its monitor, an empty buffer is refused, and a copy reads back what it wrote.

mod copy;

use super::*;
use crate::record::{Area, Shape};
use crate::table::table;
use std::path::PathBuf;
use windows::Win32::Foundation::{E_FAIL, RECT};

/// The variable that names the Windows folder, which holds the system folder.
const SYSTEM_ROOT: &str = "SystemRoot";
const SYSTEM_DIR: &str = "System32";

#[test]
fn a_missing_media_library_answers_no_video() {
    let missing = ["kx-no-such-media-library.dll".to_owned()];
    assert_eq!(media::load(&missing).err(), Some(RecordFault::NoVideo));
}

#[test]
fn the_media_libraries_load_exactly_when_the_system_folder_holds_them() {
    let dlls = &table().unwrap().media_dlls;
    let system = PathBuf::from(std::env::var_os(SYSTEM_ROOT).unwrap()).join(SYSTEM_DIR);
    let present = dlls.iter().all(|d| system.join(d).is_file());
    assert_eq!(media::load(dlls).is_ok(), present, "{present}");
}

#[test]
fn a_full_drive_reads_full_and_a_missing_encoder_no_video() {
    for code in [ERROR_DISK_FULL, ERROR_HANDLE_DISK_FULL] {
        let e = windows::core::Error::from(code.to_hresult());
        assert_eq!(fault(&e), RecordFault::Full);
    }
    let codec = windows::core::Error::from(MF_E_TOPO_CODEC_NOT_FOUND);
    assert_eq!(fault(&codec), RecordFault::NoVideo);
    assert_eq!(fault(&E_FAIL.into()), RecordFault::Failed);
}

#[test]
fn an_empty_buffer_is_refused_before_any_copy() {
    assert!(writer::memory(&[]).is_err());
}

/// A monitor of `w` by `h` pixels whose corner is at `x`, `y` on the desktop.
fn monitor(x: i32, y: i32, (w, h): (i32, i32)) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

#[test]
fn a_part_is_boxed_from_its_monitors_corner_and_refused_past_its_edge() {
    let (full, part) = ((1920, 1080), (640, 360));
    let shape = Shape {
        width: 640,
        height: 360,
        fps: 30,
    };
    let at = |x, y| Area {
        x,
        y,
        w: part.0,
        h: part.1,
    };
    let left = monitor(-full.0, 0, full);
    let on = capture::box_on(&at(-1820, 100), &shape, left).unwrap();
    let edges = (on.left, on.top, on.front, on.right, on.bottom, on.back);
    assert_eq!(edges, (100, 100, 0, 740, 460, 1));
    let main = monitor(0, 0, full);
    let corner = (full.0 - part.0, full.1 - part.1);
    assert!(capture::box_on(&at(corner.0, corner.1), &shape, main).is_ok());
    assert!(capture::box_on(&at(corner.0 + 2, corner.1), &shape, main).is_err());
    assert!(capture::box_on(&at(corner.0, corner.1 + 2), &shape, main).is_err());
    assert!(capture::box_on(&at(-10, 0), &shape, main).is_err());
}
