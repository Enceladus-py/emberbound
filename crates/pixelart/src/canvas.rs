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

    pub fn into_rgba(self) -> Vec<u8> {
        self.pixels
    }
}
