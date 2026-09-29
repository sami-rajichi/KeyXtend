//! The command-line help.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::apps::AppKind;
use crate::{g5, g6, g17, g18, g19, g20, g21, g22, g22bench, g22type, g25, hand};

/// The program's name in the help.
const PROG: &str = env!("CARGO_PKG_NAME");

/// App names joined for the usage text.
fn names(apps: &[AppKind]) -> String {
    apps.iter().map(|a| a.name()).collect::<Vec<_>>().join("|")
}

/// Command-line help; the G1 app list comes from `AppKind::ALL`.
pub fn usage() -> String {
    format!(
        "usage: {PROG} g1 <{}> [--count N] [--seed S] [--pause MS] [--attach]
       {PROG} g2 <{FACE}> [--clicks N] [--seed S]
       {PROG} g3 <{FACE}>
       {PROG} g4 <{FACE}> [--clicks N] [--seed S]
       {PROG} g5 <{}|{}>
       {PROG} assist <{}|{}> [--secs N]
       {PROG} g17 <{}>
       {PROG} g18 <{}>
       {PROG} g6 <{}>
       {PROG} g7 <{FACE}>
       {PROG} g12 <{FACE}>
       {PROG} g19 <{}|{FACE}>
       {PROG} g20 <{}>
       {PROG} g21 <{}>
       {PROG} g22 <{FACE}|{}|{}|{}>
       {PROG} g23 <{FACE}>
       {PROG} g24 <{FACE}>
       {PROG} g25 <{}>
       {PROG} close <{FACE}>
--attach types into the app's window already open; --pause sets the gap between characters.",
        names(&AppKind::ALL),
        g5::PLAIN,
        g5::ADMIN,
        hand::RIGHT,
        hand::GRAB,
        names(&g17::APPS),
        names(&g18::APPS),
        names(&g6::APPS),
        g19::CORE,
        names(&g20::APPS),
        g21::NAMES.join("|"),
        g22::BENCH,
        g22bench::WITH_CLOUD,
        g22type::TYPING,
        names(&g25::APPS),
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
