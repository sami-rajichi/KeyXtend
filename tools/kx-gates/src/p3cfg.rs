//! Settings of the gates P3 adds besides G8: G9 window flags, G10 screen readers, G11 DPI, G13 themes and G26 other
//! languages, from `[g9]` to `[g26]` of `kx-gates-p3.toml`; the check covers G8's `[osk]` too.
#![cfg(windows)]

use serde::Deserialize;

use crate::config::GatesConfig;

/// G9 settings: the top bar's buttons that hide and show the keyboard; resizing uses `[g8]`'s resize settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G9 {
    /// The button that puts the keyboard away, by UI Automation id.
    pub put_away: String,
    /// The bubble that brings it back, by UI Automation id.
    pub bring_back: String,
    /// Wait after each step before the flags are read, in ms.
    pub settle_ms: u64,
}

/// G10 settings: finding and pressing each key the way a screen reader does.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G10 {
    /// Pause after each press through UI Automation, in ms.
    pub press_gap_ms: u64,
    /// Wait for the last character to reach target-window, in ms.
    pub drain_ms: u64,
    /// Wait after each key step for its keys to reach target-window, in ms.
    pub step_wait_ms: u64,
    /// The send key pressed after each modifier latches, so the modifier goes out with it.
    pub probe: String,
    /// Keys that failed the check by point listed in full; all of them are counted.
    pub failures_shown: usize,
}

/// G11 settings: the scales the owner sets, one run each.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G11 {
    /// Display scales in percent that must each have a run.
    pub scales: Vec<u32>,
    /// Largest difference in the keyboard's logical size between scales, in px.
    pub slack_px: f64,
}

/// G13 settings: watching the keyboard follow Windows' look while the owner changes it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G13 {
    /// How long the gate watches, in seconds.
    pub watch_secs: u64,
    /// The keyboard redraws within this time of a change, in ms.
    pub follow_ms: u64,
    /// The watch ends early after this many changes.
    pub changes: usize,
    /// Parts of the look that must each change at least once, named as `g13::KINDS` names them.
    pub needs: Vec<String>,
    /// The share of each key's width and height left out on every side, so only its solid face is compared.
    pub key_inset: f64,
}

/// G26 settings: the languages checked on the CI runner.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G26 {
    /// Language tags such as `de-DE`; each is checked in the first installed layout of its language part.
    pub languages: Vec<String>,
    /// Clicks on character keys per language.
    pub clicks: usize,
    /// Longest wait for the keyboard to show, in ms; the runner starts it just before.
    pub show_wait_ms: u64,
    /// Longest wait, in ms, for every character key to be listed and named after a new layout.
    pub label_wait_ms: u64,
}

/// Refuses settings that could pass with nothing checked.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    c.osk.check()?;
    let (g9, g11) = (&c.g9, &c.g11);
    if g9.put_away.is_empty() || g9.bring_back.is_empty() {
        return Err("g9.put_away and g9.bring_back must name a button".to_string());
    }
    let roles = crate::board::roles()?;
    crate::g10keys::probe_scan(&roles, &c.g10.probe).map_err(|e| format!("g10.probe: {e}"))?;
    if g11.scales.is_empty() || !g11.slack_px.is_finite() || g11.slack_px < 0.0 {
        return Err("g11 needs scales and a finite slack of 0 px or more".to_string());
    }
    check_g13(&c.g13)?;
    check_g26(&c.g26)
}

/// Refuses G13 settings that could pass with nothing checked.
fn check_g13(g13: &G13) -> Result<(), String> {
    if g13.watch_secs == 0 || g13.changes == 0 {
        return Err("g13.watch_secs and g13.changes must be above 0".to_string());
    }
    let known = |n: &String| crate::g13::KINDS.contains(&n.as_str());
    if g13.needs.is_empty() || !g13.needs.iter().all(known) {
        return Err(format!(
            "g13.needs must name some of {:?}",
            crate::g13::KINDS
        ));
    }
    if g13.changes < g13.needs.len() {
        return Err("g13.changes must leave room for each of g13.needs".to_string());
    }
    if !(0.0..crate::g13::MAX_INSET).contains(&g13.key_inset) {
        return Err(format!(
            "g13.key_inset must be 0 or more and below {}, so part of each key is left",
            crate::g13::MAX_INSET
        ));
    }
    Ok(())
}

/// Refuses G26 settings that could pass with nothing checked.
fn check_g26(g26: &G26) -> Result<(), String> {
    if g26.languages.is_empty() || g26.clicks == 0 {
        return Err("g26 needs languages and clicks".to_string());
    }
    let keys = crate::board::char_keys()?.len();
    if g26.clicks < keys {
        return Err(format!(
            "g26.clicks must be {keys} or more, so the sweep types every character key"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;

    fn checked(c: &GatesConfig) -> Result<(), String> {
        check(c)
    }

    #[test]
    fn the_shipped_settings_pass_and_g26_checks_the_roadmaps_four_languages() {
        let c = load().expect("kx-gates.toml loads");
        assert_eq!(checked(&c), Ok(()));
        assert_eq!(c.g26.languages, ["de-DE", "ru-RU", "he-IL", "fa-IR"]);
        assert_eq!(c.g11.scales, [100, 150, 175]);
        assert_eq!(c.g13.needs, crate::g13::KINDS, "the roadmap's three");
    }

    #[test]
    fn settings_that_check_nothing_are_refused() {
        let bad: [fn(&mut GatesConfig); 14] = [
            |c| c.osk.key.clear(),
            |c| c.g9.put_away.clear(),
            |c| c.g9.bring_back.clear(),
            |c| c.g10.probe = "shift".into(),
            |c| c.g11.scales.clear(),
            |c| c.g11.slack_px = -1.0,
            |c| c.g11.slack_px = f64::INFINITY,
            |c| c.g13.changes = 0,
            |c| c.g13.changes = c.g13.needs.len() - 1,
            |c| c.g13.needs.clear(),
            |c| c.g13.needs.push("blue".into()),
            |c| c.g13.key_inset = 0.5,
            |c| c.g13.key_inset = f64::NAN,
            |c| c.g26.languages.clear(),
        ];
        for (i, f) in bad.iter().enumerate() {
            let mut c = load().expect("loads");
            f(&mut c);
            assert!(checked(&c).is_err(), "case {i}");
        }
    }

    #[test]
    fn g26_clicks_must_reach_every_character_key_once() {
        let keys = crate::board::char_keys().expect("the board reads").len();
        let mut c = load().expect("loads");
        assert!(c.g26.clicks >= keys, "the shipped clicks sweep the board");
        c.g26.clicks = keys - 1;
        assert!(checked(&c).is_err(), "a key would never be typed");
    }

    #[test]
    fn the_ci_job_can_read_the_g26_languages_line_as_json() {
        // ci.yml's G26 job finds this line with `^languages = (\[.*\])$` and reads the list as JSON.
        let lines: Vec<&str> = include_str!("../kx-gates-p3.toml")
            .lines()
            .filter_map(|l| l.strip_prefix("languages = "))
            .collect();
        assert_eq!(lines.len(), 1, "one languages line in the file");
        let tags: Vec<String> = serde_json::from_str(lines[0]).expect("a JSON list");
        assert_eq!(tags, load().expect("loads").g26.languages);
    }

    #[test]
    fn the_gates_name_top_bar_buttons_by_their_icons_toml_bar_ids() {
        let c = load().expect("loads");
        let themes = kx_theme::Themes::shipped().expect("the tokens read");
        let icons = serde_json::to_value(&themes.icons).expect("serialises");
        let g8 = [&c.g8.grow, &c.g8.shrink, &c.g8.churn_theme];
        for id in [&c.g9.put_away, &c.g9.bring_back].into_iter().chain(g8) {
            let part = id.strip_prefix("bar.").expect("a bar id");
            assert!(icons["bar"].get(part).is_some(), "{id}");
        }
    }
}
