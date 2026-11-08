//! The app before Qt: the hub first, with the clipboard history's folder, then the kernel with the platform's services
//! and every shipped module, readied for a normal or a hidden run. The hub hears the kernel's bus before the settings
//! load, so no notice is missed; the app opens its log between load and start.

use crate::hidden::Run;
use kx_kernel::{Kernel, KernelError, Policy};
use kx_module_api::ports::{
    Boxes, BoxesKey, Calendar, CalendarKey, Clipboard, ClipboardKey, ClipboardWriter,
    ClipboardWriterKey, FieldWatch, FieldWatchKey, InputInjector, InputInjectorKey, LayoutSource,
    LayoutSourceKey, Pictures, PicturesKey, Places, PlacesKey, PointerHook, PointerHookKey,
    PowerDialog, PowerDialogKey, Random, RandomKey, ScrollTarget, ScrollTargetKey, SealedShelf,
    SealedShelfKey, SealedStore, SealedStoreKey, SelectionWatch, SelectionWatchKey, Shortcuts,
    ShortcutsKey, VaultStoreKey, WebLink, WebLinkKey, WordPacks, WordPacksKey,
};
use kx_module_api::{Module, Subscription};
use kx_platform::Platform;
use kx_ui::{Hub, Request, UiModule, UiPorts};
use std::error::Error;
use std::io::ErrorKind;
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
    /// The local day, for the history's day folders and Quick-fill's stamps.
    pub calendar: Arc<dyn Calendar>,
    /// Keeps Quick-fill's notebook sealed in its own folder, under its own key.
    pub vault: Arc<dyn SealedStore>,
    /// The OS's random bytes, for Quick-fill's ids and made passwords.
    pub random: Arc<dyn Random>,
    /// Seals and opens Quick-fill's backup boxes in the vault worker.
    pub boxes: Arc<dyn Boxes>,
    /// The folders and drives Quick-fill's backup boxes go to.
    pub places: Arc<dyn Places>,
    /// Watches the focused box's selection, for the selection chips.
    pub selection: Arc<dyn SelectionWatch>,
    /// Opens Translate's page in the user's browser.
    pub link: Arc<dyn WebLink>,
}

/// The modules this build ships; the `ui` module feeds `hub`, and `os`, the system's display language, is the one
/// Translate follows.
#[must_use]
pub fn modules(hub: &Arc<Hub>, os: &str) -> Vec<Box<dyn Module>> {
    let mut list = picked(os);
    list.push(Box::new(UiModule::new(Arc::clone(hub))));
    list
}

/// The modules the build's features pick (one feature per module), with the system's display language `os`.
#[allow(
    unused_mut,
    unused_variables,
    clippy::vec_init_then_push,
    reason = "each push sits behind its module's feature, and a build may turn them all off"
)]
fn picked(os: &str) -> Vec<Box<dyn Module>> {
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
    #[cfg(feature = "mod-vault")]
    list.push(Box::new(kx_mod_vault::VaultModule::default()));
    #[cfg(feature = "mod-selection")]
    list.push(Box::new(kx_mod_selection::SelectionModule::new(os)));
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
    kernel.provide_platform::<CalendarKey>(services.calendar)?;
    kernel.provide_platform::<VaultStoreKey>(services.vault)?;
    kernel.provide_platform::<RandomKey>(services.random)?;
    kernel.provide_platform::<BoxesKey>(services.boxes)?;
    kernel.provide_platform::<PlacesKey>(services.places)?;
    kernel.provide_platform::<SelectionWatchKey>(services.selection)?;
    kernel.provide_platform::<WebLinkKey>(services.link)
}

/// The hub, the kernel and the queue of the UI's requests, before Qt runs.
pub struct Boot {
    /// Starts, stops and contains the modules.
    pub kernel: Kernel,
    /// What the UI and the modules share.
    pub hub: Arc<Hub>,
    /// Try again, answers and Quit, which the kernel's thread drains.
    pub requests: Receiver<Request>,
    /// The hub hears the kernel's notices, mouse-assist and scroll events while these live.
    pub heard: Vec<Subscription>,
    /// Why the history's folder could not be made; the run says so once its log is open.
    pub no_clip_dir: Option<ErrorKind>,
}

impl Boot {
    /// Makes the hub before any window, makes the history's folder under `platform`'s data folder and hands it to the
    /// hub unless `run` is hidden, readies the kernel for `run`, adds `services` and every module, and loads the
    /// settings; nothing starts. The system's display language is read once here, from `ports`' look.
    ///
    /// # Errors
    /// When the hub's compiled-in data is broken, or the kernel or its bus refuses a step.
    pub fn load(
        platform: Arc<dyn Platform>,
        services: Services,
        ports: UiPorts,
        run: &Run,
    ) -> Result<Self, BootError> {
        let os = ports.look.ui_language();
        let (hub, requests) = Hub::new(ports)?;
        let folder = crate::clip::folder(platform.dirs());
        let no_clip_dir = run.make_dir(&folder);
        if !run.is_hidden() {
            hub.set_clip_folder(folder);
        }
        let modules = modules(&hub, &os);
        let mut kernel = Kernel::new(platform, policy(&modules));
        run.ready(&mut kernel, &hub)?;
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
            no_clip_dir,
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
