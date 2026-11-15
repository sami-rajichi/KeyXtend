//! Tests of `usage`.

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
fn g19_lists_its_seven_runs_and_asks_to_close_the_product_first() {
    let text = usage();
    let (read, beside, terms) = (g19chips::READ, g19chips::BESIDE, g19chips::TERMINALS);
    let (look, hung, core) = (g19chips::LOOK, g19chips::HUNG, g19::CORE);
    let runs = format!("g19 <{core}|{NAME}|{read}|{beside}|{terms}|{hung}|{look}>");
    assert!(text.contains(&runs), "{text}");
    let told = |run: &str| text.lines().find(|l| l.starts_with(&format!("g19 {run} ")));
    for run in [beside, look] {
        let free = told(run).is_some_and(|l| l.contains("starts nothing"));
        assert!(free, "{run}: {text}");
    }
    for run in [NAME, terms, hung] {
        let asks = told(run).is_some_and(|l| l.contains("close the product first"));
        assert!(asks, "{run}: {text}");
    }
}

#[test]
fn p9_lists_g23_on_the_product_only_and_asks_to_close_it_first() {
    let text = usage();
    assert!(text.contains(&format!("g23 <{NAME}>")), "{text}");
    assert!(!text.contains(&format!("g23 <{FACE}>")), "{text}");
    let told = text.lines().find(|l| l.starts_with(&format!("g23 {NAME}")));
    assert!(
        told.is_some_and(|l| l.contains("close the product first")),
        "{text}"
    );
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
    let starts = format!("g20 {NAME}, g8 {} and g25 start", g8clip::CLIP);
    let told = text.lines().find(|l| l.starts_with(&starts));
    let warns =
        |l: &str| l.contains("overwrite the clipboard") && l.contains("close the product first");
    assert!(told.is_some_and(warns), "{text}");
    let worker = text
        .lines()
        .find(|l| l.starts_with(&format!("g20 {}", workertok::WORKER)));
    assert!(worker.is_some_and(|l| l.contains("only reads")), "{text}");
}

#[test]
fn p11_lists_the_bench_which_moves_nothing_and_names_the_gate_worker() {
    let text = usage();
    let runs = format!("g22 <{FACE}|{}|{}>", g22::BENCH, g22type::TYPING);
    assert!(text.contains(&runs), "{text}");
    let told = text
        .lines()
        .find(|l| l.starts_with(&format!("g22 {} ", g22::BENCH)));
    let says = |l: &str| l.contains("moves nothing") && l.contains("--features gate");
    assert!(told.is_some_and(says), "{text}");
}
