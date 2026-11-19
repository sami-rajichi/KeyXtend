//! The cloud speech port (ADR-0047): at Stop, a dictation's pieces go to one service the user switched on, through the
//! worker's `speech-net` job, and each piece's text or a fault comes back. Only `Network` reaches it, and the key, the
//! sound and the texts stay secret.

mod ask;
mod fault;
mod shape;
mod url;

pub use ask::{CloudAsk, CloudLimits, CloudTest, MAX_KEY_BYTES};
pub use fault::CloudFault;
pub use shape::{
    CloudShape, FormField, FormFields, KeyHeader, MAX_FIELD_VALUE, MAX_FIXED, OwnFields,
};
pub use url::HttpsUrl;

use crate::{Capability, Secret, ServiceId, ServiceKey};

/// What a run tells, in order, on any thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudNote {
    /// A piece's text, the first piece at index 0, told in the pieces' order; it is typed only once all are told.
    Piece {
        /// The piece's place in the ask.
        index: u16,
        /// Its text, an untrusted reply.
        text: Secret,
    },
    /// Every piece was told.
    Done,
    /// The run stopped on this fault; pieces told stay valid, and the caller keeps the recording for Try again.
    Fault(CloudFault),
}

/// A run on the platform's side. Dropping it cancels it: the requests in flight are dropped and nothing more is told.
/// The drop never blocks, so Cancel never waits on the network.
pub trait CloudRun: Send {}

/// Sends speech to a cloud service in the worker's `speech-net` job, which has the network and no microphone; the
/// platform provides it.
pub trait CloudSpeech: Send + Sync {
    /// Sends `ask`'s pieces to its service, several at a time within its limits; `tell` hears each note, on any thread.
    ///
    /// # Errors
    /// `Range` when `ask.check` refuses it, `Gone` when the job cannot start; the rest come as `Fault` notes.
    fn send(
        &self,
        ask: CloudAsk,
        tell: Box<dyn Fn(CloudNote) + Send>,
    ) -> Result<Box<dyn CloudRun>, CloudFault>;

    /// Says whether `ask`'s key works, by a call that sends no sound; it waits on the network, so call it off the
    /// module's thread.
    ///
    /// # Errors
    /// `Key` when the service refuses the key, `Range` when `ask.check` refuses it, else the fault that kept the answer.
    fn test(&self, ask: CloudTest) -> Result<(), CloudFault>;
}

/// The cloud speech service; it needs `Network`, which covers the services the user switched on (ADR-0047 decision 11).
pub struct CloudSpeechKey;

impl ServiceKey for CloudSpeechKey {
    type Api = dyn CloudSpeech;
    const ID: ServiceId = ServiceId::new("cloud-speech");
    const CAPABILITY: Option<Capability> = Some(Capability::Network);
}

#[cfg(test)]
mod tests;
