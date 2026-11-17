//! Only kx-gates built with `voice` holds kx-ipc's sound flag, so the default build that `dev-install` signs passes its
//! gate-flag check and a `voice` build fails it.
#![cfg(windows)]

#[test]
fn only_the_voice_build_holds_the_sound_flag() {
    let exe = std::fs::read(env!("CARGO_BIN_EXE_kx-gates")).expect("the built kx-gates");
    let flag = kx_ipc::SOUND_FLAG.as_bytes();
    let held = exe.windows(flag.len()).any(|w| w == flag);
    assert_eq!(held, cfg!(feature = "voice"));
}
