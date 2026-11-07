//! G23's Chrome login page: a user field and a password field in a form with a submit button. Its script keeps the
//! password's length, the Enter key-downs and the submits in the page title, each behind its mark from `[g23]`.
#![cfg(windows)]

use crate::apps::Ctx;
use crate::g21;

/// What the page's title counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// The password field's length, in UTF-16 units as the page counts.
    pub length: usize,
    /// Enter key-downs anywhere on the page.
    pub enters: usize,
    /// Form submits.
    pub submits: usize,
}

/// The page body, with `[g21]`'s fields as the old gate built them; every count starts at 0 at load.
pub fn page(ctx: &Ctx) -> String {
    let (f, g) = (&ctx.cfg.g21, &ctx.cfg.g23);
    let user = g21::input(ctx, &f.user_name, "text", "");
    let typed = "oninput=\"kx.len=this.value.length;kx.show()\"";
    let pass = g21::input(ctx, &f.pass_name, "password", typed);
    let [len, enter, sent] = g.marks();
    let key = &g.enter_key;
    format!(
        "<form onsubmit=\"kx.sent++;kx.show();return false\">{user}{pass}<input type=\"submit\"></form><script>\
         var kx={{base:document.title,len:0,enter:0,sent:0,show:function(){{document.title=kx.base+'{len}'+kx.len\
         +'{enter}'+kx.enter+'{sent}'+kx.sent}}}};addEventListener('keydown',function(e){{if(e.key==='{key}')\
         {{kx.enter++;kx.show()}}}},true);kx.show()</script>"
    )
}

/// The digits at the start of `s` as a number, and what follows them; none without a digit.
fn number(s: &str) -> Option<(usize, &str)> {
    let n = s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    Some((s.get(..n)?.parse().ok()?, &s[n..]))
}

/// The counts in window title `title`, read from the last length mark on, as Chrome ends the title with its own
/// name; none when a mark or its count is missing.
pub fn counts(title: &str, marks: [&str; 3]) -> Option<Counts> {
    let [len, enter, sent] = marks;
    let (_, rest) = title.rsplit_once(len)?;
    let (length, rest) = number(rest)?;
    let (enters, rest) = number(rest.strip_prefix(enter)?)?;
    let (submits, _) = number(rest.strip_prefix(sent)?)?;
    Some(Counts {
        length,
        enters,
        submits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKS: [&str; 3] = [" #", " ~", " ^"];

    /// Counts as a title gives them.
    fn read(length: usize, enters: usize, submits: usize) -> Counts {
        Counts {
            length,
            enters,
            submits,
        }
    }

    #[test]
    fn the_length_the_enters_and_the_submits_come_from_the_title() {
        assert_eq!(
            counts("KeyXtend probe 1 #12 ~0 ^0", MARKS),
            Some(read(12, 0, 0))
        );
        let chrome = "KeyXtend probe 1 #0 ~2 ^1 - Google Chrome";
        assert_eq!(
            counts(chrome, MARKS),
            Some(read(0, 2, 1)),
            "Chrome's own name after"
        );
        let early = "KeyXtend #probe #3 ~0 ^0";
        assert_eq!(
            counts(early, MARKS),
            Some(read(3, 0, 0)),
            "the last length mark counts"
        );
    }

    #[test]
    fn a_title_without_its_marks_or_counts_reads_as_none() {
        for title in [
            "KeyXtend probe 1 - Google Chrome",
            "KeyXtend probe 1 #12",
            "KeyXtend probe 1 #12 ~0",
            "KeyXtend probe 1 #x ~0 ^0",
            "KeyXtend probe 1 #12 ~ ^0",
            "KeyXtend probe 1 #12 ^0 ~0",
        ] {
            assert_eq!(counts(title, MARKS), None, "{title}");
        }
    }
}
