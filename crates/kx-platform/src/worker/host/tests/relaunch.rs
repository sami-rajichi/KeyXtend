//! A relaunch: the same times and launch keep the worker, new ones say goodbye to it, start the next as they say,
//! and hold a later failure back by their own restart waits.

use super::*;

/// `TIMES` with each wait doubled.
const LONGER: Times = Times {
    deadline_ms: TIMES.deadline_ms * 2,
    idle_s: TIMES.idle_s * 2,
    restart_ms: TIMES.restart_ms * 2,
    restart_max_ms: TIMES.restart_max_ms * 2,
};

#[test]
fn new_times_or_launch_end_the_worker_with_a_goodbye_and_start_the_next_as_they_say() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    assert_eq!(ping(&mut host), Ok(1));
    host.relaunch(TIMES, launch());
    assert_eq!(ping(&mut host), Ok(2));
    assert_eq!(script.spawns(), 1, "the same keep the worker");
    let bigger = Launch {
        memory_mb: launch().memory_mb * 2,
        ..launch()
    };
    host.relaunch(LONGER, bigger.clone());
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
    assert_eq!(ping(&mut host), Ok(3));
    assert_eq!(script.launches(), [launch(), bigger]);
    let within = ms(LONGER.deadline_ms);
    assert!(script.calls().ends_with(&[Call::Reply { within }]));
}

#[test]
fn a_failure_after_a_relaunch_rests_its_new_first_wait() {
    let (script, clock) = (Script::default(), Hands::default());
    let mut host = host(&script, &clock);
    host.relaunch(LONGER, launch());
    script.act(Act::Silent);
    assert_eq!(ping(&mut host), Err(Miss::Lost(Lost::Late)));
    assert_eq!(ping(&mut host), down(ms(LONGER.restart_ms)));
}
