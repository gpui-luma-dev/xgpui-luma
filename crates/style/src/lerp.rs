//! Interpolation between two resolved styles, for state animations.

use gpui::{BoxShadow, FontWeight, Hsla, SharedString};

/// Blend `a` toward `b` by `t` in 0..=1. Types that cannot blend switch at the
/// midpoint.
pub trait Lerp: Sized {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

fn step<T: Clone>(a: &T, b: &T, t: f32) -> T {
    if t < 0.5 { a.clone() } else { b.clone() }
}

impl Lerp for f32 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Hsla {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        gpui::hsla(
            f32::lerp(&a.h, &b.h, t),
            f32::lerp(&a.s, &b.s, t),
            f32::lerp(&a.l, &b.l, t),
            f32::lerp(&a.a, &b.a, t),
        )
    }
}

/// `Some` to `Some` blends; a change to or from `None` switches at the midpoint.
impl<T: Lerp + Clone> Lerp for Option<T> {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        match (a, b) {
            (Some(a), Some(b)) => Some(T::lerp(a, b, t)),
            _ => step(a, b, t),
        }
    }
}

impl Lerp for Vec<BoxShadow> {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        step(a, b, t)
    }
}

impl Lerp for SharedString {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        step(a, b, t)
    }
}

impl Lerp for FontWeight {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        step(a, b, t)
    }
}
