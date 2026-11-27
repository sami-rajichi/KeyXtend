//! A fake sound source: 10 ms packets whose samples follow the script's level, each handed over once the clock passes
//! its end and the script's latency, until the script says the stream failed.

use std::sync::Arc;
use std::time::Duration;

use super::{FakeStudio, Script};
use crate::record::{CHANNELS, Ear, Packet, RATE, RecordFault, Switch};

/// A fake source's packet length, as WASAPI's shared mode gives.
pub const PACKET: Duration = Duration::from_millis(10);

/// A sample's value, by when it was heard since the clock's zero.
type Level = Arc<dyn Fn(Duration) -> f32 + Send + Sync>;

/// How a fake source behaves.
#[derive(Clone)]
pub struct Voice {
    /// What opening it answers.
    pub opens: Result<(), RecordFault>,
    /// Each sample's value, by when it was heard.
    pub level: Level,
    /// How long after its end a packet is handed over.
    pub latency: Duration,
    /// Its stream fails from this time on.
    pub fails_at: Option<Duration>,
}

impl Voice {
    /// A source that opens and gives `level` all along, handed over one packet late.
    #[must_use]
    pub fn steady(level: f32) -> Self {
        Self::shaped(Arc::new(move |_| level))
    }

    /// A source that opens and gives `level`, handed over one packet late.
    #[must_use]
    pub fn shaped(level: Level) -> Self {
        Self {
            opens: Ok(()),
            level,
            latency: PACKET,
            fails_at: None,
        }
    }

    /// A source that never opens, answering `fault`.
    #[must_use]
    pub fn refused(fault: RecordFault) -> Self {
        Self {
            opens: Err(fault),
            ..Self::steady(0.0)
        }
    }
}

/// An open fake source: its studio, which source it is, and where its next packet starts.
struct FakeEar {
    studio: FakeStudio,
    which: Switch,
    next: Duration,
}

/// The script's voice for `which`.
fn voice(script: &Script, which: Switch) -> &Voice {
    match which {
        Switch::Mic => &script.mic,
        Switch::Sound => &script.sound,
    }
}

/// Opens `which` as the script says, its first packet starting now.
pub(super) fn open(studio: FakeStudio, which: Switch) -> Result<Box<dyn Ear>, RecordFault> {
    let next = {
        let world = studio.world();
        voice(&world.script, which).opens?;
        world.elapsed
    };
    Ok(Box::new(FakeEar {
        studio,
        which,
        next,
    }))
}

impl Ear for FakeEar {
    fn heard(&mut self) -> Vec<Packet> {
        let world = self.studio.world();
        let v = voice(&world.script, self.which);
        let end = v.fails_at.unwrap_or(Duration::MAX);
        let frames = usize::try_from(u128::from(RATE) * PACKET.as_nanos() / NS_S).unwrap_or(0);
        let mut packets = Vec::new();
        while self.next + PACKET + v.latency <= world.elapsed && self.next + PACKET <= end {
            let at = self.next;
            let samples = (0..frames)
                .flat_map(|i| {
                    let t = at + Duration::from_nanos(u64::try_from(i).unwrap_or(0) * NS_FRAME);
                    std::iter::repeat_n((v.level)(t), usize::from(CHANNELS))
                })
                .collect();
            packets.push(Packet {
                at: world.base + at,
                samples,
            });
            self.next += PACKET;
        }
        packets
    }

    fn failed(&self) -> bool {
        let world = self.studio.world();
        voice(&world.script, self.which)
            .fails_at
            .is_some_and(|t| t <= world.elapsed)
    }
}

/// Nanoseconds in a second.
const NS_S: u128 = 1_000_000_000;
/// Nanoseconds in one sound frame at the track's rate, whole.
const NS_FRAME: u64 = 1_000_000_000 / RATE as u64;
