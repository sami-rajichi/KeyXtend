//! Hands a later copy's commands to the hub. The app listens right after it claims the session, before the hub
//! exists, so commands that come first wait here, in order, until `attach`.

use kx_platform::ui::Command;
use kx_ui::Hub;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Where commands go now.
enum Stage {
    /// No hub yet: the commands so far, oldest first.
    Early(Vec<Command>),
    /// Straight to the hub.
    Hub(Arc<Hub>),
}

/// The running copy's listener; share it in an `Arc` with the claim.
pub struct Relay(Mutex<Stage>);

impl Default for Relay {
    fn default() -> Self {
        Self(Mutex::new(Stage::Early(Vec::new())))
    }
}

impl Relay {
    /// Hands `command` to the hub, or keeps it until the hub is attached.
    pub fn hear(&self, command: Command) {
        let mut stage = self.lock();
        match &mut *stage {
            Stage::Early(waiting) => waiting.push(command),
            Stage::Hub(hub) => hub.command(command),
        }
    }

    /// Hands the waiting commands to `hub` in order, then every later one. The lock keeps the order; the hub never
    /// calls back here, and its `command` never blocks.
    pub fn attach(&self, hub: &Arc<Hub>) {
        let mut stage = self.lock();
        let before = std::mem::replace(&mut *stage, Stage::Hub(Arc::clone(hub)));
        if let Stage::Early(waiting) = before {
            for command in waiting {
                hub.command(command);
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, Stage> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
