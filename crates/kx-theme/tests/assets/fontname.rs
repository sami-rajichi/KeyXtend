//! Family names from a TrueType or OpenType font's `name` table, read without a font crate.

/// Name ids Qt matches a family by: the family (1) and the typographic family (16).
const FAMILY_IDS: [u16; 2] = [1, 16];
/// The Macintosh platform id, whose names are single bytes; other platforms use UTF-16.
const MAC: u16 = 1;
/// The `name` table's tag.
const NAME: &[u8] = b"name";
/// Where a font's table records start, and each record's size.
const TABLES: (usize, usize) = (12, 16);
/// Where a `name` table's records start, and each record's size.
const RECORDS: (usize, usize) = (6, 12);

/// The big-endian `u16` at `at`.
fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// The big-endian `u32` at `at`, as an offset or length.
fn u32_at(b: &[u8], at: usize) -> Option<usize> {
    usize::try_from(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?)).ok()
}

/// The `name` table of font `b`, if it has a whole one.
fn name_table(b: &[u8]) -> Option<&[u8]> {
    let (start, size) = TABLES;
    let count = usize::from(u16_at(b, 4)?);
    (0..count).find_map(|i| {
        let at = start + i * size;
        (b.get(at..at + 4)? == NAME).then_some(())?;
        let (off, len) = (u32_at(b, at + 8)?, u32_at(b, at + 12)?);
        b.get(off..off.checked_add(len)?)
    })
}

/// One name's text: single bytes on the Macintosh platform (family names are ASCII), UTF-16 elsewhere.
fn text(platform: u16, raw: &[u8]) -> String {
    if platform == MAC {
        return raw.iter().map(|&c| char::from(c)).collect();
    }
    let units: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_be_bytes(c))
        .collect();
    String::from_utf16_lossy(&units)
}

/// Every family name in font `b`; a damaged or foreign file gives none.
pub fn families(b: &[u8]) -> Vec<String> {
    let (start, size) = RECORDS;
    let Some(t) = name_table(b) else {
        return Vec::new();
    };
    let (Some(count), Some(strings)) = (u16_at(t, 2), u16_at(t, 4)) else {
        return Vec::new();
    };
    let name = |i: usize| {
        let r = start + i * size;
        let (platform, id) = (u16_at(t, r)?, u16_at(t, r + 6)?);
        let (len, off) = (u16_at(t, r + 8)?, u16_at(t, r + 10)?);
        let from = usize::from(strings) + usize::from(off);
        let raw = t.get(from..from + usize::from(len))?;
        FAMILY_IDS.contains(&id).then(|| text(platform, raw))
    };
    (0..usize::from(count)).filter_map(name).collect()
}

/// A font with one `name` table holding `names`, as (platform, name id, text).
fn sample(names: &[(u16, u16, &str)]) -> Vec<u8> {
    let encode = |p: u16, s: &str| -> Vec<u8> {
        if p == MAC {
            s.bytes().collect()
        } else {
            s.encode_utf16().flat_map(u16::to_be_bytes).collect()
        }
    };
    let texts: Vec<Vec<u8>> = names.iter().map(|&(p, _, s)| encode(p, s)).collect();
    let n = u16::try_from(names.len()).expect("a few names");
    let strings = RECORDS.0 + names.len() * RECORDS.1;
    let mut table: Vec<u8> = [0, n, u16::try_from(strings).expect("small")]
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect();
    let mut off = 0;
    for (&(p, id, _), t) in names.iter().zip(&texts) {
        let len = u16::try_from(t.len()).expect("short");
        table.extend(
            [p, 1, 0, id, len, off]
                .into_iter()
                .flat_map(u16::to_be_bytes),
        );
        off += len;
    }
    table.extend(texts.concat());
    let at = u32::try_from(TABLES.0 + TABLES.1).expect("small");
    let len = u32::try_from(table.len()).expect("small");
    let mut font = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    font.extend(NAME);
    font.extend([0, at, len].into_iter().flat_map(u32::to_be_bytes));
    font.extend(table);
    font
}

#[test]
fn a_font_names_its_family_and_typographic_family() {
    let font = sample(&[
        (3, 1, "Rubik Medium"),
        (3, 16, "Rubik"),
        (1, 1, "Rubik"),
        (3, 4, "x"),
    ]);
    assert_eq!(families(&font), ["Rubik Medium", "Rubik", "Rubik"]);
}

#[test]
fn a_damaged_font_gives_no_names() {
    let font = sample(&[(3, 1, "Rubik")]);
    for cut in [0, 5, 20, font.len() - 1] {
        assert!(families(&font[..cut]).is_empty(), "cut at {cut}");
    }
}
