//! Fixed brush and captured-layer subjects from the upstream brush gallery.
pub(in crate::shader) const BRUSH_STOPS: [(f32, [f32; 4]); 3] =
    [(0., [0.1, 0.3, 1., 1.]), (0.5, [0.7, 0.2, 0.9, 0.65]), (1., [1., 0.5, 0.1, 1.])];
pub(in crate::shader) const SOLID_BLUE: [f32; 4] = [0.15, 0.3, 0.75, 1.];
pub(in crate::shader) const RING_STOPS: [(f32, [f32; 4]); 2] = [(0., [1., 1., 1., 0.15]), (1., [1., 1., 1., 0.9])];
pub(in crate::shader) const MATERIAL_STOPS: [(f32, [f32; 4]); 2] =
    [(0., [0.12, 0.2, 0.4, 1.]), (1., [0.55, 0.25, 0.65, 1.])];
pub(in crate::shader) const TYPE_STOPS: [(f32, [f32; 4]); 2] = [(0., [0.05, 0.3, 1., 1.]), (1., [1., 0.18, 0.02, 1.])];
pub(in crate::shader) const FREEFORM_POINTS: [([f32; 2], [f32; 4]); 3] = [
    ([0.15, 0.2], [1., 0.05, 0.08, 1.]),
    ([0.85, 0.2], [0.05, 0.3, 1., 1.]),
    ([0.5, 0.85], [0.05, 0.9, 0.25, 1.]),
];
pub(in crate::shader) const MESH_COLORS: [[f32; 4]; 4] =
    [[0.05, 0.3, 1., 1.], [1., 0.18, 0.02, 1.], [0.9, 0.05, 0.7, 1.], [0., 0.8, 0.7, 1.]];
pub(in crate::shader) const BACKDROP_STRIPES: [[u32; 2]; 2] = [[0x165dff, 0xff6408], [0xff0020, 0xffff00]];
pub(in crate::shader) const FROST_FILL: u32 = 0xffffff24;
pub(in crate::shader) const FROST_BORDER: u32 = 0xffffff90;
pub(in crate::shader) const OPACITY_SHAPES: [u32; 2] = [0xff3808, 0x164dff];
pub(in crate::shader) const FILTER_BACKGROUND: u32 = 0x213c54;
pub(in crate::shader) const FILTER_SHAPES: [u32; 2] = [0xe86a47, 0x4f87c5];
pub(in crate::shader) const FILTER_GLYPHS: u32 = 0xffdf28;
pub(in crate::shader) const BLEND_BACKDROP: [u32; 2] = [0xf4cc68, 0x349dba];
pub(in crate::shader) const PATH_STOPS: [(f32, [f32; 4]); 2] = [(0., [0.05, 0.3, 1., 1.]), (1., [1., 0.22, 0.02, 1.])];
pub(in crate::shader) const PATH_MESH: [[f32; 4]; 4] =
    [[1., 0.1, 0., 1.], [0., 0.8, 1., 1.], [0.4, 0., 1., 1.], [1., 0.7, 0., 1.]];
pub(in crate::shader) const NOISE_RAMP: [(f32, [f32; 4]); 2] =
    [(0., [0.25, 0.25, 0.25, 1.]), (1., [0.75, 0.75, 0.75, 1.])];
pub(in crate::shader) const NOISE_BASE: [f32; 4] = [0.5, 0.5, 0.5, 1.];
pub(in crate::shader) const EXPANDED_COLORS: [[f32; 4]; 8] = [
    [1., 0.03, 0.08, 1.],
    [1., 0.5, 0., 1.],
    [1., 0.95, 0., 1.],
    [0.05, 1., 0.2, 1.],
    [0., 0.9, 1., 1.],
    [0.05, 0.2, 1., 1.],
    [0.65, 0.05, 1., 1.],
    [1., 0., 0.65, 1.],
];
pub(in crate::shader) const BATCH_STOPS: [(f32, [f32; 4]); 2] = [(0., [0., 0.8, 1., 0.8]), (1., [1., 0.1, 0.6, 0.8])];
