//! A derived face for inline code: IBM Plex Mono Regular with a larger em.
//!
//! GPUI lays a paragraph out at one font size (`TextRun` and
//! `HighlightStyle` carry no size), but prose sets inline code smaller than
//! the text around it (`.g-prose code, .t3-amsg-body code { font-size:
//! 0.8125rem }` in 14px prose). Enlarging the em (`head.unitsPerEm`) while
//! keeping every outline and metric in font units shrinks the drawn glyphs
//! by the same ratio, so a run in this face inside a 14px paragraph draws
//! 13px Plex Mono on the paragraph's baseline and still wraps with it.
//!
//! The face is built in memory at startup from the bundled Plex Mono. Under
//! the SIL OFL a modified font may not use the reserved name, so it is
//! renamed; its copyright and license records are kept.
//!
//! One more change: the narrow no-break space (U+202F) gets the advance of
//! the inline padding, so `padding: 1px 4px` can be written as one such
//! space on each side of the code and never breaks from it.

/// Build the derived face from a TrueType font: em `units_per_em`, family
/// `family`, U+202F advance `pad_advance` (font units). `None` when the
/// source is not a font this reader understands.
pub fn derive(src: &[u8], units_per_em: u16, family: &str, pad_advance: u16) -> Option<Vec<u8>> {
    let tables = read_tables(src)?;
    let find = |tag: &[u8; 4]| tables.iter().find(|t| &t.0 == tag).map(|t| t.1);
    let hhea = find(b"hhea")?;
    let num_h_metrics = be16(hhea, 34)? as usize;
    let pad_glyph = cmap_lookup(find(b"cmap")?, 0x202F);
    let mut out: Vec<([u8; 4], Vec<u8>)> = Vec::new();
    for (tag, data) in &tables {
        let mut data = data.to_vec();
        match tag {
            // The signature cannot survive the edit.
            b"DSIG" => continue,
            b"head" => {
                // checkSumAdjustment is recomputed below; unitsPerEm at 18.
                data.get_mut(8..12)?.copy_from_slice(&[0; 4]);
                data.get_mut(18..20)?
                    .copy_from_slice(&units_per_em.to_be_bytes());
            }
            b"hmtx" => {
                if let Some(g) = pad_glyph.filter(|&g| (g as usize) < num_h_metrics) {
                    let at = g as usize * 4;
                    data.get_mut(at..at + 2)?
                        .copy_from_slice(&pad_advance.to_be_bytes());
                }
            }
            b"name" => data = rename(&data, family)?,
            _ => {}
        }
        out.push((*tag, data));
    }
    Some(write_font(be32(src, 0)?, out))
}

fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The sfnt table directory: (tag, data) in file order.
fn read_tables(src: &[u8]) -> Option<Vec<([u8; 4], &[u8])>> {
    let n = be16(src, 4)? as usize;
    (0..n)
        .map(|i| {
            let rec = 12 + 16 * i;
            let tag: [u8; 4] = src.get(rec..rec + 4)?.try_into().ok()?;
            let off = be32(src, rec + 8)? as usize;
            let len = be32(src, rec + 12)? as usize;
            Some((tag, src.get(off..off + len)?))
        })
        .collect()
}

/// The glyph for `ch` from the cmap's Unicode BMP (format 4) subtable.
pub(super) fn cmap_lookup(cmap: &[u8], ch: u16) -> Option<u16> {
    let n = be16(cmap, 2)? as usize;
    for i in 0..n {
        let rec = 4 + 8 * i;
        let (platform, encoding) = (be16(cmap, rec)?, be16(cmap, rec + 2)?);
        if !matches!((platform, encoding), (3, 1) | (0, 3) | (0, 4)) {
            continue;
        }
        let s = cmap.get(be32(cmap, rec + 4)? as usize..)?;
        if be16(s, 0)? != 4 {
            continue;
        }
        let segs = be16(s, 6)? as usize / 2;
        let (ends, starts) = (14, 16 + 2 * segs);
        let (deltas, ranges) = (16 + 4 * segs, 16 + 6 * segs);
        for k in 0..segs {
            if ch > be16(s, ends + 2 * k)? || ch < be16(s, starts + 2 * k)? {
                continue;
            }
            let delta = be16(s, deltas + 2 * k)?;
            let range = be16(s, ranges + 2 * k)? as usize;
            if range == 0 {
                return Some(ch.wrapping_add(delta));
            }
            let start = be16(s, starts + 2 * k)?;
            let at = ranges + 2 * k + range + 2 * (ch - start) as usize;
            let g = be16(s, at)?;
            return (g != 0).then(|| g.wrapping_add(delta));
        }
    }
    None
}

/// A name table that keeps every record except the family, unique, full
/// and PostScript names (1, 3, 4, 6, 16, 17), which become `family`.
fn rename(name: &[u8], family: &str) -> Option<Vec<u8>> {
    let count = be16(name, 2)? as usize;
    let storage = be16(name, 4)? as usize;
    let replaced = [1u16, 3, 4, 6, 16, 17];
    let postscript: String = family.chars().filter(|c| !c.is_whitespace()).collect();
    let mut records: Vec<(u16, u16, u16, u16, Vec<u8>)> = Vec::new();
    for i in 0..count {
        let rec = 6 + 12 * i;
        let id = be16(name, rec + 6)?;
        if replaced.contains(&id) {
            continue;
        }
        let len = be16(name, rec + 8)? as usize;
        let off = storage + be16(name, rec + 10)? as usize;
        records.push((
            be16(name, rec)?,
            be16(name, rec + 2)?,
            be16(name, rec + 4)?,
            id,
            name.get(off..off + len)?.to_vec(),
        ));
    }
    for (id, text) in [(1, family), (3, family), (4, family), (6, &postscript)] {
        // Macintosh Roman (ASCII here) and Windows Unicode English.
        records.push((1, 0, 0, id, text.as_bytes().to_vec()));
        let utf16 = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        records.push((3, 1, 0x409, id, utf16));
    }
    records.sort_by_key(|r| (r.0, r.1, r.2, r.3));
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&(records.len() as u16).to_be_bytes());
    out.extend_from_slice(&((6 + 12 * records.len()) as u16).to_be_bytes());
    let mut strings = Vec::new();
    for (platform, encoding, language, id, bytes) in &records {
        for v in [*platform, *encoding, *language, *id, bytes.len() as u16] {
            out.extend_from_slice(&v.to_be_bytes());
        }
        out.extend_from_slice(&(strings.len() as u16).to_be_bytes());
        strings.extend_from_slice(bytes);
    }
    out.extend_from_slice(&strings);
    Some(out)
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, c| {
        let mut word = [0u8; 4];
        word[..c.len()].copy_from_slice(c);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

/// Serialize an sfnt with `tables` sorted by tag, each 4-byte aligned, and
/// fix `head.checkSumAdjustment`.
fn write_font(version: u32, mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by_key(|t| t.0);
    let n = tables.len() as u16;
    let pow = 1u16 << (15 - n.leading_zeros());
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_be_bytes());
    for v in [n, pow * 16, pow.trailing_zeros() as u16, n * 16 - pow * 16] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    let mut offset = 12 + 16 * tables.len();
    let mut head_at = None;
    for (tag, data) in &tables {
        if tag == b"head" {
            head_at = Some(offset);
        }
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(data).to_be_bytes());
        out.extend_from_slice(&(offset as u32).to_be_bytes());
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in &tables {
        out.extend_from_slice(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    if let Some(at) = head_at {
        let adjust = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        out[at + 8..at + 12].copy_from_slice(&adjust.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONO: &[u8] = include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf");

    fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
        read_tables(font)
            .unwrap()
            .into_iter()
            .find(|t| &t.0 == tag)
            .unwrap()
            .1
    }

    #[test]
    fn derived_face_scales_the_em_and_renames_the_family() {
        let face = derive(MONO, 1077, "u0 Inline Mono", 308).expect("derived");
        assert_eq!(be16(table(&face, b"head"), 18), Some(1077));
        // Outlines and metrics are untouched: same glyf, same ascender.
        assert_eq!(table(&face, b"glyf"), table(MONO, b"glyf"));
        assert_eq!(be16(table(&face, b"hhea"), 4), Some(1025));
        assert!(read_tables(&face).unwrap().iter().all(|t| &t.0 != b"DSIG"));
        let name = table(&face, b"name");
        let utf16: Vec<u8> = "u0 Inline Mono"
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect();
        assert!(name.windows(utf16.len()).any(|w| w == utf16));
        let plex: Vec<u8> = "IBM Plex Mono"
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect();
        assert!(!name.windows(plex.len()).any(|w| w == plex));
        // The whole font checksums to the magic number.
        assert_eq!(checksum(&face), 0xB1B0_AFBA);
    }

    #[test]
    fn the_narrow_no_break_space_takes_the_padding_advance() {
        let face = derive(MONO, 1077, "u0 Inline Mono", 308).unwrap();
        let g = cmap_lookup(table(&face, b"cmap"), 0x202F).expect("U+202F mapped") as usize;
        assert_eq!(be16(table(&face, b"hmtx"), g * 4), Some(308));
        assert_eq!(be16(table(MONO, b"hmtx"), g * 4), Some(600));
        // Other glyphs keep their 600-unit advance.
        let a = cmap_lookup(table(&face, b"cmap"), u16::from(b'a')).unwrap() as usize;
        assert_eq!(be16(table(&face, b"hmtx"), a * 4), Some(600));
    }
}
