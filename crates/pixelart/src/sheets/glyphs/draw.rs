//! Shape helpers shared by the boon icons and the boon effect sprites, so
//! every flame, heart and blade in the set is drawn the same way.

use crate::canvas::{Canvas, Rgba};
use crate::palette::{BARK, CREAM, GOLD, ORANGE, OUTLINE, RED, STEEL, STONE_LIGHT, TAN, WHITE};

/// Icon frames are square.
pub const ICON: u32 = 24;

/// The four layers of an ordinary flame, outside in.
pub const FIRE: [Rgba; 4] = [RED, ORANGE, GOLD, CREAM];

/// A fresh transparent icon-sized canvas with `draw` applied.
pub fn layer(draw: impl FnOnce(&mut Canvas)) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    draw(&mut canvas);
    canvas
}

/// Draws `part` onto `icon` with its own plum outline, so overlapping parts
/// stay readable as separate objects.
pub fn stack(icon: &mut Canvas, part: &Canvas) {
    icon.blit(&part.outlined(OUTLINE), 0, 0);
}

/// Sets every pixel whose centre `(x + 0.5, y + 0.5)` maps to a colour.
pub fn paint(canvas: &mut Canvas, shade: impl Fn(f32, f32) -> Option<Rgba>) {
    for y in 0..canvas.height() as i32 {
        for x in 0..canvas.width() as i32 {
            if let Some(color) = shade(x as f32 + 0.5, y as f32 + 0.5) {
                canvas.set(x, y, color);
            }
        }
    }
}

/// A teardrop flame standing on `base`, `h` tall and `w` half-wide; `lean`
/// bends the tip sideways. Each colour is a smaller flame nested inside the
/// previous one.
#[allow(clippy::too_many_arguments)]
pub fn flame(canvas: &mut Canvas, cx: f32, base: f32, h: f32, w: f32, lean: f32, colors: &[Rgba]) {
    for (k, &color) in colors.iter().enumerate() {
        let k = k as f32;
        let (hk, wk, bk) = (h * (1.0 - 0.2 * k), w * (1.0 - 0.24 * k), base - 0.8 * k);
        paint(canvas, |x, y| {
            let t = (bk - y) / hk;
            if !(0.0..=1.0).contains(&t) {
                return None;
            }
            let profile =
                if t < 0.3 { (1.0 - ((0.3 - t) / 0.3).powi(2)).sqrt() } else { ((1.0 - t) / 0.7).powf(1.3) };
            ((x - cx - lean * hk * t * t).abs() <= wk * profile).then_some(color)
        });
    }
}

/// The classic heart curve, centred on `(cx, cy)`: it reaches `1.24 * sy`
/// above the centre (lobes) and `sy` below it (tip), `1.14 * sx` to each side.
pub fn heart(canvas: &mut Canvas, cx: f32, cy: f32, sx: f32, sy: f32, color: Rgba) {
    paint(canvas, |x, y| {
        let (u, v) = ((x - cx) / sx, (cy - y) / sy);
        let a = u * u + v * v - 1.0;
        (a * a * a - u * u * v * v * v <= 0.0).then_some(color)
    });
}

/// A band along the circle of radius `r` around `(cx, cy)`, from angle
/// `from` (radians, y down) through `span`. It is `thickness` thick in the
/// middle and tapers to both tips; `colors` run from the outer edge inwards.
/// A `skew` below 1 moves the thickest part towards the end of the sweep.
#[allow(clippy::too_many_arguments)]
pub fn arc(
    canvas: &mut Canvas,
    cx: f32,
    cy: f32,
    r: f32,
    thickness: f32,
    from: f32,
    span: f32,
    colors: &[Rgba],
    skew: f32,
) {
    paint(canvas, |x, y| {
        let (dx, dy) = (x - cx, y - cy);
        let along = (dy.atan2(dx) - from).rem_euclid(std::f32::consts::TAU) / span;
        if along > 1.0 {
            return None;
        }
        let width = thickness * (1.0 - (2.0 * along.powf(skew) - 1.0).powi(2)).max(0.0).sqrt();
        let dist = dx.hypot(dy);
        if dist > r || dist < r - width {
            return None;
        }
        let depth = ((r - dist) / width.max(0.01) * colors.len() as f32) as usize;
        Some(colors[depth.min(colors.len() - 1)])
    });
}

/// A sword pointing up-right at 45°: the guard crosses `(hx, hy)` and the
/// blade runs `n` pixels diagonally to its tip. The lit edge is upper-left.
pub fn sword(canvas: &mut Canvas, hx: i32, hy: i32, n: i32) {
    for y in 0..canvas.height() as i32 {
        for x in 0..canvas.width() as i32 {
            let (rx, ry) = (x - hx, hy - y);
            let (along, across) = (rx + ry, ry - rx);
            let color = match along {
                a if (2..=2 * n).contains(&a) && across.abs() <= (2 * n - a).min(2) => match across {
                    1.. => WHITE,
                    -1..=0 => STEEL,
                    _ => STONE_LIGHT,
                },
                0 | 1 if across.abs() <= 5 => {
                    if across > 0 {
                        GOLD
                    } else {
                        ORANGE
                    }
                }
                -5..=-1 if across.abs() <= 1 => {
                    if across > 0 {
                        TAN
                    } else {
                        BARK
                    }
                }
                -7 | -6 if across.abs() <= 2 => GOLD,
                _ => continue,
            };
            canvas.set(x, y, color);
        }
    }
}

/// A four-pointed glint: white heart, cream then gold arms `r` long.
pub fn sparkle(canvas: &mut Canvas, x: i32, y: i32, r: i32) {
    for d in 1..=r {
        let color = if d == 1 { CREAM } else { GOLD };
        for (dx, dy) in [(d, 0), (-d, 0), (0, d), (0, -d)] {
            canvas.set(x + dx, y + dy, color);
        }
    }
    canvas.set(x, y, WHITE);
}

/// A glowing ember: an orange disc with a gold heart and a cream glint.
pub fn ember(canvas: &mut Canvas, cx: f32, cy: f32, r: f32) {
    canvas.disc(cx, cy, r, ORANGE);
    canvas.disc(cx - 0.3, cy - 0.3, r * 0.6, GOLD);
    canvas.set((cx - 0.6).floor() as i32, (cy - 0.6).floor() as i32, CREAM);
}

/// The canvas turned a quarter turn clockwise (square canvases only).
pub fn rotated_cw(canvas: &Canvas) -> Canvas {
    let n = canvas.width() as i32;
    let mut out = Canvas::new(canvas.width(), canvas.height());
    for y in 0..n {
        for x in 0..n {
            out.set(x, y, canvas.get(y, n - 1 - x));
        }
    }
    out
}

/// The canvas mirrored top to bottom.
pub fn flipped_v(canvas: &Canvas) -> Canvas {
    let (w, h) = (canvas.width() as i32, canvas.height() as i32);
    let mut out = Canvas::new(canvas.width(), canvas.height());
    for y in 0..h {
        for x in 0..w {
            out.set(x, y, canvas.get(x, h - 1 - y));
        }
    }
    out
}
