//! Drawing helpers on top of tiny-skia: rounded rectangles, CSS-like soft shadows, gradients,
//! image scaling and the few line icons of the window. Coordinates are DIPs unless a name says
//! `px`; every helper takes the scale.

use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, Path, PathBuilder, Pixmap, PixmapPaint,
    Point, SpreadMode, Stroke, Transform,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct R {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl R {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        R { x, y, w, h }
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
    pub fn inset(&self, d: f32) -> R {
        R::new(self.x + d, self.y + d, self.w - 2.0 * d, self.h - 2.0 * d)
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba8(r, g, b, (a.clamp(0.0, 1.0) * 255.0).round() as u8)
}

pub fn hex(v: u32) -> Color {
    Color::from_rgba8((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        a.red() + (b.red() - a.red()) * t,
        a.green() + (b.green() - a.green()) * t,
        a.blue() + (b.blue() - a.blue()) * t,
        a.alpha() + (b.alpha() - a.alpha()) * t,
    )
    .unwrap_or(a)
}

/// Rounded rectangle path in device pixels (`r` is scaled with the rest).
pub fn round_rect(rect: R, radius: f32, s: f32) -> Option<Path> {
    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let r = (radius * s).min(w / 2.0).min(h / 2.0);
    let k = r * 0.552_284_8;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

pub fn fill(px: &mut Pixmap, path: &Path, paint: &Paint, t: Transform) {
    px.fill_path(path, paint, FillRule::Winding, t, None);
}

pub fn solid(c: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(c);
    p.anti_alias = true;
    p
}

/// Vertical gradient over `rect` (DIPs): `top` to `bottom`, with a 1 DIP inner highlight of
/// `highlight` along the top edge (CSS `inset 0 1px 0`).
pub fn vgradient(
    rect: R,
    s: f32,
    top: Color,
    bottom: Color,
    highlight: Option<Color>,
) -> Paint<'static> {
    let mut stops = Vec::new();
    if let Some(hl) = highlight {
        let edge = (1.0 / rect.h).min(0.5);
        stops.push(GradientStop::new(0.0, hl));
        stops.push(GradientStop::new(edge, top));
    } else {
        stops.push(GradientStop::new(0.0, top));
    }
    stops.push(GradientStop::new(1.0, bottom));
    let shader = LinearGradient::new(
        Point::from_xy(0.0, rect.y * s),
        Point::from_xy(0.0, rect.bottom() * s),
        stops,
        SpreadMode::Pad,
        Transform::identity(),
    );
    let mut p = Paint::default();
    if let Some(sh) = shader {
        p.shader = sh;
    }
    p.anti_alias = true;
    p
}

pub fn hgradient(rect: R, s: f32, left: Color, right: Color) -> Paint<'static> {
    let shader = LinearGradient::new(
        Point::from_xy(rect.x * s, 0.0),
        Point::from_xy(rect.right() * s, 0.0),
        vec![GradientStop::new(0.0, left), GradientStop::new(1.0, right)],
        SpreadMode::Pad,
        Transform::identity(),
    );
    let mut p = Paint::default();
    if let Some(sh) = shader {
        p.shader = sh;
    }
    p.anti_alias = true;
    p
}

/// Strokes the inside of a rounded rectangle with a `width` line (CSS `inset 0 0 0 <width>`).
pub fn inset_outline(
    px: &mut Pixmap,
    rect: R,
    radius: f32,
    width: f32,
    c: Color,
    s: f32,
    t: Transform,
) {
    if let Some(path) = round_rect(rect.inset(width / 2.0), (radius - width / 2.0).max(0.0), s) {
        let stroke = Stroke {
            width: width * s,
            ..Stroke::default()
        };
        px.stroke_path(&path, &solid(c), &stroke, t, None);
    }
}

fn erf(x: f32) -> f32 {
    // Abramowitz and Stegun 7.1.26, error below 1.5e-7.
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_4 * t - 1.453_152_1) * t) + 1.421_413_8) * t - 0.284_496_74) * t
            + 0.254_829_6)
            * t
            * (-x * x).exp();
    sign * y
}

/// CSS `box-shadow: <dx> <dy> <blur> <spread> <color>` of a box `rect` (DIPs), drawn as the
/// blurred rectangle (rounded corners make no visible difference at these blur sizes).
#[allow(clippy::too_many_arguments)]
pub fn box_shadow(
    px: &mut Pixmap,
    rect: R,
    dx: f32,
    dy: f32,
    blur: f32,
    spread: f32,
    c: Color,
    s: f32,
    opacity: f32,
) {
    let sigma = (blur / 2.0).max(0.01) * s;
    let x0 = (rect.x + dx - spread) * s;
    let x1 = (rect.right() + dx + spread) * s;
    let y0 = (rect.y + dy - spread) * s;
    let y1 = (rect.bottom() + dy + spread) * s;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let reach = 3.0 * sigma;
    let (w, h) = (px.width() as i32, px.height() as i32);
    let bx0 = ((x0 - reach).floor() as i32).max(0);
    let bx1 = ((x1 + reach).ceil() as i32).min(w);
    let by0 = ((y0 - reach).floor() as i32).max(0);
    let by1 = ((y1 + reach).ceil() as i32).min(h);
    if bx0 >= bx1 || by0 >= by1 {
        return;
    }
    let k = std::f32::consts::FRAC_1_SQRT_2 / sigma;
    let fx: Vec<f32> = (bx0..bx1)
        .map(|x| {
            let cx = x as f32 + 0.5;
            0.5 * (erf((cx - x0) * k) - erf((cx - x1) * k))
        })
        .collect();
    let base_a = c.alpha() * opacity;
    let (cr, cg, cb) = (c.red(), c.green(), c.blue());
    let stride = w as usize;
    let data = px.data_mut();
    for y in by0..by1 {
        let cy = y as f32 + 0.5;
        let fy = 0.5 * (erf((cy - y0) * k) - erf((cy - y1) * k));
        if fy < 0.002 {
            continue;
        }
        for (i, x) in (bx0..bx1).enumerate() {
            let a = base_a * fx[i] * fy;
            if a < 0.002 {
                continue;
            }
            let o = (y as usize * stride + x as usize) * 4;
            let inv = 1.0 - a;
            data[o] = (cr * a * 255.0 + data[o] as f32 * inv).round() as u8;
            data[o + 1] = (cg * a * 255.0 + data[o + 1] as f32 * inv).round() as u8;
            data[o + 2] = (cb * a * 255.0 + data[o + 2] as f32 * inv).round() as u8;
            data[o + 3] = (a * 255.0 + data[o + 3] as f32 * inv).round() as u8;
        }
    }
}

/// Scales a premultiplied image to `w` x `h` by area averaging (sharp and alias-free when
/// shrinking, which is all the window does with its 3x art).
pub fn resize(src: &Pixmap, w: u32, h: u32) -> Pixmap {
    let mut dst = Pixmap::new(w.max(1), h.max(1)).unwrap();
    let (sw, sh) = (src.width() as f32, src.height() as f32);
    let fx = sw / w as f32;
    let fy = sh / h as f32;
    let sdata = src.data();
    let sstride = src.width() as usize;
    let ddata = dst.data_mut();
    for dy in 0..h {
        let y0 = dy as f32 * fy;
        let y1 = y0 + fy;
        for dx in 0..w {
            let x0 = dx as f32 * fx;
            let x1 = x0 + fx;
            let mut acc = [0f32; 4];
            let mut area = 0f32;
            let mut sy = y0.floor() as usize;
            while (sy as f32) < y1 && sy < src.height() as usize {
                let wy = (y1.min(sy as f32 + 1.0) - y0.max(sy as f32)).max(0.0);
                let mut sx = x0.floor() as usize;
                while (sx as f32) < x1 && sx < src.width() as usize {
                    let wx = (x1.min(sx as f32 + 1.0) - x0.max(sx as f32)).max(0.0);
                    let wgt = wx * wy;
                    let o = (sy * sstride + sx) * 4;
                    for c in 0..4 {
                        acc[c] += sdata[o + c] as f32 * wgt;
                    }
                    area += wgt;
                    sx += 1;
                }
                sy += 1;
            }
            let o = (dy as usize * w as usize + dx as usize) * 4;
            for c in 0..4 {
                ddata[o + c] = (acc[c] / area.max(1e-6)).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    dst
}

pub fn draw_image(px: &mut Pixmap, img: &Pixmap, x: i32, y: i32, opacity: f32) {
    let paint = PixmapPaint {
        opacity,
        ..PixmapPaint::default()
    };
    px.draw_pixmap(x, y, img.as_ref(), &paint, Transform::identity(), None);
}

/// Line icon on a 24-unit grid, drawn into a `size` DIP square at (`x`, `y`).
#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    Lock,
    Check,
    Download,
    Folder,
    Minimize,
    Close,
}

fn icon_path(icon: &Icon) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let k = 0.552_284_8;
    match icon {
        Icon::Lock => {
            // Body: 16 x 10.5 at (4, 10.5), radius 2.5.
            let (x, y, w, h, r) = (4.0f32, 10.5f32, 16.0f32, 10.5f32, 2.5f32);
            let c = r * k;
            pb.move_to(x + r, y);
            pb.line_to(x + w - r, y);
            pb.cubic_to(x + w - r + c, y, x + w, y + r - c, x + w, y + r);
            pb.line_to(x + w, y + h - r);
            pb.cubic_to(x + w, y + h - r + c, x + w - r + c, y + h, x + w - r, y + h);
            pb.line_to(x + r, y + h);
            pb.cubic_to(x + r - c, y + h, x, y + h - r + c, x, y + h - r);
            pb.line_to(x, y + r);
            pb.cubic_to(x, y + r - c, x + r - c, y, x + r, y);
            pb.close();
            // Shackle: up from (8, 10.5) to (8, 7.5), a half circle of radius 4, down to (16, 10.5).
            let rr = 4.0f32;
            let cc = rr * k;
            pb.move_to(8.0, 10.5);
            pb.line_to(8.0, 7.5);
            pb.cubic_to(8.0, 7.5 - cc, 12.0 - cc, 3.5, 12.0, 3.5);
            pb.cubic_to(12.0 + cc, 3.5, 16.0, 7.5 - cc, 16.0, 7.5);
            pb.line_to(16.0, 10.5);
        }
        Icon::Check => {
            pb.move_to(5.0, 12.5);
            pb.line_to(9.5, 17.0);
            pb.line_to(19.0, 7.5);
        }
        Icon::Download => {
            pb.move_to(12.0, 4.0);
            pb.line_to(12.0, 16.0);
            pb.move_to(7.0, 11.0);
            pb.line_to(12.0, 16.0);
            pb.line_to(17.0, 11.0);
            pb.move_to(5.0, 20.0);
            pb.line_to(19.0, 20.0);
        }
        Icon::Folder => {
            // Lucide "folder": corners of radius 2, the tab on the top left.
            pb.move_to(20.0, 20.0);
            pb.quad_to(22.0, 20.0, 22.0, 18.0);
            pb.line_to(22.0, 8.0);
            pb.quad_to(22.0, 6.0, 20.0, 6.0);
            pb.line_to(12.1, 6.0);
            pb.quad_to(11.0, 6.0, 10.41, 5.1);
            pb.line_to(9.6, 3.9);
            pb.quad_to(9.0, 3.0, 7.93, 3.0);
            pb.line_to(4.0, 3.0);
            pb.quad_to(2.0, 3.0, 2.0, 5.0);
            pb.line_to(2.0, 18.0);
            pb.quad_to(2.0, 20.0, 4.0, 20.0);
            pb.close();
        }
        Icon::Minimize => {
            pb.move_to(0.0, 13.2);
            pb.line_to(24.0, 13.2);
        }
        Icon::Close => {
            pb.move_to(1.2, 1.2);
            pb.line_to(22.8, 22.8);
            pb.move_to(22.8, 1.2);
            pb.line_to(1.2, 22.8);
        }
    }
    pb.finish()
}

/// Draws `icon` stroked with `width` (in grid units) in a `size` DIP box.
#[allow(clippy::too_many_arguments)]
pub fn icon(
    px: &mut Pixmap,
    icon: Icon,
    x: f32,
    y: f32,
    size: f32,
    width: f32,
    c: Color,
    s: f32,
    t: Transform,
) {
    let Some(path) = icon_path(&icon) else { return };
    let k = size / 24.0 * s;
    let stroke = Stroke {
        width,
        line_cap: if matches!(icon, Icon::Minimize | Icon::Close) {
            tiny_skia::LineCap::Butt
        } else {
            tiny_skia::LineCap::Round
        },
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    };
    let tr = t.pre_translate(x * s, y * s).pre_scale(k, k);
    px.stroke_path(&path, &solid(c), &stroke, tr, None);
}
