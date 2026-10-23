//! `Ui`'s texts: the keyboard's, in the language of the layout in front, shortcut tips among them, and the tray's, in
//! the system's (ADR-0020).

use super::qobject;
use crate::bridges::qs;
use crate::hub::{Hub, NULL};
use cxx_qt_lib::QString;
use kx_module_api::ports::KeyName;

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

    pub(super) fn layer_page_text(&self, page: i32, pages: i32) -> QString {
        let (page, pages) = (usize::try_from(page), usize::try_from(pages));
        let (page, pages) = (page.unwrap_or_default(), pages.unwrap_or_default());
        qs(&self.with(String::new(), |h| h.i18n().layer_page(page, pages)))
    }

    pub(super) fn keys_tip(&self, name_key: &QString, steps: &QString) -> QString {
        let (key, steps) = (name_key.to_string(), steps_of(&steps.to_string()));
        qs(&self.with(key.clone(), |h| h.i18n().keys_tip(&key, &steps)))
    }

    pub(super) fn keys_text(&self, steps: &QString) -> QString {
        let steps = steps_of(&steps.to_string());
        qs(&self.with(String::new(), |h| h.i18n().keys(&steps)))
    }
}

/// A shortcut's steps from QML's JSON `json`; none when it does not read, as a key name of a bad form or length.
fn steps_of(json: &str) -> Vec<Vec<KeyName>> {
    let steps = serde_json::from_str::<Vec<Vec<KeyName>>>(json);
    steps.unwrap_or_else(|_| {
        tracing::warn!("QML sent shortcut steps that do not read");
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::MAX_ID_LEN;

    /// `names` as key names.
    fn step(names: &[&str]) -> Vec<KeyName> {
        names.iter().map(|n| KeyName::new(n).unwrap()).collect()
    }

    #[test]
    fn a_shortcuts_steps_read_from_qmls_json() {
        let read = steps_of(r#"[["ctrl", "a"], ["ctrl", "c"]]"#);
        assert_eq!(read, vec![step(&["ctrl", "a"]), step(&["ctrl", "c"])]);
        assert!(steps_of("[]").is_empty());
    }

    #[test]
    fn steps_that_do_not_read_give_no_keys_and_no_panic() {
        let long = format!(r#"[["{}"]]"#, "a".repeat(MAX_ID_LEN + 1));
        let deep = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
        let bad = [
            "oops",
            "",
            r#"{"steps": [["ctrl"]]}"#,
            r#"["ctrl", "c"]"#,
            "[[1, 2]]",
            r#"[["Ctrl", "c"]]"#,
            r#"[["ctrl", "-c"]]"#,
            r#"[["ctrl", "c d"]]"#,
            &long,
            &deep,
        ];
        for json in bad {
            assert!(
                steps_of(json).is_empty(),
                "{}",
                json.get(..40).unwrap_or(json)
            );
        }
    }
}
