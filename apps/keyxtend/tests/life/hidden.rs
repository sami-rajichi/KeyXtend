//! A hidden run over the fakes (ADR-0037): it reads no settings file, writes nothing in its data folder, opens no log
//! file, shows its banner, keeps its off list off for that run and gives Open folder no folder.

use crate::A_KEY;
use crate::rig::{Rig, char_key};
use keyxtend::drain;
use keyxtend::hidden::Run;
use kx_module_api::ModuleState::{Active, Stopped};
use kx_module_api::Redacted;
use kx_module_api::ports::{FaceAction, KeyId};
use kx_ui::I18n;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The app the run was started from.
const APP: &str = "Claude";

/// Everything under `dir`, A to Z, with each file's bytes.
pub fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut all = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            all.push((path.clone(), None));
            all.extend(tree(&path));
        } else {
            let bytes = std::fs::read(&path).unwrap();
            all.push((path, Some(bytes)));
        }
    }
    all.sort();
    all
}

#[test]
fn a_hidden_run_writes_nothing_in_its_data_folder_and_shows_its_banner() {
    let rig = Rig::new();
    let files = rig.files();
    let off = format!("[kernel]\ndisabled = [\"{}\"]\n", kx_mod_mouse::ID);
    std::fs::write(&files.settings, off).unwrap();
    let before = tree(&files.dir);
    let run = Run::Hidden {
        app: Redacted::new(APP.into()),
        off: vec![kx_mod_power::ID],
    };
    let _log = run.log(None, &files).unwrap();
    let mut boot = rig.load_as(&run);
    boot.start().unwrap();
    let states = [kx_mod_mouse::ID, kx_mod_power::ID].map(|id| boot.kernel.state(id));
    assert_eq!(
        states,
        [Some(Active), Some(Stopped)],
        "the file is unread; power is kept off"
    );
    boot.kernel.set_enabled(kx_mod_scroll::ID, false).unwrap();
    let tap = FaceAction::Tap {
        id: KeyId::new(char_key(A_KEY)),
        back: false,
    };
    let _ = boot.hub.act(tap);
    assert!(!rig.injector.strokes().is_empty(), "it still types");
    let en = I18n::new(kx_ui::find("en").unwrap());
    let banner = en.notice(&run.banner().unwrap());
    let shown = boot.hub.take_notices();
    assert!(shown.iter().any(|n| n.text == banner), "{shown:?}");
    let hub = Arc::clone(&boot.hub);
    let thread = drain::spawn(boot.kernel, Arc::clone(&hub), boot.requests, None).unwrap();
    assert!(drain::stop(&hub, thread));
    drop(boot.heard);
    assert_eq!(
        tree(&files.dir),
        before,
        "no settings, clipboard folder or log was written"
    );
}

#[test]
fn a_hidden_run_gives_open_folder_no_folder_to_open() {
    let rig = Rig::new();
    let run = Run::Hidden {
        app: Redacted::new(APP.into()),
        off: Vec::new(),
    };
    let boot = rig.load_as(&run);
    assert!(!boot.hub.open_folder(), "this run keeps no folder");
    assert!(rig.reveal.shown().is_empty());
}
