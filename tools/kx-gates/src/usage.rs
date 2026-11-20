//! The command-line help.
#![cfg(windows)]

use spike_core::config::FACE;

use crate::apps::AppKind;
use crate::product::NAME;
use crate::{
    capture, g5, g6, g6rec, g8, g8clip, g8hook, g8lang, g8lit, g8predict, g17, g18, g19, g19chips,
    g20, g21, g22, g25, hand, workertok,
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
    let (bench, staged) = (g22::BENCH, g22::STAGED);
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
       {PROG} g22 <{NAME}|{bench}|{staged}>
       {PROG} g23 <{NAME}>
       {PROG} g26 <{NAME}>
       {PROG} g27 <{NAME}>"
    )
}

/// The help lines of the gates that still drive the frozen spike face.
fn face_lines() -> String {
    format!(
        "       {PROG} g7 <{FACE}>
       {PROG} g24 <{FACE}>
       {PROG} close <{NAME}|{FACE}>",
    )
}

/// The help lines of P11's voice gates.
fn voice_lines() -> String {
    let (bench, staged, build) = (g22::BENCH, g22::STAGED, g22::worker_build());
    format!(
        "g22 {NAME} starts the build with --features gate-voice beside the gate worker on a data folder it makes, so \
         close the product first; it dictates each shipped clip into target-window by clicks and prints numbers and \
         the id of the device that wrote each text.
g22 {bench} starts no screen and moves nothing: each language's default voice model reads its shipped clip through \
         the worker built with {build}, normal then weak, and it prints a table of numbers.
g22 {staged} starts no screen and moves nothing: it copies the worker cargo xtask stage wrote, checks edges 28, 29 \
         and 39 on it, then deletes every copy.
g22 {NAME} and g22 {bench} pass the sound flag, so they run only from kx-gates built with --features voice, never \
         the signed one."
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
        "g20 {NAME}, g8 {clip} and g25 start the test build on its own data folder from the plain kx-gates (cargo run -p \
         kx-gates), so close the product first; they overwrite the clipboard and Windows' clipboard history.
g20 {worker} only reads the tokens of the running product's workers: start the dev install first.
g23 {NAME} starts the test build on a data folder it makes, saves a test login through Quick-fill and fills Chrome \
         login pages with it, so close the product first; the folder goes at the end."
    )
}

/// The help lines of P10's selection strip gate.
fn sel_lines() -> String {
    let (read, beside, terms) = (g19chips::READ, g19chips::BESIDE, g19chips::TERMINALS);
    let (look, hung) = (g19chips::LOOK, g19chips::HUNG);
    format!(
        "g19 {NAME} starts the test build on a data folder it makes and drives the selection strip in its own Notepad, \
         Chrome page, target-window and terminals, so close the product first; the folder goes at the end. It \
         overwrites the clipboard and Windows' clipboard history, then puts the clipboard back and forgets its copies.
g19 {terms} does the same in the terminals alone, in about a minute, so close the product first.
g19 {hung} does the same for the hung app alone and notes the z-order, so close the product first.
g19 {read} starts the test build the same way and selects the long Arabic line: click Translate, then Yes; the gate \
         only reads the page that opens.
g19 {beside} starts nothing: it does the same beside the uiAccess dev install, and runs only from the installed \
         uiAccess kx-gates, as Windows hides that keyboard from a plain one.
g19 {look} starts nothing and moves nothing: it runs the product's selection look on the focused box and prints each \
         change, never text."
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
       {PROG} g19 <{}>
{}
       {PROG} g20 <{}|{NAME}|{}>
       {PROG} g21 <{}>
       {PROG} g25 <{}>
       {PROG} {}
--attach types into the app's window already open; --pause sets the gap between characters.
{NAME} is the product keyboard; {FACE} is the frozen spike face; g8 {} starts the product capped like a weak PC.
{}
g5 {} is the owner's manual check on the installed product, as the test build cannot reach admin windows.",
        names(&AppKind::ALL),
        product_lines(),
        g5::PLAIN,
        hand::RIGHT,
        hand::GRAB,
        names(&g17::APPS),
        names(&g18::APPS),
        g19::NAMES.join("|"),
        face_lines(),
        names(&g20::APPS),
        workertok::WORKER,
        g21::NAMES.join("|"),
        names(&g25::APPS),
        capture::COMMAND,
        g8::WEAK,
        notes(),
        g5::ADMIN,
    )
}

/// The help's notes on each phase's gates, in order.
fn notes() -> String {
    [
        g8_notes(),
        typing_lines(),
        clip_lines(),
        sel_lines(),
        voice_lines(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests;
