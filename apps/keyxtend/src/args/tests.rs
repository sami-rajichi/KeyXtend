use super::*;

/// `list` as command-line arguments.
fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

#[test]
fn nothing_or_show_shows_and_quit_quits() {
    assert_eq!(parse(args(&[])), Ok(Command::Show));
    assert_eq!(parse(args(&["--show"])), Ok(Command::Show));
    assert_eq!(parse(args(&["--quit"])), Ok(Command::Quit));
}

#[test]
fn each_flag_reads_back_as_its_command() {
    for command in COMMANDS {
        assert_eq!(parse([OsString::from(flag(command))]), Ok(command));
    }
    assert_eq!(flag(Command::Show), "--show");
    assert_eq!(flag(Command::Quit), "--quit");
}

#[test]
fn every_command_has_a_flag() {
    // Exhaustive on purpose: a new command stops this test building until `COMMANDS` lists it.
    let listed = |c: Command| match c {
        Command::Show | Command::Quit => COMMANDS.contains(&c),
    };
    assert!(listed(Command::Show) && listed(Command::Quit));
}

#[test]
fn anything_else_is_refused_and_named() {
    for bad in [
        "show", "-show", "--Show", "--show ", "--exit", "--", "", "/quit", "--quit=1",
    ] {
        let refused = parse(args(&[bad]));
        assert_eq!(refused, Err(ArgError::Unknown(bad.to_owned())), "{bad:?}");
    }
    let said = ArgError::Unknown("--exit".into()).to_string();
    assert!(said.contains("\"--exit\""), "{said}");
    assert!(said.contains("--show") && said.contains("--quit"), "{said}");
}

#[test]
fn a_qt_option_never_passes() {
    for qt in [
        "-platformpluginpath",
        "-qmljsdebugger=port:1234",
        "-platform",
    ] {
        assert!(
            matches!(parse(args(&[qt])), Err(ArgError::Unknown(_))),
            "{qt}"
        );
    }
}

#[test]
fn two_arguments_are_refused() {
    assert_eq!(parse(args(&["--show", "--quit"])), Err(ArgError::Many));
    assert_eq!(parse(args(&["--show", "--show"])), Err(ArgError::Many));
    let said = ArgError::Many.to_string();
    assert!(said.contains("--show") && said.contains("--quit"), "{said}");
}

#[cfg(windows)]
#[test]
fn an_argument_that_is_not_unicode_is_refused() {
    use std::os::windows::ffi::OsStringExt;
    let lone_surrogate = OsString::from_wide(&[0xD800]);
    assert!(matches!(parse([lone_surrogate]), Err(ArgError::Unknown(_))));
}

proptest::proptest! {
    #[test]
    fn only_the_two_flags_pass(text in r"\PC{0,16}") {
        let got = parse([OsString::from(&text)]);
        let known = text == "--show" || text == "--quit";
        proptest::prop_assert_eq!(got.is_ok(), known, "{:?}", text);
    }
}
