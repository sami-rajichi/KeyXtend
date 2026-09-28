//! P1 spike harness: drives gates G1-G6, G17-G25 and the hand try, and prints one JSON line of results per run.
//!
//! It moves the real mouse, types into real apps and opens Start: run it only when the owner agrees.

mod admin;
mod apps;
mod assist;
mod clicks;
mod clip;
mod clipkeep;
mod cliplisten;
mod config;
mod diff;
mod facetools;
mod featcfg;
mod g1;
mod g17;
mod g18;
mod g19;
mod g19pill;
mod g2;
mod g20;
mod g21;
mod g22;
mod g22bench;
mod g22type;
mod g23;
mod g24;
mod g25;
mod g3;
mod g4;
mod g5;
mod g6;
mod hand;
mod hookhost;
mod hookio;
mod keys;
mod launch;
mod mouse;
mod out;
mod probe;
mod probecfg;
mod readback;
mod rng;
mod scroll;
mod shot;
mod simuser;
mod stats;
mod text;
mod tlog;
mod wer;
mod win;
mod winclip;

use std::process::ExitCode;

use serde_json::{Value, json};
use windows::Win32::Foundation::{HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation};
use windows::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};

use apps::{AppKind, Ctx};

/// App names joined for the usage text.
fn names(apps: &[AppKind]) -> String {
    apps.iter().map(|a| a.name()).collect::<Vec<_>>().join("|")
}

/// Command-line help; the G1 app list comes from `AppKind::ALL`.
fn usage() -> String {
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

/// Gates whose runs take a random seed.
const SEEDED: [&str; 3] = ["g1", "g2", "g4"];

/// Parsed command line.
#[derive(Debug, PartialEq)]
struct Args {
    gate: String,
    name: String,
    count: Option<usize>,
    seed: Option<u64>,
    /// G1: use the app's open window instead of starting it.
    attach: bool,
    /// G1: pause between characters, instead of the one in harness.toml.
    pause_ms: Option<u64>,
}

fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("bad number: {value}"))
}

fn parse(args: &[String]) -> Result<Args, String> {
    let [gate, name, rest @ ..] = args else {
        return Err(usage());
    };
    let count_flag = match gate.as_str() {
        "g1" => Some("--count"),
        "g2" | "g4" => Some("--clicks"),
        "assist" => Some("--secs"),
        "g3" => None,
        g if named(g).is_some() => None,
        _ => return Err(usage()),
    };
    let g1 = gate == "g1";
    let mut out = Args {
        gate: gate.clone(),
        name: name.clone(),
        count: None,
        seed: None,
        attach: false,
        pause_ms: None,
    };
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        if g1 && flag == "--attach" {
            out.attach = true;
            continue;
        }
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            f if Some(f) == count_flag => out.count = Some(number(value)?),
            "--seed" if SEEDED.contains(&gate.as_str()) => out.seed = Some(number(value)?),
            "--pause" if g1 => out.pause_ms = Some(number(value)?),
            _ => return Err(format!("unknown option {flag}\n{}", usage())),
        }
    }
    Ok(out)
}

/// A gate run that takes only a name.
type NamedRun = fn(&Ctx, &str) -> Result<Value, String>;

/// Gates that take only a name: no count, no seed.
const NAMED: [(&str, NamedRun); 12] = [
    ("g5", g5::run),
    ("g6", g6::run),
    ("g17", g17::run),
    ("g18", g18::run),
    ("g19", g19::run),
    ("g20", g20::run),
    ("g21", g21::run),
    ("g22", g22::run),
    ("g23", g23::run),
    ("g24", g24::run),
    ("g25", g25::run),
    ("close", facetools::close),
];

/// The run of `gate` when it takes only a name.
fn named(gate: &str) -> Option<NamedRun> {
    NAMED.iter().find(|(g, _)| *g == gate).map(|&(_, run)| run)
}

fn dispatch(ctx: &Ctx, args: &Args) -> Result<Value, String> {
    if let Some(run) = named(&args.gate) {
        return run(ctx, &args.name);
    }
    let seed = args.seed.unwrap_or_else(out::clock_seed);
    match args.gate.as_str() {
        "g1" => {
            let kind = AppKind::parse(&args.name)
                .ok_or_else(|| format!("unknown app {}\n{}", args.name, usage()))?;
            let run = g1::Run {
                count: args.count.unwrap_or(ctx.cfg.g1.count),
                seed,
                attach: args.attach,
                pause_ms: args.pause_ms.unwrap_or(ctx.cfg.g1.chunk_pause_ms),
            };
            g1::run(ctx, kind, &run)
        }
        "g2" => g2::run(
            ctx,
            &args.name,
            args.count.unwrap_or(ctx.cfg.g2.clicks),
            seed,
        ),
        "g4" => g4::run(
            ctx,
            &args.name,
            args.count.unwrap_or(ctx.cfg.g4.clicks),
            seed,
        ),
        "assist" => hand::run(
            ctx,
            &args.name,
            args.count.map_or(ctx.cfg.assist.try_secs, |n| n as u64),
        ),
        _ => g3::run(ctx, &args.name),
    }
}

/// Runs one gate; the result line (or its error) is also appended to results.jsonl.
fn run(args: &Args) -> Result<Value, String> {
    let cfg = config::load()?;
    let spike = spike_core::config::load()?;
    let out = out::out_dir(&cfg.out_dir())?;
    let base = format!("{}-{}-{}", args.gate, args.name, out::stamp());
    let ctx = Ctx {
        cfg: &cfg,
        spike: &spike,
        out,
        base,
    };
    let result = dispatch(&ctx, args).unwrap_or_else(
        |e| json!({ "gate": args.gate.to_uppercase(), "name": args.name, "error": e }),
    );
    out::save_result(&ctx.out, &result.to_string())?;
    Ok(result)
}

/// Stops apps we start from inheriting our stdin, stdout and stderr, so they cannot hold a pipe open.
fn keep_stdio_private() {
    for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: plain calls on our own standard handles; a missing handle only gives an error.
        unsafe {
            if let Ok(h) = GetStdHandle(id) {
                let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
            }
        }
    }
}

fn main() -> ExitCode {
    // Per-monitor DPI awareness comes from the manifest that build.rs embeds.
    keep_stdio_private();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let result = parse(&raw).and_then(|args| run(&args));
    match result {
        Ok(v) => {
            println!("{v}");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn usage_lists_every_g1_app() {
        let names: Vec<&str> = AppKind::ALL.iter().map(|a| a.name()).collect();
        assert!(usage().contains(&format!("g1 <{}>", names.join("|"))));
    }

    #[test]
    fn parses_each_gate() {
        let a = parse(&args("g1 notepad --count 5 --seed 7")).expect("parses");
        assert_eq!(
            a,
            Args {
                gate: "g1".into(),
                name: "notepad".into(),
                count: Some(5),
                seed: Some(7),
                attach: false,
                pause_ms: None
            }
        );
        let a = parse(&args("g1 word --attach --pause 100")).expect("parses");
        assert!(a.attach);
        assert_eq!(a.pause_ms, Some(100));
        assert_eq!(
            parse(&args("g2 qt --clicks 10")).expect("parses").count,
            Some(10)
        );
        assert_eq!(parse(&args("g3 slint")).expect("parses").count, None);
        assert_eq!(parse(&args("g5 admin")).expect("parses").gate, "g5");
        let a = parse(&args("assist right --secs 30")).expect("parses");
        assert_eq!((a.name.as_str(), a.count), ("right", Some(30)));
        let a = parse(&args("g4 qt --clicks 50 --seed 3")).expect("parses");
        assert_eq!(
            (a.gate.as_str(), a.count, a.seed),
            ("g4", Some(50), Some(3))
        );
    }

    #[test]
    fn parses_gates_that_take_only_a_name() {
        for probe in [
            "g6 chrome",
            "g17 explorer",
            "g18 notepad",
            "g19 core",
            "g20 notepad",
            "g21 win32",
            "g19 slint",
            "g22 slint",
            "g22 bench",
            "g22 bench-cloud",
            "g22 typing",
            "g23 qt",
            "g24 slint",
            "g25 notepad",
            "close slint",
        ] {
            let a = parse(&args(probe)).expect("parses");
            assert_eq!((a.count, a.seed), (None, None));
            assert!(named(&a.gate).is_some(), "{probe} has no run");
        }
    }

    #[test]
    fn rejects_bad_input() {
        for bad in [
            "",
            "g1",
            "g9 x",
            "g1 target --clicks 3",
            "g3 qt --seed 1",
            "g2 qt --clicks",
            "g1 target --count x",
            "g2 qt --attach",
            "g2 qt --pause 5",
            "g1 word --pause",
            "g4 qt --attach",
            "g4 qt --count 5",
            "g4 qt --pause 5",
            "g5 plain --seed 1",
            "assist grab --seed 1",
            "g17 notepad --seed 1",
            "g18 chrome --clicks 5",
            "g6 word --count 2",
            "g19 core --seed 1",
            "g20 notepad --count 3",
            "g21 chrome --clicks 2",
            "g22",
            "g22 bench --seed 1",
            "g22 typing --pause 10",
            "g25 notepad --attach",
            "g23 slint --seed 2",
            "g24 qt --clicks 3",
        ] {
            assert!(parse(&args(bad)).is_err(), "{bad} should fail");
        }
    }
}
