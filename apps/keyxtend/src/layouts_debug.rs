//! Debug builds only: the layout source the kernel serves, with Caps Lock read the other way round while the churn's
//! flag is set (ADR-0033 decision 5), so the memory check changes Caps Lock with no key sent.

use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, LayoutSource, Scan, SwitchError};
use std::sync::Arc;

/// The platform's layout source, every call passed through but `caps_on`, which the churn's flag turns over.
struct ChurnCaps(Arc<dyn LayoutSource>);

/// `source`, wrapped so the churn's flag turns its Caps Lock over.
#[must_use]
pub fn wrap(source: Arc<dyn LayoutSource>) -> Arc<dyn LayoutSource> {
    Arc::new(ChurnCaps(source))
}

impl LayoutSource for ChurnCaps {
    fn installed(&self) -> Vec<LayoutId> {
        self.0.installed()
    }

    fn foreground(&self) -> Option<LayoutId> {
        self.0.foreground()
    }

    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
        self.0.keys(layout, scans)
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        self.0.facts(layout)
    }

    fn caps_on(&self) -> bool {
        self.0.caps_on() != kx_ui::caps_flipped()
    }

    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
        self.0.types_any(layout, key, held)
    }

    fn request(&self, layout: LayoutId) -> Result<(), SwitchError> {
        self.0.request(layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_platform_fake::FakeLayoutSource;

    /// The captured English layout, the one the fake source holds.
    const EN: &str = "en-us";

    #[test]
    fn the_churn_flag_turns_caps_lock_over_and_back_and_the_rest_passes_through() {
        let fake = Arc::new(FakeLayoutSource::golden(&[EN]).unwrap());
        let source = wrap(fake.clone());
        assert!(!source.caps_on());
        kx_ui::flip_caps();
        assert!(source.caps_on(), "flipped");
        fake.set_caps(true);
        assert!(!source.caps_on(), "the system's on, turned over");
        kx_ui::flip_caps();
        assert!(source.caps_on(), "and back");
        assert_eq!(source.installed(), fake.installed());
        assert_eq!(source.foreground(), fake.foreground());
        let en = fake.installed()[0];
        assert_eq!(source.facts(en), fake.facts(en));
        let a = [Scan::new(0x1E)];
        assert_eq!(source.keys(en, &a), fake.keys(en, &a));
        assert_eq!(
            source.types_any(en, a[0], &[]),
            fake.types_any(en, a[0], &[])
        );
        assert_eq!(source.request(en), Ok(()));
        assert_eq!(fake.requests(), [en], "the request reached the system");
    }
}
