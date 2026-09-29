//! The micro-kernel that starts, stops and contains feature modules.
//! It holds each module's lifecycle and the order in which modules start and stop.

#![forbid(unsafe_code)]

pub mod bus;
mod grants;
mod host;
mod kernel;
mod lifecycle;
pub mod log;
mod order;
mod registry;

pub use grants::Policy;
pub use kernel::{KERNEL, Kernel, KernelError, KernelSettings, Phase};
