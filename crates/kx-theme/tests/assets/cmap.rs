//! Whether a TrueType or OpenType font maps a character, from its `cmap` table's Unicode subtables (formats 4 and 12),
//! read without a font crate.

/// The `cmap` table's tag.
const CMAP: &[u8] = b"cmap";
/// Where a font's table records start, and each record's size.
const TABLES: (usize, usize) = (12, 16);
/// Where a `cmap` table's encoding records start, and each record's size.
const ENCODINGS: (usize, usize) = (4, 8);
/// Unicode encodings, as (platform, encoding): any Unicode platform one, Windows BMP and Windows full repertoire.
const UNICODE: [(u16, Option<u16>); 3] = [(0, None), (3, Some(1)), (3, Some(10))];
/// The subtable formats read: segments over the BMP, and groups over all of Unicode.
const SEGMENTS: u16 = 4;
const GROUPS: u16 = 12;

/// The big-endian `u16` at `at`.
fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// The big-endian `u32` at `at`.
fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The big-endian `u32` at `at`, as an offset or a length.
fn len_at(b: &[u8], at: usize) -> Option<usize> {
    usize::try_from(u32_at(b, at)?).ok()
}

/// The `cmap` table of font `b`, from its offset to the font's end.
fn cmap(b: &[u8]) -> Option<&[u8]> {
    let (start, size) = TABLES;
    let count = usize::from(u16_at(b, 4)?);
    (0..count).find_map(|i| {
        let at = start + i * size;
        (b.get(at..at + 4)? == CMAP).then_some(())?;
        b.get(len_at(b, at + 8)?..)
    })
}

/// The glyph of `c` in a format 4 subtable `t`; 0 when it has none.
fn by_segments(t: &[u8], c: u32) -> Option<u16> {
    let c = u16::try_from(c).ok()?;
    let segs = usize::from(u16_at(t, 6)? / 2);
    let ends = 14;
    let (starts, deltas, ranges) = (
        ends + 2 * segs + 2,
        ends + 4 * segs + 2,
        ends + 6 * segs + 2,
    );
    let i = (0..segs).find(|&i| u16_at(t, ends + 2 * i).is_some_and(|end| end >= c))?;
    let first = u16_at(t, starts + 2 * i)?;
    if first > c {
        return Some(0);
    }
    let (delta, range) = (u16_at(t, deltas + 2 * i)?, u16_at(t, ranges + 2 * i)?);
    if range == 0 {
        return Some(c.wrapping_add(delta));
    }
    let at = ranges + 2 * i + usize::from(range) + 2 * usize::from(c - first);
    let glyph = u16_at(t, at)?;
    Some(if glyph == 0 {
        0
    } else {
        glyph.wrapping_add(delta)
    })
}

/// The glyph of `c` in a format 12 subtable `t`; 0 when it has none.
fn by_groups(t: &[u8], c: u32) -> Option<u32> {
    let count = len_at(t, 12)?;
    let group = |i: usize| {
        let at = 16 + 12 * i;
        Some((u32_at(t, at)?, u32_at(t, at + 4)?, u32_at(t, at + 8)?))
    };
    let hit = (0..count)
        .map_while(group)
        .find(|&(first, last, _)| (first..=last).contains(&c));
    Some(hit.map_or(0, |(first, _, glyph)| glyph.saturating_add(c - first)))
}

/// True when font `b` maps `c` to a glyph in any Unicode subtable it has; a damaged or foreign file maps nothing.
pub fn maps(b: &[u8], c: char) -> bool {
    let Some(table) = cmap(b) else {
        return false;
    };
    let (start, size) = ENCODINGS;
    let count = usize::from(u16_at(table, 2).unwrap_or_default());
    let unicode = |p: u16, e: u16| {
        UNICODE
            .iter()
            .any(|&(up, ue)| up == p && ue.is_none_or(|ue| ue == e))
    };
    (0..count).any(|i| {
        let at = start + i * size;
        let found = || {
            let (p, e) = (u16_at(table, at)?, u16_at(table, at + 2)?);
            let sub = table.get(len_at(table, at + 4)?..)?;
            let glyph = match (unicode(p, e), u16_at(sub, 0)?) {
                (true, SEGMENTS) => u32::from(by_segments(sub, u32::from(c))?),
                (true, GROUPS) => by_groups(sub, u32::from(c))?,
                _ => 0,
            };
            Some(glyph != 0)
        };
        found().unwrap_or(false)
    })
}

/// A font with one `cmap` table: a Windows BMP subtable in format 4 mapping `from..=to` to glyphs from 1, with the
/// closing segment every format 4 table ends with.
fn sample(from: u16, to: u16) -> Vec<u8> {
    let delta = 1u16.wrapping_sub(from);
    let words: [u16; 16] = [
        SEGMENTS, 32, 0, 4, 4, 1, 0, // format, length, language, segCountX2, search fields
        to, 0xFFFF, 0, // end codes, then the pad
        from, 0xFFFF, // start codes
        delta, 1, // deltas
        0, 0, // range offsets
    ];
    let sub: Vec<u8> = words.into_iter().flat_map(u16::to_be_bytes).collect();
    let mut table: Vec<u8> = [0u16, 1, 3, 1]
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect();
    table.extend(12u32.to_be_bytes());
    table.extend(sub);
    let at = u32::try_from(TABLES.0 + TABLES.1).unwrap_or_default();
    let len = u32::try_from(table.len()).unwrap_or_default();
    let mut font = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    font.extend(CMAP);
    font.extend([0, at, len].into_iter().flat_map(u32::to_be_bytes));
    font.extend(table);
    font
}

#[test]
fn a_font_maps_the_characters_of_its_segments_only() {
    let font = sample(0x25CC, 0x25CC);
    assert!(maps(&font, '\u{25CC}'));
    assert!(!maps(&font, '\u{25CB}') && !maps(&font, 'a'));
    assert!(!maps(&font, '\u{1F600}'), "past the BMP");
}

#[test]
fn a_font_cut_before_its_segments_maps_nothing() {
    let font = sample(0x25CC, 0x25CC);
    for cut in [0, 5, 30, 50] {
        assert!(!maps(&font[..cut], '\u{25CC}'), "cut at {cut}");
    }
}
