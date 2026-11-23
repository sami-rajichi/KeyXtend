//! The crate's tests: addresses, host lists, ranges, the redirect and resume rules over the fake, calls with headers,
//! and the fault map. None touches the network or sleeps; a stall is a scripted quiet the fake weighs against the wait.

mod call;
mod faults;
mod follow;
mod hosts;
mod range;
mod sweep;
mod target;

use crate::{Answer, NetError};
use std::borrow::Cow;

/// The hosts the tests' downloads list.
const HOSTS: [Cow<'static, str>; 2] = [
    Cow::Borrowed("models.example"),
    Cow::Borrowed("cdn.example"),
];

/// Every byte left in `answer`'s body, or the error that stopped it.
fn drain(answer: &mut Answer) -> Result<Vec<u8>, NetError> {
    let (mut out, mut buf) = (Vec::new(), [0u8; 3]);
    loop {
        match answer.body.read(&mut buf)? {
            0 => return Ok(out),
            n => out.extend_from_slice(&buf[..n]),
        }
    }
}
