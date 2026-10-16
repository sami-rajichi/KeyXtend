//! Tests of `g6track`: the face sits so the step before it lies in the document's body, and the track keeps that step.

use super::*;

/// A 1080p screen's work area, without its taskbar.
const WORK: RECT = RECT {
    left: 0,
    top: 0,
    right: 1920,
    bottom: 1032,
};

fn shipped() -> (G6, i32) {
    let cfg = crate::config::load().expect("kx-gates.toml loads");
    (cfg.g6, cfg.probes.move_steps)
}

/// Where the glide's last point outside the face lands when the pointer goes exactly where it is sent.
fn planned(doc: &RECT) -> Option<Pt> {
    let (g, steps) = shipped();
    let f = face(doc, &WORK, &g);
    let mut track = Track::new(f);
    for p in glide(uia::centre(doc), &f, steps) {
        track.see(p);
    }
    track.last()
}

#[test]
fn the_step_before_the_face_lies_in_the_documents_body() {
    let (g, _) = shipped();
    let docs = [
        RECT {
            left: 300,
            top: 200,
            right: 1200,
            bottom: 800,
        },
        RECT {
            left: 0,
            top: 0,
            right: 900,
            bottom: 500,
        },
        RECT {
            left: 1100,
            top: 500,
            right: 1900,
            bottom: 1100,
        },
    ];
    for doc in docs {
        let p = planned(&doc).expect("the glide leaves the face");
        assert!(simin::inside(&doc, p), "{doc:?}: {p:?} is off the window");
        assert!(
            p.y >= uia::centre(&doc).y,
            "{doc:?}: {p:?} is above the centre"
        );
        assert!(
            doc.bottom - p.y > g.face_overlap_px,
            "{doc:?}: {p:?} is on the lower edge"
        );
    }
}

#[test]
fn the_face_is_centred_under_the_document_and_stays_in_the_work_area() {
    let (g, _) = shipped();
    let doc = RECT {
        left: -200,
        top: 600,
        right: 400,
        bottom: 1200,
    };
    let f = face(&doc, &WORK, &g);
    assert_eq!((f.right - f.left, f.bottom - f.top), (g.face_px, g.face_px));
    assert!(f.left >= WORK.left && f.bottom <= WORK.bottom, "{f:?}");
    let mid = RECT {
        left: 300,
        top: 200,
        right: 1200,
        bottom: 800,
    };
    let f = face(&mid, &WORK, &g);
    assert_eq!(uia::centre(&f).x, uia::centre(&mid).x);
    assert_eq!(f.top, mid.bottom - g.face_overlap_px);
}

/// A face square at 0,0, 10 px wide.
const FACE: RECT = RECT {
    left: 0,
    top: 0,
    right: 10,
    bottom: 10,
};

#[test]
fn the_track_keeps_the_last_point_outside_the_face_and_ignores_points_inside() {
    let mut track = Track::new(FACE);
    let pts = [
        Pt { x: 50, y: 40 },
        Pt { x: 10, y: 3 },
        Pt { x: 9, y: 9 },
        Pt { x: 0, y: 0 },
    ];
    for p in pts {
        track.see(p);
    }
    assert_eq!(
        track.last(),
        Some(Pt { x: 10, y: 3 }),
        "the right edge is outside"
    );
}

#[test]
fn the_track_gives_none_when_the_pointer_never_left_the_face() {
    let mut track = Track::new(FACE);
    track.see(Pt { x: 5, y: 5 });
    assert_eq!(track.last(), None);
}

#[test]
fn the_scroll_point_is_reported_from_the_windows_top_left_never_as_a_screen_point() {
    let doc = RECT {
        left: 300,
        top: 200,
        right: 1200,
        bottom: 800,
    };
    assert_eq!(offset(&doc, Pt { x: 750, y: 630 }), [450, 430]);
}
