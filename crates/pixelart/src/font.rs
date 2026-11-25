//! The game's pixel font, authored as tiny bitmaps and compiled into a
//! TrueType file at start-up, so text is drawn in the same blocky style as
//! every sprite and the repository still holds no asset files.
//!
//! Glyphs sit on a grid of font pixels: capitals and digits are
//! [`CAP_HEIGHT`] pixels tall, and one em is [`EM_PIXELS`] pixels. Text is
//! therefore crispest at font sizes that are a multiple of [`EM_PIXELS`].
//! Lowercase letters map to the capitals.

/// Font pixels per em: a font size of `8 * n` draws each font pixel as an
/// `n` x `n` square.
pub const EM_PIXELS: u16 = 8;
/// Height of capitals and digits, in font pixels.
pub const CAP_HEIGHT: u16 = 7;

/// Font units per font pixel.
const UNIT: i16 = 128;
const UNITS_PER_EM: u16 = EM_PIXELS * UNIT as u16;
/// Font pixels above the baseline and below it, for line layout.
const ASCENT: i16 = 8;
const DESCENT: i16 = 2;
/// Blank font pixels after every glyph.
const SPACING: u16 = 1;

/// One glyph: `#` is ink, `.` is paper, rows separated by `/`, top row
/// first. The first [`CAP_HEIGHT`] rows stand on the baseline; any further
/// rows hang below it.
struct Glyph {
    chars: &'static str,
    rows: &'static str,
}

const fn g(chars: &'static str, rows: &'static str) -> Glyph {
    Glyph { chars, rows }
}

#[rustfmt::skip]
const GLYPHS: &[Glyph] = &[
    g(" ", ".../.../.../.../.../.../..."),
    g("Aa", ".###./#...#/#...#/#####/#...#/#...#/#...#"),
    g("Bb", "####./#...#/#...#/####./#...#/#...#/####."),
    g("Cc", ".###./#...#/#..../#..../#..../#...#/.###."),
    g("Dd", "####./#...#/#...#/#...#/#...#/#...#/####."),
    g("Ee", "####/#.../#.../###./#.../#.../####"),
    g("Ff", "####/#.../#.../###./#.../#.../#..."),
    g("Gg", ".###./#...#/#..../#.###/#...#/#...#/.####"),
    g("Hh", "#...#/#...#/#...#/#####/#...#/#...#/#...#"),
    g("Ii", "###/.#./.#./.#./.#./.#./###"),
    g("Jj", "..###/...#./...#./...#./#..#./#..#./.##.."),
    g("Kk", "#...#/#..#./#.#../##.../#.#../#..#./#...#"),
    g("Ll", "#.../#.../#.../#.../#.../#.../####"),
    g("Mm", "#...#/##.##/#.#.#/#.#.#/#...#/#...#/#...#"),
    g("Nn", "#...#/#...#/##..#/#.#.#/#..##/#...#/#...#"),
    g("Oo", ".###./#...#/#...#/#...#/#...#/#...#/.###."),
    g("Pp", "####./#...#/#...#/####./#..../#..../#...."),
    g("Qq", ".###./#...#/#...#/#...#/#.#.#/#..#./.##.#"),
    g("Rr", "####./#...#/#...#/####./#.#../#..#./#...#"),
    g("Ss", ".####/#..../#..../.###./....#/....#/####."),
    g("Tt", "#####/..#../..#../..#../..#../..#../..#.."),
    g("Uu", "#...#/#...#/#...#/#...#/#...#/#...#/.###."),
    g("Vv", "#...#/#...#/#...#/#...#/#...#/.#.#./..#.."),
    g("Ww", "#...#/#...#/#...#/#.#.#/#.#.#/#.#.#/.#.#."),
    g("Xx", "#...#/#...#/.#.#./..#../.#.#./#...#/#...#"),
    g("Yy", "#...#/#...#/.#.#./..#../..#../..#../..#.."),
    g("Zz", "#####/....#/...#./..#../.#.../#..../#####"),
    g("0", ".###./#...#/#..##/#.#.#/##..#/#...#/.###."),
    g("1", "..#../.##../..#../..#../..#../..#../.###."),
    g("2", ".###./#...#/....#/...#./..#../.#.../#####"),
    g("3", "####./....#/....#/.###./....#/....#/####."),
    g("4", "...#./..##./.#.#./#..#./#####/...#./...#."),
    g("5", "#####/#..../####./....#/....#/#...#/.###."),
    g("6", ".###./#..../#..../####./#...#/#...#/.###."),
    g("7", "#####/....#/...#./..#../.#.../.#.../.#..."),
    g("8", ".###./#...#/#...#/.###./#...#/#...#/.###."),
    g("9", ".###./#...#/#...#/.####/....#/....#/.###."),
    g(".", "././././././#"),
    g(",", "../../../../../.#/.#/#."),
    g(":", "././#/./././#"),
    g(";", "../../.#/../../.#/.#/#."),
    g("!", "#/#/#/#/#/./#"),
    g("?", ".###./#...#/....#/...#./..#../...../..#.."),
    g("'", "#/#/././././."),
    g("\"", "#.#/#.#/.../.../.../.../..."),
    g("-", "..../..../..../####/..../..../...."),
    g("_", "..../..../..../..../..../..../####"),
    g("+", "...../..#../..#../#####/..#../..#../....."),
    g("=", "..../..../####/..../####/..../...."),
    g("*", "...../#.#.#/.###./#####/.###./#.#.#/....."),
    g("/", "....#/....#/...#./..#../.#.../#..../#...."),
    g("%", "##..#/##..#/...#./..#../.#.../#..##/#..##"),
    g("#", ".#.#./.#.#./#####/.#.#./#####/.#.#./.#.#."),
    g("(", "..#/.#./#../#../#../.#./..#"),
    g(")", "#../.#./..#/..#/..#/.#./#.."),
    g("[", "###/#../#../#../#../#../###"),
    g("]", "###/..#/..#/..#/..#/..#/###"),
    g("<", "...#/..#./.#../#.../.#../..#./...#"),
    g(">", "#.../.#../..#./...#/..#./.#../#..."),
    g("&", ".##../#..#./#.#../.#.../#.#.#/#..#./.##.#"),
];

/// Shown for any character the font lacks: a hollow box.
const MISSING: &str = "#####/#...#/#...#/#...#/#...#/#...#/#####";

/// A glyph's ink as rectangles `(x, y, width, height)` in font pixels,
/// `y` up from the baseline, and its width.
fn shape(rows: &str) -> (Vec<(u16, i16, u16, u16)>, u16) {
    let rows: Vec<&[u8]> = rows.split('/').map(str::as_bytes).collect();
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as u16;
    let mut rects: Vec<(u16, i16, u16, u16)> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let y = CAP_HEIGHT as i16 - 1 - index as i16;
        let mut x = 0;
        while x < row.len() {
            if row[x] != b'#' {
                x += 1;
                continue;
            }
            let start = x;
            while x < row.len() && row[x] == b'#' {
                x += 1;
            }
            let run = (start as u16, y, (x - start) as u16, 1);
            // Grow the run above into this row when they line up exactly,
            // so vertical strokes become one tall rectangle.
            match rects.iter_mut().find(|r| r.0 == run.0 && r.2 == run.2 && r.1 - r.3 as i16 + 1 == y + 1) {
                Some(above) => above.3 += 1,
                None => rects.push(run),
            }
        }
    }
    (rects, width)
}

/// The font as a TrueType (`.ttf`) file.
pub fn pixel_font_ttf() -> Vec<u8> {
    let mut outlines = vec![shape(MISSING)];
    let mut chars: Vec<(char, u16)> = Vec::new();
    for glyph in GLYPHS {
        let id = outlines.len() as u16;
        outlines.push(shape(glyph.rows));
        chars.extend(glyph.chars.chars().map(|c| (c, id)));
    }
    chars.sort_unstable();
    build(&outlines, &chars)
}

/// Big-endian byte writer.
#[derive(Default)]
struct Bytes(Vec<u8>);

impl Bytes {
    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend(v.to_be_bytes());
        self
    }
    fn i16(&mut self, v: i16) -> &mut Self {
        self.0.extend(v.to_be_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend(v.to_be_bytes());
        self
    }
    fn zeros(&mut self, n: usize) -> &mut Self {
        self.0.resize(self.0.len() + n, 0);
        self
    }
    fn pad4(&mut self) {
        while !self.0.len().is_multiple_of(4) {
            self.0.push(0);
        }
    }
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn build(outlines: &[(Vec<(u16, i16, u16, u16)>, u16)], chars: &[(char, u16)]) -> Vec<u8> {
    let px = |v: i16| v * UNIT;
    let num_glyphs = outlines.len() as u16;

    // glyf + loca: every rectangle is a clockwise four-point contour.
    let mut glyf = Bytes::default();
    let mut loca = Bytes::default();
    let (mut max_points, mut max_contours) = (0u16, 0u16);
    let (mut x_max_all, mut y_min_all, mut y_max_all) = (0i16, 0i16, 0i16);
    for (rects, _) in outlines {
        loca.u32(glyf.0.len() as u32);
        if rects.is_empty() {
            continue;
        }
        let bounds = rects.iter().fold((i16::MAX, i16::MAX, i16::MIN, i16::MIN), |b, &(x, y, w, h)| {
            let (x0, y0) = (x as i16, y - h as i16 + 1);
            (b.0.min(x0), b.1.min(y0), b.2.max(x0 + w as i16), b.3.max(y + 1))
        });
        let (x_min, y_min, x_max, y_max) = (px(bounds.0), px(bounds.1), px(bounds.2), px(bounds.3));
        (x_max_all, y_min_all, y_max_all) =
            (x_max_all.max(x_max), y_min_all.min(y_min), y_max_all.max(y_max));
        let contours = rects.len() as u16;
        (max_points, max_contours) = (max_points.max(contours * 4), max_contours.max(contours));

        glyf.i16(contours as i16).i16(x_min).i16(y_min).i16(x_max).i16(y_max);
        for i in 0..contours {
            glyf.u16(i * 4 + 3);
        }
        glyf.u16(0); // no instructions
        glyf.0.extend(std::iter::repeat_n(0x01u8, contours as usize * 4)); // on-curve, long coords
        let points: Vec<(i16, i16)> = rects
            .iter()
            .flat_map(|&(x, y, w, h)| {
                let (x0, x1) = (px(x as i16), px((x + w) as i16));
                let (y0, y1) = (px(y - h as i16 + 1), px(y + 1));
                [(x0, y0), (x0, y1), (x1, y1), (x1, y0)]
            })
            .collect();
        let mut last = 0;
        for &(x, _) in &points {
            glyf.i16(x - last);
            last = x;
        }
        last = 0;
        for &(_, y) in &points {
            glyf.i16(y - last);
            last = y;
        }
        glyf.pad4();
    }
    loca.u32(glyf.0.len() as u32);

    let advances: Vec<u16> = outlines.iter().map(|(_, w)| (w + SPACING) * UNIT as u16).collect();
    let max_advance = advances.iter().copied().max().unwrap_or(0);
    let mut hmtx = Bytes::default();
    for (advance, (rects, _)) in advances.iter().zip(outlines) {
        let lsb = rects.iter().map(|r| r.0 as i16).min().unwrap_or(0);
        hmtx.u16(*advance).i16(px(lsb));
    }

    let (ascent, descent) = (px(ASCENT), px(-DESCENT));

    let mut head = Bytes::default();
    head.u32(0x0001_0000).u32(0x0001_0000).u32(0).u32(0x5F0F_3CF5);
    head.u16(0b1001).u16(UNITS_PER_EM).zeros(16); // flags, em, created + modified
    head.i16(0).i16(y_min_all).i16(x_max_all).i16(y_max_all);
    head.u16(0).u16(6).i16(2).i16(1).i16(0); // macStyle, lowestRecPPEM, direction, long loca, format

    let mut hhea = Bytes::default();
    hhea.u32(0x0001_0000).i16(ascent).i16(descent).i16(0).u16(max_advance);
    hhea.i16(0).i16(0).i16(x_max_all).i16(1).i16(0).i16(0).zeros(8).i16(0).u16(num_glyphs);

    let mut maxp = Bytes::default();
    maxp.u32(0x0001_0000).u16(num_glyphs).u16(max_points).u16(max_contours);
    maxp.u16(0).u16(0).u16(2).zeros(16);

    // cmap: one format 12 subtable for both Unicode and Windows Unicode.
    let mut subtable = Bytes::default();
    subtable.u16(12).u16(0).u32(16 + 12 * chars.len() as u32).u32(0).u32(chars.len() as u32);
    for &(c, id) in chars {
        subtable.u32(c as u32).u32(c as u32).u32(id as u32);
    }
    let mut cmap = Bytes::default();
    cmap.u16(0).u16(2).u16(0).u16(4).u32(20).u16(3).u16(10).u32(20);
    cmap.0.extend(&subtable.0);

    let names = [(1u16, "Emberbound Pixel"), (2, "Regular"), (4, "Emberbound Pixel"), (6, "EmberboundPixel")];
    let mut strings = Bytes::default();
    let mut name = Bytes::default();
    name.u16(0).u16(names.len() as u16).u16(6 + 12 * names.len() as u16);
    for (id, text) in names {
        let offset = strings.0.len() as u16;
        text.encode_utf16().for_each(|unit| {
            strings.u16(unit);
        });
        name.u16(3).u16(1).u16(0x409).u16(id).u16(strings.0.len() as u16 - offset).u16(offset);
    }
    name.0.extend(&strings.0);

    let mut post = Bytes::default();
    post.u32(0x0003_0000).u32(0).i16(px(-1)).i16(UNIT).zeros(20);

    let first = chars.first().map_or(0, |c| c.0 as u16);
    let last = chars.last().map_or(0, |c| c.0 as u16);
    let average = (advances.iter().map(|&a| a as u32).sum::<u32>() / advances.len() as u32) as i16;
    let mut os2 = Bytes::default();
    os2.u16(4).i16(average).u16(400).u16(5).u16(0);
    os2.zeros(20).i16(0).zeros(10); // sub/superscript & strikeout, family class, panose
    os2.u32(1).zeros(12).0.extend(b"EMBR"); // Basic Latin
    os2.u16(0x00C0).u16(first).u16(last); // regular, use typo metrics
    os2.i16(ascent).i16(descent).i16(0).u16(ascent as u16).u16(-descent as u16);
    os2.u32(1).u32(0).i16(px(CAP_HEIGHT as i16)).i16(px(CAP_HEIGHT as i16));
    os2.u16(0).u16(' ' as u16).u16(1);

    let tables: [(&[u8; 4], Vec<u8>); 10] = [
        (b"OS/2", os2.0),
        (b"cmap", cmap.0),
        (b"glyf", glyf.0),
        (b"head", head.0),
        (b"hhea", hhea.0),
        (b"hmtx", hmtx.0),
        (b"loca", loca.0),
        (b"maxp", maxp.0),
        (b"name", name.0),
        (b"post", post.0),
    ];

    let mut font = Bytes::default();
    let count = tables.len() as u16;
    let entry_selector = 15 - count.leading_zeros() as u16;
    let search_range = (1u16 << entry_selector) * 16;
    font.u32(0x0001_0000).u16(count).u16(search_range).u16(entry_selector).u16(count * 16 - search_range);
    let mut offset = 12 + 16 * tables.len();
    let mut head_offset = 0;
    for (tag, data) in &tables {
        if *tag == b"head" {
            head_offset = offset;
        }
        font.0.extend(*tag);
        font.u32(checksum(data)).u32(offset as u32).u32(data.len() as u32);
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in &tables {
        font.0.extend(data);
        font.pad4();
    }
    let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&font.0));
    font.0[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    font.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_well_formed() {
        for glyph in GLYPHS.iter().map(|g| g.rows).chain([MISSING]) {
            let rows: Vec<&str> = glyph.split('/').collect();
            assert!(rows.len() >= CAP_HEIGHT as usize, "{glyph}: too short");
            assert!(rows.len() <= (CAP_HEIGHT as i16 + DESCENT) as usize, "{glyph}: hangs too low");
            assert!(rows.iter().all(|row| row.len() == rows[0].len()), "{glyph}: ragged rows");
            assert!(rows.iter().all(|row| row.bytes().all(|b| b == b'#' || b == b'.')), "{glyph}");
        }
    }

    #[test]
    fn no_character_is_drawn_twice() {
        let mut chars: Vec<char> = GLYPHS.iter().flat_map(|g| g.chars.chars()).collect();
        let total = chars.len();
        chars.sort_unstable();
        chars.dedup();
        assert_eq!(chars.len(), total);
    }

    #[test]
    fn rectangles_cover_exactly_the_ink() {
        for glyph in GLYPHS.iter().map(|g| g.rows).chain([MISSING]) {
            let (rects, width) = shape(glyph);
            let rows: Vec<&[u8]> = glyph.split('/').map(str::as_bytes).collect();
            for (index, row) in rows.iter().enumerate() {
                let y = CAP_HEIGHT as i16 - 1 - index as i16;
                for x in 0..width {
                    let covering = rects
                        .iter()
                        .filter(|r| {
                            (r.0..r.0 + r.2).contains(&x) && (r.1 - r.3 as i16 + 1..=r.1).contains(&y)
                        })
                        .count();
                    assert_eq!(covering, (row[x as usize] == b'#') as usize, "{glyph} at ({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn the_font_file_is_consistent() {
        let font = pixel_font_ttf();
        assert_eq!(&font[..4], &[0, 1, 0, 0]);
        assert_eq!(font.len() % 4, 0);
        assert_eq!(checksum(&font), 0xB1B0_AFBA, "whole-file checksum matches head");
    }
}
