//! The `--layout` switch: an installed layout the window types into, so a gate knows the layout.

use kx_module_api::ports::{CodeError, LayoutId};
use thiserror::Error;

/// Why the window could not take the layout it was given.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutError {
    /// The switch has no layout id after it.
    #[error("{0} needs a layout id after it")]
    Missing(String),
    /// The text after the switch is not a layout id.
    #[error(transparent)]
    Bad(#[from] CodeError),
    /// The layout is not installed on this PC; the window never installs one.
    #[error("layout {0} is not installed")]
    NotInstalled(LayoutId),
    /// Windows did not switch the window's thread to the layout.
    #[error("Windows did not switch to layout {0}")]
    Refused(LayoutId),
}

/// The layout named after switch `flag` in `args`; `None` when the switch is absent.
///
/// # Errors
/// `Missing` when nothing follows the switch, `Bad` when the text is not a `0x` hex layout id.
pub fn parse(args: &[String], flag: &str) -> Result<Option<LayoutId>, LayoutError> {
    let Some(at) = args.iter().position(|a| a == flag) else {
        return Ok(None);
    };
    let text = args
        .get(at + 1)
        .ok_or_else(|| LayoutError::Missing(flag.to_owned()))?;
    Ok(Some(text.parse()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAG: &str = "--layout";

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn without_the_switch_there_is_no_layout() {
        assert_eq!(parse(&args(""), FLAG), Ok(None));
        assert_eq!(parse(&args("--password"), FLAG), Ok(None));
    }

    #[test]
    fn the_id_after_the_switch_is_read_as_hex() {
        let fr = LayoutId::new(0x040C_040C);
        assert_eq!(parse(&args("--layout 0x040C040C"), FLAG), Ok(Some(fr)));
        let ar = LayoutId::new(0xFFFF_FFFF_F029_1C01);
        let text = "--password --layout 0xFFFFFFFFF0291C01";
        assert_eq!(parse(&args(text), FLAG), Ok(Some(ar)));
    }

    #[test]
    fn a_switch_without_an_id_is_missing_one() {
        let err = parse(&args("--layout"), FLAG).unwrap_err();
        assert_eq!(err, LayoutError::Missing(FLAG.into()));
        assert!(err.to_string().contains(FLAG), "{err}");
    }

    #[test]
    fn text_that_is_not_an_id_is_refused() {
        for bad in ["--layout 040C040C", "--layout 0xZZ", "--layout --password"] {
            let err = parse(&args(bad), FLAG).unwrap_err();
            assert!(matches!(err, LayoutError::Bad(_)), "{bad}: {err}");
        }
    }

    #[test]
    fn each_error_names_the_layout() {
        let id = LayoutId::new(0x0419_0419);
        for err in [LayoutError::NotInstalled(id), LayoutError::Refused(id)] {
            assert!(err.to_string().contains("0x0000000004190419"), "{err}");
        }
    }
}
