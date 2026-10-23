//! A shortcut's tip in this language: its name and its keys as keyboards print them, in the order `tip.keys` gives.

use super::keys::{COMBO_JOIN, COMBO_THEN, KEYNAME, KEYS_TIP, LABEL};
use super::{I18n, find_in, join};
use fluent_bundle::FluentArgs;
use kx_module_api::ports::KeyName;

/// `tip.keys`'s variable for the shortcut's name.
const NAME_VAR: &str = "name";
/// `tip.keys`'s variable for the keys it sends.
const KEYS_VAR: &str = "keys";
/// What starts a function key's name, as in `f5`.
const F_KEY: char = 'f';
/// The last function key keyboards and Windows know: F24.
const LAST_F_KEY: u8 = 24;

impl I18n {
    /// The text keyboards print for key `name`: the board's label, else its `keyname` text, else the name in upper
    /// case when it is one letter, one digit or F1 to F24; none for any other name.
    #[must_use]
    pub fn key_name(&self, name: &KeyName) -> Option<String> {
        let name = name.as_str();
        let text = |group: &&str| find_in(&self.bundles, &join(group, name), None);
        let named = [LABEL, KEYNAME].iter().find_map(text);
        named.or_else(|| as_itself(name).then(|| name.to_ascii_uppercase()))
    }

    /// The keys of a shortcut's `steps`, each its key names with the tapped key last, such as `Ctrl+A, Ctrl+C`; a
    /// name no rule covers shows in upper case, and an empty step is left out.
    #[must_use]
    pub fn keys(&self, steps: &[Vec<KeyName>]) -> String {
        let plus = self.text(COMBO_JOIN);
        let shown = |n: &KeyName| {
            self.key_name(n)
                .unwrap_or_else(|| n.as_str().to_ascii_uppercase())
        };
        let step = |s: &Vec<KeyName>| s.iter().map(shown).collect::<Vec<_>>().join(&plus);
        let steps: Vec<String> = steps.iter().filter(|s| !s.is_empty()).map(step).collect();
        steps.join(&self.text(COMBO_THEN))
    }

    /// A shortcut's tip: the text of its name key `name` and the keys of its `steps`, as `tip.keys` orders them; the
    /// name alone when it has no keys.
    #[must_use]
    pub fn keys_tip(&self, name: &str, steps: &[Vec<KeyName>]) -> String {
        let (name, keys) = (self.text(name), self.keys(steps));
        if keys.is_empty() {
            return name;
        }
        let mut values = FluentArgs::new();
        values.set(NAME_VAR, name);
        values.set(KEYS_VAR, keys);
        self.format(KEYS_TIP, Some(&values))
    }
}

/// True when key name `name` prints as itself: one letter, one digit, or a function key.
fn as_itself(name: &str) -> bool {
    let one = name.len() == 1 && name.bytes().all(|b| b.is_ascii_alphanumeric());
    one || f_key(name)
}

/// True for `f1` to `f24`, written without a leading zero.
fn f_key(name: &str) -> bool {
    let Some(n) = name.strip_prefix(F_KEY) else {
        return false;
    };
    let digits = !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit());
    digits && n.parse::<u8>().is_ok_and(|n| (1..=LAST_F_KEY).contains(&n))
}
