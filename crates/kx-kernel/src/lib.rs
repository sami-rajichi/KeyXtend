//! The micro-kernel that starts, stops and contains feature modules.
//! It holds each module's lifecycle and the order in which modules start and stop.

#![forbid(unsafe_code)]

pub mod bus;
pub mod grants;
pub mod host;
pub mod lifecycle;
pub mod order;
pub mod registry;
