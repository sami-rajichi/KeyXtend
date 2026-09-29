//! kx-gates: the Windows gate runner. It drives gates G1-G7, G12, G17-G25 and the hand try, and prints one JSON line of results per run.
//!
//! It moves the real mouse, types into real apps and opens Start: run it only when the owner agrees. Windows only.
#![deny(unsafe_code)]

mod admin;
mod apps;
mod assist;
mod cfgfile;
mod cli;
mod clicks;
mod clip;
mod clipkeep;
mod cliplisten;
mod cliptext;
mod config;
mod diff;
mod facearg;
mod facetools;
mod featcfg;
mod g1;
mod g12;
mod g12pts;
mod g17;
mod g18;
mod g19;
mod g19pill;
mod g2;
mod g20;
mod g21;
mod g22;
mod g22bench;
mod g22cfg;
mod g22type;
mod g23;
mod g24;
mod g25;
mod g3;
mod g4;
mod g5;
mod g6;
mod g7;
mod gatecfg;
mod hand;
mod hookhost;
mod hookio;
mod hookstate;
mod keys;
mod launch;
mod launchterm;
mod mouse;
mod out;
mod probe;
mod probecfg;
mod probetext;
mod readback;
mod rng;
mod scroll;
mod shot;
mod simuser;
mod stats;
mod text;
mod tlog;
mod usage;
mod wer;
mod win;
mod winclip;
mod winfind;

use std::process::ExitCode;

/// What the tool says on any other system.
#[cfg(not(windows))]
const WINDOWS_ONLY: &str = "kx-gates runs only on Windows.";

/// Stops apps we start from inheriting our stdin, stdout and stderr, so they cannot hold a pipe open.
#[cfg(windows)]
#[allow(unsafe_code, reason = "Plain calls on our own standard handles.")]
fn keep_stdio_private() {
    use windows::Win32::Foundation::{HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation};
    use windows::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: plain calls on our own standard handles; a missing handle only gives an error.
        unsafe {
            if let Ok(h) = GetStdHandle(id) {
                let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
            }
        }
    }
}

#[cfg(windows)]
fn main() -> ExitCode {
    // Per-monitor DPI awareness comes from the manifest that build.rs embeds.
    keep_stdio_private();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let result = cli::parse(&raw).and_then(|args| cli::run(&args));
    match result {
        Ok(v) => {
            out::say!("{v}");
            if v.get("error").is_some() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("{WINDOWS_ONLY}");
    ExitCode::FAILURE
}
