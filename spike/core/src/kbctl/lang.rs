//! The layout in front as the spike reads it: what each key types, the product's labels and the names' turn.

use std::sync::Arc;

use kx_module_api::ports::{LangLabel, LayoutId, LayoutSource, LayoutTable};
use kx_platform_windows::WinLayouts;

use super::Kb;
use crate::{langinfo, langkey, layout};

impl Kb {
    /// The current layout as the product names it.
    pub(super) fn layout_id(&self) -> LayoutId {
        LayoutId::new(self.hkl as u64)
    }

    /// What each key types in the current layout, with its label, read once per layout.
    pub(super) fn table(&mut self) -> Arc<LayoutTable> {
        let label = self.label_of(self.hkl);
        let spec = &self.spec;
        let table = self.chars.entry(self.hkl).or_insert_with(|| {
            let scans = spec.char_scans();
            let keys = scans.iter().copied().zip(WinLayouts.keys(label.id, &scans));
            let keys = keys.collect();
            Arc::new(LayoutTable { label, keys })
        });
        Arc::clone(table)
    }

    /// The product's label of layout `hkl`, read once.
    fn label_of(&mut self, hkl: isize) -> LangLabel {
        let c = &self.cfg.lang;
        let label = self.labels.entry(hkl);
        label.or_insert_with(|| langinfo::label(hkl, c)).clone()
    }

    /// Every installed layout's label, in the order the key steps.
    pub(super) fn labels(&mut self) -> Vec<LangLabel> {
        let installed = langkey::installed();
        installed.into_iter().map(|h| self.label_of(h)).collect()
    }

    /// The layout is now `h` among `ids`; the names turn the way the last step asked if it landed here, and once.
    pub(super) fn switched(&mut self, h: isize, ids: &[LayoutId]) {
        self.hkl = h;
        let now = self.layout_id();
        self.turn.seen(now, ids);
    }

    /// True when the app in front switched layout since the last look.
    pub fn relabel(&mut self) -> bool {
        let installed = WinLayouts.installed();
        let moved = self.follow.see(layout::other_front(), &installed);
        if let Some(id) = self.follow.shown().filter(|_| moved) {
            self.switched(id.get() as isize, &installed);
        }
        moved
    }
}
