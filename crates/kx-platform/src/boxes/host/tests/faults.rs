//! The worker's faults become each call's own errors, `Failed` where a call has no such error, and keep the worker; an
//! ask the frame would refuse is the port's error before any worker starts.

use super::*;
use BoxError::{Damaged, Failed, TooLarge, Unsupported, WrongPassword};
use kx_ipc::{
    Fault, MAX_BOX_BYTES, MAX_BOX_WORK, MAX_KEEPASS_BYTES, MAX_LOCKED_BYTES, MAX_PASSWORD_BYTES,
};

#[test]
fn each_fault_is_the_calls_own_error_and_keeps_the_worker() {
    // A fault's error in padlock, seal, open, check and keepass, in that order.
    let table = [
        (
            Fault::WrongPassword,
            [Failed, Failed, WrongPassword, Failed, WrongPassword],
        ),
        (Fault::Damaged, [Failed, Damaged, Damaged, Damaged, Damaged]),
        (Fault::Foreign, [Failed, Damaged, Damaged, Damaged, Damaged]),
        (
            Fault::TooLarge,
            [TooLarge, TooLarge, TooLarge, Failed, TooLarge],
        ),
        (
            Fault::Unsupported,
            [Failed, Failed, Unsupported, Failed, Unsupported],
        ),
        (Fault::Internal, [Failed; 5]),
    ];
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for (fault, errors) in table {
        for (kind, error) in KINDS.into_iter().zip(errors) {
            script.act(VaultAct::Fail(fault));
            assert_eq!(call(&mut host, kind), Err(error), "{fault:?} in {kind:?}");
        }
    }
    assert_eq!(script.spawns(), 1);
    assert!(!script.calls().contains(&Call::End), "a fault is an answer");
}

#[test]
fn a_seal_needs_each_known_part_once_and_no_more_than_the_box_holds() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let twice = vec![part(NOTEBOOK_PART), part(NOTEBOOK_PART)];
    for parts in [Vec::new(), vec![part("settings")], twice] {
        assert_eq!(host.seal(padlock(), parts), Err(Failed), "a module's slip");
    }
    let bytes = Redacted::new(Zeroizing::new(vec![0; MAX_BOX_BYTES + 1]));
    let huge = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes,
    };
    assert_eq!(host.seal(padlock(), vec![huge]), Err(TooLarge));
    assert_eq!(script.tries(), 0);
}

#[test]
fn a_padlock_the_frame_will_not_carry_is_damaged() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let empty = Padlock {
        recipient: Redacted::new(String::new()),
        ..padlock()
    };
    let long = Padlock {
        locked: Redacted::new(vec![1; MAX_LOCKED_BYTES + 1]),
        ..padlock()
    };
    let parts = || vec![part(NOTEBOOK_PART)];
    assert_eq!(host.seal(empty.clone(), parts()), Err(Damaged));
    assert_eq!(host.seal(long.clone(), parts()), Err(Damaged));
    let unlocked = Padlock {
        locked: Redacted::new(Vec::new()),
        ..padlock()
    };
    assert_eq!(host.check(unlocked, secret(PASSWORD)), Err(Damaged));
    assert_eq!(host.check(long, secret(PASSWORD)), Err(Damaged));
    assert_eq!(script.tries(), 0);
}

#[test]
fn files_the_frame_will_not_carry_and_work_past_its_range_start_nothing() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    for work in [MIN_BOX_WORK - 1, MAX_BOX_WORK + 1] {
        assert_eq!(
            host.padlock(secret(PASSWORD), work),
            Err(TooLarge),
            "{work}"
        );
    }
    let open = |host: &mut Vault, bytes| host.open(bytes, secret(PASSWORD)).map(drop);
    assert_eq!(open(&mut host, Vec::new()), Err(Damaged), "cut short");
    assert_eq!(open(&mut host, vec![0; MAX_BOX_BYTES + 1]), Err(TooLarge));
    let read = |host: &mut Vault, bytes| host.keepass(bytes, secret(PASSWORD)).map(drop);
    assert_eq!(read(&mut host, Vec::new()), Err(Damaged));
    assert_eq!(
        read(&mut host, vec![0; MAX_KEEPASS_BYTES + 1]),
        Err(TooLarge)
    );
    assert_eq!(script.tries(), 0);
}

#[test]
fn a_password_past_the_frames_cap_fails_every_call_before_any_start() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    let long = || secret(&"x".repeat(MAX_PASSWORD_BYTES + 1));
    assert_eq!(host.padlock(long(), WORK), Err(Failed));
    assert_eq!(host.open(BOX.to_vec(), long()).map(drop), Err(Failed));
    assert_eq!(host.check(padlock(), long()), Err(Failed));
    assert_eq!(host.keepass(KDBX.to_vec(), long()).map(drop), Err(Failed));
    assert_eq!(script.tries(), 0);
    let longest = secret(&"x".repeat(MAX_PASSWORD_BYTES));
    script.act(VaultAct::Answer(checked));
    assert_eq!(
        host.check(padlock(), longest),
        Ok(false),
        "the cap itself goes"
    );
}
