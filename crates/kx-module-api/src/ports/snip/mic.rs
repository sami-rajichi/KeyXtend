//! The microphone's pass (ADR-0056 decision 10): a service has one gate, so the recorder's is `ScreenCapture`, and a
//! recording hears the microphone only with a pass from the app's one `MicSeal`, which only `MicPassKey`, gated by
//! `Microphone`, reaches. A module may make its own seal, but the recorder opens no pass but its seal's.

use crate::{Capability, ServiceId, ServiceKey};
use std::sync::Arc;

/// The app's one microphone seal, made once and handed only to `MicPassKey` and the recorder; the recorder hears the
/// microphone only with a pass this seal made, so a pass from any other is refused (`RecordError::BadPass`).
#[derive(Clone, Debug, Default)]
pub struct MicSeal(Arc<()>);

impl MicSeal {
    /// A new seal; a pass it makes opens only this seal and its clones.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// True when this seal, or a clone of it, made `pass`.
    #[must_use]
    pub fn opens(&self, pass: &MicPass) -> bool {
        Arc::ptr_eq(&self.0, &pass.0)
    }
}

impl MicPasses for MicSeal {
    fn pass(&self) -> MicPass {
        MicPass(Arc::clone(&self.0))
    }
}

/// A seal's pass to hear the microphone, which only a recorder holding the same seal opens.
#[derive(Clone, Debug)]
pub struct MicPass(Arc<()>);

/// Two passes are alike when one seal made both.
impl PartialEq for MicPass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for MicPass {}

/// Hands out the app's microphone passes; the app provides its `MicSeal`.
pub trait MicPasses: Send + Sync {
    /// A pass of the app's seal.
    fn pass(&self) -> MicPass;
}

/// The microphone pass service; it needs `Microphone`.
pub struct MicPassKey;

impl ServiceKey for MicPassKey {
    type Api = dyn MicPasses;
    const ID: ServiceId = ServiceId::new("mic-pass");
    const CAPABILITY: Option<Capability> = Some(Capability::Microphone);
}
