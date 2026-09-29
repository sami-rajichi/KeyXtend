//! What the samples record, in order, and the switch that tells them how to behave.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use kx_module_api::{ModuleId, ServiceError, ServiceId};
use std::sync::{Arc, Mutex, PoisonError};

/// What a sample did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Did {
    Start(ModuleId),
    Stop(ModuleId),
    Ping(ModuleId),
    Got(ModuleId, &'static str),
    Refused(ModuleId, ServiceError),
    Saw(ModuleId, i64),
    Dropped(ServiceId),
}

/// The shared log every sample writes to.
#[derive(Clone, Default)]
pub struct Log(Arc<Mutex<Vec<Did>>>);

impl Log {
    pub fn push(&self, did: Did) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(did);
    }

    pub fn all(&self) -> Vec<Did> {
        self.0.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }

    fn pick(&self, pick: fn(Did) -> Option<ModuleId>) -> Vec<ModuleId> {
        self.all().into_iter().filter_map(pick).collect()
    }

    /// The modules that started, in order.
    pub fn starts(&self) -> Vec<ModuleId> {
        self.pick(|d| if let Did::Start(m) = d { Some(m) } else { None })
    }

    /// The modules that stopped, in order.
    pub fn stops(&self) -> Vec<ModuleId> {
        self.pick(|d| if let Did::Stop(m) = d { Some(m) } else { None })
    }

    /// The modules that saw a `Ping`, in order.
    pub fn pings(&self) -> Vec<ModuleId> {
        self.pick(|d| if let Did::Ping(m) = d { Some(m) } else { None })
    }
}

/// How a sample behaves; tests flip it through a shared `Switch`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Succeed,
    Fail,
    #[allow(dead_code, reason = "only the panics test binary builds it")]
    Panic,
    #[allow(dead_code, reason = "only the panics test binary builds it")]
    PanicInStop,
    #[allow(dead_code, reason = "only the kernel test binary builds it")]
    PingInStop,
}

/// The payload of every planned sample panic, so a test hook can hide exactly these.
pub struct SamplePanic;

/// A shared, changeable `Mode`.
#[derive(Clone)]
pub struct Switch(Arc<Mutex<Mode>>);

impl Switch {
    pub fn new(mode: Mode) -> Self {
        Self(Arc::new(Mutex::new(mode)))
    }

    pub fn set(&self, mode: Mode) {
        *self.0.lock().unwrap() = mode;
    }

    pub fn get(&self) -> Mode {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
