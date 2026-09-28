//! Shared core of the P1 spike faces. Throwaway: deleted at the end of P1.

pub mod backdrop;
pub mod capture;
pub mod clock;
pub mod com;
pub mod config;
pub mod facecfg;
pub mod fill;
pub mod hello;
pub mod hold;
pub mod inject;
pub mod kbctl;
pub mod kbgeom;
pub mod kbview;
pub mod langinfo;
pub mod langkey;
pub mod latch;
pub mod layout;
pub mod legend;
pub mod lines;
pub mod lookcfg;
pub mod lookview;
pub mod note;
pub mod paint;
pub mod place;
pub mod screen;
pub mod selwatch;
pub mod sizer;
pub mod snip;
pub mod status;
pub mod sysui;
pub mod targetlog;
pub mod theme;
pub mod typer;
pub mod uia;
pub mod uiaccess;
pub mod voice;
pub mod voicecfg;
pub mod voiceproto;
pub mod voiceworker;
pub mod weak;
pub mod window;

/// Scan-code prefix that marks an extended key (`0xE0xx`).
pub const EXTENDED: u32 = 0xE000;
/// Low byte of a scan code.
pub const SCAN_MASK: u32 = 0xFF;
/// Prefix byte of a scan code.
pub const PREFIX_MASK: u32 = 0xFF00;

/// True for an extended key (`0xE0xx`).
pub fn is_extended(code: u32) -> bool {
    code & PREFIX_MASK == EXTENDED
}

/// The scan code without its prefix byte.
pub fn scan_byte(code: u32) -> u32 {
    code & SCAN_MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_codes_split_into_prefix_and_byte() {
        assert!(is_extended(0xE01D));
        assert!(!is_extended(0x1D));
        assert!(!is_extended(0xE11D));
        assert_eq!(scan_byte(0xE035), 0x35);
        assert_eq!(scan_byte(0x2A), 0x2A);
    }
}
