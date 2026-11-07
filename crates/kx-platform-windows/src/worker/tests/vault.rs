//! The real vault job, started as the host starts it: a padlock seals a box that opens with its password alone, a
//! check tells a wrong password, foreign bytes are damaged, and its word and exit codes match the host's.

use super::*;
use kx_ipc::VAULT_JOB;
use kx_module_api::ports::{BoxConfig, BoxError, BoxPart, Boxes, MIN_BOX_WORK, NOTEBOOK_PART};
use kx_module_api::{Secret, Zeroizing};
use kx_platform::boxes::WorkerBoxes;

/// Test values; the real ones come from the vault module's settings. A dev build derives keys slower than a release.
const BOX_CFG: BoxConfig = BoxConfig {
    deadline_ms: 60_000,
    idle_s: 60,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

#[test]
fn the_real_vault_job_seals_and_opens_a_box_through_the_host() {
    let boxes = WorkerBoxes::new(spawner(), Arc::new(Still));
    let mut run = boxes.start(BOX_CFG);
    let password = secret("كلمة السر");
    let padlock = run.padlock(&password, MIN_BOX_WORK).unwrap();
    let note = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new("سري الخاص".as_bytes().to_vec())),
    };
    let sealed = run.seal(&padlock, std::slice::from_ref(&note)).unwrap();
    let opened = run.open(sealed.expose(), &password).unwrap();
    assert_eq!(
        (opened.padlock, opened.parts),
        (padlock.clone(), vec![note])
    );
    assert_eq!(run.check(&padlock, &password), Ok(true));
    assert_eq!(run.check(&padlock, &secret("wrong")), Ok(false));
    let wrong = run.open(sealed.expose(), &secret("wrong"));
    assert_eq!(wrong.map(drop), Err(BoxError::WrongPassword));
    let foreign = run.open(b"not a box", &password).map(drop);
    assert_eq!(foreign, Err(BoxError::Damaged));
    let file = run.keepass(b"not a kdbx file", &password).map(drop);
    assert_eq!(file, Err(BoxError::Damaged));
}

#[test]
fn the_vault_jobs_word_and_exit_codes_match_the_host() {
    let exit = |c: u8| Some(u32::from(c));
    let (input, _output, mut child) = spawner().start(&Launch::vault()).unwrap();
    drop(input);
    assert_eq!(child.code(WAIT), exit(CLEAN_EXIT), "the end of input");
    let (mut input, _output, mut child) = spawner().start(&Launch::vault()).unwrap();
    input.write_all(b"not a frame at all").unwrap();
    assert_eq!(child.code(WAIT), exit(REFUSED_EXIT), "a broken frame");
    let (_input, _output, mut child) = start(&[VAULT_JOB, MAX_PIXELS]);
    assert_eq!(child.code(WAIT), exit(REFUSED_EXIT), "alone on its line");
}
