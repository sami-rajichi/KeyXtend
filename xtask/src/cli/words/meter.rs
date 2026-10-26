//! Parsing of `cargo xtask words meter`: optional flags, each at most once, in any order.

use super::{FLAG_PACKS, UsageError, is_path};

/// Flag naming the held-back sentences' folder, instead of the build's.
const FLAG_HELD: &str = "--held";
/// Flag giving the chips in the lane, instead of the `[predict]` default.
const FLAG_CHIPS: &str = "--chips";
/// Flag naming the one language to measure.
const FLAG_LANG: &str = "--lang";
/// Flag that times the scorer with a full learner instead of counting presses.
const FLAG_SPEED: &str = "--speed";

/// What `cargo xtask words meter` was given.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct MeterArgs {
    /// The packs folder.
    pub(crate) packs: Option<String>,
    /// The held-back sentences' folder.
    pub(crate) held: Option<String>,
    /// The chips in the lane.
    pub(crate) chips: Option<u8>,
    /// The one language to measure.
    pub(crate) lang: Option<String>,
    /// Time the scorer instead of counting presses.
    pub(crate) speed: bool,
}

/// The usage form, after `prefix`.
pub(super) fn form(prefix: &str) -> String {
    format!(
        "{prefix} [{FLAG_PACKS} <dir>] [{FLAG_HELD} <dir>] [{FLAG_CHIPS} <n>] [{FLAG_LANG} <iso>] [{FLAG_SPEED}]"
    )
}

/// Parses the arguments that follow `meter`.
pub(super) fn parse(args: &[String]) -> Result<MeterArgs, UsageError> {
    let mut out = MeterArgs::default();
    let mut chips = None;
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        if flag == FLAG_SPEED && !out.speed {
            out.speed = true;
            continue;
        }
        let value = rest.next().filter(|v| is_path(v)).ok_or(UsageError)?;
        let slot = match flag.as_str() {
            FLAG_PACKS => &mut out.packs,
            FLAG_HELD => &mut out.held,
            FLAG_LANG => &mut out.lang,
            FLAG_CHIPS => &mut chips,
            _ => return Err(UsageError),
        };
        if slot.replace(value.clone()).is_some() {
            return Err(UsageError);
        }
    }
    out.chips = chips
        .map(|n| n.parse())
        .transpose()
        .map_err(|_| UsageError)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::args;
    use super::super::super::{Command, parse as parse_all};
    use super::super::WordsAction;
    use super::*;

    fn meter(values: &[&str]) -> Result<Command, UsageError> {
        let all = [&["words", "meter"][..], values].concat();
        parse_all(&args(&all))
    }

    #[test]
    fn meter_parses_with_no_flag_or_every_flag_in_any_order() {
        let none = Command::Words(WordsAction::Meter(MeterArgs::default()));
        assert_eq!(meter(&[]), Ok(none));
        let every = MeterArgs {
            packs: Some("p".into()),
            held: Some("h".into()),
            chips: Some(5),
            lang: Some("fr".into()),
            speed: true,
        };
        let given = [
            "--lang", "fr", "--speed", "--chips", "5", "--held", "h", "--packs", "p",
        ];
        assert_eq!(meter(&given), Ok(Command::Words(WordsAction::Meter(every))));
    }

    #[test]
    fn a_repeated_unknown_or_empty_flag_or_a_bad_count_is_a_usage_error() {
        for bad in [
            &["--packs"][..],
            &["--packs", "--held"],
            &["--packs", "p", "--packs", "q"],
            &["--speed", "--speed"],
            &["--src", "w"],
            &["p"],
            &["--chips", "five"],
            &["--chips", "300"],
            &["--chips", "-1"],
        ] {
            assert_eq!(meter(bad), Err(UsageError), "{bad:?}");
        }
    }
}
