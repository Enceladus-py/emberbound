//! The 24x24 boon card icons. Each is built from a few outlined parts on a
//! transparent canvas, lit from the top-left, and gets a final plum outline.

use super::draw::{
    FIRE, arc, ember, flame, flipped_v, heart, layer, paint, rotated_cw, sparkle, stack, sword,
};
use crate::BoonGlyph;
use crate::canvas::{CLEAR, Canvas, Rgba};
use crate::palette::{art, *};
use std::f32::consts::PI;

pub fn icon(glyph: BoonGlyph) -> Canvas {
    use BoonGlyph::*;
    let icon = match glyph {
        Whetstone => whetstone(),
        ShieldHeart => shield_heart(),
        WingedBoot => winged_boot(),
        Eye => eye(),
        FlameHeart => flame_heart(),
        Wing => wing(),
        Magnet => magnet(),
        Scales => scales(),
        Crescent => crescent(),
        Campfire => campfire(),
        QuickSword => quick_sword(),
        Bellows => bellows(),
        FlameSteps => flame_steps(),
        Thorns => thorns(),
        BloodBlade => blood_blade(),
        Wave => wave(),
        TwinFlames => twin_flames(),
        CrackedShield => cracked_shield(),
        MoltenHeart => molten_heart(),
        Feather => feather(),
        EmberPile => ember_pile(),
        BurningSkull => burning_skull(),
        Hearth => hearth(),
    };
    centred(&finish(&icon))
}

/// The icon shifted so its bounding box sits in the middle of the frame.
fn centred(icon: &Canvas) -> Canvas {
    let (mut lo, mut hi) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
    for y in 0..icon.height() as i32 {
        for x in 0..icon.width() as i32 {
            if icon.get(x, y)[3] > 0 {
                (lo, hi) = ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)));
            }
        }
    }
    let last = icon.width() as i32 - 1;
    icon.shifted((last - hi.0 - lo.0) / 2, (last - hi.1 - lo.1) / 2)
}

/// Closes the outline around everything not yet outlined (glints, rays),
/// without thickening the outlines the stacked parts already have.
fn finish(icon: &Canvas) -> Canvas {
    let mut out = icon.clone();
    for y in 0..icon.height() as i32 {
        for x in 0..icon.width() as i32 {
            let lit = |dx: i32, dy: i32| {
                let px = icon.get(x + dx, y + dy);
                px[3] > 0 && px != OUTLINE
            };
            if icon.get(x, y)[3] == 0 && (lit(1, 0) || lit(-1, 0) || lit(0, 1) || lit(0, -1)) {
                out.set(x, y, OUTLINE);
            }
        }
    }
    out
}

/// The white-gold light of sword arcs, outer edge first.
const LIGHT: [Rgba; 4] = [WHITE, CREAM, GOLD, ORANGE];

fn whetstone() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let stone = layer(|c| {
        c.fill_rect(3, 15, 18, 6, STONE);
        c.fill_rect(3, 15, 18, 2, STONE_LIGHT);
        c.fill_rect(3, 20, 18, 1, STONE_DARK);
        c.fill_rect(20, 17, 1, 4, STONE_DARK);
        for (x, y) in [(3, 15), (20, 15), (3, 20), (20, 20)] {
            c.set(x, y, CLEAR);
        }
        c.set(7, 18, STONE_LIGHT);
        c.set(14, 19, STONE_DARK);
    });
    stack(&mut icon, &stone);
    // Drawn pointing up-right, then flipped so the tip rests on the stone.
    let blade = flipped_v(&layer(|c| sword(c, 7, 17, 10)));
    stack(&mut icon, &blade);
    let glints = layer(|c| {
        sparkle(c, 10, 9, 2);
        sparkle(c, 19, 12, 2);
        c.set(21, 15, GOLD);
        c.set(18, 10, CREAM);
    });
    icon.blit(&glints, 0, 0);
    icon
}

/// A heater shield, `inset` pixels smaller than the full silhouette.
fn shield(c: &mut Canvas, inset: f32, color: Rgba) {
    let (top, bottom, half) = (2.5 + inset, 21.5 - inset * 1.6, 8.5 - inset);
    paint(c, |x, y| {
        if y < top || y > bottom {
            return None;
        }
        let t = (y - top) / (bottom - top);
        let hw = if t < 0.4 { half } else { half * (1.0 - ((t - 0.4) / 0.6).powf(1.6)) };
        let corner = y - top < 1.0 && (x - 12.0).abs() > hw - 1.0;
        ((x - 12.0).abs() <= hw && !corner).then_some(color)
    });
}

/// The shield body: a steel rim around a blue field.
fn shield_body() -> Canvas {
    let mut body = layer(|c| shield(c, 0.0, STEEL)).bevel(STEEL, WHITE, STONE_LIGHT, 1, 1);
    body.blit(&layer(|c| shield(c, 1.6, BLUE)).bevel(BLUE, SKY, NAVY, 1, 2), 0, 0);
    body
}

fn shield_heart() -> Canvas {
    let mut icon = shield_body();
    let love = layer(|c| heart(c, 12.0, 10.5, 4.3, 4.3, RED)).bevel(RED, CORAL, MAROON, 1, 1);
    stack(&mut icon, &love);
    icon.set(9, 8, WHITE);
    icon
}

fn winged_boot() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    // Three feathers fanning back from the heel, the top one in front.
    for (from, to) in [((9.0, 13.0), (4.0, 13.0)), ((9.0, 11.0), (3.5, 8.0)), ((10.0, 9.0), (5.0, 3.0))] {
        let feather = layer(|c| c.line(from, to, 3.2, WHITE)).bevel(WHITE, WHITE, STEEL, 1, 1);
        stack(&mut icon, &feather);
    }
    let boot = layer(|c| {
        c.fill_rect(10, 5, 6, 10, BARK);
        c.polygon(
            &[(10.0, 13.0), (16.0, 13.0), (19.0, 14.0), (21.5, 16.5), (21.5, 19.0), (10.0, 19.0)],
            BARK,
        );
    })
    .bevel(BARK, TAN, BARK_DARK, 1, 1);
    let mut boot = boot;
    boot.fill_rect(9, 3, 8, 3, TAN);
    boot.fill_rect(9, 3, 8, 1, SAND);
    boot.fill_rect(10, 19, 12, 2, BARK_DARK);
    for y in [8, 10, 12] {
        boot.set(15, y, SAND);
    }
    stack(&mut icon, &boot);
    icon
}

fn eye() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let lens = |x: f32, y: f32| {
        let dx = (x - 12.0) / 9.6;
        (y - 12.5).abs() <= 6.5 * (1.0 - dx * dx)
    };
    let white = layer(|c| paint(c, |x, y| lens(x, y).then_some(WHITE))).bevel(WHITE, WHITE, STEEL, 1, 2);
    stack(&mut icon, &white);
    let mut iris = layer(|c| {
        c.disc(12.0, 12.5, 5.0, ORANGE);
        c.disc(11.6, 12.1, 3.9, GOLD);
        c.ellipse(12.0, 12.5, 1.0, 4.6, OUTLINE);
        c.fill_rect(9, 9, 2, 2, WHITE);
    });
    iris.set(12, 16, RED);
    icon.blit_inside(&iris, 0, 0);
    let rays = layer(|c| {
        c.fill_rect(12, 1, 1, 3, GOLD);
        c.line((5.0, 3.0), (6.9, 5.9), 1.0, GOLD);
        c.line((19.0, 3.0), (17.1, 5.9), 1.0, GOLD);
    });
    icon.blit(&rays, 0, 0);
    icon
}

fn flame_heart() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let fire = layer(|c| {
        flame(c, 7.0, 10.0, 9.5, 3.4, 0.3, &FIRE[..3]);
        flame(c, 12.5, 11.0, 8.0, 2.8, -0.2, &FIRE[..3]);
        flame(c, 17.0, 9.5, 7.5, 3.0, -0.35, &FIRE[..3]);
        heart(c, 12.0, 13.0, 7.2, 7.6, RED);
        heart(c, 12.0, 13.6, 5.6, 6.0, ORANGE);
        flame(c, 12.0, 19.5, 10.0, 3.6, 0.0, &FIRE[2..]);
    });
    stack(&mut icon, &fire);
    icon
}

fn wing() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let (shoulder, elbow, wrist) = ((3.0, 15.0), (7.0, 6.0), (19.0, 2.5));
    let tips = [(21.5, 16.0), (15.0, 20.5), (8.0, 20.5)];
    // The trailing edge scallops up towards the wrist between finger tips.
    let mut outline = vec![shoulder, elbow, wrist, tips[0]];
    for (from, to, pull) in
        [(tips[0], tips[1], (15.5, 11.0)), (tips[1], tips[2], (10.5, 13.0)), (tips[2], shoulder, (5.5, 14.5))]
    {
        for i in 1..=6 {
            let t = i as f32 / 6.0;
            let (a, b, c) = ((1.0 - t) * (1.0 - t), 2.0 * t * (1.0 - t), t * t);
            outline.push((a * from.0 + b * pull.0 + c * to.0, a * from.1 + b * pull.1 + c * to.1));
        }
    }
    let membrane = layer(|c| c.polygon(&outline, ORANGE)).bevel(ORANGE, GOLD, RED, 1, 1);
    stack(&mut icon, &membrane);
    let bones = layer(|c| {
        for tip in tips {
            c.line(wrist, tip, 1.0, MAROON);
        }
        c.line(shoulder, elbow, 2.4, RED);
        c.line(elbow, wrist, 2.4, RED);
        c.line((elbow.0 - 0.6, elbow.1 - 0.6), (wrist.0 - 0.4, wrist.1 - 0.6), 1.0, CORAL);
        c.line((shoulder.0 - 0.6, shoulder.1), (elbow.0 - 0.6, elbow.1), 1.0, CORAL);
    });
    icon.blit(&bones, 0, 0);
    let claw = layer(|c| {
        c.set(20, 1, CREAM);
        c.set(21, 2, CREAM);
        c.set(20, 2, SAND);
    });
    icon.blit(&claw, 0, 0);
    icon
}

fn magnet() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let (cx, cy) = (12.0, 13.0);
    let horseshoe = |x: f32, y: f32, top: f32| {
        let d = (x - cx).hypot(y - cy);
        if y >= cy { (4.0..=8.5).contains(&d) } else { y >= top && (4.0..=8.5).contains(&(x - cx).abs()) }
    };
    let body =
        layer(|c| paint(c, |x, y| horseshoe(x, y, 10.0).then_some(RED))).bevel(RED, CORAL, MAROON, 1, 1);
    stack(&mut icon, &body);
    let poles = layer(|c| paint(c, |x, y| (horseshoe(x, y, 6.5) && y < 10.0).then_some(STEEL))).bevel(
        STEEL,
        WHITE,
        STONE_LIGHT,
        1,
        1,
    );
    stack(&mut icon, &poles);
    let embers = layer(|c| {
        ember(c, 12.0, 9.0, 2.2);
        ember(c, 7.0, 3.0, 1.6);
        ember(c, 17.0, 2.5, 1.6);
        c.set(12, 4, ORANGE);
        c.set(12, 2, RED);
    });
    stack(&mut icon, &embers);
    icon
}

fn scales() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    // Lowest row first, so each row overlaps the top of the one below and
    // only the pointed free edges of the lower scales show.
    let rows: [(f32, &[f32]); 4] =
        [(15.3, &[8.75, 15.25]), (11.5, &[5.5, 12.0, 18.5]), (7.7, &[8.75, 15.25]), (3.9, &[12.0])];
    for (cy, xs) in rows {
        for &cx in xs {
            let mut scale = layer(|c: &mut Canvas| {
                c.ellipse(cx, cy + 1.0, 3.5, 3.0, CORAL);
                c.polygon(&[(cx - 3.2, cy + 2.0), (cx + 3.2, cy + 2.0), (cx, cy + 6.0)], CORAL);
            })
            .bevel(CORAL, SKIN, RED, 1, 2);
            for d in 0..3 {
                scale.set((cx - 0.5) as i32, (cy + 2.5) as i32 + d, if d == 0 { CREAM } else { SKIN });
            }
            stack(&mut icon, &scale);
        }
    }
    icon
}

fn crescent() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let sweep = layer(|c| arc(c, 12.0, 17.5, 9.8, 7.2, PI - 0.15, PI + 0.3, &LIGHT, 0.6));
    stack(&mut icon, &sweep);
    icon
}

fn campfire() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let log = |from, to| {
        let mut log = layer(|c| c.line(from, to, 3.2, BARK)).bevel(BARK, TAN, BARK_DARK, 1, 1);
        log.disc(to.0, to.1, 1.3, SAND);
        log
    };
    stack(&mut icon, &log((19.0, 15.5), (4.5, 20.0)));
    let fire = layer(|c| {
        flame(c, 7.5, 18.0, 8.0, 2.5, -0.3, &FIRE[..3]);
        flame(c, 16.5, 18.0, 9.0, 2.5, 0.3, &FIRE[..3]);
        flame(c, 12.0, 19.0, 17.0, 5.5, 0.05, &FIRE);
    });
    stack(&mut icon, &fire);
    stack(&mut icon, &log((5.0, 15.5), (19.5, 20.0)));
    icon
}

fn quick_sword() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let streaks = layer(|c| {
        for (y, x0) in [(7, 3), (12, 2), (17, 4)] {
            c.fill_rect(x0, y, 8 - x0, 1, STONE_LIGHT);
            c.fill_rect(x0 + 2, y, 6 - x0, 1, STEEL);
            c.fill_rect(x0 + 4, y, 4 - x0, 1, WHITE);
        }
    });
    icon.blit(&streaks, 0, 0);
    let blade = layer(|c| {
        c.fill_rect(9, 11, 2, 3, GOLD);
        c.fill_rect(11, 8, 2, 9, GOLD);
        c.fill_rect(12, 8, 1, 9, ORANGE);
        c.fill_rect(13, 11, 8, 1, WHITE);
        c.fill_rect(13, 12, 9, 1, STEEL);
        c.fill_rect(13, 13, 8, 1, STONE_LIGHT);
        c.set(9, 11, CREAM);
    });
    stack(&mut icon, &blade);
    icon
}

fn bellows() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    // Accordion leather between the boards, pleats fanning to the nozzle.
    let leather = layer(|c| {
        c.polygon(
            &[
                (4.0, 6.5),
                (15.5, 11.0),
                (15.5, 13.0),
                (4.0, 17.5),
                (2.6, 15.5),
                (4.0, 14.0),
                (2.6, 12.0),
                (4.0, 10.0),
                (2.6, 8.5),
            ],
            RED,
        );
    });
    let mut leather = leather.bevel(RED, CORAL, MAROON, 1, 1);
    leather.line((4.0, 10.0), (15.0, 11.7), 1.0, MAROON);
    leather.line((4.0, 14.0), (15.0, 12.3), 1.0, MAROON);
    stack(&mut icon, &leather);
    for (end, knob) in [((3.8, 5.0), (3.2, 4.0)), ((3.8, 19.0), (3.2, 20.0))] {
        let board = layer(|c| {
            c.line(end, (15.5, if end.1 < 12.0 { 10.3 } else { 13.7 }), 2.6, BARK);
            c.disc(knob.0, knob.1, 1.4, BARK);
        })
        .bevel(BARK, TAN, BARK_DARK, 1, 1);
        stack(&mut icon, &board);
    }
    let nozzle = layer(|c| {
        c.fill_rect(15, 11, 3, 3, GOLD);
        c.fill_rect(15, 13, 3, 1, ORANGE);
        c.fill_rect(15, 11, 3, 1, CREAM);
    });
    stack(&mut icon, &nozzle);
    // A flame drawn pointing up, turned to blow out of the nozzle.
    let puff = rotated_cw(&layer(|c| flame(c, 12.0, 6.0, 4.4, 3.0, 0.0, &FIRE)));
    stack(&mut icon, &puff);
    icon
}

fn flame_steps() -> Canvas {
    // A right footprint glowing like a coal, its toes flickering flame tips.
    const FOOT: [&str; 14] = [
        "..Y.....", ".yYy.y..", ".OYO.O.y", ".rOr.r.O", "......rr", "..rOOOr.", ".rOyyyOr", ".rOyYyOr",
        ".rOyYyO.", "..rOyOr.", "..rOyOr.", "..rOyOr.", "..rOOOr.", "...rrr..",
    ];
    let right = art(&FOOT);
    let mut left = Canvas::new(8, 14);
    for y in 0..14 {
        for x in 0..8 {
            left.set(7 - x, y, right.get(x, y));
        }
    }
    let mut icon = Canvas::new(24, 24);
    for (foot, x, y) in [(&right, 13, 2), (&left, 3, 8)] {
        let mut print = Canvas::new(24, 24);
        print.blit(foot, x, y);
        stack(&mut icon, &print);
    }
    icon
}

fn thorns() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let (start, end) = ((5.0f32, 19.0f32), (19.0f32, 5.0f32));
    let len = (end.0 - start.0).hypot(end.1 - start.1);
    let (dx, dy) = ((end.0 - start.0) / len, (end.1 - start.1) / len);
    let point = |s: f32| {
        let wiggle = 2.8 * (s * 2.0 * PI).sin();
        (start.0 + dx * len * s - dy * wiggle, start.1 + dy * len * s + dx * wiggle)
    };
    let vine = layer(|c| {
        let mut prev = point(0.0);
        for i in 1..=40 {
            let next = point(i as f32 / 40.0);
            c.line(prev, next, 3.8, GREEN);
            prev = next;
        }
    })
    .bevel(GREEN, LIME, GREEN_DEEP, 1, 1);
    stack(&mut icon, &vine);
    let thorns = layer(|c| {
        for (i, s) in [0.1, 0.3, 0.5, 0.7, 0.9].into_iter().enumerate() {
            let (p, q) = (point(s), point(s + 0.02));
            let (tx, ty) = (q.0 - p.0, q.1 - p.1);
            let tl = tx.hypot(ty);
            let (tx, ty) = (tx / tl, ty / tl);
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let (nx, ny) = (-ty * side, tx * side);
            let root = (p.0 + nx * 1.2, p.1 + ny * 1.2);
            let tip = (p.0 + nx * 5.5 + tx * 1.5, p.1 + ny * 5.5 + ty * 1.5);
            c.polygon(
                &[(root.0 - tx * 1.8, root.1 - ty * 1.8), (root.0 + tx * 1.8, root.1 + ty * 1.8), tip],
                SAND,
            );
        }
    })
    .bevel(SAND, SAND_LIGHT, TAN, 1, 1);
    stack(&mut icon, &thorns);
    icon
}

fn blood_blade() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let blade = layer(|c| {
        c.fill_rect(10, 2, 3, 1, GOLD);
        c.fill_rect(11, 3, 1, 2, BARK);
        c.fill_rect(5, 5, 13, 2, GOLD);
        c.fill_rect(5, 6, 13, 1, ORANGE);
        c.polygon(&[(9.0, 7.0), (14.0, 7.0), (14.0, 14.0), (11.5, 18.5), (9.0, 14.0)], STEEL);
        c.fill_rect(9, 7, 1, 8, WHITE);
        c.fill_rect(13, 7, 1, 8, STONE_LIGHT);
        c.fill_rect(11, 8, 1, 7, WHITE);
        // Blood coats the lower blade, running down in uneven streaks.
        for (x, top) in [(9, 13), (10, 11), (11, 12), (12, 10), (13, 12)] {
            for y in top..19 {
                if c.get(x, y)[3] > 0 {
                    c.set(x, y, if y == top { CORAL } else { RED });
                }
            }
        }
        c.set(12, 16, MAROON);
        c.set(10, 15, MAROON);
    });
    stack(&mut icon, &blade);
    let drop = layer(|c| {
        c.disc(11.5, 20.6, 1.6, RED);
        c.set(11, 19, RED);
        c.set(11, 20, CORAL);
    });
    stack(&mut icon, &drop);
    icon
}

fn wave() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let streaks = layer(|c| {
        for (y, x0, x1) in [(8, 2, 9), (12, 1, 10), (16, 2, 9)] {
            c.fill_rect(x0, y, x1 - x0, 1, GOLD);
            c.fill_rect(x0 + 2, y, x1 - x0 - 2, 1, CREAM);
        }
    });
    icon.blit(&streaks, 0, 0);
    let wave = layer(|c| arc(c, 9.0, 12.0, 11.5, 6.5, -1.0, 2.0, &LIGHT, 1.0));
    stack(&mut icon, &wave);
    icon
}

fn twin_flames() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    stack(&mut icon, &layer(|c| flame(c, 8.0, 21.5, 15.0, 5.0, -0.15, &FIRE)));
    stack(&mut icon, &layer(|c| flame(c, 15.5, 21.5, 19.0, 5.5, 0.12, &FIRE)));
    icon
}

fn cracked_shield() -> Canvas {
    let mut icon = shield_body();
    let crack = [(13.0, 2.0), (11.0, 7.0), (14.0, 11.0), (11.0, 15.0), (12.5, 18.0)];
    for pair in crack.windows(2) {
        icon.line((pair[0].0 + 0.5, pair[0].1), (pair[1].0 + 0.5, pair[1].1), 1.0, STONE_LIGHT);
        icon.line(pair[0], pair[1], 1.0, OUTLINE);
    }
    // A chip knocked out of the rim.
    for (x, y) in [(19, 3), (20, 3), (20, 4), (20, 5)] {
        icon.set(x, y, CLEAR);
    }
    icon
}

fn molten_heart() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let crust = layer(|c| heart(c, 12.0, 11.5, 8.2, 8.0, STONE_DARK)).bevel(STONE_DARK, STONE, COAL, 1, 2);
    stack(&mut icon, &crust);
    let core = (12.0, 11.0);
    let cracks = layer(|c| {
        for path in [
            [core, (9.0, 8.0), (6.0, 5.0)],
            [core, (15.0, 8.0), (16.5, 4.0)],
            [core, (11.0, 15.0), (12.0, 19.0)],
            [core, (16.0, 13.0), (19.5, 10.0)],
            [core, (7.5, 12.5), (4.0, 9.0)],
        ] {
            for pair in path.windows(2) {
                c.line(pair[0], pair[1], 2.0, RED);
            }
            for pair in path.windows(2) {
                c.line(pair[0], pair[1], 1.0, ORANGE);
            }
        }
        c.disc(12.0, 11.0, 2.6, ORANGE);
        c.disc(12.0, 11.0, 1.6, GOLD);
        c.set(11, 10, CREAM);
    });
    icon.blit_inside(&cracks, 0, 0);
    let drip = layer(|c| {
        c.fill_rect(12, 19, 1, 2, ORANGE);
        c.set(12, 21, RED);
    });
    icon.blit(&drip, 0, 0);
    icon
}

fn feather() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let (base, tip) = ((4.0f32, 21.5f32), (20.5f32, 2.5f32));
    let len = (tip.0 - base.0).hypot(tip.1 - base.1);
    let (dx, dy) = ((tip.0 - base.0) / len, (tip.1 - base.1) / len);
    let width = |t: f32| 5.5 * ((t - 0.22) / 0.78 * PI).sin().max(0.0).powf(0.6);
    // Flames licking off the lower edge of the vane.
    let fire = layer(|c| {
        for (t, h) in [(0.42, 6.0), (0.6, 7.0), (0.78, 5.0)] {
            let w = width(t) + 1.8 * (t * PI).sin();
            let (x, y) = (base.0 + dx * len * t - dy * w, base.1 + dy * len * t + dx * w);
            flame(c, x + 0.5, y + 1.0, h, 1.8, 0.25, &FIRE[..3]);
        }
    });
    stack(&mut icon, &fire);
    let vane = layer(|c| {
        paint(c, |x, y| {
            let (px, py) = (x - base.0, y - base.1);
            let t = (px * dx + py * dy) / len;
            let across = px * dy - py * dx - 1.8 * (t * PI).sin();
            if !(0.0..=1.0).contains(&t) {
                return None;
            }
            if across.abs() <= 0.6 && t < 0.95 {
                return Some(CREAM);
            }
            let w = width(t);
            if t < 0.22 || across.abs() > w {
                return None;
            }
            // Notches between the barbs.
            let along_px = t * len;
            if across.abs() > 0.55 * w && ((along_px + across.abs() * 0.9) as i32).rem_euclid(5) == 0 {
                return None;
            }
            let edge = across.abs() > 0.6 * w;
            Some(match (t, edge) {
                (t, false) if t > 0.75 => CREAM,
                (t, true) if t > 0.75 => GOLD,
                (t, false) if t > 0.5 => GOLD,
                (t, true) if t > 0.5 => ORANGE,
                (t, false) if t > 0.33 => ORANGE,
                _ => RED,
            })
        });
    });
    stack(&mut icon, &vane);
    let glints = layer(|c| {
        sparkle(c, 5, 6, 2);
        sparkle(c, 20, 18, 2);
        c.set(9, 2, GOLD);
    });
    icon.blit(&glints, 0, 0);
    icon
}

fn ember_pile() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    for (cx, cy) in [(6.0, 18.0), (12.0, 19.0), (18.0, 18.0), (9.0, 14.5), (15.0, 14.5), (12.0, 10.5)] {
        let coin = layer(|c: &mut Canvas| {
            c.ellipse(cx, cy + 1.4, 3.6, 2.2, RED);
            c.ellipse(cx, cy, 3.6, 2.2, ORANGE);
            c.ellipse(cx - 0.2, cy - 0.2, 2.4, 1.3, GOLD);
            c.set(cx as i32, cy as i32, ORANGE);
            c.set((cx - 2.0) as i32, (cy - 1.0) as i32, CREAM);
        });
        stack(&mut icon, &coin);
    }
    let sparks = layer(|c| {
        sparkle(c, 6, 8, 2);
        sparkle(c, 18, 5, 1);
        c.set(15, 3, GOLD);
        c.set(9, 4, ORANGE);
    });
    icon.blit(&sparks, 0, 0);
    icon
}

fn burning_skull() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let fire = layer(|c| {
        flame(c, 6.0, 20.0, 12.0, 3.0, -0.25, &FIRE[..3]);
        flame(c, 18.0, 20.0, 12.0, 3.0, 0.25, &FIRE[..3]);
        flame(c, 12.0, 20.0, 19.5, 8.0, 0.0, &FIRE[..3]);
    });
    stack(&mut icon, &fire);
    let mut skull = layer(|c| {
        c.ellipse(12.0, 13.5, 5.6, 5.0, SAND_LIGHT);
        c.fill_rect(9, 16, 7, 5, SAND_LIGHT);
    })
    .bevel(SAND_LIGHT, WHITE, SAND, 1, 1);
    for cx in [9.5, 14.5] {
        skull.ellipse(cx, 14.5, 1.6, 1.7, COAL);
    }
    skull.set(9, 14, ORANGE);
    skull.set(14, 14, ORANGE);
    skull.set(12, 17, COAL);
    skull.fill_rect(10, 19, 5, 1, COAL);
    skull.set(12, 20, COAL);
    stack(&mut icon, &skull);
    icon
}

fn hearth() -> Canvas {
    let mut icon = Canvas::new(24, 24);
    let opening =
        |x: f32, y: f32| (x - 12.0).abs() <= 5.0 && (y >= 12.0 || (x - 12.0).hypot((y - 12.0) * 1.2) <= 5.0);
    let stone = layer(|c| {
        c.fill_rect(3, 7, 18, 15, STONE);
        for y in [10, 14, 18] {
            c.fill_rect(3, y, 18, 1, STONE_DARK);
        }
        for (row, top) in [7, 11, 15, 19].into_iter().enumerate() {
            for x in (3 + (row as i32 % 2) * 3..21).step_by(6) {
                c.fill_rect(x, top, 1, 3, STONE_DARK);
            }
        }
        paint(c, |x, y| opening(x, y).then_some(COAL));
    })
    .bevel(STONE, STONE_LIGHT, STONE, 1, 0);
    stack(&mut icon, &stone);
    let mantel = layer(|c| c.fill_rect(2, 4, 20, 3, BARK)).bevel(BARK, TAN, BARK_DARK, 1, 1);
    stack(&mut icon, &mantel);
    let fire = layer(|c| flame(c, 12.0, 22.0, 14.0, 4.8, 0.0, &FIRE[1..]));
    icon.blit_inside(&fire, 0, 0);
    // Too small for the heart curve to keep its notch, so drawn by hand.
    let mut love = Canvas::new(24, 24);
    love.blit(&art(&[".rr.rr.", "rwRrrrr", "rRrrrrm", ".rrrrm.", "..rrm..", "...m..."]), 9, 14);
    stack(&mut icon, &love);
    icon
}
