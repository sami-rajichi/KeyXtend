use super::super::{Command, parse as parse_all};
use super::*;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

fn sounds(action: SoundsAction) -> Command {
    Command::Sounds(action)
}

fn import_of(start_ms: Option<u32>, len_ms: Option<u32>) -> Command {
    sounds(SoundsAction::Import {
        input: "in.wav".into(),
        output: "out.wav".into(),
        start_ms,
        len_ms,
    })
}

#[test]
fn render_parses_with_and_without_a_folder() {
    assert_eq!(
        parse_all(&args(&["sounds", "render"])),
        Ok(sounds(SoundsAction::Render { out: None }))
    );
    assert_eq!(
        parse_all(&args(&["sounds", "render", "--out", "x"])),
        Ok(sounds(SoundsAction::Render {
            out: Some("x".into())
        }))
    );
}

#[test]
fn render_with_a_bad_flag_or_no_folder_is_a_usage_error() {
    for bad in [
        &["sounds"][..],
        &["sounds", "render", "--out"],
        &["sounds", "render", "--dir", "x"],
        &["sounds", "render", "--out", "--len-ms"],
        &["sounds", "render", "--out", "x", "y"],
    ] {
        assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
    }
}

#[test]
fn import_parses_its_paths_and_flags_in_any_order() {
    let base = ["sounds", "import", "in.wav", "out.wav"];
    assert_eq!(parse_all(&args(&base)), Ok(import_of(None, None)));
    let both = [&base[..], &["--len-ms", "80", "--start-ms", "12"]].concat();
    assert_eq!(parse_all(&args(&both)), Ok(import_of(Some(12), Some(80))));
    let start = [&base[..], &["--start-ms", "0"]].concat();
    assert_eq!(parse_all(&args(&start)), Ok(import_of(Some(0), None)));
}

#[test]
fn import_with_bad_flags_is_a_usage_error() {
    let base = ["sounds", "import", "in.wav", "out.wav"];
    for extra in [
        &["--start-ms"][..],
        &["--start-ms", "-5"],
        &["--start-ms", "ten"],
        &["--start-ms", "1", "--start-ms", "2"],
        &["--fade-ms", "5"],
    ] {
        let all = [&base[..], extra].concat();
        assert_eq!(parse_all(&args(&all)), Err(UsageError), "{extra:?}");
    }
}

#[test]
fn import_needs_two_paths_that_are_not_flags() {
    for bad in [
        &["sounds", "import"][..],
        &["sounds", "import", "in.wav"],
        &["sounds", "import", "--start-ms", "5", "out.wav"],
        &["sounds", "import", "in.wav", "--len-ms", "5"],
    ] {
        assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
    }
}

#[test]
fn join_parses_three_paths_and_the_start_of_the_second() {
    let line = ["sounds", "join", "a.wav", "b.wav", "c.wav", "--at-ms", "95"];
    assert_eq!(
        parse_all(&args(&line)),
        Ok(sounds(SoundsAction::Join {
            first: "a.wav".into(),
            second: "b.wav".into(),
            output: "c.wav".into(),
            at_ms: 95,
        }))
    );
}

#[test]
fn join_without_a_good_start_or_three_paths_is_a_usage_error() {
    for bad in [
        &["sounds", "join", "a.wav", "b.wav", "c.wav"][..],
        &["sounds", "join", "a.wav", "b.wav", "c.wav", "--at-ms"],
        &["sounds", "join", "a.wav", "b.wav", "c.wav", "--at-ms", "-1"],
        &["sounds", "join", "a.wav", "b.wav", "c.wav", "--len-ms", "9"],
        &["sounds", "join", "a.wav", "b.wav", "--at-ms", "9"],
        &["sounds", "join", "a.wav", "--x", "c.wav", "--at-ms", "9"],
        &[
            "sounds", "join", "a.wav", "b.wav", "c.wav", "--at-ms", "9", "x",
        ],
    ] {
        assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
    }
}

#[test]
fn usage_names_every_action_and_flag() {
    let message = UsageError.to_string();
    for name in [
        RENDER,
        IMPORT,
        JOIN,
        FLAG_OUT,
        FLAG_START_MS,
        FLAG_LEN_MS,
        FLAG_AT_MS,
    ] {
        assert!(message.contains(name), "usage message missing {name}");
    }
}
