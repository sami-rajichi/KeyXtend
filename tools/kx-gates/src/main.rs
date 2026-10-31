//! kx-gates: the Windows gate runner for gates G1-G13, G17-G27 and the hand try; it prints one JSON line per run.
//! It moves the real mouse and types into real apps, so it runs only with the owner's yes; `capture-layouts` only reads.
#![deny(unsafe_code)]

mod admin;
mod aim;
mod apps;
mod board;
mod budget;
mod busbench;
mod capture;
mod cfgfile;
mod cfgload;
mod cli;
mod clicks;
mod clipcopy;
mod clipfill;
mod clipkeep;
mod clipnames;
mod clipown;
mod clippanel;
mod close;
mod config;
mod copyend;
mod diff;
mod facearg;
mod facetools;
mod featcfg;
mod fit;
mod ftl;
mod g1;
mod g10;
mod g10keys;
mod g10pad;
mod g10point;
mod g10steps;
mod g11;
mod g12;
mod g12frames;
mod g12pts;
mod g13;
mod g17;
mod g18;
mod g19;
mod g19pill;
mod g2;
mod g20;
mod g20hear;
mod g20keys;
mod g20plan;
mod g20prod;
mod g20steps;
mod g21;
mod g21prod;
mod g22;
mod g22bench;
mod g22cfg;
mod g22type;
mod g23;
mod g24;
mod g25;
mod g25layer;
mod g25watch;
mod g25wins;
mod g26;
mod g27;
mod g27lane;
mod g27type;
mod g2hand;
mod g2raw;
mod g3;
mod g4;
mod g5;
mod g5play;
mod g6;
mod g6cases;
mod g6judge;
mod g6rec;
mod g6rows;
mod g6scene;
mod g6spot;
mod g6uia;
mod g7;
mod g8;
mod g8cfg;
mod g8churn;
mod g8clip;
mod g8frames;
mod g8hook;
mod g8host;
mod g8lang;
mod g8lit;
mod g8loss;
mod g8parts;
mod g8pops;
mod g8predict;
mod g8rounds;
mod g8scroll;
mod g8size;
mod g8start;
mod g8steps;
mod g8tips;
mod g8type;
mod g8ui;
mod g8use;
mod g8words;
mod g9;
mod g9pad;
mod g9win;
mod gatecfg;
mod hand;
mod hooklog;
mod keymap;
mod keys;
mod langs;
mod launch;
mod launchterm;
mod modekey;
mod mouse;
mod mouseguard;
mod out;
mod outline;
mod owncopy;
mod ownset;
mod p3cfg;
mod p4cfg;
mod p5cfg;
mod p7cfg;
mod p8cfg;
mod padgeom;
mod padread;
mod pin;
mod presentmon;
mod probe;
mod probecfg;
mod probetext;
mod proc;
mod product;
mod readback;
mod rng;
mod scroll;
mod scrolllog;
mod shot;
mod simin;
mod simuser;
mod stats;
mod testbuild;
mod text;
mod tlog;
mod typebox;
mod uilook;
mod usage;
mod walk;
mod wer;
mod win;
mod winclip;
mod winfind;
mod workertok;

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
    let result = match raw.as_slice() {
        [command] if command == capture::COMMAND => {
            config::load().and_then(|cfg| capture::run(&cfg.capture))
        }
        _ => cli::parse(&raw).and_then(|args| cli::run(&args)),
    };
    match result {
        Ok(v) => {
            out::say!("{v}");
            if out::failed(&v) {
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
