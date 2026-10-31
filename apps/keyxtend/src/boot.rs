//! The app before Qt: the hub first, with the clipboard history's folder, then the kernel with the platform's services
//! and every shipped module. The hub hears the kernel's bus before the settings load, so no notice is missed; the app
//! opens its log between load and start.

use kx_kernel::{Kernel, KernelError, Policy};
use kx_module_api::ports::{
    Calendar, CalendarKey, Clipboard, ClipboardKey, ClipboardWriter, ClipboardWriterKey,
    FieldWatch, FieldWatchKey, InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey,
    Pictures, PicturesKey, PointerHook, PointerHookKey, PowerDialog, PowerDialogKey, ScrollTarget,
    ScrollTargetKey, SealedShelf, SealedShelfKey, SealedStore, SealedStoreKey, Shortcuts,
    ShortcutsKey, WordPacks, WordPacksKey,
};
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
    /// The mouse hook the mouse module's hold engine runs on.
    pub pointer: Arc<dyn PointerHook>,
    /// Scrolls what the user pointed at, for the scroll module.
    pub scroll: Arc<dyn ScrollTarget>,
    /// Sends the system shortcuts, for the keyboard's shortcuts layer.
    pub shortcuts: Arc<dyn Shortcuts>,
    /// Opens the system's power dialog, for the power module.
    pub power: Arc<dyn PowerDialog>,
    /// Watches the focused box, for the modules that read typed text.
    pub field: Arc<dyn FieldWatch>,
    /// Opens the word packs, for prediction.
    pub packs: Arc<dyn WordPacks>,
    /// Keeps the learned words sealed, for prediction.
    pub sealed: Arc<dyn SealedStore>,
    /// Watches copies and checks copied files, for the clipboard history.
    pub clipboard: Arc<dyn Clipboard>,
    /// Sets the clipboard for each paste, for the clipboard history.
    pub writer: Arc<dyn ClipboardWriter>,
    /// Keeps the clipboard history sealed in day folders.
    pub shelf: Arc<dyn SealedShelf>,
    /// Draws the history's thumbnails and previews in the media worker.
    pub pictures: Arc<dyn Pictures>,
    /// The local day, for the history's day folders.
    pub calendar: Arc<dyn Calendar>,
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
    #[cfg(feature = "mod-mouse")]
    list.push(Box::new(kx_mod_mouse::MouseModule::default()));
    #[cfg(feature = "mod-scroll")]
    list.push(Box::new(kx_mod_scroll::ScrollModule::default()));
    #[cfg(feature = "mod-power")]
    list.push(Box::new(kx_mod_power::PowerModule::default()));
    #[cfg(feature = "mod-predict")]
    list.push(Box::new(kx_mod_predict::PredictModule::default()));
    #[cfg(feature = "mod-clipboard")]
    list.push(Box::new(kx_mod_clipboard::ClipboardModule::default()));
    list
}

/// The layout source the kernel serves: the platform's, which debug builds wrap so the churn can turn Caps Lock over.
fn layout_source(source: Arc<dyn LayoutSource>) -> Arc<dyn LayoutSource> {
    #[cfg(debug_assertions)]
    let source = crate::layouts_debug::wrap(source);
    source
}

/// The grants: each shipped module may use what its manifest declares, since the app vouches for what it ships.
#[must_use]
pub fn policy(modules: &[Box<dyn Module>]) -> Policy {
    let manifests = modules.iter().map(|m| m.manifest());
    manifests.fold(Policy::new(), |p, m| p.allow(m.id, m.capabilities))
}

/// Registers each of `services` with `kernel` under its key.
fn provide(kernel: &mut Kernel, services: Services) -> Result<(), KernelError> {
    kernel.provide_platform::<InputInjectorKey>(services.injector)?;
    kernel.provide_platform::<LayoutSourceKey>(layout_source(services.layouts))?;
    kernel.provide_platform::<PointerHookKey>(services.pointer)?;
    kernel.provide_platform::<ScrollTargetKey>(services.scroll)?;
    kernel.provide_platform::<ShortcutsKey>(services.shortcuts)?;
    kernel.provide_platform::<PowerDialogKey>(services.power)?;
    kernel.provide_platform::<FieldWatchKey>(services.field)?;
    kernel.provide_platform::<WordPacksKey>(services.packs)?;
    kernel.provide_platform::<SealedStoreKey>(services.sealed)?;
    kernel.provide_platform::<ClipboardKey>(services.clipboard)?;
    kernel.provide_platform::<ClipboardWriterKey>(services.writer)?;
    kernel.provide_platform::<SealedShelfKey>(services.shelf)?;
    kernel.provide_platform::<PicturesKey>(services.pictures)?;
    kernel.provide_platform::<CalendarKey>(services.calendar)
}

/// The hub, the kernel and the queue of the UI's requests, before Qt runs.
pub struct Boot {
    /// Starts, stops and contains the modules.
    pub kernel: Kernel,
    /// What the UI and the modules share.
    pub hub: Arc<Hub>,
    /// Try again and Quit, which the kernel's thread drains.
    pub requests: Receiver<Request>,
    /// The hub hears the kernel's notices, mouse-assist and scroll events while these live.
    pub heard: Vec<Subscription>,
}

impl Boot {
    /// Makes the hub before any window and hands it the history's folder under `platform`'s data folder, adds
    /// `services` and every module, and loads the settings; nothing starts.
    ///
    /// # Errors
    /// When the hub's compiled-in data is broken, or the kernel or its bus refuses a step.
    pub fn load(
        platform: Arc<dyn Platform>,
        services: Services,
        ports: UiPorts,
    ) -> Result<Self, BootError> {
        let (hub, requests) = Hub::new(ports)?;
        hub.set_clip_folder(crate::clip::folder(platform.dirs()));
        let modules = modules(&hub);
        let mut kernel = Kernel::new(platform, policy(&modules));
        provide(&mut kernel, services)?;
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
