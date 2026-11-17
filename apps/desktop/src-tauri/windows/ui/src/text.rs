//! Text: the embedded fonts, shaping (rustybuzz, so kerning and ligatures work), simple line
//! breaking and rasterising glyph outlines with tiny-skia.
//!
//! Onest comes as two static faces per weight, Latin and Cyrillic (the app ships the same
//! subsets); each character picks its face. JetBrains Mono is subset to digits and a few signs
//! for the version chip and the percentage.

use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};
use rustybuzz::{Face, UnicodeBuffer};
use std::sync::OnceLock;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Font {
    Onest600,
    Onest700,
    Onest800,
    Mono700,
}

pub struct Fonts {
    latin: [Face<'static>; 3],
    cyrillic: [Face<'static>; 3],
    mono: Face<'static>,
}

fn face(data: &'static [u8]) -> Face<'static> {
    Face::from_slice(data, 0).expect("embedded font")
}

pub fn fonts() -> &'static Fonts {
    static FONTS: OnceLock<Fonts> = OnceLock::new();
    FONTS.get_or_init(|| Fonts {
        latin: [
            face(include_bytes!("../fonts/onest-latin-600.ttf")),
            face(include_bytes!("../fonts/onest-latin-700.ttf")),
            face(include_bytes!("../fonts/onest-latin-800.ttf")),
        ],
        cyrillic: [
            face(include_bytes!("../fonts/onest-cyrillic-600.ttf")),
            face(include_bytes!("../fonts/onest-cyrillic-700.ttf")),
            face(include_bytes!("../fonts/onest-cyrillic-800.ttf")),
        ],
        mono: face(include_bytes!("../fonts/jbm-700.ttf")),
    })
}

impl Fonts {
    /// Main face of a font and the face for Cyrillic letters.
    fn faces(&self, font: Font) -> (&Face<'static>, &Face<'static>) {
        match font {
            Font::Onest600 => (&self.latin[0], &self.cyrillic[0]),
            Font::Onest700 => (&self.latin[1], &self.cyrillic[1]),
            Font::Onest800 => (&self.latin[2], &self.cyrillic[2]),
            Font::Mono700 => (&self.mono, &self.mono),
        }
    }

    /// Ascent and descent (both positive) of `font` at `size` px, as a browser computes the
    /// line box from the primary face.
    pub fn metrics(&self, font: Font, size: f32) -> (f32, f32) {
        let f = self.faces(font).0;
        let k = size / f.units_per_em() as f32;
        (f.ascender() as f32 * k, -(f.descender() as f32) * k)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub font: Font,
    pub size: f32,
    /// Letter spacing in em, added after every glyph (as CSS `letter-spacing`).
    pub tracking: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
struct Glyph {
    cyrillic: bool,
    id: u16,
    x: f32,
    y: f32,
}

/// A shaped single-line run, positions in px from the run's origin on the baseline.
#[derive(Clone)]
pub struct Run {
    glyphs: Vec<Glyph>,
    pub width: f32,
    pub style: Style,
}

fn is_cyrillic(c: char) -> bool {
    matches!(c as u32, 0x0400..=0x052F | 0x1C80..=0x1C8F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F | 0x2116)
}

pub fn shape(text: &str, style: Style) -> Run {
    let fonts = fonts();
    let (latin, cyr) = fonts.faces(style.font);
    // Split into runs of one face: Cyrillic letters use the Cyrillic face, everything else the
    // Latin one; spaces and signs stay with the current run when its face has them.
    let mut segments: Vec<(bool, String)> = Vec::new();
    for c in text.chars() {
        let own = is_cyrillic(c) && style.font != Font::Mono700;
        let neutral = !c.is_alphabetic();
        match segments.last_mut() {
            Some((cy, s)) if *cy == own => s.push(c),
            Some((cy, s))
                if neutral && (if *cy { cyr } else { latin }).glyph_index(c).is_some() =>
            {
                s.push(c)
            }
            _ => segments.push((own, c.to_string())),
        }
    }
    let mut glyphs = Vec::new();
    let mut pen = 0.0f32;
    let track = style.tracking * style.size;
    for (cy, s) in segments {
        let face = if cy { cyr } else { latin };
        let k = style.size / face.units_per_em() as f32;
        let mut buf = UnicodeBuffer::new();
        buf.push_str(&s);
        buf.guess_segment_properties();
        let out = rustybuzz::shape(face, &[], buf);
        for (info, pos) in out.glyph_infos().iter().zip(out.glyph_positions()) {
            glyphs.push(Glyph {
                cyrillic: cy,
                id: info.glyph_id as u16,
                x: pen + pos.x_offset as f32 * k,
                y: pos.y_offset as f32 * k,
            });
            pen += pos.x_advance as f32 * k + track;
        }
    }
    Run {
        glyphs,
        width: pen,
        style,
    }
}

struct Outline<'a> {
    pb: &'a mut PathBuilder,
    x: f32,
    y: f32,
    k: f32,
}

impl OutlineBuilder for Outline<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pb.move_to(self.x + x * self.k, self.y - y * self.k);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.pb.line_to(self.x + x * self.k, self.y - y * self.k);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.pb.quad_to(
            self.x + x1 * self.k,
            self.y - y1 * self.k,
            self.x + x * self.k,
            self.y - y * self.k,
        );
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.pb.cubic_to(
            self.x + x1 * self.k,
            self.y - y1 * self.k,
            self.x + x2 * self.k,
            self.y - y2 * self.k,
            self.x + x * self.k,
            self.y - y * self.k,
        );
    }
    fn close(&mut self) {
        self.pb.close();
    }
}

impl Run {
    /// Draws the run with its baseline origin at (`x`, `y`) in DIPs, scaled by `scale`.
    pub fn draw(
        &self,
        px: &mut Pixmap,
        x: f32,
        y: f32,
        scale: f32,
        color: Option<Color>,
        t: Transform,
    ) {
        let fonts = fonts();
        let (latin, cyr) = fonts.faces(self.style.font);
        let mut pb = PathBuilder::new();
        for g in &self.glyphs {
            let face = if g.cyrillic { cyr } else { latin };
            let k = self.style.size * scale / face.units_per_em() as f32;
            let mut o = Outline {
                pb: &mut pb,
                x: (x + g.x) * scale,
                y: (y - g.y) * scale,
                k,
            };
            face.outline_glyph(GlyphId(g.id), &mut o);
        }
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color.unwrap_or(self.style.color));
            paint.anti_alias = true;
            px.fill_path(&path, &paint, FillRule::Winding, t, None);
        }
    }
}

/// A piece of a line: a run and its x offset in the line.
#[derive(Clone)]
pub struct Piece {
    pub run: Run,
    pub x: f32,
}

/// A paragraph broken into lines that fit `width`; `spans` are (text, style) pairs. Lines break
/// only at spaces, also at a space between two spans.
pub fn wrap(spans: &[(String, Style)], width: f32) -> Vec<Vec<Piece>> {
    struct Word {
        run: Run,
        space: f32,
        breakable: bool,
    }
    let mut words: Vec<Word> = Vec::new();
    let mut space_before = false;
    for (text, style) in spans {
        let space = shape(" ", *style).width;
        for (i, part) in text.split(' ').enumerate() {
            if i > 0 {
                space_before = true;
            }
            if part.is_empty() {
                continue;
            }
            words.push(Word {
                run: shape(part, *style),
                space: if space_before { space } else { 0.0 },
                breakable: space_before,
            });
            space_before = false;
        }
    }
    let mut lines: Vec<Vec<Piece>> = vec![Vec::new()];
    let mut x = 0.0f32;
    for w in words {
        let line = lines.last_mut().unwrap();
        let gap = if line.is_empty() { 0.0 } else { w.space };
        if !line.is_empty() && w.breakable && x + gap + w.run.width > width {
            x = w.run.width;
            lines.push(vec![Piece { x: 0.0, run: w.run }]);
        } else {
            line.push(Piece {
                x: x + gap,
                run: w.run,
            });
            x += gap + lines.last().unwrap().last().unwrap().run.width;
        }
    }
    lines
}

/// Width of a line of pieces.
pub fn line_width(line: &[Piece]) -> f32 {
    line.last().map(|p| p.x + p.run.width).unwrap_or(0.0)
}
