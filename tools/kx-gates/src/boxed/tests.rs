use super::*;

fn of(words: &[&str]) -> Result<Boxed, String> {
    parse(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn takes_the_cap_the_program_and_its_arguments_in_order() {
    let b = of(&["--mb", "4096", r"C:\tools\probe.exe", "mic", "3"]).expect("valid");
    assert_eq!(b.memory_mb, 4096);
    assert_eq!(b.exe, PathBuf::from(r"C:\tools\probe.exe"));
    assert_eq!(b.args, ["mic", "3"]);
}

#[test]
fn a_program_with_no_arguments_is_fine() {
    let b = of(&["--mb", "64", r"C:\tools\probe.exe"]).expect("valid");
    assert!(b.args.is_empty());
}

#[test]
fn the_cap_is_required_and_must_be_a_whole_number() {
    assert!(of(&[r"C:\tools\probe.exe"]).is_err());
    assert!(of(&["--mb", "lots", r"C:\tools\probe.exe"]).is_err());
    assert!(of(&["--mb", "0", r"C:\tools\probe.exe"]).is_err());
    assert!(of(&["--mb"]).is_err());
}

#[test]
fn a_relative_or_missing_program_is_refused() {
    assert!(of(&["--mb", "64", "probe.exe"]).is_err());
    assert!(of(&["--mb", "64"]).is_err());
    assert!(of(&[]).is_err());
}

#[test]
fn a_proofs_word_is_no_program() {
    assert!(of(&["https"]).is_err());
    assert!(of(&["folder"]).is_err());
}
