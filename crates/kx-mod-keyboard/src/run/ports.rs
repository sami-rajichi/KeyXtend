//! The services a tap goes through, as the kernel hands them out, and the parts of them one tap reads.

use crate::layer::LayerCell;
use kx_module_api::ports::{
    InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey, LayoutTable, Layouts,
    LayoutsKey, Shortcuts, ShortcutsKey,
};
use kx_module_api::{Bus, Clock, ModuleCx, ServiceError};
use std::sync::Arc;

/// The ports one tap goes through, the layout table the keyboard shows and the layer's keys it draws.
#[derive(Clone, Copy)]
pub struct Io<'a> {
    /// Sends keys and text to the app in front, and names that app.
    pub injector: &'a dyn InputInjector,
    /// Says whether Caps Lock is on and whether a key types anything.
    pub source: &'a dyn LayoutSource,
    /// What each key types in the layout shown.
    pub table: &'a LayoutTable,
    /// Sends the layer's shortcuts by id.
    pub shortcuts: &'a dyn Shortcuts,
    /// The layer's keys drawn now; none while it is hidden.
    pub layer: &'a [LayerCell],
}

/// The services typing needs, as the kernel hands them out, the kernel's clock and its bus.
#[derive(Clone)]
pub struct Ports {
    /// The input injector.
    pub injector: Arc<dyn InputInjector>,
    /// The layout source.
    pub source: Arc<dyn LayoutSource>,
    /// The layouts service, whose current table the keyboard shows.
    pub layouts: Arc<dyn Layouts>,
    /// The shortcuts the platform knows and sends, for the layer.
    pub shortcuts: Arc<dyn Shortcuts>,
    /// The kernel's clock, which times a held key's repeats.
    pub clock: Arc<dyn Clock>,
    /// The kernel's bus, limited to the module's grants, which carries the other modules' events.
    pub bus: Bus,
}

impl Ports {
    /// The four services, the clock and the bus from the kernel.
    ///
    /// # Errors
    /// The `ServiceError` of the first service the module cannot have.
    pub fn from_cx(cx: &ModuleCx<'_>) -> Result<Self, ServiceError> {
        Ok(Self {
            injector: cx.service::<InputInjectorKey>()?,
            source: cx.service::<LayoutSourceKey>()?,
            layouts: cx.service::<LayoutsKey>()?,
            shortcuts: cx.service::<ShortcutsKey>()?,
            clock: cx.clock(),
            bus: cx.bus(),
        })
    }

    /// The ports of one tap, with `table`, the current table of `layouts`, and `layer`, the layer's keys drawn now.
    #[must_use]
    pub fn io<'a>(&'a self, table: &'a LayoutTable, layer: &'a [LayerCell]) -> Io<'a> {
        Io {
            injector: &*self.injector,
            source: &*self.source,
            table,
            shortcuts: &*self.shortcuts,
            layer,
        }
    }
}
