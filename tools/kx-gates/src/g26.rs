//! G26 other languages (roadmap G26; the Japanese IME part moved to F16): in each `[g26]` language, every character key
//! is listed and named after the layout target-window types in, and G2's clicks arrive intact. It switches the
//! session's input language through `kx-target-window --layout`, so it runs on GitHub's Windows runner only.
#![cfg(windows)]

use std::time::Instant;

use kx_module_api::ports::LayoutId;
use kx_platform_windows::WinLayouts;
use serde_json::{Value, json};

use crate::apps::{self, Ctx, Opened};
use crate::clicks::Run;
use crate::keymap::{KeyRead, Reader};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{capture, g2, launch, out, tlog, walk};

/// The language part of tag `tag`, such as `de` in `de-DE`, which layouts are matched by.
fn language(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// True when target-window closed by itself and its `log`, if read, shows no `# layout-kept`: the layout came back.
fn put_back(g2: &Value, log: Option<&str>) -> bool {
    let closed = g2["left_open"].as_array().is_some_and(Vec::is_empty);
    closed && log.is_some_and(|l| !tlog::layout_kept(l))
}

/// True when every language was found, named right, typed intact and put back.
fn passed(langs: &[Value]) -> bool {
    !langs.is_empty() && langs.iter().all(|l| l["pass"] == true)
}

/// Character keys named wrongly, then those UI Automation did not list.
type Labels = (Vec<String>, Vec<String>);

/// The character keys in `want` that UI Automation's `read` names wrongly, and those it does not list at all.
fn labels(read: &[KeyRead], want: &[walk::CharKey]) -> Labels {
    let ids: Vec<String> = want.iter().map(|c| c.id.clone()).collect();
    (walk::misnamed(read, want), walk::missing(read, &ids))
}

/// Calls `read` every `poll_ms` until every key is listed and named right, or `wait_ms` has passed; the last read
/// counts, so a failed read fails only when no later one came right.
fn settle(
    wait_ms: u64,
    poll_ms: u64,
    mut read: impl FnMut() -> Result<Labels, String>,
) -> Result<Labels, String> {
    // `poll_until` calls at least once, so the first read always replaces this.
    let mut last = Err(String::new());
    win::poll_until(wait_ms, poll_ms, || {
        last = read();
        let right = |(m, missing): &Labels| m.is_empty() && missing.is_empty();
        last.as_ref().is_ok_and(right).then_some(())
    });
    last
}

/// The character keys of keyboard `kb` not named after `layout` or not listed, from one follow wait until they are
/// right or `[g26] label_wait_ms` has passed, and the ms that took.
fn misnamed(ctx: &Ctx, kb: &Keyboard, layout: LayoutId) -> Result<(Labels, u64), String> {
    let t = Instant::now();
    sleep_ms(product::follow_ms(ctx.cfg.timing.poll_ms)?);
    let (reader, want) = (Reader::new()?, walk::char_keys(layout)?);
    let read = || Ok(labels(&reader.walk(kb.hwnd)?, &want));
    let got = settle(ctx.cfg.g26.label_wait_ms, ctx.cfg.timing.poll_ms, read)?;
    Ok((got, win::ms_since(t)))
}

/// Target-window, in front, in `layout`, its key labels and the ms they took; it is closed if reading them fails.
fn open_in(ctx: &Ctx, kb: &Keyboard, layout: LayoutId) -> Result<(Opened, (Labels, u64)), String> {
    let args = [ctx.target.layout_arg.clone(), layout.to_string()];
    let target = launch::to_front(ctx, launch::start_target_with(ctx, &args)?)?;
    match misnamed(ctx, kb, layout) {
        Ok(m) => Ok((target, m)),
        Err(e) => {
            apps::clean_up(ctx, target);
            Err(e)
        }
    }
}

/// One language: the labels, then G2's clicks, which close target-window and so put the layout back. The clicks
/// type each key once in board order first, so every key's text is checked.
fn one(ctx: &Ctx, kb: &Keyboard, tag: &str) -> Result<Value, String> {
    let layout = capture::find(&WinLayouts, language(tag))
        .ok_or_else(|| format!("no installed {tag} layout"))?;
    let (target, ((misnamed, missing), labels_ms)) = open_in(ctx, kb, layout)?;
    let lang_ctx = Ctx {
        out: ctx.out.clone(),
        base: format!("{}-{tag}", ctx.base),
        ..*ctx
    };
    let run = Run {
        count: ctx.cfg.g26.clicks,
        seed: out::clock_seed(),
        sweep: true,
    };
    let typed = g2::on_target(&lang_ctx, product::NAME, kb, target, run)?;
    let log = std::fs::read_to_string(&ctx.target.log).ok();
    let back = put_back(&typed, log.as_deref());
    let pass = misnamed.is_empty() && missing.is_empty() && g2::typed_intact(&typed) && back;
    Ok(json!({
        "language": tag, "layout": layout.to_string(), "misnamed": misnamed, "missing": missing, "labels_ms": labels_ms,
        "typed": typed, "layout_put_back": back, "pass": pass,
    }))
}

/// Runs G26 on the product, called `name`, for every language in `[g26] languages`; it waits for the keyboard first.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let g = &ctx.cfg.g26;
    let shown = win::poll_until(g.show_wait_ms, ctx.cfg.timing.poll_ms, || {
        Keyboard::find().ok()
    });
    let kb = shown.map_or_else(Keyboard::find, Ok)?;
    let each = |tag: &String| {
        one(ctx, &kb, tag).unwrap_or_else(|e| json!({ "language": tag, "error": e, "pass": false }))
    };
    let langs: Vec<Value> = g.languages.iter().map(each).collect();
    Ok(json!({ "gate": "G26", "keyboard": name, "languages": langs, "pass": passed(&langs) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_back_only_after_a_clean_close_and_a_log_without_the_mark() {
        let ok = json!({ "gate": "G2", "left_open": [] });
        assert!(put_back(&ok, Some("# start\t0\n10\t0041\n")));
        assert!(
            !put_back(&ok, Some("# start\t0\n# layout-kept\t20\n")),
            "the window said so"
        );
        assert!(!put_back(&ok, None), "an unread log proves nothing");
        let mut killed = ok.clone();
        killed["left_open"] = json!(["target-window did not close; killed it"]);
        assert!(
            !put_back(&killed, Some("# start\t0\n")),
            "a killed window could not put it back"
        );
    }

    #[test]
    fn a_character_key_ui_automation_does_not_list_is_named_as_missing() {
        use crate::keymap::KeyRead;
        use crate::walk::CharKey;
        use windows::Win32::Foundation::RECT;
        let read = [KeyRead {
            id: "c-0x0010".into(),
            name: "q".into(),
            rect: RECT::default(),
            invokable: true,
        }];
        let want = [
            CharKey::new("c-0x0010", "q", Some("q")),
            CharKey::new("c-0x0011", "w", Some("w")),
        ];
        let (misnamed, missing) = labels(&read, &want);
        assert!(misnamed.is_empty());
        assert_eq!(missing, ["c-0x0011"]);
    }

    /// Labels with `missing` keys not listed and none misnamed.
    fn with_missing(missing: &[&str]) -> Labels {
        (
            Vec::new(),
            missing.iter().map(ToString::to_string).collect(),
        )
    }

    #[test]
    fn the_labels_are_read_again_until_every_key_is_listed_and_named_right() {
        let mut reads = vec![
            with_missing(&[]),
            with_missing(&["c-0x0010"]),
            with_missing(&["c-0x0010", "c-0x0011"]),
        ];
        let mut calls = 0;
        let got = settle(60_000, 0, || {
            calls += 1;
            Ok(reads.pop().expect("a read is left"))
        });
        assert_eq!((got, calls), (Ok(with_missing(&[])), 3));
    }

    #[test]
    fn the_last_read_counts_when_the_labels_never_come_right_in_time() {
        let mut calls = 0;
        let got = settle(0, u64::MAX, || {
            calls += 1;
            Ok(with_missing(&["c-0x0010"]))
        });
        assert_eq!((got, calls), (Ok(with_missing(&["c-0x0010"])), 1));
    }

    #[test]
    fn a_failed_read_is_tried_again_and_fails_the_language_only_when_it_is_the_last() {
        let mut reads = vec![
            Ok(with_missing(&[])),
            Err("UI Automation: busy".to_string()),
        ];
        assert_eq!(
            settle(60_000, 0, || reads.pop().expect("a read is left")),
            Ok(with_missing(&[]))
        );
        let got = settle(0, u64::MAX, || Err("UI Automation: busy".to_string()));
        assert_eq!(got, Err("UI Automation: busy".to_string()));
    }

    #[test]
    fn g26_passes_only_when_every_language_passed() {
        let good = json!({ "language": "de-DE", "pass": true });
        let bad =
            json!({ "language": "fa-IR", "error": "no installed fa-IR layout", "pass": false });
        assert!(passed(&[good.clone(), good.clone()]));
        assert!(!passed(&[good, bad]));
        assert!(!passed(&[]));
    }

    #[test]
    fn layouts_are_matched_by_the_language_part_of_a_tag() {
        assert_eq!(language("fa-IR"), "fa");
        assert_eq!(language("he"), "he");
        assert_eq!(capture::find(&WinLayouts, language("zz-ZZ")), None);
    }
}
