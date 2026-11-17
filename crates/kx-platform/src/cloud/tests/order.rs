//! A run takes a text only at the next untold index below its piece count and within its cap in all; anything else
//! ends the run and cancels the job's run (carry 1). The job's faults are the port's, `Range` naming the shape.

use super::*;
use kx_ipc::{MAX_NET_RUN_TEXT_BYTES, MAX_NET_TEXT_BYTES};

/// A run under watch: its host and run stay alive while the test plays the worker.
struct Watched {
    _host: Host,
    _run: Box<dyn CloudRun>,
    script: Script<NetFrames>,
    heard: Heard,
    called: Receiver<Call>,
}

/// Sends a run of `n` one-second pieces, watching the worker's calls.
fn watched(n: usize) -> Watched {
    let (host, script, _) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (run, heard) = sent(&host, ask(&vec![PER_S; n]));
    Watched {
        _host: host,
        _run: run,
        script,
        heard,
        called,
    }
}

#[test]
fn a_text_out_of_order_ends_the_run_as_gone_and_cancels_it() {
    let w = watched(2);
    w.script.say(Ok(text(1, SAID)));
    assert_eq!(next(&w.heard, 1), [CloudNote::Fault(CloudFault::Gone)]);
    took(&w.called, &NetAsk::Cancel { id: ID });
}

#[test]
fn a_text_past_the_last_piece_ends_the_run_as_gone_and_cancels_it() {
    let w = watched(1);
    w.script.say(Ok(text(0, SAID)));
    w.script.say(Ok(text(1, SAID)));
    let gone = CloudNote::Fault(CloudFault::Gone);
    assert_eq!(next(&w.heard, 2), [piece(0, SAID), gone]);
    took(&w.called, &NetAsk::Cancel { id: ID });
}

#[test]
fn an_end_before_every_text_is_gone_and_cancelled() {
    let w = watched(2);
    w.script.say(Ok(text(0, SAID)));
    w.script.say(Ok(NetNote::Done { id: ID }));
    let gone = CloudNote::Fault(CloudFault::Gone);
    assert_eq!(next(&w.heard, 2), [piece(0, SAID), gone]);
    took(&w.called, &NetAsk::Cancel { id: ID });
}

#[test]
fn texts_past_the_runs_cap_end_it_as_strange_and_cancel_it() {
    let fit = MAX_NET_RUN_TEXT_BYTES / MAX_NET_TEXT_BYTES;
    let w = watched(fit + 1);
    let long = "a".repeat(MAX_NET_TEXT_BYTES);
    for index in 0..=fit {
        w.script.say(Ok(text(u16::try_from(index).unwrap(), &long)));
    }
    let told = next(&w.heard, fit + 1);
    let pieces = told[..fit]
        .iter()
        .all(|n| matches!(n, CloudNote::Piece { .. }));
    assert!(pieces, "up to the cap");
    assert_eq!(told[fit], CloudNote::Fault(CloudFault::Strange));
    took(&w.called, &NetAsk::Cancel { id: ID });
}

#[test]
fn each_fault_of_the_job_is_the_ports_and_range_names_the_shape() {
    use std::time::Duration as D;
    let renews_s = Some(3_600);
    let pairs = [
        (NetFault::Key, CloudFault::Key),
        (NetFault::Credit, CloudFault::Credit),
        (
            NetFault::Allowance { renews_s },
            CloudFault::Allowance {
                renews_in: Some(D::from_secs(3_600)),
            },
        ),
        (
            NetFault::Allowance { renews_s: None },
            CloudFault::Allowance { renews_in: None },
        ),
        (NetFault::Rate, CloudFault::Rate),
        (NetFault::TooBig, CloudFault::TooBig),
        (NetFault::Language, CloudFault::Language),
        (NetFault::Empty, CloudFault::Empty),
        (NetFault::Strange, CloudFault::Strange),
        (NetFault::Network, CloudFault::Network),
        (NetFault::Timeout, CloudFault::Timeout),
        (NetFault::Range, CloudFault::Range(SHAPE)),
        (NetFault::Gone, CloudFault::Gone),
    ];
    for (net, port) in pairs {
        let w = watched(1);
        w.script.say(Ok(fault(net)));
        assert_eq!(next(&w.heard, 1), [CloudNote::Fault(port)], "{net:?}");
    }
}
