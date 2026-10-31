//! The command-line help.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::apps::AppKind;
use crate::product::NAME;
use crate::{
    capture, g5, g6, g6rec, g8, g8clip, g8hook, g8lang, g8lit, g8predict, g17, g18, g19, g20, g21,
    g22, g22bench, g22type, g25, hand, workertok,
};

/// The program's name in the help.
const PROG: &str = env!("CARGO_PKG_NAME");

/// App names joined for the usage text.
fn names(apps: &[AppKind]) -> String {
    apps.iter().map(|a| a.name()).collect::<Vec<_>>().join("|")
}

/// The help lines of the gates aimed at the product or its test build.
fn product_lines() -> String {
    let (weak, arabic, lit, osk) = (g8::WEAK, g8lang::ARABIC, g8lit::LIT, g8lit::OSK);
    let (hook, hook_weak, predict) = (g8hook::HOOK, g8hook::HOOK_WEAK, g8predict::PREDICT);
    let (apps, records, clip) = (names(&g6::APPS), g6rec::NAMES.join("|"), g8clip::CLIP);
    format!(
        "       {PROG} g2 <{NAME}> [--clicks N] [--seed S]
       {PROG} g3 <{NAME}>
       {PROG} g4 <{NAME}> [--clicks N] [--seed S]
       {PROG} g6 <{apps}|{records}>
       {PROG} g8 <{NAME}|{weak}|{arabic}|{lit}|{osk}|{hook}|{hook_weak}|{clip}|{predict}>
       {PROG} g9 <{NAME}>
       {PROG} g10 <{NAME}>
       {PROG} g11 <{NAME}>
       {PROG} g12 <{NAME}>
       {PROG} g13 <{NAME}>
       {PROG} g26 <{NAME}>
       {PROG} g27 <{NAME}>"
    )
}

/// The help lines of the gates that still drive the frozen spike face.
fn face_lines() -> String {
    format!(
        "       {PROG} g7 <{FACE}>
       {PROG} g19 <{}|{FACE}>
       {PROG} g22 <{FACE}|{}|{}|{}>
       {PROG} g23 <{FACE}>
       {PROG} g24 <{FACE}>
       {PROG} close <{NAME}|{FACE}>",
        g19::CORE,
        g22::BENCH,
        g22bench::WITH_CLOUD,
        g22type::TYPING,
    )
}

/// The help lines of P7's gates that type through the product's chips.
fn typing_lines() -> String {
    let predict = g8predict::PREDICT;
    format!(
        "g8 {predict} reads the predict part's memory from two cold starts and types words through chips; the owner's \
         settings come back.
g21 {NAME} and g27 {NAME} start their own copy with learning paused, so close the product first; they type \
         through its keys and chips into target-window and Chrome, and the owner's settings come back."
    )
}

/// The help lines of P8's clipboard gates.
fn clip_lines() -> String {
    let (clip, worker) = (g8clip::CLIP, workertok::WORKER);
    format!(
        "g20 {NAME} and g8 {clip} start the test build on its own data folder from the plain kx-gates (cargo run -p \
         kx-gates), so close the product first; they overwrite the clipboard and Windows' clipboard history.
g20 {worker} only reads the tokens of the running product's workers: start the dev install first."
    )
}

/// The help lines of G8's parts on a layout, a key light or the hook.
fn g8_notes() -> String {
    let (arabic, lit, osk) = (g8lang::ARABIC, g8lit::LIT, g8lit::OSK);
    let (hook, hook_weak) = (g8hook::HOOK, g8hook::HOOK_WEAK);
    format!(
        "g8 {arabic} puts an Arabic layout in front before the cold start and the owner's layout back after.
g8 {lit} times the running product's key light only; g8 {osk} times the shown On-Screen Keyboard's, which it never starts.
g8 {hook} and {hook_weak} judge the hook's p99 and lost echoes on the product's test build, driven with G5's cases; the second in weak mode."
    )
}

/// Command-line help; the G1 app list comes from `AppKind::ALL`.
pub fn usage() -> String {
    format!(
        "usage: {PROG} g1 <{}> [--count N] [--seed S] [--pause MS] [--attach]
{}
       {PROG} g5 <{}>
       {PROG} assist <{}|{}> [--secs N]
       {PROG} g17 <{}>
       {PROG} g18 <{}>
{}
       {PROG} g20 <{}|{NAME}|{}>
       {PROG} g21 <{}>
       {PROG} g25 <{}>
       {PROG} {}
--attach types into the app's window already open; --pause sets the gap between characters.
{NAME} is the product keyboard; {FACE} is the frozen spike face; g8 {} starts the product capped like a weak PC.
{}
{}
{}
g5 {} is the owner's manual check on the installed product, as the test build cannot reach admin windows.",
        names(&AppKind::ALL),
        product_lines(),
        g5::PLAIN,
        hand::RIGHT,
        hand::GRAB,
        names(&g17::APPS),
        names(&g18::APPS),
        face_lines(),
        names(&g20::APPS),
        workertok::WORKER,
        g21::NAMES.join("|"),
        names(&g25::APPS),
        capture::COMMAND,
        g8::WEAK,
        g8_notes(),
        typing_lines(),
        clip_lines(),
        g5::ADMIN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_lists_every_g1_app() {
        let names: Vec<&str> = AppKind::ALL.iter().map(|a| a.name()).collect();
        assert!(usage().contains(&format!("g1 <{}>", names.join("|"))));
    }

    #[test]
    fn g6_lists_its_apps_and_its_records_but_never_word() {
        let text = usage();
        let line = text
            .lines()
            .find(|l| l.contains("g6 <"))
            .unwrap_or_default();
        let apps = g6::APPS.iter().map(|a| a.name());
        for name in apps.chain(g6rec::NAMES) {
            assert!(line.contains(name), "{text}");
        }
        assert!(!line.contains(AppKind::Word.name()), "{line}");
    }

    #[test]
    fn g5_runs_plain_and_names_admin_as_the_owners_manual_check() {
        let text = usage();
        assert!(text.contains(&format!("g5 <{}>", g5::PLAIN)), "{text}");
        let admin = text
            .lines()
            .find(|l| l.contains(&format!("g5 {}", g5::ADMIN)));
        assert!(admin.is_some_and(|l| l.contains("manual")), "{text}");
    }

    #[test]
    fn g12_runs_on_the_product_and_g8_lists_its_hook_names() {
        let text = usage();
        assert!(text.contains(&format!("g12 <{NAME}>")), "{text}");
        let (hook, weak) = (g8hook::HOOK, g8hook::HOOK_WEAK);
        let g8 = format!("|{hook}|{weak}|");
        assert!(
            text.lines().any(|l| l.contains("g8 <") && l.contains(&g8)),
            "{text}"
        );
        let told = text.lines().find(|l| l.contains(&format!("g8 {hook}")));
        assert!(
            told.is_some_and(|l| l.contains(weak) && l.contains("test build")),
            "{text}"
        );
        let arabic = text
            .lines()
            .find(|l| l.contains(&format!("g8 {}", g8lang::ARABIC)));
        assert!(arabic.is_some_and(|l| l.contains("back")), "{text}");
    }

    #[test]
    fn p7_lists_g27_g21s_product_half_and_g8_predict() {
        let text = usage();
        assert!(text.contains(&format!("g27 <{NAME}>")), "{text}");
        assert!(
            text.contains(&format!("g21 <{}>", g21::NAMES.join("|"))),
            "{text}"
        );
        assert!(g21::NAMES.contains(&NAME), "G21's product half");
        let predict = format!("|{}>", g8predict::PREDICT);
        assert!(
            text.lines()
                .any(|l| l.contains("g8 <") && l.contains(&predict)),
            "{text}"
        );
        let told = text
            .lines()
            .find(|l| l.contains(&format!("g8 {}", g8predict::PREDICT)));
        assert!(
            told.is_some_and(|l| l.contains("settings come back")),
            "{text}"
        );
        let own = text.lines().find(|l| l.starts_with(&format!("g21 {NAME}")));
        assert!(
            own.is_some_and(|l| l.contains("close the product first")),
            "{text}"
        );
    }

    #[test]
    fn every_command_line_names_kx_gates() {
        let text = usage();
        assert!(!text.contains("harness"), "{text}");
        assert!(text.starts_with("usage: kx-gates g1 "), "{text}");
        let lines = text.lines().filter(|l| l.contains('<'));
        assert!(lines.clone().count() > 1);
        for line in lines {
            assert!(line.contains("kx-gates "), "{line}");
        }
    }

    #[test]
    fn p8_lists_g20_on_the_product_and_g8_clip_and_warns_of_the_clipboard() {
        let text = usage();
        let g20 = format!("g20 <{}|{NAME}|{}>", names(&g20::APPS), workertok::WORKER);
        assert!(text.contains(&g20), "{text}");
        let clip = format!("|{}|", g8clip::CLIP);
        assert!(
            text.lines()
                .any(|l| l.contains("g8 <") && l.contains(&clip)),
            "{text}"
        );
        let told = text.lines().find(|l| l.starts_with(&format!("g20 {NAME}")));
        let warns = |l: &str| {
            l.contains("overwrite the clipboard") && l.contains("close the product first")
        };
        assert!(told.is_some_and(warns), "{text}");
        let worker = text
            .lines()
            .find(|l| l.starts_with(&format!("g20 {}", workertok::WORKER)));
        assert!(worker.is_some_and(|l| l.contains("only reads")), "{text}");
    }
}
