//! `g19 translate-beside`: the translate read beside the keyboard the owner started, the uiAccess dev install. Windows
//! hides a uiAccess window from a plain UI Automation client, so it runs only from the installed uiAccess kx-gates.
#![cfg(windows)]

use std::path::Path;

use kx_platform_windows::uiaccess;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g19chips::read::{BESIDE, line, read};
use crate::product::Keyboard;
use crate::{clipkeep, win};

/// Refuses the step unless kx-gates has uiAccess, as `own` says, with the reason.
fn from_ui_access(own: Result<bool, String>) -> Result<(), String> {
    match own {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!(
            "{BESIDE} runs only from the installed uiAccess kx-gates: Windows hides the uiAccess keyboard from a \
             plain UI Automation client"
        )),
        Err(e) => Err(format!("kx-gates cannot tell whether it has uiAccess: {e}")),
    }
}

/// Runs the step next to the keyboard the owner started, starting nothing; a plain kx-gates is refused at once.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    from_ui_access(uiaccess::check().map_err(|e| e.to_string()))?;
    let kb = Keyboard::find()?;
    let (pid, program) = (win::owner(kb.hwnd).0, win::program(kb.hwnd)?);
    let got = clipkeep::keep_changed(ctx.cfg, || read(ctx, pid, Path::new(&program)));
    let run = got.unwrap_or_else(|e| json!({ "ok": false, "error": e }));
    Ok(line(BESIDE, &run, Ok(())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_beside_runs_only_from_a_ui_access_kx_gates() {
        assert_eq!(from_ui_access(Ok(true)), Ok(()));
        let plain = from_ui_access(Ok(false)).expect_err("a plain kx-gates");
        assert!(plain.contains("installed uiAccess kx-gates"), "{plain}");
        let unsure = from_ui_access(Err("no token".into())).expect_err("unsure");
        assert!(unsure.contains("no token"), "{unsure}");
    }
}
