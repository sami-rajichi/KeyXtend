//! Prediction over the whole app: predict watches the box, reads the sealed store and opens the language in front
//! whichever of it and layouts starts first, and asks the keyboard that started first for its typing mode; Quit ends
//! its watch, and switched off it touches nothing. The packs are read beside the program, and in a debug build where
//! `cargo xtask words` writes them.

use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::{Boot, policy};
use keyxtend::drain;
use keyxtend::words::pack_dirs;
use kx_kernel::Kernel;
use kx_mod_layouts::{LayoutsModule, LayoutsSettings, label};
use kx_mod_predict::PredictModule;
use kx_module_api::ModuleState::Active;
use kx_module_api::ports::{
    FieldWatchKey, LayoutChanged, LayoutSource, LayoutSourceKey, PACK_DIR, PACK_INDEX,
    SealedStoreKey, TypingMode, TypingModeAsk, WordPacksKey,
};
use kx_module_api::{Bus, Event, Module, Notice, NoticeArg, Subscription, args, keys};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

/// The layout in front, as layouts tells it: its language's code and name.
fn front(rig: &Rig) -> LayoutChanged {
    let short = LayoutsSettings::from_defaults().unwrap().short;
    let id = rig.layouts.foreground().unwrap();
    LayoutChanged::from(&label(id, &rig.layouts.facts(id), &short))
}

/// True once predict tells, by `front`'s name, that its language, which the build shipped, has no pack: it opened it.
fn told_missing(notices: &Receiver<Notice>, front: &LayoutChanged) -> bool {
    let lang = (args::LANG, NoticeArg::Text(front.name.clone()));
    let missing = |n: &Notice| n.key == keys::WORDS_MISSING && n.args.contains(&lang);
    while let Ok(notice) = notices.recv_timeout(WAIT) {
        if missing(&notice) {
            return true;
        }
    }
    false
}

/// A loaded kernel with only `modules`, added in that order, over the rig's layouts and predict's three services.
fn kernel(rig: &Rig, modules: Vec<Box<dyn Module>>) -> Kernel {
    let mut kernel = Kernel::new(rig.platform(), policy(&modules));
    kernel
        .provide_platform::<LayoutSourceKey>(rig.layouts.clone())
        .unwrap();
    kernel
        .provide_platform::<FieldWatchKey>(rig.field.clone())
        .unwrap();
    kernel
        .provide_platform::<WordPacksKey>(rig.packs.clone())
        .unwrap();
    kernel
        .provide_platform::<SealedStoreKey>(rig.sealed.clone())
        .unwrap();
    for module in modules {
        kernel.add(module).unwrap();
    }
    kernel.load().unwrap();
    kernel
}

#[test]
fn predict_starts_with_the_box_watch_and_the_sealed_store_and_quit_ends_its_watch() {
    let rig = Rig::new();
    let shown = front(&rig);
    rig.packs.ship(&shown.iso);
    let mut boot = rig.load();
    let (_ear, notices) = hear::<Notice>(&boot.kernel.bus());
    boot.start().unwrap();
    assert_eq!(boot.kernel.state(kx_mod_predict::ID), Some(Active));
    assert!(rig.field.watching(), "it watches the box");
    assert!(told_missing(&notices, &shown), "it opened {}", shown.iso);
    assert!(rig.sealed.loads() > 0, "it read the sealed store");
    let Boot {
        kernel,
        hub,
        requests,
        heard,
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests).unwrap();
    assert!(hub.quit());
    thread.join().unwrap();
    assert!(!rig.field.watching(), "Quit ended the watch");
    drop(heard);
}

#[test]
fn predict_opens_the_language_in_front_whether_it_starts_before_or_after_layouts() {
    for predict_first in [false, true] {
        let mut modules: Vec<Box<dyn Module>> =
            vec![Box::new(LayoutsModule), Box::new(PredictModule::default())];
        if predict_first {
            modules.reverse();
        }
        let rig = Rig::new();
        let shown = front(&rig);
        rig.packs.ship(&shown.iso);
        let mut kernel = kernel(&rig, modules);
        let (_ear, notices) = hear::<Notice>(&kernel.bus());
        kernel.start().unwrap();
        let why = format!("predict first: {predict_first}");
        assert!(
            told_missing(&notices, &shown),
            "{why}: it opened {}",
            shown.iso
        );
        // Dropping the kernel stops every module, so a late second open is counted too.
        drop(kernel);
        assert_eq!(rig.packs.opens(&shown.iso), 1, "{why}: once");
    }
}

#[test]
fn with_predict_switched_off_nothing_watches_the_box_or_opens_a_pack() {
    let rig = Rig::new();
    let section = format!("[kernel]\ndisabled = [\"{}\"]\n", kx_mod_predict::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert_ne!(boot.kernel.state(kx_mod_predict::ID), Some(Active));
    let keyboard = boot.kernel.state(kx_mod_keyboard::ID);
    assert_eq!(keyboard, Some(Active), "the others run");
    assert_eq!(rig.field.watches(), 0, "no box watch");
    assert_eq!(rig.packs.opens(&front(&rig).iso), 0, "no pack");
    assert_eq!(rig.sealed.loads(), 0, "no sealed file");
    let thread = drain::spawn(boot.kernel, Arc::clone(&boot.hub), boot.requests).unwrap();
    assert!(boot.hub.quit(), "Quit still reaches the kernel's thread");
    thread.join().unwrap();
}

#[test]
fn packs_are_read_from_the_words_folder_beside_the_program_first() {
    let exe = Path::new("app");
    let dirs = pack_dirs(exe);
    assert_eq!(dirs.first(), Some(&exe.join(PACK_DIR)));
    let debug = usize::from(cfg!(debug_assertions));
    assert_eq!(
        dirs.len(),
        1 + debug,
        "only a debug build reads a second folder"
    );
}

#[cfg(debug_assertions)]
#[test]
fn a_debug_build_also_reads_the_packs_in_the_workspace_assets() {
    let dirs = pack_dirs(Path::new("app"));
    let want = crate::rig::words_out();
    let real = |p: &Path| std::fs::canonicalize(p).unwrap();
    assert_eq!(dirs.get(1).map(|d| real(d)), Some(real(&want)));
    assert!(
        want.join(PACK_INDEX).is_file(),
        "the build's index is there"
    );
}

/// Each `TypingModeAsk` and `TypingMode` on `bus`, by name, in order.
fn modes(bus: &Bus) -> ([Subscription; 2], Receiver<&'static str>) {
    let (tx, rx) = mpsc::channel();
    let asked = tx.clone();
    let ask = bus.subscribe(move |_: &TypingModeAsk| {
        let _ = asked.send(TypingModeAsk::NAME);
    });
    let mode = bus.subscribe(move |_: &TypingMode| {
        let _ = tx.send(TypingMode::NAME);
    });
    ([ask.unwrap(), mode.unwrap()], rx)
}

#[test]
fn predict_asks_the_keyboard_that_started_first_for_the_typing_mode_and_hears_it() {
    let rig = Rig::new();
    let mut boot = rig.load();
    let (_ears, heard) = modes(&boot.kernel.bus());
    boot.start().unwrap();
    let (mut log, mut asked) = (Vec::new(), false);
    while let Ok(name) = heard.recv_timeout(WAIT) {
        log.push(name);
        asked |= name == TypingModeAsk::NAME;
        if asked && name == TypingMode::NAME {
            break;
        }
    }
    assert_eq!(
        log.first(),
        Some(&TypingMode::NAME),
        "the keyboard told first: {log:?}"
    );
    assert_eq!(
        log.last(),
        Some(&TypingMode::NAME),
        "the ask was answered: {log:?}"
    );
    assert!(log.contains(&TypingModeAsk::NAME), "{log:?}");
}
