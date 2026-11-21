//! The three elder dragons.
//!
//! They share one anatomy, assembled from parts (tail, body, neck, legs,
//! head, wings) in a 48x48 design space and scaled to the sprite size. A
//! [`Look`] gives each dragon its colours, proportions and ornaments.

use super::hash;
use crate::canvas::{Canvas, Rgba};
use crate::palette::{
    BARK, BLUE, COAL, CREAM, FROST, GOLD, GRASS, GREEN, GREEN_DEEP, ICE, LIME, MAROON, ORANGE, OUTLINE, RED,
    SAND, SKY, STEEL, STONE, STONE_DARK, STONE_LIGHT, TAN, WHITE,
};

type Point = (f32, f32);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Moss,
    Frost,
    Ashen,
}

struct Look {
    kind: Kind,
    /// Sprite size in pixels; the design space is 48 units.
    size: u32,
    /// Scales: dark, mid, light.
    hide: [Rgba; 3],
    /// Belly plates and the seams between them.
    belly: [Rgba; 2],
    /// Wing membrane: dark, mid, light.
    membrane: [Rgba; 3],
    /// Wing membrane of the far wing.
    far_membrane: Rgba,
    /// Wing finger bones.
    bone: Rgba,
    /// Horns and claws: base, highlight.
    horn: [Rgba; 2],
    /// Dorsal spikes.
    spike: Rgba,
    eye: Rgba,
    /// How far dorsal spikes stick out, in design units.
    spike_reach: f32,
    horn_length: f32,
    horn_width: f32,
    wing_scale: f32,
}

/// One part of the dragon, drawn in design units.
struct Layer {
    canvas: Canvas,
    scale: f32,
    /// Raises the part (the body heaves up on the wing downstroke).
    lift: f32,
}

impl Layer {
    fn new(look: &Look, lift: f32) -> Self {
        Self { canvas: Canvas::new(look.size, look.size), scale: look.size as f32 / 48.0, lift }
    }

    fn at(&self, p: Point) -> Point {
        (p.0 * self.scale, (p.1 - self.lift) * self.scale)
    }

    fn ellipse(&mut self, centre: Point, rx: f32, ry: f32, color: Rgba) {
        let c = self.at(centre);
        self.canvas.ellipse(c.0, c.1, rx * self.scale, ry * self.scale, color);
    }

    fn disc(&mut self, centre: Point, r: f32, color: Rgba) {
        self.ellipse(centre, r, r, color);
    }

    fn polygon(&mut self, points: &[Point], color: Rgba) {
        let points: Vec<Point> = points.iter().map(|&p| self.at(p)).collect();
        self.canvas.polygon(&points, color);
    }

    fn line(&mut self, from: Point, to: Point, thickness: f32, color: Rgba) {
        let (from, to) = (self.at(from), self.at(to));
        self.canvas.line(from, to, thickness * self.scale, color);
    }

    /// A small square detail of `size` design units (at least one pixel).
    fn dot(&mut self, p: Point, size: f32, color: Rgba) {
        let (x, y) = self.at(p);
        let px = (size * self.scale).round().max(1.0) as i32;
        self.canvas.fill_rect(x.floor() as i32, y.floor() as i32, px, px, color);
    }

    /// A tapering tube along `curve`, e.g. a horn.
    fn taper(&mut self, curve: [Point; 4], from: f32, to: f32, color: Rgba) {
        for (t, p, _) in samples(curve, 24) {
            self.disc(p, from + (to - from) * t, color);
        }
    }

    /// A triangular spike standing on `base` and pointing along `normal`.
    fn spike(&mut self, base: Point, normal: Point, reach: f32, half_width: f32, color: Rgba) {
        let side = (-normal.1 * half_width, normal.0 * half_width);
        self.polygon(
            &[
                (base.0 + side.0, base.1 + side.1),
                (base.0 + normal.0 * reach, base.1 + normal.1 * reach),
                (base.0 - side.0, base.1 - side.1),
            ],
            color,
        );
    }

    fn shade(&mut self, [dark, mid, light]: [Rgba; 3]) {
        let depth = if self.scale > 1.2 { 2 } else { 1 };
        self.canvas = self.canvas.bevel(mid, light, dark, depth, depth + 1);
    }
}

fn bezier(curve: [Point; 4], t: f32) -> Point {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    (
        w[0] * curve[0].0 + w[1] * curve[1].0 + w[2] * curve[2].0 + w[3] * curve[3].0,
        w[0] * curve[0].1 + w[1] * curve[1].1 + w[2] * curve[2].1 + w[3] * curve[3].1,
    )
}

/// `(t, point, unit tangent)` at `count + 1` steps along a curve.
fn samples(curve: [Point; 4], count: usize) -> Vec<(f32, Point, Point)> {
    (0..=count)
        .map(|i| {
            let t = i as f32 / count as f32;
            let (a, b) = (bezier(curve, (t - 0.02).max(0.0)), bezier(curve, (t + 0.02).min(1.0)));
            let len = (b.0 - a.0).hypot(b.1 - a.1).max(1e-4);
            (t, bezier(curve, t), ((b.0 - a.0) / len, (b.1 - a.1) / len))
        })
        .collect()
}

/// Spine curves, from the body outwards.
const TAIL: [Point; 4] = [(15.0, 33.0), (7.0, 41.0), (-1.0, 38.0), (4.5, 28.5)];
const NECK: [Point; 4] = [(28.0, 28.0), (35.5, 27.0), (30.5, 19.0), (36.0, 14.5)];
const BODY: Point = (22.0, 31.0);
/// The line the belly plates follow under the body.
const UNDERSIDE: [Point; 4] = [(14.5, 35.5), (19.0, 39.0), (26.0, 38.5), (29.5, 31.5)];

fn tail_radius(t: f32) -> f32 {
    4.6 - 3.6 * t
}

fn neck_radius(t: f32) -> f32 {
    5.4 - 2.0 * t
}

fn torso(look: &Look, lift: f32, pulse: bool) -> Canvas {
    let mut layer = Layer::new(look, lift);
    let [_, mid, light] = look.hide;
    let (tail, neck) = (samples(TAIL, 30), samples(NECK, 18));

    // Tail tip ornament and dorsal spikes go first: the body covers their roots.
    let (_, tip, tip_dir) = tail[30];
    match look.kind {
        Kind::Moss => {
            layer.ellipse((tip.0 + tip_dir.0 * 2.0, tip.1 + tip_dir.1 * 2.5), 2.2, 3.2, LIME);
            layer.line(tip, (tip.0 + tip_dir.0 * 4.0, tip.1 + tip_dir.1 * 5.0), 1.0, GREEN);
        }
        Kind::Frost => {
            layer.spike(tip, tip_dir, 6.5, 2.4, FROST);
            layer.spike(tip, (-tip_dir.0, -tip_dir.1), 2.5, 2.4, ICE);
        }
        Kind::Ashen => {
            layer.spike(tip, tip_dir, 7.0, 2.8, ORANGE);
            layer.spike(tip, tip_dir, 4.5, 1.4, GOLD);
        }
    }
    for &(t, p, dir) in tail.iter().skip(2).step_by(4).take(4) {
        layer.spike(p, (-dir.1, dir.0), tail_radius(t) + look.spike_reach * (1.0 - t * 0.6), 1.5, look.spike);
    }
    for &(t, p, dir) in neck.iter().skip(2).step_by(4).take(4) {
        layer.spike(p, (dir.1, -dir.0), neck_radius(t) + look.spike_reach, 1.5, look.spike);
    }
    for angle in [62.0_f32, 86.0, 108.0, 130.0] {
        let (sin, cos) = angle.to_radians().sin_cos();
        layer.spike(
            (BODY.0 + 8.0 * cos, BODY.1 - 5.0 * sin),
            (cos * 0.5, -sin.max(0.9)),
            3.0 + look.spike_reach,
            1.6,
            look.spike,
        );
    }

    for &(t, p, _) in &tail {
        layer.disc(p, tail_radius(t), mid);
    }
    layer.ellipse(BODY, 10.8, 7.6, mid);
    for &(t, p, _) in &neck {
        layer.disc(p, neck_radius(t), mid);
    }

    // Belly plates run under the tail root, the body and up the throat.
    // Overlapping discs painted in spine order leave a crescent seam
    // between one plate and the next.
    let mut plates: Vec<(Point, f32)> = Vec::new();
    for &(t, p, dir) in tail.iter().take(14).rev() {
        let r = tail_radius(t);
        plates.push(((p.0 + dir.1 * r * 0.7, p.1 - dir.0 * r * 0.7), r * 0.55));
    }
    for (_, p, _) in samples(UNDERSIDE, 24) {
        plates.push((p, 3.5));
    }
    for &(t, p, dir) in &neck {
        let r = neck_radius(t);
        plates.push(((p.0 - dir.1 * r * 0.62, p.1 + dir.0 * r * 0.62), r * 0.6));
    }
    let mut belly = Layer::new(look, lift);
    let (mut travelled, mut previous) = (0.0_f32, plates[0].0);
    for (centre, r) in plates {
        travelled += (centre.0 - previous.0).hypot(centre.1 - previous.1);
        previous = centre;
        belly.disc(centre, r, look.belly[usize::from(travelled % 3.0 < 1.0)]);
    }
    layer.canvas.blit_inside(&belly.canvas, 0, 0);
    layer.shade(look.hide);

    let size = look.size as i32;
    match look.kind {
        Kind::Moss => {
            // Moss clings to the sunlit back.
            for y in 0..size {
                for x in 0..size {
                    let c = layer.canvas.get(x, y);
                    let near_top = (1..=3).any(|d| {
                        layer.canvas.get(x, y - d)[3] == 0 || layer.canvas.get(x, y - d) == look.spike
                    });
                    if (c == mid || c == light)
                        && near_top
                        && hash(x as u32 / 2, y as u32 / 2, 9).is_multiple_of(3)
                    {
                        layer.canvas.set(x, y, LIME);
                    }
                }
            }
        }
        Kind::Frost => {
            // Rime glitters on the scales.
            for y in 0..size {
                for x in 0..size {
                    if layer.canvas.get(x, y) == mid && hash(x as u32, y as u32, 4).is_multiple_of(11) {
                        layer.canvas.set(x, y, FROST);
                    }
                }
            }
        }
        Kind::Ashen => {
            // Ember cracks wander across the charred hide.
            const STARTS: [Point; 9] = [
                (12.0, 30.0),
                (17.0, 27.0),
                (22.0, 25.5),
                (26.0, 28.0),
                (20.0, 31.0),
                (31.0, 24.0),
                (33.0, 19.0),
                (8.0, 36.0),
                (4.0, 34.0),
            ];
            for (i, start) in STARTS.into_iter().enumerate() {
                let (mut x, mut y) = layer.at(start);
                for step in 0..(7.0 * layer.scale) as u32 {
                    let (px, py) = (x as i32, y as i32);
                    if look.hide.contains(&layer.canvas.get(px, py)) || layer.canvas.get(px, py) == GOLD {
                        let hot = (step < 2) == pulse;
                        layer.canvas.set(px, py, if hot { GOLD } else { ORANGE });
                    }
                    let h = hash(i as u32, step, 21);
                    x += [1.0, 1.0, 0.0, 1.0, -1.0][(h % 5) as usize];
                    y += [0.0, 1.0, 1.0, -1.0, 1.0][((h >> 8) % 5) as usize];
                }
            }
        }
    }
    layer.canvas
}

/// A hind and a fore leg; `shift` moves the pair (for the far side).
fn legs(look: &Look, shift: Point, colors: [Rgba; 3]) -> Canvas {
    let mut layer = Layer::new(look, 0.0);
    let mid = colors[1];
    let at = |x: f32, y: f32| (x + shift.0, y + shift.1);
    // Hind leg: big thigh, shin, splayed foot.
    layer.ellipse(at(16.5, 35.0), 4.9, 5.4, mid);
    layer.polygon(&[at(14.4, 38.0), at(18.8, 38.0), at(18.6, 44.0), at(14.8, 44.0)], mid);
    layer.ellipse(at(17.8, 43.8), 3.7, 1.7, mid);
    // Fore leg.
    layer.ellipse(at(27.6, 35.0), 3.2, 4.6, mid);
    layer.polygon(&[at(26.0, 37.0), at(29.8, 37.0), at(29.6, 44.0), at(26.2, 44.0)], mid);
    layer.ellipse(at(29.4, 43.8), 3.5, 1.7, mid);
    layer.shade(colors);
    for foot in [19.0, 30.4] {
        for claw in 0..3 {
            layer.dot(at(foot - 2.6 + claw as f32 * 1.6, 44.6), 1.0, look.horn[1]);
        }
    }
    layer.canvas
}

fn head(look: &Look, lift: f32, pulse: bool) -> Canvas {
    let mut layer = Layer::new(look, lift);
    let [dark, mid, _] = look.hide;
    let [horn, horn_light] = look.horn;
    let (len, girth) = (look.horn_length, look.horn_width);

    // Far horn (the Moss Wyrm's antlers stay a single clean beam), then
    // the near horn sweeping back over the neck.
    if look.kind != Kind::Moss {
        layer.taper(
            [(38.0, 10.0), (36.0, 7.0), (34.0, 6.0), (38.5 - len * 0.8, 9.5 - len * 0.75)],
            girth * 0.8,
            0.5,
            horn,
        );
    }
    match look.kind {
        Kind::Moss => {
            // Antlers: a main beam with two tines.
            layer.line((33.4, 9.8), (33.0, 5.2), 1.0, horn);
            layer.line((30.8, 7.8), (31.4, 3.2), 1.0, horn);
            // A leafy frill behind the jaw.
            layer.spike((35.2, 16.0), (-0.8, 0.6), 4.0, 1.6, LIME);
            layer.spike((35.0, 13.8), (-1.0, 0.1), 3.6, 1.4, GRASS);
        }
        Kind::Frost => {
            // A crown of ice shards.
            for (base, dir, reach) in [
                ((35.6, 10.6), (-0.5, -0.86), 6.0),
                ((34.6, 12.6), (-0.95, -0.3), 5.5),
                ((34.8, 15.2), (-0.9, 0.42), 4.5),
            ] {
                layer.spike(base, dir, reach, 1.5, ICE);
                layer.spike(base, dir, reach - 1.6, 0.7, WHITE);
            }
        }
        Kind::Ashen => {
            // A second, lower pair of horns and chin barbs.
            layer.taper([(35.5, 14.0), (32.5, 15.0), (30.5, 14.0), (29.0, 10.5)], 1.3, 0.5, horn);
            layer.spike((40.5, 18.2), (-0.3, 0.95), 3.0, 1.0, horn);
            layer.spike((43.0, 18.0), (-0.2, 0.98), 2.4, 0.9, horn);
        }
    }
    layer.taper([(36.5, 11.2), (33.5, 10.6), (31.5, 9.0), (36.0 - len, 10.2 - len * 0.7)], girth, 0.5, horn);
    layer.canvas = layer.canvas.bevel(horn, horn_light, horn, 1, 0);

    // Skull, snout and lower jaw.
    layer.ellipse((38.0, 13.4), 4.7, 4.1, mid);
    layer.ellipse((42.4, 14.4), 4.3, 2.8, mid);
    layer.ellipse((41.4, 17.2), 3.8, 1.5, look.belly[0]);
    layer.shade(look.hide);

    // Mouth line with fangs; the Ashen Tyrant's maw glows.
    let mouth = if look.kind == Kind::Ashen { if pulse { GOLD } else { ORANGE } } else { OUTLINE };
    layer.line((39.4, 16.4), (46.0, 16.1), 1.0, mouth);
    for fang in [41.0, 43.0, 45.0] {
        layer.dot((fang, 16.1), 0.7, WHITE);
    }
    // Brow ridge over a slit eye, and a nostril.
    layer.line((37.6, 11.2), (41.2, 12.3), 1.0, dark);
    layer.dot((39.0, 12.4), 1.0, look.eye);
    layer.dot((40.0, 12.6), 1.0, look.eye);
    layer.dot((45.0, 13.4), 0.8, dark);
    layer.canvas
}

/// A wing hinged on `shoulder`. `flap` runs from 1 (raised) to -1 (swept down).
fn wing(
    look: &Look,
    lift: f32,
    shoulder: Point,
    flap: f32,
    scale: f32,
    membrane: [Rgba; 3],
    near: bool,
) -> Canvas {
    let mut layer = Layer::new(look, lift);
    let turn = (flap - 1.0) * 0.5 * 44.0;
    // Fold the fingers together a little on the downstroke.
    let fan = 0.82 + 0.18 * (flap + 1.0) * 0.5;
    let pt = |angle: f32, reach: f32| {
        let (sin, cos) = (angle * fan + turn).to_radians().sin_cos();
        let reach = reach * scale * look.wing_scale;
        (shoulder.0 - reach * cos, shoulder.1 - reach * sin)
    };
    let (elbow, wrist) = (pt(108.0, 8.5), pt(84.0, 16.5));
    let tips = [pt(64.0, 25.0), pt(40.0, 24.0), pt(17.0, 20.5)];
    let outline = [
        shoulder,
        elbow,
        wrist,
        tips[0],
        pt(52.0, 17.0),
        tips[1],
        pt(28.0, 15.0),
        tips[2],
        pt(4.0, 10.5),
        pt(-14.0, 7.0),
    ];
    layer.polygon(&outline, membrane[1]);
    layer.canvas = layer.canvas.bevel(membrane[1], membrane[2], membrane[0], 1, 2);
    let arm = if near { look.hide[1] } else { look.hide[0] };
    for tip in tips {
        layer.line(wrist, tip, 1.0, look.bone);
    }
    layer.line(shoulder, elbow, 2.4, arm);
    layer.line(elbow, wrist, 2.0, arm);
    if near {
        layer.line(shoulder, elbow, 1.0, look.hide[2]);
        // The wing claw at the wrist.
        layer.spike(wrist, (0.5, -0.86), 2.6, 0.9, look.horn[1]);
    }
    layer.canvas
}

fn frame(look: &Look, flap: f32, lift: f32, pulse: bool) -> Canvas {
    let mut sprite = Canvas::new(look.size, look.size);
    let far_hide = [look.hide[0]; 3];
    let parts = [
        wing(look, lift, (28.5, 23.5), flap * 0.85 + 0.15, 0.9, [look.far_membrane; 3], false),
        legs(look, (4.0, -1.0), far_hide),
        torso(look, lift, pulse),
        legs(look, (0.0, 0.0), look.hide),
        head(look, lift, pulse),
        wing(look, lift, (24.0, 25.0), flap, 1.0, look.membrane, true),
    ];
    for part in parts {
        sprite.blit(&part.outlined(OUTLINE), 0, 0);
    }
    sprite
}

/// Wings raised, level, swept down (the body heaves up), level again.
fn flap_cycle(look: &Look) -> Vec<Canvas> {
    [(1.0, 0.0, false), (0.1, 0.0, false), (-0.8, 1.0, true), (0.1, 0.0, true)]
        .into_iter()
        .map(|(flap, lift, pulse)| frame(look, flap, lift, pulse))
        .collect()
}

/// The Moss Wyrm: forest green, antlered, overgrown.
pub fn moss() -> Vec<Canvas> {
    flap_cycle(&Look {
        kind: Kind::Moss,
        size: 48,
        hide: [GREEN_DEEP, GREEN, GRASS],
        belly: [SAND, TAN],
        membrane: [BARK, TAN, SAND],
        far_membrane: BARK,
        bone: GREEN_DEEP,
        horn: [BARK, SAND],
        spike: LIME,
        eye: GOLD,
        spike_reach: 2.0,
        horn_length: 8.0,
        horn_width: 1.0,
        wing_scale: 0.94,
    })
}

/// The Frost Wyrm: glacier blue with a crown of ice.
pub fn frost() -> Vec<Canvas> {
    flap_cycle(&Look {
        kind: Kind::Frost,
        size: 48,
        hide: [BLUE, SKY, ICE],
        belly: [FROST, ICE],
        membrane: [SKY, ICE, FROST],
        far_membrane: BLUE,
        bone: BLUE,
        horn: [STEEL, WHITE],
        spike: FROST,
        eye: GOLD,
        spike_reach: 3.4,
        horn_length: 5.0,
        horn_width: 1.4,
        wing_scale: 1.0,
    })
}

/// The Ashen Tyrant: charcoal hide split by glowing ember cracks.
pub fn ashen() -> Vec<Canvas> {
    flap_cycle(&Look {
        kind: Kind::Ashen,
        size: 64,
        hide: [COAL, STONE_DARK, STONE],
        belly: [MAROON, RED],
        membrane: [COAL, MAROON, RED],
        far_membrane: COAL,
        bone: ORANGE,
        horn: [STONE_LIGHT, CREAM],
        spike: STONE_LIGHT,
        eye: GOLD,
        spike_reach: 2.8,
        horn_length: 9.5,
        horn_width: 1.6,
        wing_scale: 1.0,
    })
}
