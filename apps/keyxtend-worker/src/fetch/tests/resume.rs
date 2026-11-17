//! Whole and resumed downloads: a cut download keeps its part and the next ask goes on from its length, a server that
//! sends the whole file restarts it, a range from elsewhere is refused, and a complete part is checked without asking.

use super::*;

#[test]
fn a_whole_download_is_checked_then_named_and_leaves_no_part() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), progress(1, 40), done(1)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
    assert_eq!(server.seen(), [seen(URL, 0)]);
}

#[test]
fn a_cut_download_keeps_its_part_and_the_next_ask_resumes_from_its_length() {
    let mut cut = reads(&FILE[..20]);
    cut.push(Step::Fail(NetError::Net));
    let cut = Act::Answer(Head::whole(FILE.len()), cut);
    let (dir, server) = (Dir::new(), Server::new([cut]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Net)]);
    assert_eq!(dir.files(), (None, Some(FILE[..20].to_vec())));
    server.0.push(Act::part(&FILE[20..], 20, 40));
    out.clear();
    take(&mut job, get(2, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(2, 20), progress(2, 40), done(2)]);
    assert_eq!(server.seen(), [seen(URL, 0), seen(URL, 20)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_new_job_resumes_the_part_an_earlier_one_left() {
    let dir = Dir::new();
    fs::write(dir.part(), &FILE[..30]).unwrap();
    let server = Server::new([Act::part(&FILE[30..], 30, 40)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(7, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(7, 30), progress(7, 40), done(7)]);
    assert_eq!(server.seen(), [seen(URL, 30)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_server_that_sends_the_whole_file_to_a_range_restarts_the_part() {
    let dir = Dir::new();
    fs::write(dir.part(), b"stale part").unwrap();
    let server = Server::new([whole(FILE)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    let want = [progress(1, 10), progress(1, 0), progress(1, 40), done(1)];
    assert_eq!(out, want, "the part's bytes, then the emptied part");
    assert_eq!(server.seen(), [seen(URL, 10)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_partial_answer_that_starts_elsewhere_keeps_the_part_as_it_was() {
    let dir = Dir::new();
    fs::write(dir.part(), &FILE[..20]).unwrap();
    let server = Server::new([Act::part(&FILE[10..], 10, 40)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 20), fault(1, FetchFault::Server)]);
    assert_eq!(server.seen(), [seen(URL, 20)]);
    assert_eq!(dir.files(), (None, Some(FILE[..20].to_vec())));
}

#[test]
fn a_complete_part_is_checked_and_named_without_asking_the_server() {
    let dir = Dir::new();
    fs::write(dir.part(), FILE).unwrap();
    let server = Server::new([]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 40), done(1)]);
    assert!(
        server.seen().is_empty(),
        "a range from its end would be 416"
    );
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_part_longer_than_the_file_starts_over() {
    let dir = Dir::new();
    fs::write(dir.part(), [FILE, b"tail"].concat()).unwrap();
    let server = Server::new([whole(FILE)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(server.seen(), [seen(URL, 0)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_pause_while_reading_the_part_back_goes_on_at_resume() {
    let dir = Dir::new();
    fs::write(dir.part(), &FILE[..20]).unwrap();
    let server = Server::new([Act::part(&FILE[20..], 20, 40)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    take(&mut job, FetchAsk::Pause { id: 1 }, &mut out);
    assert!(!job.busy());
    take(&mut job, FetchAsk::Resume { id: 1 }, &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 20), progress(1, 40), done(1)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}
