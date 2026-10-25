use proptest::prelude::*;

use super::*;

/// Where the test image puts its parts: section start in the file and in memory, tables and names.
const RAW: usize = 0x200;
const VA: usize = 0x1000;
const IMPORT_TABLE: usize = 0x200;
const DELAY_TABLE: usize = 0x280;
const IMPORT_NAMES: usize = 0x300;
const DELAY_NAMES: usize = 0x380;
const NAME_GAP: usize = 0x20;
const IMAGE_LEN: usize = 0x400;
/// Data directories in the test image, as linkers write them.
const DIR_COUNT: usize = 16;

fn put16(b: &mut [u8], at: usize, v: u16) {
    b[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn put32(b: &mut [u8], at: usize, v: usize) {
    let v = u32::try_from(v).unwrap();
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// The address in memory of file offset `at` inside the one section.
fn rva(at: usize) -> usize {
    at - RAW + VA
}

/// Writes one table of descriptors `len` bytes long and their names; `flags` marks delay-load descriptors.
fn table(b: &mut [u8], at: usize, names_at: usize, t: &Table, libs: &[&str], flags: u32) {
    for (i, lib) in libs.iter().enumerate() {
        let desc = at + i * t.len;
        let name = names_at + i * NAME_GAP;
        if t.flags_at.is_some() {
            put32(b, desc, usize::try_from(flags).unwrap());
        }
        put32(b, desc + t.name_at, rva(name));
        b[name..name + lib.len()].copy_from_slice(lib.as_bytes());
    }
}

/// Writes the one section's header at `sec`: it maps the file from `RAW` on to memory at `VA`.
fn section(b: &mut [u8], sec: usize) {
    let fields = [
        (V_SIZE_AT, RAW),
        (VA_AT, VA),
        (RAW_SIZE_AT, RAW),
        (RAW_AT, RAW),
    ];
    for (at, v) in fields {
        put32(b, sec + at, v);
    }
}

/// A small PE image with one section holding `imports` and delay-loaded `delayed`; `wide` picks PE32+.
fn image(imports: &[&str], delayed: &[&str], wide: bool, flags: u32) -> Vec<u8> {
    let mut b = vec![0u8; IMAGE_LEN];
    b[..2].copy_from_slice(DOS_MAGIC);
    let pe = 0x40;
    put32(&mut b, PE_AT, pe);
    b[pe..pe + 4].copy_from_slice(PE_MAGIC);
    let coff = pe + PE_MAGIC.len();
    let (magic, dirs) = if wide {
        (PE64, DIRS_AT_64)
    } else {
        (PE32, DIRS_AT_32)
    };
    let opt_len = dirs + DIR_COUNT * DIR_LEN;
    put16(&mut b, coff + SECTIONS_AT, 1);
    put16(&mut b, coff + OPT_SIZE_AT, u16::try_from(opt_len).unwrap());
    let opt = coff + COFF_LEN;
    put16(&mut b, opt, magic);
    put32(&mut b, opt + dirs - DIR_COUNT_LEN, DIR_COUNT);
    put32(
        &mut b,
        opt + dirs + IMPORTS.index * DIR_LEN,
        rva(IMPORT_TABLE),
    );
    put32(
        &mut b,
        opt + dirs + DELAYED.index * DIR_LEN,
        rva(DELAY_TABLE),
    );
    section(&mut b, opt + opt_len);
    table(&mut b, IMPORT_TABLE, IMPORT_NAMES, &IMPORTS, imports, flags);
    table(&mut b, DELAY_TABLE, DELAY_NAMES, &DELAYED, delayed, flags);
    b
}

#[test]
fn imports_then_delayed_imports_are_read() {
    let b = image(&["A.dll", "B.dll"], &["C.dll"], true, RVA_FLAG);
    assert_eq!(
        imports(&b),
        Some(vec!["A.dll".into(), "B.dll".into(), "C.dll".into()])
    );
}

#[test]
fn a_32_bit_image_is_read_too() {
    let b = image(&["A.dll"], &[], false, RVA_FLAG);
    assert_eq!(imports(&b), Some(vec!["A.dll".to_string()]));
}

#[test]
fn an_image_without_tables_imports_nothing() {
    let mut b = image(&[], &[], true, RVA_FLAG);
    let dirs = 0x40 + PE_MAGIC.len() + COFF_LEN + DIRS_AT_64;
    put32(&mut b, dirs + IMPORTS.index * DIR_LEN, 0);
    put32(&mut b, dirs + DELAYED.index * DIR_LEN, 0);
    assert_eq!(imports(&b), Some(Vec::new()));
}

#[test]
fn the_bare_image_imports_nothing() {
    assert_eq!(imports(&bare_image()), Some(Vec::new()));
}

#[test]
fn an_old_style_delay_table_is_refused() {
    let b = image(&[], &["C.dll"], true, 0);
    assert_eq!(imports(&b), None);
}

#[test]
fn text_is_not_an_image() {
    assert_eq!(imports(b"MZ is not enough"), None);
    assert_eq!(imports(b""), None);
}

#[test]
fn every_cut_of_an_image_reads_without_panicking() {
    let b = image(&["A.dll"], &["C.dll"], true, RVA_FLAG);
    for len in 0..b.len() {
        let _ = imports(&b[..len]);
    }
    assert_eq!(imports(&b[..RAW]), None);
}

#[test]
fn read_names_a_file_that_is_not_an_image() {
    let dir = crate::test_support::TempDir::new("pe");
    let path = dir.path().join("x.dll");
    std::fs::write(&path, b"not a program").unwrap();
    assert_eq!(
        read(&path),
        Err(DevError::Stage(StageError::BadPe(paths::text(&path))))
    );
}

#[cfg(windows)]
#[test]
fn the_running_test_imports_kernel32() {
    let names = read(&std::env::current_exe().unwrap()).unwrap();
    assert!(names.iter().any(|n| n.eq_ignore_ascii_case("kernel32.dll")));
}

proptest! {
    #[test]
    fn any_bytes_read_without_panicking(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let _ = imports(&bytes);
    }

    #[test]
    fn any_changed_byte_reads_without_panicking(at in 0..IMAGE_LEN, value in any::<u8>()) {
        let mut b = image(&["A.dll"], &["C.dll"], true, RVA_FLAG);
        b[at] = value;
        let _ = imports(&b);
    }
}
