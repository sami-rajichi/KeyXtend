//! The command-line help.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::apps::AppKind;
use crate::product::NAME;
use crate::{
    capture, g5, g6, g8, g8lit, g17, g18, g19, g20, g21, g22, g22bench, g22type, g25, hand,
};

/// The program's name in the help.
const PROG: &str = env!("CARGO_PKG_NAME");

/// App names joined for the usage text.
fn names(apps: &[AppKind]) -> String {
    apps.iter().map(|a| a.name()).collect::<Vec<_>>().join("|")
}

/// The help lines of the gates aimed at the product.
fn product_lines() -> String {
    let (weak, lit, osk) = (g8::WEAK, g8lit::LIT, g8lit::OSK);
    format!(
        "       {PROG} g2 <{NAME}> [--clicks N] [--seed S]
       {PROG} g3 <{NAME}>
       {PROG} g4 <{NAME}> [--clicks N] [--seed S]
       {PROG} g8 <{NAME}|{weak}|{lit}|{osk}>
       {PROG} g9 <{NAME}>
       {PROG} g10 <{NAME}>
       {PROG} g11 <{NAME}>
       {PROG} g13 <{NAME}>
       {PROG} g26 <{NAME}>"
    )
}

/// The help lines of the gates that still drive the frozen spike face.
fn face_lines() -> String {
    format!(
        "       {PROG} g7 <{FACE}>
       {PROG} g12 <{FACE}>
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

/// Command-line help; the G1 app list comes from `AppKind::ALL`.
pub fn usage() -> String {
    format!(
        "usage: {PROG} g1 <{}> [--count N] [--seed S] [--pause MS] [--attach]
{}
       {PROG} g5 <{}>
       {PROG} assist <{}|{}> [--secs N]
       {PROG} g17 <{}>
       {PROG} g18 <{}>
       {PROG} g6 <{}>
{}
       {PROG} g20 <{}>
       {PROG} g21 <{}>
       {PROG} g25 <{}>
       {PROG} {}
--attach types into the app's window already open; --pause sets the gap between characters.
{NAME} is the product keyboard; {FACE} is the frozen spike face; g8 {} starts the product capped like a weak PC.
g8 {} times the running product's key light only; g8 {} times the shown On-Screen Keyboard's, which it never starts.
g5 {} is the owner's manual check on the installed product, as the test build cannot reach admin windows.",
        names(&AppKind::ALL),
        product_lines(),
        g5::PLAIN,
        hand::RIGHT,
        hand::GRAB,
        names(&g17::APPS),
        names(&g18::APPS),
        names(&g6::APPS),
        face_lines(),
        names(&g20::APPS),
        g21::NAMES.join("|"),
        names(&g25::APPS),
        capture::COMMAND,
        g8::WEAK,
        g8lit::LIT,
        g8lit::OSK,
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
    fn g5_runs_plain_and_names_admin_as_the_owners_manual_check() {
        let text = usage();
        assert!(text.contains(&format!("g5 <{}>", g5::PLAIN)), "{text}");
        let admin = text
            .lines()
            .find(|l| l.contains(&format!("g5 {}", g5::ADMIN)));
        assert!(admin.is_some_and(|l| l.contains("manual")), "{text}");
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
}
