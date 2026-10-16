//! The command line: parses the arguments, runs one gate and saves its result line.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx};
use crate::config::GatesConfig;
use crate::mouseguard::Mouse;
use crate::usage::usage;
use crate::{
    close, config, facearg, fit, g1, g2, g3, g4, g5, g6, g7, g8, g8hook, g9, g10, g11, g12, g13,
    g17, g18, g19, g20, g21, g22, g23, g24, g25, g26, hand, mouseguard, out,
};

/// Gates whose runs take a random seed.
const SEEDED: [&str; 3] = ["g1", "g2", "g4"];

/// Parsed command line.
#[derive(Debug, PartialEq)]
pub struct Args {
    /// Gate name, such as `g1`.
    gate: String,
    /// App, keyboard or gate-specific name.
    name: String,
    /// Characters, clicks or seconds, depending on the gate.
    count: Option<usize>,
    /// Random seed of a seeded gate.
    seed: Option<u64>,
    /// G1: use the app's open window instead of starting it.
    attach: bool,
    /// G1: pause between characters, instead of the one in kx-gates.toml.
    pause_ms: Option<u64>,
}

fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("bad number: {value}"))
}

/// Parses the arguments after the program name.
pub fn parse(args: &[String]) -> Result<Args, String> {
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
    facearg::check(gate, name)?;
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

/// Every gate: whether it holds the mouse, and its run when it takes only a name. G5, G6, G12, G17 and G18 drive the
/// test build, as `DRIVEN`'s names do; the hand try, G13 and G23 wait on the owner's own mouse, G26 runs on the CI
/// runner and `close` moves nothing.
const GATES: [(&str, Mouse, Option<NamedRun>); 25] = [
    ("g1", Mouse::Held, None),
    ("g2", Mouse::Held, None),
    ("g3", Mouse::Held, None),
    ("g4", Mouse::Held, None),
    ("assist", Mouse::Free, None),
    ("g5", Mouse::Drives, Some(g5::run)),
    ("g6", Mouse::Drives, Some(g6::run)),
    ("g7", Mouse::Held, Some(g7::run)),
    ("g8", Mouse::Held, Some(g8::run)),
    ("g9", Mouse::Held, Some(g9::run)),
    ("g10", Mouse::Held, Some(g10::run)),
    ("g11", Mouse::Held, Some(g11::run)),
    ("g12", Mouse::Drives, Some(g12::run)),
    ("g13", Mouse::Free, Some(g13::run)),
    ("g17", Mouse::Drives, Some(g17::run)),
    ("g18", Mouse::Drives, Some(g18::run)),
    ("g19", Mouse::Held, Some(g19::run)),
    ("g20", Mouse::Held, Some(g20::run)),
    ("g21", Mouse::Held, Some(g21::run)),
    ("g22", Mouse::Held, Some(g22::run)),
    ("g23", Mouse::Free, Some(g23::run)),
    ("g24", Mouse::Held, Some(g24::run)),
    ("g25", Mouse::Held, Some(g25::run)),
    ("g26", Mouse::Free, Some(g26::run)),
    ("close", Mouse::Free, Some(close::run)),
];

/// A gate's names that drive the test build, though the gate itself holds the mouse.
const DRIVEN: [(&str, &str); 2] = [("g8", g8hook::HOOK), ("g8", g8hook::HOOK_WEAK)];

/// The run of `gate` when it takes only a name.
fn named(gate: &str) -> Option<NamedRun> {
    GATES.iter().find(|(g, ..)| *g == gate).and_then(|g| g.2)
}

/// Whether `gate` run on `name` holds the mouse, and how; an unknown gate never does.
fn mouse(gate: &str, name: &str) -> Mouse {
    if DRIVEN.contains(&(gate, name)) {
        return Mouse::Drives;
    }
    GATES
        .iter()
        .find(|(g, ..)| *g == gate)
        .map_or(Mouse::Free, |g| g.1)
}

/// The clicks of a G2 or G4 run, `--clicks` or else the gate's setting; refused when the run would pass the mouse
/// guard's limit.
fn clicks(cfg: &GatesConfig, args: &Args) -> Result<usize, String> {
    let set = if args.gate == "g4" {
        cfg.g4.clicks
    } else {
        cfg.g2.clicks
    };
    let count = args.count.unwrap_or(set);
    fit::clicks_fit(cfg, &args.gate, count)?;
    Ok(count)
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
        "g2" => g2::run(ctx, &args.name, clicks(ctx.cfg, args)?, seed),
        "g4" => g4::run(ctx, &args.name, clicks(ctx.cfg, args)?, seed),
        "assist" => hand::run(
            ctx,
            &args.name,
            args.count.map_or(ctx.cfg.hand.try_secs, |n| n as u64),
        ),
        "g3" => g3::run(ctx, &args.name),
        _ => Err(usage()),
    }
}

/// Runs one gate; the result line (or its error) is also appended to results.jsonl.
pub fn run(args: &Args) -> Result<Value, String> {
    let cfg = config::load()?;
    let spike = spike_core::config::load()?;
    let target = kx_target_window::config::load().map_err(|e| e.to_string())?;
    let out = out::out_dir(&cfg.out_dir())?;
    let base = format!("{}-{}-{}", args.gate, args.name, out::stamp());
    let ctx = Ctx {
        cfg: &cfg,
        spike: &spike,
        target: &target,
        out,
        base,
    };
    // The guard lets go when the gate ends; dropping `guard` lets go on a panic too.
    let guard = mouseguard::begin(&cfg.mouse_guard, mouse(&args.gate, &args.name))?;
    let result = dispatch(&ctx, args);
    let ended = guard.finish(result.is_ok());
    let mut result = result.unwrap_or_else(
        |e| json!({ "gate": args.gate.to_uppercase(), "name": args.name, "error": e }),
    );
    mouseguard::mark(&mut result, ended);
    out::save_result(&ctx.out, &result.to_string())?;
    Ok(result)
}

#[cfg(test)]
mod tests;
