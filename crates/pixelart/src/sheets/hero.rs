//! The Wyrmwarden: a small knight with a steel helm, red cloak and sword.

use crate::canvas::Canvas;
use crate::palette::{OUTLINE, art};

/// Plume, helm, face, cloak, tunic and sword arm, down to the belt.
const UPPER: [&str; 11] = [
    "......rR........",
    ".....344444.....",
    "....33444443....",
    "....2333yy33....",
    "....223fffff....",
    "....222fofof..w.",
    ".....22ffff..w4.",
    "...rr.uuuu..w4..",
    "..rrruUuuunyy...",
    "..rmruuuuunf....",
    "..mrrbbybbn.....",
];

/// Hips (only seen when the body lifts), legs and boots.
const STAND: [&str; 3] = ["..mrrnnnnnn.....", "..mmr.nn.nn.....", "...m..bb.bb....."];
const STRIDE_A: [&str; 3] = ["..mrrnnnnnn.....", "..mmrnn...nn....", "...mbb.....bb..."];
const PASS: [&str; 3] = ["..mrrnnnnnn.....", "...mr..nnn......", "..mm...bbb......"];
const STRIDE_B: [&str; 3] = ["..mrrnnnnnn.....", "..mmr.nn..nn....", "..m..bb....bb..."];

/// Stacks the upper body (raised by `lift` pixels) on a leg pose.
fn pose(legs: &[&str; 3], lift: i32) -> Canvas {
    let mut frame = Canvas::new(16, 16);
    frame.blit(&art(legs), 0, 12);
    frame.blit(&art(&UPPER), 0, 2 - lift);
    frame.outlined(OUTLINE)
}

pub fn idle() -> Vec<Canvas> {
    vec![pose(&STAND, 0), pose(&STAND, 1)]
}

pub fn walk() -> Vec<Canvas> {
    vec![pose(&STRIDE_A, 0), pose(&PASS, 1), pose(&STRIDE_B, 0), pose(&PASS, 1)]
}
