//! The media helpers: each picture at its side, failures passed on, file facts in order, and both threads ending
//! when the actor lets go, the worker session with them.

use std::sync::mpsc;
use std::time::Duration;

use kx_module_api::ports::{FileFact, PictureConfig, PictureError, Pictures};
use kx_platform_fake::{FakeClipboard, FakePictures};

use super::*;
use crate::fixture::{files, text};

/// The longest a threaded test waits; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);
/// The plan's sides.
const SIDES: Sides = Sides {
    thumb: 128,
    preview: 960,
};
/// The plan's worker.
const PICS: PictureConfig = PictureConfig {
    deadline_ms: 3000,
    idle_s: 30,
    memory_mb: 512,
    max_pixels: 100_000_000,
    restart_ms: 1000,
    restart_max_ms: 30_000,
};

/// A 2 by 1 picture.
fn pixels() -> Pixels {
    Pixels::new(2, 1, vec![0; 8], SIDES.thumb).unwrap()
}

/// An image part of `bytes`.
fn image(bytes: &[u8]) -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(bytes.to_vec()),
    }
}

#[test]
fn a_job_takes_the_image_part_or_none() {
    let job = job_of(7, Size::Thumb, &[text("x"), image(&[1, 2])]).unwrap();
    assert_eq!(
        (job.id, job.size, job.codec),
        (7, Size::Thumb, ImageCodec::Dib)
    );
    assert_eq!(job.bytes.expose(), &[1, 2]);
    assert!(job_of(7, Size::Preview, &[text("only words")]).is_none());
}

#[test]
fn a_thumb_draws_at_its_side_and_a_preview_at_its_own() {
    let pics = FakePictures::new();
    pics.queue(Ok(pixels()));
    let mut run = pics.open(PICS);
    let drawn = draw(
        &mut *run,
        job_of(3, Size::Thumb, &[image(&[9])]).unwrap(),
        SIDES,
    );
    assert_eq!(
        (drawn.id, drawn.size, drawn.pixels),
        (3, Size::Thumb, Ok(pixels()))
    );
    assert_eq!(pics.last_ask().map(|a| a.side), Some(SIDES.thumb));
    draw(
        &mut *run,
        job_of(3, Size::Preview, &[image(&[9])]).unwrap(),
        SIDES,
    );
    assert_eq!(pics.last_ask().map(|a| a.side), Some(SIDES.preview));
}

#[test]
fn a_failed_draw_is_told_as_it_is() {
    let pics = FakePictures::new();
    pics.queue(Err(PictureError::Timeout));
    let mut run = pics.open(PICS);
    let drawn = draw(
        &mut *run,
        job_of(1, Size::Thumb, &[image(&[1])]).unwrap(),
        SIDES,
    );
    assert_eq!(drawn.pixels, Err(PictureError::Timeout));
}

#[test]
fn a_look_tells_each_entrys_facts_in_order() {
    let clip = FakeClipboard::new();
    clip.set_file("C:\\a", FileFact::Size(5));
    clip.set_file("E:\\b", FileFact::Away);
    let ClipPart::Files(paths) = files(&["C:\\a", "C:\\gone", "E:\\b"]) else {
        unreachable!()
    };
    let facts = look(
        &clip,
        &Look {
            why: Why::Insert(4),
            items: vec![(9, paths)],
        },
    );
    assert_eq!(facts.why, Why::Insert(4));
    assert_eq!(
        facts.items,
        [(9, vec![FileFact::Size(5), FileFact::Gone, FileFact::Away])]
    );
}

#[test]
fn the_media_thread_draws_each_job_and_ends_with_its_inbox_and_the_session() {
    let pics = FakePictures::new();
    pics.queue(Ok(pixels()));
    let (jobs, inbox) = mpsc::channel();
    let (tell, heard) = mpsc::channel();
    let thread = spawn_media(pics.open(PICS), SIDES, inbox, move |d| {
        let _ = tell.send(d);
    })
    .unwrap();
    jobs.send(job_of(5, Size::Thumb, &[image(&[1])]).unwrap())
        .unwrap();
    let drawn = heard.recv_timeout(WAIT).unwrap();
    assert_eq!((drawn.id, drawn.pixels), (5, Ok(pixels())));
    drop(jobs);
    thread.join().unwrap();
    assert!(!pics.running(), "the worker session went with the thread");
}

#[test]
fn the_files_thread_looks_and_ends_with_its_inbox() {
    let clip = FakeClipboard::new();
    let (looks, inbox) = mpsc::channel();
    let (tell, heard) = mpsc::channel();
    let thread = spawn_files(Arc::new(clip.clone()), inbox, move |f| {
        let _ = tell.send(f);
    })
    .unwrap();
    let ClipPart::Files(paths) = files(&["C:\\x"]) else {
        unreachable!()
    };
    looks
        .send(Look {
            why: Why::Rows,
            items: vec![(2, paths)],
        })
        .unwrap();
    let facts = heard.recv_timeout(WAIT).unwrap();
    assert_eq!(
        (facts.why, facts.items),
        (Why::Rows, vec![(2, vec![FileFact::Gone])])
    );
    assert_eq!(clip.checks(), 1);
    drop(looks);
    thread.join().unwrap();
}

#[test]
fn no_debug_shows_an_image_or_a_path() {
    let job = job_of(1, Size::Thumb, &[image(b"SECRETBYTES")]).unwrap();
    let ClipPart::Files(paths) = files(&["C:\\secret.txt"]) else {
        unreachable!()
    };
    let look = Look {
        why: Why::Rows,
        items: vec![(1, paths)],
    };
    for shown in [format!("{job:?}"), format!("{look:?}")] {
        assert!(
            !shown.contains("secret") && !shown.contains("83, 69, 67"),
            "{shown}"
        );
    }
}

#[test]
fn the_files_thread_serves_inserts_first_and_only_the_newest_row_look() {
    let ClipPart::Files(paths) = files(&["C:\\x"]) else {
        unreachable!()
    };
    let (looks, inbox) = mpsc::channel();
    let ask = |why, id| Look {
        why,
        items: vec![(id, paths.clone())],
    };
    let asked = [
        ask(Why::Rows, 1),
        ask(Why::Insert(1), 2),
        ask(Why::Rows, 3),
        ask(Why::Insert(2), 4),
    ];
    for look in asked {
        looks.send(look).unwrap();
    }
    drop(looks);
    let (tell, heard) = mpsc::channel();
    let clip = Arc::new(FakeClipboard::new());
    let thread = spawn_files(clip, inbox, move |f| drop(tell.send(f))).unwrap();
    thread.join().unwrap();
    let order: Vec<(Why, u64)> = heard.try_iter().map(|f| (f.why, f.items[0].0)).collect();
    let want = [(Why::Insert(1), 2), (Why::Insert(2), 4), (Why::Rows, 3)];
    assert_eq!(order, want, "the older row look is dropped");
}

#[test]
fn one_thumbnail_draws_at_a_time_and_only_the_newest_preview_waits() {
    let job = |id| job_of(id, Size::Preview, &[image(b"x")]).unwrap();
    let mut draws = Draws::default();
    draws.went(Size::Thumb);
    assert!(!draws.free(Size::Thumb) && draws.free(Size::Preview));
    draws.went(Size::Preview);
    draws.hold(job(1));
    draws.hold(job(2));
    assert!(draws.done(Size::Thumb).is_none());
    assert!(draws.free(Size::Thumb));
    let next = draws.done(Size::Preview).map(|j| j.id);
    assert_eq!(next, Some(2), "the older preview was dropped");
    assert!(
        !draws.free(Size::Preview),
        "the one handed back goes at once"
    );
    assert!(draws.done(Size::Preview).is_none());
    assert!(draws.free(Size::Preview));
}
