//! A tiny RGBA drawing surface and an ASCII-art loader.

/// One RGBA8 colour.
pub type Rgba = [u8; 4];

/// Builds an opaque colour from `0xRRGGBB`.
pub const fn rgb(hex: u32) -> Rgba {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255]
}

pub const CLEAR: Rgba = [0, 0, 0, 0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    /// A fully transparent canvas.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![0; (width * height * 4) as usize] }
    }

    pub fn filled(width: u32, height: u32, color: Rgba) -> Self {
        let mut canvas = Self::new(width, height);
        for px in canvas.pixels.chunks_mut(4) {
            px.copy_from_slice(&color);
        }
        canvas
    }

    /// Builds a canvas from ASCII art. Each character is looked up in
    /// `palette`; `.` and space (and unknown characters) are transparent.
    /// The canvas is as wide as the longest row.
    pub fn from_art(rows: &[&str], palette: &[(char, Rgba)]) -> Self {
        let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as u32;
        let mut canvas = Self::new(width, rows.len() as u32);
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some((_, color)) = palette.iter().find(|(c, _)| *c == ch) {
                    canvas.set(x as i32, y as i32, *color);
                }
            }
        }
        canvas
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Sets a pixel; coordinates outside the canvas are ignored.
    pub fn set(&mut self, x: i32, y: i32, color: Rgba) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let i = ((y as u32 * self.width + x as u32) * 4) as usize;
            self.pixels[i..i + 4].copy_from_slice(&color);
        }
    }

    /// Reads a pixel; outside the canvas is transparent.
    pub fn get(&self, x: i32, y: i32) -> Rgba {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let i = ((y as u32 * self.width + x as u32) * 4) as usize;
            [self.pixels[i], self.pixels[i + 1], self.pixels[i + 2], self.pixels[i + 3]]
        } else {
            CLEAR
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgba) {
        for py in y..y + h {
            for px in x..x + w {
                self.set(px, py, color);
            }
        }
    }

    /// Draws `other` on top of this canvas at `(x, y)`, skipping transparent pixels.
    pub fn blit(&mut self, other: &Canvas, x: i32, y: i32) {
        for oy in 0..other.height as i32 {
            for ox in 0..other.width as i32 {
                let color = other.get(ox, oy);
                if color[3] > 0 {
                    self.set(x + ox, y + oy, color);
                }
            }
        }
    }

    /// A copy shifted by `(dx, dy)`; pixels moved off the canvas are lost.
    pub fn shifted(&self, dx: i32, dy: i32) -> Canvas {
        let mut out = Canvas::new(self.width, self.height);
        out.blit(self, dx, dy);
        out
    }

    /// A copy with every pixel of colour `from` replaced by `to`.
    pub fn recolored(&self, from: Rgba, to: Rgba) -> Canvas {
        let mut out = self.clone();
        for px in out.pixels.chunks_mut(4) {
            if px == from {
                px.copy_from_slice(&to);
            }
        }
        out
    }

    /// Adds a 1px outline of `color` around all opaque pixels.
    pub fn outlined(&self, color: Rgba) -> Canvas {
        let mut out = self.clone();
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if self.get(x, y)[3] == 0
                    && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| self.get(x + dx, y + dy)[3] > 0)
                {
                    out.set(x, y, color);
                }
            }
        }
        out
    }

    /// Draws `other` at `(x, y)`, but only onto pixels that are already opaque.
    pub fn blit_inside(&mut self, other: &Canvas, x: i32, y: i32) {
        for oy in 0..other.height as i32 {
            for ox in 0..other.width as i32 {
                let color = other.get(ox, oy);
                if color[3] > 0 && self.get(x + ox, y + oy)[3] > 0 {
                    self.set(x + ox, y + oy, color);
                }
            }
        }
    }

    /// Fills the ellipse centred on `(cx, cy)` (pixel centres are at `+0.5`).
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgba) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
            for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
                let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x, y, color);
                }
            }
        }
    }

    /// Fills the circle of radius `r` centred on `(cx, cy)`.
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, color: Rgba) {
        self.ellipse(cx, cy, r, r, color);
    }

    /// Fills a polygon (even-odd rule, sampled at pixel centres).
    pub fn polygon(&mut self, points: &[(f32, f32)], color: Rgba) {
        for y in 0..self.height as i32 {
            let yc = y as f32 + 0.5;
            let mut crossings: Vec<f32> = Vec::new();
            for (i, &(x0, y0)) in points.iter().enumerate() {
                let (x1, y1) = points[(i + 1) % points.len()];
                if (y0 <= yc) != (y1 <= yc) {
                    crossings.push(x0 + (yc - y0) / (y1 - y0) * (x1 - x0));
                }
            }
            crossings.sort_by(f32::total_cmp);
            for pair in crossings.chunks_exact(2) {
                for x in 0..self.width as i32 {
                    let xc = x as f32 + 0.5;
                    if xc >= pair[0] && xc < pair[1] {
                        self.set(x, y, color);
                    }
                }
            }
        }
    }

    /// Draws a line; `thickness <= 1` gives a crisp one pixel line.
    pub fn line(&mut self, from: (f32, f32), to: (f32, f32), thickness: f32, color: Rgba) {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let thin = thickness <= 1.0;
        let steps = if thin { dx.abs().max(dy.abs()).ceil() } else { (dx.hypot(dy) * 2.0).ceil() }.max(1.0);
        for i in 0..=steps as i32 {
            let t = i as f32 / steps;
            let (x, y) = (from.0 + dx * t, from.1 + dy * t);
            if thin {
                self.set(x.floor() as i32, y.floor() as i32, color);
            } else {
                self.disc(x, y, thickness / 2.0, color);
            }
        }
    }

    /// Shades a flat shape as if lit from the top-left: pixels of colour
    /// `base` whose upper/left neighbours (within `light_depth`) are not
    /// `base` become `light`, those whose lower/right neighbours (within
    /// `dark_depth`) are not `base` become `dark`.
    pub fn bevel(&self, base: Rgba, light: Rgba, dark: Rgba, light_depth: i32, dark_depth: i32) -> Canvas {
        let mut out = self.clone();
        let edge = |x: i32, y: i32| self.get(x, y) != base;
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if edge(x, y) {
                    continue;
                }
                if (1..=light_depth).any(|d| edge(x - d, y) || edge(x, y - d)) {
                    out.set(x, y, light);
                } else if (1..=dark_depth).any(|d| edge(x + d, y) || edge(x, y + d)) {
                    out.set(x, y, dark);
                }
            }
        }
        out
    }

    /// A copy where every opaque pixel has its alpha replaced by `alpha`.
    pub fn with_alpha(&self, alpha: u8) -> Canvas {
        let mut out = self.clone();
        for px in out.pixels.chunks_mut(4) {
            if px[3] > 0 {
                px[3] = alpha;
            }
        }
        out
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.pixels
    }
}
