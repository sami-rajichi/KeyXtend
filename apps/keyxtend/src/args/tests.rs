use super::*;

/// `list` as command-line arguments.
fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

/// `command` with the hand's input.
fn hand(command: Command) -> (Command, Input) {
    (command, Input::Hand)
}

#[test]
fn nothing_or_show_shows_and_quit_quits() {
    assert_eq!(parse(args(&[])), Ok(hand(Command::Show)));
    assert_eq!(parse(args(&["--show"])), Ok(hand(Command::Show)));
    assert_eq!(parse(args(&["--quit"])), Ok(hand(Command::Quit)));
}

#[test]
fn each_flag_reads_back_as_its_command() {
    for command in COMMANDS {
        assert_eq!(parse([OsString::from(flag(command))]), Ok(hand(command)));
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

#[cfg(not(feature = "gate-input"))]
#[test]
fn without_the_gate_build_the_gate_flag_is_unknown() {
    let refused = Err(ArgError::Unknown(SIM_INPUT.to_owned()));
    assert_eq!(parse(args(&[SIM_INPUT])), refused);
    assert_eq!(parse(args(&[SIM_INPUT, "--show"])), Err(ArgError::Many));
}

#[cfg(feature = "gate-input")]
#[test]
fn the_gate_build_takes_the_gate_flag_alone_or_before_show() {
    let sim = Ok((Command::Show, Input::Sim));
    assert_eq!(parse(args(&[SIM_INPUT])), sim);
    assert_eq!(parse(args(&[SIM_INPUT, "--show"])), sim);
    assert_eq!(parse(args(&["--show"])), Ok(hand(Command::Show)));
    assert_eq!(parse(args(&["--quit"])), Ok(hand(Command::Quit)));
}

#[cfg(feature = "gate-input")]
#[test]
fn the_gate_build_still_refuses_anything_else() {
    for many in [
        &[SIM_INPUT, "--quit"][..],
        &["--show", SIM_INPUT],
        &[SIM_INPUT, SIM_INPUT],
        &[SIM_INPUT, "--show", "--show"],
        &[SIM_INPUT, "-platform"],
    ] {
        assert_eq!(parse(args(many)), Err(ArgError::Many), "{many:?}");
    }
    for bad in ["--sim-input=1", "--Sim-Input", "-sim-input", "--sim-input "] {
        let refused = Err(ArgError::Unknown(bad.to_owned()));
        assert_eq!(parse(args(&[bad])), refused, "{bad:?}");
    }
}

proptest::proptest! {
    #[test]
    fn only_the_known_flags_pass(text in r"\PC{0,16}") {
        let got = parse([OsString::from(&text)]);
        let gate = cfg!(feature = "gate-input") && text == SIM_INPUT;
        let known = text == "--show" || text == "--quit" || gate;
        proptest::prop_assert_eq!(got.is_ok(), known, "{:?}", text);
    }
}

/// A clip as the voice gate build takes it: an absolute path.
const CLIP: &str = r"D:\clips\check-en.wav";

#[cfg(not(feature = "gate-voice"))]
#[test]
fn without_the_voice_gate_build_the_sound_flag_is_refused() {
    let flag = kx_ipc::SOUND_FLAG;
    assert_eq!(parse(args(&[flag, CLIP])), Err(ArgError::Many));
    let unknown = Err(ArgError::Unknown(flag.to_owned()));
    assert_eq!(parse(args(&[flag])), unknown);
}

#[cfg(feature = "gate-voice")]
#[test]
fn the_voice_gate_build_shows_with_the_hands_input_and_names_its_clip() {
    let line = args(&[SOUND_FLAG, CLIP]);
    assert_eq!(parse(line.clone()), Ok(hand(Command::Show)));
    assert_eq!(clip(&line), Some(std::path::PathBuf::from(CLIP)));
    assert_eq!(clip(&args(&["--show"])), None);
    assert_eq!(clip(&args(&[])), None);
}

#[cfg(feature = "gate-voice")]
#[test]
fn the_voice_gate_build_takes_one_absolute_clip_after_its_flag_only() {
    for bad in [
        &[SOUND_FLAG][..],
        &[SOUND_FLAG, "clip.wav"],
        &[SOUND_FLAG, CLIP, CLIP],
        &[CLIP, SOUND_FLAG],
        &["--show", SOUND_FLAG, CLIP],
    ] {
        assert!(parse(args(bad)).is_err(), "{bad:?}");
        assert_eq!(clip(&args(bad)), None, "{bad:?}");
    }
}

#[test]
fn read_gives_the_command_and_the_flags_beside_it() {
    let (command, flags) = read(args(&["--quit"])).expect("quit reads");
    assert_eq!((command, flags.input), hand(Command::Quit));
    assert_eq!(
        read(args(&["--quit", "--show"])).err(),
        Some(ArgError::Many)
    );
    #[cfg(feature = "gate-voice")]
    {
        assert_eq!(flags.clip, None);
        let (command, flags) = read(args(&[SOUND_FLAG, CLIP])).expect("a clip reads");
        assert_eq!(command, Command::Show);
        assert_eq!(flags.clip, Some(std::path::PathBuf::from(CLIP)));
    }
}
