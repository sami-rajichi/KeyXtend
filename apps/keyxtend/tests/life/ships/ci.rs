//! Reads the CI workflow's cargo commands: which package each builds, with which features, and whether the default
//! features are off.

/// What starts a comment line.
const COMMENT: char = '#';
/// What starts a list item.
const ITEM: &str = "- ";
/// The key of a step's command.
const RUN: &str = "run:";
/// The word that starts each cargo command.
const CARGO: &str = "cargo";
/// What ends cargo's own flags; the words after it go to the tool cargo runs.
const ARGS_END: &str = "--";
/// The flags that name the package.
const PACKAGE: [&str; 2] = ["--package", "-p"];
/// The flags that list features.
const FEATURES: [&str; 2] = ["--features", "-F"];
/// The flag that turns the default features off.
const NO_DEFAULT: &str = "--no-default-features";
/// What joins a flag to its value in one word.
const JOIN: char = '=';
/// What separates the features in one list.
const SEP: char = ',';
/// The quotes a shell may put around a list.
const QUOTES: [char; 2] = ['"', '\''];

/// What one cargo command builds.
#[derive(Default)]
struct Build<'a> {
    package: Option<&'a str>,
    no_default: bool,
    features: Vec<&'a str>,
}

impl<'a> Build<'a> {
    /// Reads cargo's own flags from `args`, the words after `cargo`.
    fn read(args: &[&'a str]) -> Self {
        let mut build = Self::default();
        let mut rest = args.iter().copied().take_while(|a| *a != ARGS_END);
        while let Some(arg) = rest.next() {
            if let Some(list) = value(arg, &FEATURES, &mut rest) {
                let list = list.trim_matches(QUOTES).split(SEP);
                build.features.extend(list.filter(|f| !f.is_empty()));
            } else if let Some(name) = value(arg, &PACKAGE, &mut rest) {
                build.package = Some(name);
            } else {
                build.no_default |= arg == NO_DEFAULT;
            }
        }
        build
    }
}

/// The value when `arg` is one of `flags`: the next word, or what follows `=` in the same word.
fn value<'a>(
    arg: &'a str,
    flags: &[&str],
    rest: &mut impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    flags.iter().find_map(|flag| {
        let tail = arg.strip_prefix(flag)?;
        if tail.is_empty() {
            rest.next()
        } else {
            tail.strip_prefix(JOIN)
        }
    })
}

/// The value of the `run:` key that starts `line`, with the key's column.
fn run(line: &str) -> Option<(usize, &str)> {
    let body = line.trim_start();
    let item = body.strip_prefix(ITEM).map_or(body, str::trim_start);
    let value = item.strip_prefix(RUN)?;
    Some((line.len() - item.len(), value))
}

/// The words of each `run:` step in `yaml`, its lines joined, whatever their spacing, and comment lines left out.
fn steps(yaml: &str) -> Vec<Vec<&str>> {
    let mut steps = Vec::new();
    let mut open: Option<(usize, Vec<&str>)> = None;
    let lines = yaml
        .lines()
        .filter(|l| !l.trim_start().starts_with(COMMENT));
    for line in lines {
        let indent = line.len() - line.trim_start().len();
        let inside = |key: &usize| line.trim().is_empty() || indent > *key;
        if let Some((_, words)) = open.as_mut().filter(|(key, _)| inside(key)) {
            words.extend(line.split_whitespace());
            continue;
        }
        steps.extend(open.take().map(|(_, words)| words));
        open = run(line).map(|(key, value)| (key, value.split_whitespace().collect()));
    }
    steps.extend(open.map(|(_, words)| words));
    steps
}

/// The sorted features of each cargo command in `yaml`'s steps that builds `package` with default features off.
pub fn lists<'a>(yaml: &'a str, package: &str) -> Vec<Vec<&'a str>> {
    let steps = steps(yaml);
    let commands = steps.iter().flat_map(|w| w.split(|w| *w == CARGO).skip(1));
    let builds = commands.map(Build::read);
    let off = builds.filter(|b| b.no_default && b.package == Some(package));
    let sorted = |mut b: Build<'a>| {
        b.features.sort_unstable();
        b.features
    };
    off.map(sorted).collect()
}

mod tests;
