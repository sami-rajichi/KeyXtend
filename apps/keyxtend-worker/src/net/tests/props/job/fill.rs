//! The job property's server: it answers a key test by its address and a piece by the byte its sound is filled with,
//! so each run's outcome follows from what the property sent.

use super::super::super::bench::fault;
use super::super::super::*;
use kx_ipc::{NetFault, NetNote};
use kx_platform_net::{Call, Hop, Http, NetError, Reply, Verb};

/// A piece's least size, so its fill shows as a run of this many bytes.
pub(super) const FILLED: usize = 4;
/// The fill of a part for no live run, which no request may carry.
pub(super) const STRAY: u8 = 0x5A;
/// What a text piece says.
const SAID: &str = "x";

/// How the server answers a run's pieces, by the byte their sound is filled with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Text,
    Key,
    Down,
}

impl Kind {
    pub(super) const ALL: [Self; 3] = [Self::Text, Self::Key, Self::Down];

    /// The byte its pieces' sound is filled with.
    pub(super) const fn fill(self) -> u8 {
        match self {
            Self::Text => 0x11,
            Self::Key => 0x22,
            Self::Down => 0x33,
        }
    }

    /// The server's answer to one of its pieces.
    fn answer(self) -> Act {
        match self {
            Self::Text => said(SAID),
            Self::Key => reply(401, KEY_BAD),
            Self::Down => reply(503, DOWN),
        }
    }

    /// The note that ends a run answered so.
    pub(super) fn end(self, id: u32) -> NetNote {
        match self {
            Self::Text => NetNote::Done { id },
            Self::Key => fault(id, NetFault::Key),
            Self::Down => fault(id, NetFault::Network),
        }
    }
}

/// True when `body` holds `FILLED` bytes of `fill` in a row.
pub(super) fn holds(body: &[u8], fill: u8) -> bool {
    body.windows(FILLED).any(|w| w.iter().all(|&b| b == fill))
}

/// A server that answers a key test by its address and a piece by its fill, keeping every body it was sent.
#[derive(Default)]
pub(super) struct ByFill(pub Mutex<Vec<Vec<u8>>>);

impl Http for ByFill {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }

    fn deliver(&self, call: &Call<'_>) -> Result<Reply, NetError> {
        lock(&self.0).push(call.body.to_vec());
        let act = if call.verb != Verb::Get {
            let kind = Kind::ALL.into_iter().find(|k| holds(call.body, k.fill()));
            kind.map_or_else(|| said(SAID), Kind::answer)
        } else if MODELS.ends_with(call.target.path()) {
            reply(OK, LIST)
        } else {
            reply(401, KEY_BAD)
        };
        FakeHttp::new([act]).call(call)
    }
}
