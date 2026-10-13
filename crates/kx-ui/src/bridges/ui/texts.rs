//! `Ui`'s texts: the keyboard's, in the language of the layout in front, and the tray's, in the system's (ADR-0020).

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;

impl qobject::Ui {
    pub(super) fn lang_json(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::lang_json))
    }

    pub(super) fn text(&self, key: &QString) -> QString {
        let key = key.to_string();
        qs(&self.with(key.clone(), |h| h.i18n().text(&key)))
    }

    pub(super) fn tray_text(&self, key: &QString) -> QString {
        let key = key.to_string();
        qs(&self.with(key.clone(), |h| h.tray_i18n().text(&key)))
    }

    pub(super) fn name_of(&self, group: &QString, id: &QString) -> QString {
        let (group, id) = (group.to_string(), id.to_string());
        qs(&self.with(id.clone(), |h| h.i18n().name_of(&group, &id)))
    }

    pub(super) fn space(&self, parts: &QString) -> QString {
        let Ok(parts) = serde_json::from_str::<Vec<String>>(&parts.to_string()) else {
            tracing::warn!("QML sent space bar parts that do not read");
            return QString::default();
        };
        qs(&self.with(String::new(), |h| h.i18n().space(&parts)))
    }

    pub(super) fn more_text(&self, n: i32) -> QString {
        let n = usize::try_from(n).unwrap_or_default();
        qs(&self.with(String::new(), |h| h.i18n().more(n)))
    }

    pub(super) fn page_text(&self, at: i32, count: i32) -> QString {
        let (at, count) = (usize::try_from(at), usize::try_from(count));
        let (at, count) = (at.unwrap_or_default(), count.unwrap_or_default());
        qs(&self.with(String::new(), |h| h.i18n().page(at, count)))
    }
}
