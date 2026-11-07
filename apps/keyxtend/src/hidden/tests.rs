use super::fixture::{APP, Bed, hub};
use super::*;
use kx_crypto::{GuardError, Zeroizing};
use kx_module_api::ports::BlobName;
use kx_platform::ui::HiddenCopy;
use kx_platform::{CLIP_DIR, SEALED_DIR, VAULT_DIR};
use kx_platform_fake::FakeHidden;
use kx_ui::I18n;
use std::fs;
use std::io;

/// The package folder a hidden run's saves would land in.
const PACKAGE: &str = "Claude_pzs8sxrjxfjjc";
/// A settings file a run that reads it would follow: a log level that is not the default.
const FILE: &str = "[kernel]\nlog_level = \"debug\"\n";
/// The level that file sets.
const LEVEL: &str = "debug";

/// A key guard that keeps the key as it is, so the sealed files can save in a test.
struct Plain;

impl KeyGuard for Plain {
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError> {
        Ok(key.to_vec())
    }

    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError> {
        Ok(Zeroizing::new(wrapped.to_vec()))
    }
}

/// A drive with room for anything.
struct Roomy;

impl DiskRoom for Roomy {
    fn free(&self, _path: &Path) -> io::Result<u64> {
        Ok(u64::MAX / 2)
    }

    fn drive(&self, _path: &Path) -> io::Result<String> {
        Ok("D:".into())
    }
}

/// The run a probe answering `answer` gives.
fn probed(answer: Result<Probe, HiddenError>) -> Probed {
    let port = FakeHidden::new();
    port.set_probe(answer);
    Run::probe(&port, Path::new("data"))
}

/// A run started from inside `APP`.
fn hidden() -> Run {
    Run::hidden(APP.into())
}

/// The kernel of `run` over `bed`, readied and booted, with the hub that heard it.
fn booted(run: &Run, bed: &Bed) -> (Kernel, Arc<Hub>) {
    let (hub, _requests) = hub();
    let mut kernel = bed.kernel();
    run.ready(&mut kernel, &hub).unwrap();
    kernel.boot().unwrap();
    (kernel, hub)
}

/// Saves one blob in each of `stores` and one in `shelf`.
fn save_all(stores: [&dyn SealedStore; 2], shelf: &dyn SealedShelf) {
    let name = BlobName::new("learned").unwrap();
    for store in stores {
        store.save(&name, b"words").unwrap();
        assert_eq!(store.load(&name).unwrap().as_deref(), Some(&b"words"[..]));
    }
    shelf.save(&name, &name, b"copied").unwrap();
}

#[test]
fn a_hidden_probe_names_its_app_and_keeps_the_listed_modules_off() {
    let probe = Probe::Hidden {
        package: PACKAGE.into(),
        app: APP.into(),
    };
    let got = probed(Ok(probe));
    let want = Run::Hidden {
        app: Redacted::new(APP.into()),
        off: KEPT_OFF.to_vec(),
    };
    assert_eq!((got.run, got.error), (want, None));
    #[cfg(feature = "mod-vault")]
    assert_eq!(
        KEPT_OFF,
        [kx_mod_vault::ID],
        "Quick-fill never opens its notebook here"
    );
}

#[test]
fn a_normal_probe_is_a_normal_run_and_so_is_a_failed_one() {
    let normal = probed(Ok(Probe::Normal));
    assert_eq!((normal.run, normal.error), (Run::Normal, None));
    let failed = probed(Err(HiddenError::Unreadable));
    assert_eq!(failed.run, Run::Normal);
    assert_eq!(failed.error, Some(HiddenError::Unreadable));
}

#[test]
fn a_hidden_run_reads_no_settings_saves_none_and_shows_its_banner() {
    let bed = Bed::new();
    fs::write(bed.files().settings, FILE).unwrap();
    let run = hidden();
    assert_eq!(
        run.peek(&bed.files()),
        None,
        "the single-copy reach uses its defaults"
    );
    let (kernel, hub) = booted(&run, &bed);
    let defaults = KernelSettings {
        disabled: KEPT_OFF.iter().map(|id| id.as_str().to_owned()).collect(),
        ..KernelSettings::from_defaults().unwrap()
    };
    assert_eq!(
        kernel.kernel_settings(),
        Some(defaults),
        "the defaults, with the kept-off list off"
    );
    let shown = hub.take_notices();
    let en = I18n::new(kx_ui::find("en").unwrap());
    let banner = en.notice(&about_app(keys::HIDDEN_RUN, APP));
    assert_eq!(shown.len(), 1);
    assert_eq!(
        (shown[0].text.as_str(), shown[0].ask.is_none()),
        (banner.as_str(), true)
    );
    assert!(banner.contains(APP), "{banner}");
    assert_eq!(fs::read_to_string(bed.files().settings).unwrap(), FILE);
}

#[test]
fn a_normal_run_reads_its_settings_and_shows_no_banner() {
    let bed = Bed::new();
    fs::write(bed.files().settings, FILE).unwrap();
    assert!(Run::Normal.peek(&bed.files()).is_some());
    let (kernel, hub) = booted(&Run::Normal, &bed);
    assert_eq!(kernel.kernel_settings().unwrap().log_level, LEVEL);
    assert!(hub.take_notices().is_empty());
}

#[test]
fn a_hidden_run_keeps_its_words_copies_and_notebook_in_memory_and_makes_no_folder() {
    let bed = Bed::new();
    let run = hidden();
    let (words, clip) = (bed.data().join(SEALED_DIR), bed.data().join(CLIP_DIR));
    let store = run.store(words, || Box::new(Plain));
    let vault = run.store(bed.data().join(VAULT_DIR), || Box::new(Plain));
    let shelf = run.shelf(clip.clone(), || Box::new(Plain), Box::new(Roomy));
    save_all([&*store, &*vault], &*shelf);
    assert_eq!(run.make_dir(&clip), None);
    assert_eq!(
        bed.listing(),
        Vec::<PathBuf>::new(),
        "nothing reached the data folder"
    );
}

#[test]
fn a_normal_run_seals_its_words_copies_and_notebook_in_the_data_folder() {
    let bed = Bed::new();
    let (words, clip) = (bed.data().join(SEALED_DIR), bed.data().join(CLIP_DIR));
    assert_eq!(Run::Normal.make_dir(&clip), None);
    assert!(clip.is_dir(), "the history's folder is made");
    let store = Run::Normal.store(words.clone(), || Box::new(Plain));
    let notebook = bed.data().join(VAULT_DIR);
    let vault = Run::Normal.store(notebook.clone(), || Box::new(Plain));
    let shelf = Run::Normal.shelf(clip.clone(), || Box::new(Plain), Box::new(Roomy));
    save_all([&*store, &*vault], &*shelf);
    let held = |dir: &Path| fs::read_dir(dir).unwrap().count();
    assert!(
        [&words, &clip, &notebook].into_iter().all(|d| held(d) > 0),
        "each is sealed on disk in its own folder"
    );
}

#[test]
fn a_hidden_run_opens_no_log_file_while_a_normal_run_does() {
    let bed = Bed::new();
    let log = hidden().log(None, &bed.files()).unwrap();
    assert!(log.no_file.is_none());
    assert!(!bed.files().logs.exists(), "no log folder or file");
    let log = Run::Normal.log(None, &bed.files()).unwrap();
    assert!(log.no_file.is_none() && bed.files().log.is_file());
}

#[test]
fn the_probe_is_told_in_the_log_and_neither_names_the_app() {
    let bed = Bed::new();
    let log = Run::Normal.log(None, &bed.files()).unwrap();
    let probed = Probed {
        run: hidden(),
        error: Some(HiddenError::Unreadable),
    };
    tracing::dispatcher::with_default(&log.dispatch, || probed.tell());
    let text = fs::read_to_string(bed.files().log).unwrap();
    assert_eq!(
        text.lines().count(),
        2,
        "a failed probe, then a run that saves nothing: {text}"
    );
    assert!(
        !text.contains(APP) && !format!("{probed:?}").contains(APP),
        "{text}"
    );
}

#[test]
fn only_a_normal_run_asks_about_the_hidden_copies() {
    let bed = Bed::new();
    let port = Arc::new(FakeHidden::new());
    let copy = HiddenCopy::new(bed.data().join(PACKAGE), PACKAGE.into(), APP.into());
    port.set_copies(Ok(vec![copy]));
    let (kernel, hub) = booted(&hidden(), &bed);
    let _ = hub.take_notices();
    assert!(hidden().ask(port.clone(), &kernel, &hub).is_none());
    assert!(
        hub.take_notices().is_empty(),
        "a hidden run never lists the copies"
    );
    let (kernel, hub) = booted(&Run::Normal, &bed);
    assert!(Run::Normal.ask(port, &kernel, &hub).is_some());
    assert_eq!(hub.take_notices().len(), 1);
}
