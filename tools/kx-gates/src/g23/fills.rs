//! One G23 fill on a fresh login page: a click into its field, the `fill` key while the panel is shut, then a row
//! button of the test entry. It passes when only the wanted text arrived, the clipboard never changed, no Enter or
//! submit happened and the panel stayed open.
#![cfg(windows)]

use kx_platform_windows::clip::open;
use kx_platform_windows::uia::{self, Uia};
use serde_json::{Value, json};

use crate::apps::{Ctx, Opened};
use crate::g21;
use crate::g23::page::{self, Counts};
use crate::g23::panel::{Panel, fill_key};
use crate::ownkeys::{self, Real};
use crate::p9cfg::G23;
use crate::win::{self, sleep_ms};

/// A fill G23 runs: its row button, the field clicked first, and whether the user and the password arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// The user, a Tab and the password, from the user field.
    Both,
    /// The password alone, from the password field.
    Password,
    /// The user alone, from the user field.
    User,
}

/// What a fill must leave: the test user in the user field or nothing, and the password's length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Want {
    /// The user field holds the test user; else it stays empty.
    pub user: bool,
    /// The password field's length, in UTF-16 units.
    pub length: usize,
}

/// What a fill left on its page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Left {
    /// The user field holds exactly the test user.
    pub user_is_test: bool,
    /// The user field's characters; none when it was not read.
    pub user_chars: Option<usize>,
    /// The page's counts; none when its title lost them.
    pub counts: Option<Counts>,
    /// The clipboard's sequence number stayed as it was before the click.
    pub clip_same: bool,
    /// The panel still shows after the fill.
    pub open: bool,
}

impl Fill {
    /// Its name in the result line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::Password => "password",
            Self::User => "user",
        }
    }

    /// Its row button's id.
    fn button(self, g: &G23) -> &str {
        match self {
            Self::Both => &g.both,
            Self::Password => &g.password_button,
            Self::User => &g.user_button,
        }
    }

    /// What it must leave.
    pub fn want(self, g: &G23) -> Want {
        let user = self != Self::Password;
        let length = if self == Self::User {
            0
        } else {
            g.password.encode_utf16().count()
        };
        Want { user, length }
    }
}

/// True when the user field holds what `want` asks: the test user, or nothing.
fn user_ok(left: &Left, want: &Want) -> bool {
    if want.user {
        left.user_is_test
    } else {
        left.user_chars == Some(0)
    }
}

/// True once the wanted text has arrived, whatever else happened.
pub fn arrived(left: &Left, want: &Want) -> bool {
    user_ok(left, want) && left.counts.map(|c| c.length) == Some(want.length)
}

/// The fill's verdict line: it passes only when the text arrived, the clipboard stayed, no Enter or submit happened
/// and the panel stayed open. The test values never go in it.
pub fn judge(left: &Left, want: &Want) -> Value {
    let c = left.counts;
    let (enters, submits) = (c.map(|c| c.enters), c.map(|c| c.submits));
    let quiet = enters == Some(0) && submits == Some(0);
    let ok = arrived(left, want) && quiet && left.clip_same && left.open;
    json!({
        "user_ok": user_ok(left, want), "user_chars": left.user_chars, "password_length": c.map(|c| c.length),
        "length_wanted": want.length, "enters": enters, "submits": submits, "clipboard_same": left.clip_same,
        "panel_open": left.open, "ok": ok,
    })
}

/// What `app`'s page shows now, with the clipboard's sequence number `before` the click and the panel `open`.
fn read(ctx: &Ctx, uia: &Uia, app: &Opened, before: u32, open: bool) -> Left {
    let g = &ctx.cfg.g23;
    let el = uia.named(app.hwnd, &ctx.cfg.g21.user_name);
    let user = el.and_then(|e| uia::value(&e, ctx.cfg.g1.max_read_chars));
    Left {
        user_is_test: user.as_deref() == Some(g.user.as_str()),
        user_chars: user.map(|u| u.chars().count()),
        counts: page::counts(&win::title(app.hwnd), g.marks()),
        clip_same: open::sequence() == before,
        open,
    }
}

/// Runs `fill` on `app`'s fresh page: its field, the `fill` key unless the panel stayed open from the last fill, the
/// first row's button, then the page read once the text arrived or the wait ran out, after `[g23] late_ms` for a part
/// sent after a Tab. A failed panel read ends it.
pub fn run(ctx: &Ctx, panel: &Panel, uia: &Uia, app: &Opened, fill: Fill) -> Result<Value, String> {
    let (g, f) = (&ctx.cfg.g23, &ctx.cfg.g21);
    let field = if fill == Fill::Password {
        &f.pass_name
    } else {
        &f.user_name
    };
    g21::click_field(ctx, uia, app, field)?;
    fill_key(app, panel, true)?;
    let row = panel.first_row()?;
    let before = open::sequence();
    let id = g.row_button(0, fill.button(g));
    ownkeys::click(&Real, app, &|| panel.click(&id))?;
    let want = fill.want(g);
    let poll = ctx.cfg.timing.poll_ms;
    let now = || arrived(&read(ctx, uia, app, before, true), &want).then_some(());
    let came = win::poll_until(g.fill_wait_ms, poll, now).is_some();
    sleep_ms(g.late_ms);
    let open = panel.window()?.is_some();
    let mut v = judge(&read(ctx, uia, app, before, open), &want);
    let row_ok = row.as_deref() == Some(g.name.as_str());
    v["row_ok"] = json!(row_ok);
    v["arrived"] = json!(came);
    v["ok"] = json!(v["ok"] == true && row_ok);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WANT_BOTH: Want = Want {
        user: true,
        length: 12,
    };

    /// What a good Both leaves.
    fn good() -> Left {
        Left {
            user_is_test: true,
            user_chars: Some(10),
            counts: Some(count(12, 0, 0)),
            clip_same: true,
            open: true,
        }
    }

    #[test]
    fn a_fill_passes_when_every_check_does() {
        assert_eq!(judge(&good(), &WANT_BOTH)["ok"], true);
        assert!(arrived(&good(), &WANT_BOTH));
    }

    /// Counts read from a title.
    fn count(length: usize, enters: usize, submits: usize) -> Counts {
        Counts {
            length,
            enters,
            submits,
        }
    }

    #[test]
    fn a_fill_fails_when_any_check_fails() {
        let spoils: [fn(&mut Left); 7] = [
            |l| l.user_is_test = false,
            |l| l.counts = None,
            |l| l.counts = Some(count(11, 0, 0)),
            |l| l.counts = Some(count(12, 1, 0)),
            |l| l.counts = Some(count(12, 0, 1)),
            |l| l.clip_same = false,
            |l| l.open = false,
        ];
        for (i, spoil) in spoils.iter().enumerate() {
            let mut left = good();
            spoil(&mut left);
            assert_eq!(judge(&left, &WANT_BOTH)["ok"], false, "case {i}");
        }
    }

    #[test]
    fn a_one_part_fill_wants_the_other_field_left_empty() {
        let password = Want {
            user: false,
            length: 12,
        };
        let mut left = good();
        assert_eq!(judge(&left, &password)["ok"], false, "the user came too");
        left.user_is_test = false;
        left.user_chars = Some(0);
        assert_eq!(judge(&left, &password)["ok"], true);
        assert!(arrived(&left, &password));
        let user = Want {
            user: true,
            length: 0,
        };
        assert_eq!(judge(&good(), &user)["ok"], false, "the password came too");
        assert!(!arrived(&left, &user), "the user has not come");
    }

    #[test]
    fn enters_submits_and_the_clipboard_never_hold_up_the_arrival() {
        let mut left = good();
        left.counts = Some(count(12, 1, 1));
        left.clip_same = false;
        left.open = false;
        assert!(arrived(&left, &WANT_BOTH));
    }

    #[test]
    fn each_fill_wants_its_own_parts() {
        let g = crate::config::load().expect("loads").g23;
        let len = g.password.encode_utf16().count();
        assert_eq!(
            Fill::Both.want(&g),
            Want {
                user: true,
                length: len
            }
        );
        assert_eq!(
            Fill::Password.want(&g),
            Want {
                user: false,
                length: len
            }
        );
        assert_eq!(
            Fill::User.want(&g),
            Want {
                user: true,
                length: 0
            }
        );
        assert_eq!(Fill::Both.button(&g), g.both);
    }
}
