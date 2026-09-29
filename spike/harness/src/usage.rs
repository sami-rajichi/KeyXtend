//! The harness's command-line help.

use spike_core::config::FACE;

use crate::apps::AppKind;
use crate::{g5, g6, g17, g18, g19, g20, g21, g22, g22bench, g22type, g25, hand};

/// App names joined for the usage text.
fn names(apps: &[AppKind]) -> String {
    apps.iter().map(|a| a.name()).collect::<Vec<_>>().join("|")
}

/// Command-line help; the G1 app list comes from `AppKind::ALL`.
pub fn usage() -> String {
    format!(
        "usage: harness g1 <{}> [--count N] [--seed S] [--pause MS] [--attach]
       harness g2 <{FACE}> [--clicks N] [--seed S]
       harness g3 <{FACE}>
       harness g4 <{FACE}> [--clicks N] [--seed S]
       harness g5 <{}|{}>
       harness assist <{}|{}> [--secs N]
       harness g17 <{}>
       harness g18 <{}>
       harness g6 <{}>
       harness g7 <{FACE}>
       harness g12 <{FACE}>
       harness g19 <{}|{FACE}>
       harness g20 <{}>
       harness g21 <{}>
       harness g22 <{FACE}|{}|{}|{}>
       harness g23 <{FACE}>
       harness g24 <{FACE}>
       harness g25 <{}>
       harness close <{FACE}>
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
