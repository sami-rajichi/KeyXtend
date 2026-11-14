//! Pause, resume, cancel, a stall and the job's end: each answered between reads, each leaving no named file, and a
//! cancel deleting the part. Asks for another download are ignored, and a second download is refused.

use super::*;
use kx_platform_net::wait_ms;

#[test]
fn pause_stops_reading_keeps_the_part_and_resume_asks_from_its_length() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    work(&mut job, &mut out, 3);
    take(&mut job, FetchAsk::Pause { id: 1 }, &mut out);
    assert!(!job.busy(), "a paused download reads nothing");
    assert_eq!(out, [progress(1, 0)], "a pause tells nothing");
    assert_eq!(dir.files(), (None, Some(FILE[..20].to_vec())));
    server.0.push(Act::part(&FILE[20..], 20, 40));
    take(&mut job, FetchAsk::Resume { id: 1 }, &mut out);
    drive(&mut job, &mut out);
    let want = [progress(1, 0), progress(1, 20), progress(1, 40), done(1)];
    assert_eq!(out, want);
    assert_eq!(server.seen(), [seen(URL, 0), seen(URL, 20)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_resume_asks_the_table_again_from_its_first_address() {
    let acts = [Act::Fail(NetError::Net), whole(FILE)];
    let (dir, server) = (Dir::new(), Server::new(acts));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get_of(1, &dir, &[URL, MIRROR], FILE), &mut out);
    work(&mut job, &mut out, 1);
    take(&mut job, FetchAsk::Pause { id: 1 }, &mut out);
    take(&mut job, FetchAsk::Resume { id: 1 }, &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(server.seen(), [seen(URL, 0), seen(URL, 0)]);
}

#[test]
fn cancel_deletes_the_part_and_tells_cancelled_once() {
    for paused in [false, true] {
        let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
        let (mut job, mut out) = (job(&server), Vec::new());
        take(&mut job, get(1, &dir), &mut out);
        work(&mut job, &mut out, 2);
        if paused {
            take(&mut job, FetchAsk::Pause { id: 1 }, &mut out);
        }
        take(&mut job, FetchAsk::Cancel { id: 1 }, &mut out);
        take(&mut job, FetchAsk::Cancel { id: 1 }, &mut out);
        assert!(!job.busy());
        assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Cancelled)]);
        assert_eq!(dir.files(), (None, None), "paused: {paused}");
    }
}

#[test]
fn a_stall_past_stall_s_is_a_fault_that_keeps_the_part() {
    let quiet = Step::Quiet(wait_ms(STALL_S));
    let stalled = Act::Answer(
        Head::whole(FILE.len()),
        vec![Step::Bytes(FILE[..10].to_vec()), quiet],
    );
    let (dir, server) = (Dir::new(), Server::new([stalled]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Stall)]);
    assert_eq!(dir.files(), (None, Some(FILE[..10].to_vec())));
}

#[test]
fn a_quiet_shorter_than_stall_s_is_waited_out() {
    let mut steps = reads(&FILE[..10]);
    steps.push(Step::Quiet(wait_ms(STALL_S) - 1));
    steps.extend(reads(&FILE[10..]));
    let (dir, server) = (
        Dir::new(),
        Server::new([Act::Answer(Head::whole(40), steps)]),
    );
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn reads_stop_at_the_rows_size() {
    let mut steps = reads(FILE);
    steps.push(Step::Bytes(b"more bytes".to_vec()));
    steps.push(Step::Quiet(u32::MAX));
    let no_length = Head {
        status: 200,
        ..Head::default()
    };
    let (dir, server) = (Dir::new(), Server::new([Act::Answer(no_length, steps)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), progress(1, 40), done(1)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_cancel_deletes_the_part_a_fault_left_and_tells_nothing() {
    let stalled = Act::Answer(
        Head::whole(40),
        vec![
            Step::Bytes(FILE[..10].to_vec()),
            Step::Fail(NetError::Stall),
        ],
    );
    let (dir, server) = (Dir::new(), Server::new([stalled]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    take(&mut job, FetchAsk::Cancel { id: 2 }, &mut out);
    assert!(dir.part().exists(), "another id's cancel keeps it");
    take(&mut job, FetchAsk::Cancel { id: 1 }, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Stall)]);
    assert_eq!(dir.files(), (None, None));
}

#[test]
fn asks_for_another_download_are_ignored_and_a_second_download_is_refused() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    work(&mut job, &mut out, 2);
    for ask in [
        FetchAsk::Pause { id: 9 },
        FetchAsk::Resume { id: 1 },
        FetchAsk::Cancel { id: 9 },
        get(1, &dir),
    ] {
        take(&mut job, ask, &mut out);
        assert!(job.busy());
    }
    take(&mut job, get(2, &dir), &mut out);
    assert_eq!(out, [progress(1, 0), fault(2, FetchFault::Internal)]);
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn ending_the_job_mid_download_keeps_the_part_and_names_nothing() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    work(&mut job, &mut out, 2);
    job.end();
    assert!(!job.busy());
    assert_eq!(dir.files(), (None, Some(FILE[..10].to_vec())));
    take(&mut job, get(2, &dir), &mut out);
    job.end();
    assert!(dir.part().exists(), "a part with bytes stays");
    fs::remove_file(dir.part()).unwrap();
    take(&mut job, get(3, &dir), &mut out);
    job.end();
    assert_eq!(dir.files(), (None, None), "an empty part goes");
}
