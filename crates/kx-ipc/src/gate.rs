//! Gate builds only: the line, field and stage names the voice gate build logs and kx-gates reads back, one copy for
//! both.
//! Like `SOUND_FLAG`, only the app's `gate-voice` and kx-gates' `voice` features reach them.

/// The field naming a dictation's stage.
pub const STAGE: &str = "stage";
/// The field holding a piece's index.
pub const INDEX: &str = "index";
/// The field holding a piece's sound length, in ms.
pub const SOUND_MS: &str = "sound_ms";
/// The field holding the model's load time, in ms.
pub const TOOK_MS: &str = "took_ms";
/// The field holding the id of the device that wrote a dictation.
pub const DEVICE: &str = "device";
/// The field holding the id of the cloud service that wrote or failed a dictation.
pub const SERVICE: &str = "service";
/// The field holding a cloud fault's kind.
pub const FAULT: &str = "fault";

/// The run was asked to start.
pub const START: &str = "start";
/// The microphone opened.
pub const OPEN: &str = "open";
/// Stop reached the run.
pub const STOP: &str = "stop";
/// The model's load ended.
pub const LOAD: &str = "load";
/// A piece's text came.
pub const PIECE: &str = "piece";
/// A cloud dictation's pieces went to its service.
pub const SEND: &str = "send";

/// The message of each stage's line.
pub const STAGE_LINE: &str = "voice gate build: a stage";
/// The message of the line logged as each dictation is written, with its device or service.
pub const WROTE_LINE: &str = "voice gate build: a dictation was written";
/// The message of the line logged as a cloud dictation fails, with its service and fault kind.
pub const FAILED_LINE: &str = "voice gate build: a cloud dictation failed";
