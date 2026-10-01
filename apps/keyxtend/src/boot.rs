//! The app before Qt: the hub first, then the kernel with the platform's services and every shipped module. The hub
//! hears the kernel's bus before the settings load, so no notice is missed; the app opens its log between load and
//! start.

use kx_kernel::{Kernel, KernelError, Policy};
use kx_module_api::ports::{InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey};
use kx_module_api::{Module, Subscription};
use kx_platform::Platform;
use kx_ui::{Hub, Request, UiModule, UiPorts};
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

/// Why the app could not boot, for the log.
pub type BootError = Box<dyn Error + Send + Sync>;

/// The services the platform adapter hands the kernel.
pub struct Services {
    /// Types into the app in front.
    pub injector: Arc<dyn InputInjector>,
    /// Reads the OS keyboard layouts.
    pub layouts: Arc<dyn LayoutSource>,
}

/// The modules this build ships; the `ui` module feeds `hub`.
#[must_use]
pub fn modules(hub: &Arc<Hub>) -> Vec<Box<dyn Module>> {
    let mut list = picked();
    list.push(Box::new(UiModule::new(Arc::clone(hub))));
    list
}

/// The modules the build's features pick (one feature per module).
#[allow(
    unused_mut,
    clippy::vec_init_then_push,
    reason = "each push sits behind its module's feature, and a build may turn them all off"
)]
fn picked() -> Vec<Box<dyn Module>> {
    let mut list: Vec<Box<dyn Module>> = Vec::new();
    #[cfg(feature = "mod-layouts")]
    list.push(Box::new(kx_mod_layouts::LayoutsModule));
    #[cfg(feature = "mod-keyboard")]
    list.push(Box::new(kx_mod_keyboard::KeyboardModule::default()));
    list
}

/// The grants: each shipped module may use what its manifest declares, since the app vouches for what it ships.
#[must_use]
pub fn policy(modules: &[Box<dyn Module>]) -> Policy {
    let manifests = modules.iter().map(|m| m.manifest());
    manifests.fold(Policy::new(), |p, m| p.allow(m.id, m.capabilities))
}

/// The hub, the kernel and the queue of the UI's requests, before Qt runs.
pub struct Boot {
    /// Starts, stops and contains the modules.
    pub kernel: Kernel,
    /// What the UI and the modules share.
    pub hub: Arc<Hub>,
    /// Try again and Quit, which the kernel's thread drains.
    pub requests: Receiver<Request>,
    /// The hub hears the kernel's notices while this lives.
    pub heard: Subscription,
}

impl Boot {
    /// Makes the hub before any window, adds `services` and every module, and loads the settings; nothing starts.
    ///
    /// # Errors
    /// When the hub's compiled-in data is broken, or the kernel or its bus refuses a step.
    pub fn load(
        platform: Arc<dyn Platform>,
        services: Services,
        ports: UiPorts,
    ) -> Result<Self, BootError> {
        let (hub, requests) = Hub::new(ports)?;
        let modules = modules(&hub);
        let mut kernel = Kernel::new(platform, policy(&modules));
        kernel.provide_platform::<InputInjectorKey>(services.injector)?;
        kernel.provide_platform::<LayoutSourceKey>(services.layouts)?;
        for module in modules {
            kernel.add(module)?;
        }
        let heard = hub.listen(&kernel.bus())?;
        kernel.load()?;
        Ok(Self {
            kernel,
            hub,
            requests,
            heard,
        })
    }

    /// Starts every module it can; their notices went to the hub already.
    ///
    /// # Errors
    /// `WrongPhase` when the modules started already.
    pub fn start(&mut self) -> Result<(), KernelError> {
        self.kernel.start().map(drop)
    }
}
