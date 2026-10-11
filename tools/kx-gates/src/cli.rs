//! The command line: parses the arguments, runs one gate and saves its result line.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx};
use crate::usage::usage;
use crate::{
    close, config, facearg, g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11, g12, g13, g17, g18, g19,
    g20, g21, g22, g23, g24, g25, g26, hand, mouseguard, out,
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

/// Gates that take only a name: no count, no seed.
const NAMED: [(&str, NamedRun); 20] = [
    ("g5", g5::run),
    ("g6", g6::run),
    ("g7", g7::run),
    ("g8", g8::run),
    ("g9", g9::run),
    ("g10", g10::run),
    ("g11", g11::run),
    ("g12", g12::run),
    ("g13", g13::run),
    ("g17", g17::run),
    ("g18", g18::run),
    ("g19", g19::run),
    ("g20", g20::run),
    ("g21", g21::run),
    ("g22", g22::run),
    ("g23", g23::run),
    ("g24", g24::run),
    ("g25", g25::run),
    ("g26", g26::run),
    ("close", close::run),
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
    let guard = mouseguard::begin(&cfg.mouse_guard, &args.gate)?;
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
