//! Snip's ports from the platform: the Snips folder in the data folder, none in a hidden run, and the recorder, the
//! dialogs and the player over it, all usable from another thread; none starts a worker or opens a dialog here.

use super::*;
use kx_module_api::ports::{FileKind, MicSeal, PlayError, RecordError, SnipFile};
use kx_platform::{SNIP_DIR, UNSAVED_DIR};

#[test]
fn the_snips_folder_lies_in_the_data_folder_and_a_hidden_run_has_none() {
    let platform = WindowsPlatform::new().unwrap();
    let snips = platform.snip_folder(false).snips().unwrap();
    assert_eq!(
        snips.expose(),
        platform.dirs().data_dir().path().join(SNIP_DIR)
    );
    assert_eq!(platform.snip_folder(true).snips(), None);
}

#[test]
fn the_snip_ports_answer_on_another_thread_and_refuse_before_any_worker() {
    let platform = WindowsPlatform::new().unwrap();
    let hidden = platform.snip_folder(true);
    let recorder = platform.recorder(MicSeal::new(), Arc::clone(&hidden));
    let videos = platform.media_open(Arc::clone(&hidden));
    let _dialogs = platform.file_ask(hidden);
    let data = platform.dirs().data_dir().path().to_path_buf();
    let answers = std::thread::spawn(move || {
        let video = data.join(SNIP_DIR).join(UNSAVED_DIR).join("1.mp4");
        let video = SnipFile::new(video, FileKind::Mp4).unwrap();
        (recorder.speed(&speed(video.clone())), videos.play(&video))
    });
    let (copied, played) = answers.join().unwrap();
    assert_eq!(
        copied,
        Err(RecordError::NoFolder),
        "a hidden run copies nothing"
    );
    assert_eq!(played, Err(PlayError::BadPath), "nor plays anything");
}

/// A 2x copy of `from` at the least floor, cap and waits.
fn speed(from: SnipFile) -> kx_module_api::ports::SpeedAsk {
    use kx_module_api::ports::{RecordAsk, RecordWaits, Speed, SpeedAsk};
    let waits = RecordWaits {
        start_ms: *RecordWaits::START_MS.start(),
        tick_ms: *RecordWaits::TICK_MS.start(),
        end_ms: *RecordWaits::END_MS.start(),
        switch_ms: *RecordWaits::SWITCH_MS.start(),
        speed_ms: *RecordWaits::SPEED_MS.start(),
        speed_mb_ms: *RecordWaits::SPEED_MB_MS.start(),
    };
    SpeedAsk {
        from,
        speed: Speed::new(SpeedAsk::SPEEDS[1]).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits,
    }
}
