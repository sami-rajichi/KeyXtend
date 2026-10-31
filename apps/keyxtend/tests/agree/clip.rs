//! The clipboard's settings fit the worker's frames: the largest entry the settings allow fits a draw, the largest
//! thumbnail and preview fit a reply's side, and a pixel has as many bytes on the pipe as in the port.

use kx_mod_clipboard::settings::{MAX_ENTRY_MB, PREVIEW_PX, THUMB_PX};
use kx_module_api::ports::{MB, RGBA_BYTES};

#[test]
fn the_largest_entry_the_settings_allow_fits_one_draw() {
    let top = u64::from(*MAX_ENTRY_MB.end()) * MB;
    let cap = u64::try_from(kx_ipc::MAX_IMAGE_BYTES).unwrap();
    assert!(top <= cap, "{top} bytes over the frame's {cap}");
}

#[test]
fn the_largest_thumbnail_and_preview_fit_a_replys_side() {
    let tops = [*THUMB_PX.end(), *PREVIEW_PX.end()];
    assert!(tops.iter().all(|px| *px <= kx_ipc::MAX_SIDE), "{tops:?}");
}

#[test]
fn a_pixel_has_as_many_bytes_on_the_pipe_as_in_the_port() {
    assert_eq!(kx_ipc::RGBA_BYTES, RGBA_BYTES);
}
