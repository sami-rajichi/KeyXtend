//! The Win32 side of the link: Explorer's desktop shell object opens it in the default browser, through
//! `crate::explorer`, as the reveal opens a folder.

use kx_module_api::ports::{Link, LinkError, WebLink};
use windows::core::BSTR;

use super::{Os, open};
use crate::explorer::Explorer;

/// Opens links in the user's default browser: the running Explorer starts it (Raymond Chen's "launch from the shell"
/// route), so the browser runs with Explorer's token and never inherits our uiAccess.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinWebLink;

impl WebLink for WinWebLink {
    fn open(&self, link: &Link) -> Result<(), LinkError> {
        open(&Win32, link.expose())
    }
}

/// The Win32 calls behind `WinWebLink`; no test reaches them, as they would open a browser on the owner's screen.
struct Win32;

impl Os for Win32 {
    type Shell = Explorer;

    fn shell(&self) -> Result<Explorer, i32> {
        Explorer::find()
    }

    fn open(&self, explorer: &Explorer, link: &str) -> Result<(), i32> {
        explorer.open(&BSTR::from(link))
    }
}
