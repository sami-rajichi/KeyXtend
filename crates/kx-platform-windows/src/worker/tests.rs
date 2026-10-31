//! The real worker, started as the host starts it: it draws through the host, its words and exit codes match the
//! host's, a missing or relative program is missing, and the command line reads back as written.

use super::*;
use kx_ipc::{Ask, CLEAN_EXIT, MAX_PIXELS_FLAG as MAX_PIXELS, MEDIA_JOB as MEDIA, REFUSED_EXIT};
use kx_module_api::ports::{ImageCodec, PictureAsk, PictureConfig, PictureError, Pictures};
use kx_module_api::{Clock, Mono, Redacted};
use kx_platform::pictures::{Worker, WorkerPictures};
use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, OnceLock};

mod rights;
mod verdict;

/// How long a test waits for the worker; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);
/// The worker's package, which Cargo builds beside this test program.
const PACKAGE: &str = "keyxtend-worker";
/// The folder Cargo's dev profile builds into, under a name of its own.
const DEBUG_DIR: &str = "debug";
/// Cargo's dev profile.
const DEV: &str = "dev";
/// A file Cargo keeps only at the top of its target folder, never in a target triple's folder.
const TOP_MARK: &str = ".rustc_info.json";

/// Test values; the real ones come from the clipboard module's settings.
const CFG: PictureConfig = PictureConfig {
    deadline_ms: 10_000,
    idle_s: 60,
    memory_mb: 256,
    max_pixels: 1_000_000,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// A clock that never moves, so no worker goes idle.
struct Still;

impl Clock for Still {
    fn now(&self) -> Mono {
        Mono::default()
    }
}

/// The worker program, built by Cargo before the first test uses it, so it is never stale.
fn worker() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(build).clone()
}

/// Builds the worker beside this test program, with the target folder, triple and profile its path shows.
fn build() -> PathBuf {
    let exe = env::current_exe().unwrap();
    let dir = exe.parent().and_then(Path::parent).unwrap().to_path_buf();
    let name = dir.file_name().unwrap();
    let profile = if name == DEBUG_DIR {
        OsStr::new(DEV)
    } else {
        name
    };
    let up = dir.parent().unwrap();
    let (top, triple) = if up.join(TOP_MARK).is_file() {
        (up, None)
    } else {
        (up.parent().unwrap(), up.file_name())
    };
    let cargo = env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into());
    let mut cmd = Command::new(cargo);
    cmd.args(["build", "--quiet", "-p", PACKAGE, "--profile"])
        .arg(profile)
        .arg("--target-dir")
        .arg(top)
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    if let Some(triple) = triple {
        cmd.arg("--target").arg(triple);
    }
    assert!(
        cmd.status().unwrap().success(),
        "Cargo did not build the worker"
    );
    let worker = dir.join(WORKER_EXE);
    assert!(worker.is_file(), "no worker at {}", worker.display());
    worker
}

fn spawner() -> WinSpawner {
    WinSpawner::at(worker())
}

/// A started worker on the media job, as the host drives it.
fn spawn() -> Piped<WinChild> {
    spawner().spawn(&Launch::media(&CFG)).unwrap()
}

/// A started worker's raw stdin, stdout and process, for frames the host never sends.
fn start(args: &[&str]) -> (PipeWriter, PipeReader, WinChild) {
    let args = args.iter().map(|a| (*a).to_owned()).collect();
    let launch = Launch {
        args,
        memory_mb: CFG.memory_mb,
    };
    spawner().start(&launch).unwrap()
}

/// The media job's own command line.
fn media() -> Vec<String> {
    Launch::media(&CFG).args
}

/// A 1 by 1 24-bit DIB of red 1, green 2, blue 3.
fn tiny_dib() -> Vec<u8> {
    let mut head = [0_u8; 40];
    head[0] = 40;
    head[4] = 1;
    head[8] = 1;
    head[12] = 1;
    head[14] = 24;
    [&head[..], &[3, 2, 1, 0]].concat()
}

fn ask(codec: ImageCodec, bytes: Vec<u8>) -> PictureAsk {
    PictureAsk {
        codec,
        bytes: Redacted::new(bytes),
        side: 4,
    }
}

#[test]
fn the_real_worker_draws_through_the_host() {
    let pictures = WorkerPictures::new(spawner(), Arc::new(Still));
    let mut run = pictures.open(CFG);
    let dib = ask(ImageCodec::Dib, tiny_dib());
    let drawn = run.draw(&dib).unwrap();
    assert_eq!((drawn.width(), drawn.height()), (1, 1));
    assert_eq!(drawn.rgba().expose().as_ref(), [1, 2, 3, 255]);
    let junk = ask(ImageCodec::Png, b"junk".to_vec());
    assert_eq!(run.draw(&junk), Err(PictureError::Refused));
    assert!(run.draw(&dib).is_ok(), "a refused image keeps the worker");
}

#[test]
fn the_workers_words_and_exit_codes_match_the_host() {
    let exit = |c: u8| Some(u32::from(c));
    let mut worker = spawn();
    worker.send(Ask::Bye);
    assert_eq!(worker.code(WAIT), exit(CLEAN_EXIT), "goodbye");
    let media = media();
    let words: Vec<&str> = media.iter().map(String::as_str).collect();
    let (input, _output, mut child) = start(&words);
    drop(input);
    assert_eq!(child.code(WAIT), exit(CLEAN_EXIT), "the end of input");
    let (mut input, _output, mut child) = start(&words);
    input.write_all(b"not a frame at all").unwrap();
    assert_eq!(child.code(WAIT), exit(REFUSED_EXIT), "a broken frame");
    let changed = [
        ["medium", MAX_PIXELS, "5"],
        [MEDIA, "--max-pixel", "5"],
        [MEDIA, MAX_PIXELS, "0"],
    ];
    for args in changed {
        let (_input, _output, mut child) = start(&args);
        assert_eq!(child.code(WAIT), exit(REFUSED_EXIT), "{args:?}");
    }
}

#[test]
fn a_missing_or_relative_worker_is_missing() {
    let gone = worker().with_file_name("kx-no-such-worker.exe");
    let launch = Launch::media(&CFG);
    assert!(matches!(
        WinSpawner::at(gone).spawn(&launch),
        Err(SpawnError::Missing)
    ));
    let relative = WinSpawner::at(PathBuf::from(WORKER_EXE));
    assert!(matches!(relative.spawn(&launch), Err(SpawnError::Missing)));
}

#[test]
fn a_wait_always_ends_however_long_it_is() {
    assert_eq!(wait_ms(Duration::from_millis(5)), 5);
    let longest = [u64::from(INFINITE), u64::MAX].map(Duration::from_millis);
    for within in longest.into_iter().chain([Duration::MAX]) {
        assert_eq!(wait_ms(within), LONGEST_WAIT_MS, "{within:?}");
    }
    assert_ne!(wait_ms(Duration::MAX), INFINITE, "never a wait without end");
}

#[test]
fn the_worker_is_found_in_the_programs_folder() {
    let exe = env::current_exe().unwrap();
    let dir = exe.parent().unwrap();
    assert_eq!(WinSpawner::beside(dir).exe, exe.with_file_name(WORKER_EXE));
}

#[test]
#[allow(
    unsafe_code,
    reason = "Reads Windows' own split of a command line, then frees it."
)]
fn the_command_line_reads_back_as_written() {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::UI::Shell::CommandLineToArgvW;
    let exe = OsStr::new(r"C:\Program Files\Key Xtend\keyxtend-worker.exe");
    let args = [
        "media",
        "two words",
        "",
        "a\"b",
        r"back\slash",
        r"end\",
        r#"x\"y"#,
        "tab\there",
    ];
    let args: Vec<String> = args.map(String::from).to_vec();
    let line = line::command_line(exe, &args);
    let mut count = 0;
    // SAFETY: `line` is null-ended; Windows returns one block of `count` strings, freed once below.
    let read: Vec<String> = unsafe {
        let split = CommandLineToArgvW(windows::core::PCWSTR(line.as_ptr()), &raw mut count);
        assert!(!split.is_null());
        let len = usize::try_from(count).unwrap();
        let read = (0..len)
            .map(|i| (*split.add(i)).to_string().unwrap())
            .collect();
        let _ = LocalFree(Some(HLOCAL(split.cast())));
        read
    };
    assert_eq!(read[0], exe.to_string_lossy());
    assert_eq!(read[1..], args[..]);
}
