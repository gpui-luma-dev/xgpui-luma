//! Fixed shader subjects shared across themes. Chrome is resolved from the look.
/// Default encoded gradient stop colors, preserving the paint-lab fixture.
pub(super) const GRADIENT_STOPS: [(f32, [f32; 4]); 5] = [
    (0., [1., 0.12, 0.18, 1.]),
    (0.25, [1., 0.7, 0.12, 1.]),
    (0.5, [0.25, 0.25, 1., 1.]),
    (0.75, [0.05, 0.85, 0.75, 1.]),
    (1., [1., 0.12, 0.18, 1.]),
];
pub(super) const WASH_TINT: [f32; 3] = [0.95, 0.15, 0.35];
pub(super) const MINIMAL_TINT: [f32; 3] = [0.25, 0.65, 1.];
pub(super) const COMPOSITION_BACKGROUND: u32 = 0x1c2b46;
pub(super) const COMPOSITION_DIVIDER: u32 = 0xffdf80;
pub(super) const COMPOSITION_CLIP: u32 = 0x67e8f9;
pub(super) const COMPOSITION_SHAPES: [[f32; 4]; 3] =
    [[1., 0.2, 0.3, 0.65], [0.15, 0.9, 0.55, 0.65], [0.3, 0.5, 1., 0.65]];
pub(super) fn instance_tint(index: f32, particles: bool) -> [f32; 4] {
    if particles {
        [0.4 + 0.4 * (index * 0.7).sin(), 0.65, 1., 0.8]
    } else {
        [0.3 + 0.2 * (index * 0.3).sin(), 0.55, 1., 1.]
    }
}

// Fixed light reference surfaces and shadow signal, retained from shader-examples.
pub(super) const SHADOW_CARD: u32 = 0xf8fafd;
pub(super) const SHADOW_STAGE: u32 = 0xe6ebf3;
pub(super) const BORDER_STAGE: u32 = 0xf8f9fb;
pub(super) const BORDER_SURFACE: u32 = 0xffffff;
pub(super) const SHADOW_NATIVE_GRADIENT: u32 = 0x7160d9b3;
pub(super) const SHADOW_NATIVE_SOLID: u32 = 0x00000099;
pub(super) const SHADOW_STOPS: [(f32, [f32; 4]); 4] = [
    (0., [0.98, 0.16, 0.45, 0.85]),
    (0.35, [0.55, 0.25, 1., 0.8]),
    (0.7, [0.05, 0.6, 1., 0.8]),
    (1., [0.1, 0.85, 0.65, 0.75]),
];
pub(super) const SHADOW_SOLID: [f32; 4] = [0., 0., 0., 0.6];

pub(super) mod gallery;
