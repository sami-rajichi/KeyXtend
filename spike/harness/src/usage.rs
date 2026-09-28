//! The harness's command-line help.

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
       harness g2 <slint|qt|tauri> [--clicks N] [--seed S]
       harness g3 <slint|qt|tauri>
       harness g4 <slint|qt|tauri> [--clicks N] [--seed S]
       harness g5 <{}|{}>
       harness assist <{}|{}> [--secs N]
       harness g17 <{}>
       harness g18 <{}>
       harness g6 <{}>
       harness g7 <qt>
       harness g12 <qt>
       harness g19 <{}|slint|qt>
       harness g20 <{}>
       harness g21 <{}>
       harness g22 <slint|qt|{}|{}|{}>
       harness g23 <slint|qt>
       harness g24 <slint|qt>
       harness g25 <{}>
       harness close <slint|qt>
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
