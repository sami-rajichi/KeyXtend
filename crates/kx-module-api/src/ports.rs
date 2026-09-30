//! Ports: the service traits and plain data types that modules, adapters and the UI talk through.

mod code;
mod face;
mod geom;
mod input;
mod layout;
mod layouts;

pub use code::CodeError;
pub use face::{FaceAction, FaceError, FaceReply, KeyId, KeyboardFace, KeyboardFaceKey};
pub use geom::{BASE_DPI, Corner, Point, Rect, Screen, physical};
pub use input::{InjectError, InputInjector, InputInjectorKey, Scan, TextRoute};
pub use layout::{KeyChars, LangFacts, LayoutId, LayoutSource, LayoutSourceKey, Slot, SwitchError};
pub use layouts::{LangLabel, LayoutChanged, LayoutTable, Layouts, LayoutsKey, step_to};
