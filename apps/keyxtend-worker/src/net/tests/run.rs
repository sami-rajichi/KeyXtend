//! A run's texts: those that come early are held until their turn, and the run ends `Strange` once its texts, told
//! and held together, pass `MAX_NET_RUN_TEXT_BYTES`, as the host would drop it.

use super::*;
use crate::net::run::Run;
use kx_ipc::{MAX_NET_RUN_TEXT_BYTES, MAX_NET_TEXT_BYTES, NetFault, NetNote};

#[test]
fn texts_held_out_of_order_past_the_runs_cap_are_strange() {
    let over = MAX_NET_RUN_TEXT_BYTES / MAX_NET_TEXT_BYTES + 1;
    let sizes = vec![2; over + 1];
    let mut run = Run::new(1, call(), run_of(&sizes, 1, 1, 20));
    let full = || Zeroizing::new("a".repeat(MAX_NET_TEXT_BYTES));
    for index in 1..over {
        let (notes, ended) = run.told(1, u16::try_from(index).unwrap(), Ok(full()));
        assert!(notes.is_empty() && !ended, "held at {index}");
    }
    let last = u16::try_from(over).unwrap();
    let strange = NetNote::Fault {
        id: 1,
        fault: NetFault::Strange,
    };
    assert_eq!(run.told(1, last, Ok(full())), (vec![strange], true));
}
