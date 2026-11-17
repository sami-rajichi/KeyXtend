//! The net job's one-byte codes: request shapes, key headers and faults; code 0 is never used.

use crate::code::Code;

/// A request shape's byte, which a form's names follow; the port and the frames share its `needs_model` rule.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ShapeCode {
    /// The upload form.
    Form = 1,
    /// Azure's fast transcription.
    Azure = 2,
    /// `ElevenLabs`' form.
    Eleven = 3,
    /// Gemini's single request.
    Gemini = 4,
}

impl Code for ShapeCode {
    const ALL: &'static [Self] = &[Self::Form, Self::Azure, Self::Eleven, Self::Gemini];

    fn code(self) -> u8 {
        self as u8
    }
}

impl ShapeCode {
    /// True when a run to this shape must name a model: `ElevenLabs` and Gemini have no default one.
    #[must_use]
    pub const fn needs_model(self) -> bool {
        matches!(self, Self::Eleven | Self::Gemini)
    }
}

/// A key header's byte; a header's name follows it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum HeaderCode {
    /// `Authorization: Bearer`.
    Bearer = 1,
    /// A named header.
    Named = 2,
}

impl Code for HeaderCode {
    const ALL: &'static [Self] = &[Self::Bearer, Self::Named];

    fn code(self) -> u8 {
        self as u8
    }
}

/// A fault's byte; an allowance's renewal follows it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FaultCode {
    /// `NetFault::Key`.
    Key = 1,
    /// `NetFault::Credit`.
    Credit = 2,
    /// `NetFault::Allowance`.
    Allowance = 3,
    /// `NetFault::Rate`.
    Rate = 4,
    /// `NetFault::TooBig`.
    TooBig = 5,
    /// `NetFault::Language`.
    Language = 6,
    /// `NetFault::Empty`.
    Empty = 7,
    /// `NetFault::Strange`.
    Strange = 8,
    /// `NetFault::Network`.
    Network = 9,
    /// `NetFault::Timeout`.
    Timeout = 10,
    /// `NetFault::Range`.
    Range = 11,
    /// `NetFault::Gone`.
    Gone = 12,
}

impl Code for FaultCode {
    const ALL: &'static [Self] = &[
        Self::Key,
        Self::Credit,
        Self::Allowance,
        Self::Rate,
        Self::TooBig,
        Self::Language,
        Self::Empty,
        Self::Strange,
        Self::Network,
        Self::Timeout,
        Self::Range,
        Self::Gone,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}
