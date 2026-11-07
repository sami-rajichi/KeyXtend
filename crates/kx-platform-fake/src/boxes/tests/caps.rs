//! The worker's caps as the fake holds them: a box, a `KeePass` file or parts past their cap are too large, a work
//! factor outside the bounds too, and a password past the frame's cap fails every call that carries one.

use super::*;
use kx_module_api::ports::{MAX_BOX_BYTES, MAX_KEEPASS_BYTES, MAX_PASSWORD_BYTES};

#[test]
fn a_box_or_keepass_file_past_the_cap_is_too_large() {
    let (boxes, mut run) = session();
    boxes.set_keepass(KDBX, PASSWORDS[0], entries());
    let big = vec![0; MAX_BOX_BYTES + 1];
    assert_eq!(run.open(&big, &password(0)), Err(BoxError::TooLarge));
    let file = vec![0; MAX_KEEPASS_BYTES + 1];
    assert_eq!(run.keepass(&file, &password(0)), Err(BoxError::TooLarge));
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let full = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(vec![0; MAX_BOX_BYTES])),
    };
    assert_eq!(run.seal(&padlock, &[full]), Err(BoxError::TooLarge));
}

#[test]
fn a_password_past_the_frames_cap_fails_every_call_that_carries_one() {
    let (boxes, mut run) = session();
    let padlock = run.padlock(&password(0), WORK).unwrap();
    let sealed = run.seal(&padlock, &parts()).unwrap().into_inner();
    let long = secret(&"x".repeat(MAX_PASSWORD_BYTES + 1));
    boxes.set_keepass(KDBX, long.expose(), entries());
    let failed = Err(BoxError::Failed);
    assert_eq!(run.padlock(&long, WORK).map(drop), failed);
    assert_eq!(run.open(&sealed, &long).map(drop), failed);
    assert_eq!(run.check(&padlock, &long).map(drop), failed);
    assert_eq!(run.keepass(KDBX, &long).map(drop), failed);
    let longest = secret(&"x".repeat(MAX_PASSWORD_BYTES));
    assert!(run.padlock(&longest, WORK).is_ok(), "the cap itself goes");
}

#[test]
fn a_work_factor_outside_the_bounds_is_too_large_and_the_latest_is_kept() {
    let (boxes, mut run) = session();
    for work in [MIN_BOX_WORK, MAX_BOX_WORK] {
        assert!(run.padlock(&password(0), work).is_ok(), "{work}");
    }
    for work in [MIN_BOX_WORK - 1, MAX_BOX_WORK + 1] {
        let made = run.padlock(&password(0), work);
        assert_eq!(made, Err(BoxError::TooLarge), "{work}");
    }
    assert_eq!(boxes.work(), Some(MAX_BOX_WORK + 1));
}
