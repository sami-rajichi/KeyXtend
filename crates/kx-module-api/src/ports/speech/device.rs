//! The devices a model runs on, the `device` setting's pick among them (ADR-0044 decision 8), and what a model file
//! tells about itself.

use super::TAG_LEN;

/// A graphics card's id as the engine names it, such as `Vulkan0`: ASCII letters, digits, `-`, `_` and `.`, within
/// `TAG_LEN`, and never a pick's own name, so the `device` setting reads one way only.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(String);

impl CardId {
    /// Checks and wraps `text`; `None` when it breaks a rule.
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        let plain = |b: u8| b.is_ascii_alphanumeric() || b"-_.".contains(&b);
        let named = [DevicePick::AUTO, DevicePick::CPU].contains(&text);
        let fits = TAG_LEN.contains(&text.len()) && text.bytes().all(plain);
        (fits && !named).then(|| Self(text.to_owned()))
    }
    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Where the `device` setting runs models: Automatic, the processor, or one card.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DevicePick {
    /// The processor until a card passes its check, then the best checked card with room.
    Auto,
    /// The processor.
    Cpu,
    /// This card, once its check passed; the processor serves meanwhile.
    Card(CardId),
}

impl DevicePick {
    /// Automatic's name in the setting.
    pub const AUTO: &'static str = "auto";
    /// The processor's name in the setting.
    pub const CPU: &'static str = "cpu";

    /// The pick `text` names; `None` for text that is no pick, so the setting goes back to its default.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            Self::AUTO => Some(Self::Auto),
            Self::CPU => Some(Self::Cpu),
            _ => CardId::new(text).map(Self::Card),
        }
    }

    /// The pick's name in the setting.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Auto => Self::AUTO,
            Self::Cpu => Self::CPU,
            Self::Card(id) => id.as_str(),
        }
    }
}

/// What a device is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    /// The processor.
    Cpu,
    /// A graphics card, by its id.
    Card(CardId),
}

/// A device's memory, in MB.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    /// All of it.
    pub total_mb: u32,
    /// What is free now.
    pub free_mb: u32,
}

/// A device the engine can run models on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// What it is.
    pub kind: DeviceKind,
    /// Its name as the system gives it, such as the card's model.
    pub name: String,
    /// Its memory now.
    pub memory: Memory,
    /// Its driver's version, kept with each check; empty for the processor or when unknown.
    pub driver: String,
}

/// What a model file tells about itself, read by the engine in the sandboxed job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFacts {
    /// The runner that reads it, as the models table names it, such as `whisper`.
    pub runner: String,
    /// The model's own code for each language it knows, such as `fr` or `fr-FR`.
    pub langs: Vec<String>,
}
