//! G1 text: random EN/FR/AR/AltGr text from the configured character groups.
#![cfg(windows)]

use serde::Deserialize;

use crate::rng::Rng;

/// Characters in the lam-alef pair.
const PAIR: usize = 2;

/// Character groups and how often each is picked.
#[derive(Debug, Clone, Deserialize)]
pub struct Charsets {
    /// English letters, digits and punctuation.
    pub english: String,
    /// French accented letters and punctuation.
    pub french: String,
    /// Arabic letters; harakat may follow them.
    pub arabic: String,
    /// The lam-alef pair, typed as two characters.
    pub lam_alef: String,
    /// Arabic diacritics, placed only after a letter.
    pub harakat: String,
    /// Characters typed with `AltGr`.
    pub altgr: String,
    /// How often each group is picked.
    pub weights: Weights,
}

/// Relative pick weights per group.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Weights {
    /// Weight of `english`.
    pub english: u32,
    /// Weight of `french`.
    pub french: u32,
    /// Weight of `arabic`.
    pub arabic: u32,
    /// Weight of `lam_alef`.
    pub lam_alef: u32,
    /// Weight of `harakat`.
    pub harakat: u32,
    /// Weight of `altgr`.
    pub altgr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    English,
    French,
    Arabic,
    LamAlef,
    Harakat,
    AltGr,
}

impl Charsets {
    /// Rejects empty groups that can be picked, control characters, a bad pair and stray harakat.
    pub fn validate(&self) -> Result<(), String> {
        if self.arabic.is_empty() {
            return Err("text.arabic is empty".to_string());
        }
        if self.lam_alef.chars().count() != PAIR {
            return Err(format!("text.lam_alef must be {PAIR} characters"));
        }
        if self.groups().iter().all(|(_, w)| *w == 0) {
            return Err("text.weights are all zero".to_string());
        }
        for (group, weight) in self.groups() {
            let set = self.set(group);
            if weight > 0 && set.is_empty() {
                return Err(format!("text group {group:?} is empty"));
            }
            if set.chars().any(char::is_control) {
                return Err(format!("text group {group:?} has a control character"));
            }
            if group != Group::Harakat && set.chars().any(|c| self.harakat.contains(c)) {
                return Err(format!("text group {group:?} has a haraka"));
            }
        }
        Ok(())
    }

    /// True if `c` may appear in generated text.
    #[cfg(test)]
    pub fn allows(&self, c: char) -> bool {
        self.groups().iter().any(|(g, _)| self.set(*g).contains(c))
    }

    /// True if `c` is an Arabic letter a haraka may follow.
    pub fn is_letter(&self, c: char) -> bool {
        self.arabic.contains(c) || self.lam_alef.contains(c)
    }

    fn groups(&self) -> [(Group, u32); 6] {
        let w = self.weights;
        [
            (Group::English, w.english),
            (Group::French, w.french),
            (Group::Arabic, w.arabic),
            (Group::LamAlef, w.lam_alef),
            (Group::Harakat, w.harakat),
            (Group::AltGr, w.altgr),
        ]
    }

    fn set(&self, group: Group) -> &str {
        match group {
            Group::English => &self.english,
            Group::French => &self.french,
            Group::Arabic => &self.arabic,
            Group::LamAlef => &self.lam_alef,
            Group::Harakat => &self.harakat,
            Group::AltGr => &self.altgr,
        }
    }

    fn pick_group(&self, rng: &mut Rng) -> Group {
        let groups = self.groups();
        let total: u32 = groups.iter().map(|(_, w)| w).sum();
        let mut roll = u32::try_from(rng.below(total as usize)).unwrap_or_default();
        for (group, weight) in groups {
            if roll < weight {
                return group;
            }
            roll -= weight;
        }
        Group::Arabic
    }

    fn pick(&self, group: Group, rng: &mut Rng) -> char {
        let set: Vec<char> = self.set(group).chars().collect();
        set[rng.below(set.len())]
    }
}

/// Exactly `count` random characters; harakat only after an Arabic letter.
pub fn generate(sets: &Charsets, count: usize, rng: &mut Rng) -> Result<String, String> {
    sets.validate()?;
    let pair: Vec<char> = sets.lam_alef.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(count);
    while out.len() < count {
        let left = count - out.len();
        let after_letter = out.last().is_some_and(|c| sets.is_letter(*c));
        match sets.pick_group(rng) {
            Group::LamAlef if left >= pair.len() => out.extend(&pair),
            Group::Harakat if after_letter => out.push(sets.pick(Group::Harakat, rng)),
            Group::LamAlef | Group::Harakat => out.push(sets.pick(Group::Arabic, rng)),
            group => out.push(sets.pick(group, rng)),
        }
    }
    Ok(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sets() -> Charsets {
        crate::config::load().expect("kx-gates.toml loads").text
    }

    /// Small valid groups, so the checks run without kx-gates.toml.
    fn small() -> Charsets {
        Charsets {
            english: "ab".into(),
            french: "é".into(),
            arabic: "بت".into(),
            lam_alef: "لا".into(),
            harakat: "\u{64E}".into(),
            altgr: "€".into(),
            weights: Weights {
                english: 1,
                french: 1,
                arabic: 1,
                lam_alef: 1,
                harakat: 1,
                altgr: 1,
            },
        }
    }

    /// No group can be picked.
    const ZERO: Weights = Weights {
        english: 0,
        french: 0,
        arabic: 0,
        lam_alef: 0,
        harakat: 0,
        altgr: 0,
    };

    /// Words the error must contain, and the change that spoils `small()`.
    type Spoil = (&'static str, fn(&mut Charsets));

    #[test]
    fn validate_rejects_each_bad_setting() {
        assert_eq!(small().validate(), Ok(()));
        let cases: [Spoil; 8] = [
            ("arabic is empty", |s| s.arabic.clear()),
            ("lam_alef", |s| s.lam_alef = "ل".into()),
            ("lam_alef", |s| s.lam_alef = "لال".into()),
            ("all zero", |s| s.weights = ZERO),
            ("French is empty", |s| s.french.clear()),
            ("control", |s| s.english.push('\t')),
            ("Arabic has a haraka", |s| s.arabic.push('\u{64E}')),
            ("AltGr has a haraka", |s| s.altgr.push('\u{64E}')),
        ];
        for (want, spoil) in cases {
            let mut s = small();
            spoil(&mut s);
            let err = s.validate().expect_err(want);
            assert!(err.contains(want), "{want}: {err}");
        }
        let mut unused = small();
        unused.weights.french = 0;
        unused.french.clear();
        assert_eq!(unused.validate(), Ok(()), "an empty group nobody picks");
    }

    #[test]
    fn generates_exactly_n_allowed_chars() {
        let s = sets();
        for n in [0, 1, 2, 999, 1000] {
            let text = generate(&s, n, &mut Rng::new(n as u64)).expect("generates");
            assert_eq!(text.chars().count(), n);
            assert!(text.chars().all(|c| s.allows(c)));
            assert!(!text.contains(['\n', '\r', '\t']));
        }
    }

    #[test]
    fn same_seed_same_text_and_all_groups_appear() {
        let s = sets();
        let a = generate(&s, 5000, &mut Rng::new(9)).expect("generates");
        assert_eq!(a, generate(&s, 5000, &mut Rng::new(9)).expect("generates"));
        for set in [
            &s.english,
            &s.french,
            &s.arabic,
            &s.harakat,
            &s.altgr,
            &s.lam_alef,
        ] {
            assert!(a.chars().any(|c| set.contains(c)), "missing group {set}");
        }
        assert!(a.contains(s.lam_alef.as_str()));
    }

    #[test]
    fn harakat_only_after_a_letter() {
        let s = sets();
        let text: Vec<char> = generate(&s, 20_000, &mut Rng::new(3))
            .expect("generates")
            .chars()
            .collect();
        for (i, c) in text.iter().enumerate() {
            if s.harakat.contains(*c) {
                assert!(
                    i > 0 && s.is_letter(text[i - 1]),
                    "haraka at {i} not after a letter"
                );
            }
        }
    }
}
