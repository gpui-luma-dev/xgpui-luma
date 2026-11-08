//! Radix Colors signup-stage accent mesh — Skia raster → `RenderImage`.
//!
//! Source geometry matches the Radix Themes custom-palette SVG (viewBox 2560×1920):
//! gray-2 stage + accent radial fills on diamond paths, displayed at ~0.6 opacity
//! with `width: 240%; margin-left: 70%`.

use std::sync::{Arc, LazyLock};

use gpui::{Hsla, RenderImage};
use image::{Frame, ImageBuffer, Rgba};
use smallvec::smallvec;
use tiny_skia::{
    Color, FillRule, GradientStop, Paint, Path, PathBuilder, Pixmap, Point, RadialGradient, SpreadMode, Transform,
};

use xgpui_luma_look_radix::{Look, ScaleFamily, ScaleStep, SemanticRole};

/// Signup / panel stage behind elevated cards (Radix Colors custom palette).
///
/// HTML equivalent: `background: var(--gray-2)` plus an accent mesh SVG at ~0.6 opacity.
/// Mesh paints use the **color** scale (Radix `--accent-*`) against `--color-background`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignupStage {
    /// Gray fill behind the card (`--gray-2`).
    pub fill_step: ScaleStep,
    /// Elevated card face (`--color-background` / gray 1).
    pub card_step: ScaleStep,
    /// Overall mesh layer opacity.
    pub mesh_opacity: f32,
}

impl Default for SignupStage {
    fn default() -> Self {
        Self { fill_step: 2, card_step: 1, mesh_opacity: 0.6 }
    }
}

/// Soft mesh tints taken from the Radix signup SVG radial stops (`accent-1/2/3/5/7/9`).
#[derive(Clone, Copy, Debug)]
pub struct SignupMeshColors {
    pub background: Hsla,
    pub accent_1: Hsla,
    pub accent_2: Hsla,
    pub accent_3: Hsla,
    pub accent_5: Hsla,
    pub accent_7: Hsla,
    pub accent_9: Hsla,
}

impl SignupStage {
    pub fn fill(self, look: &Look) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.fill_step).hsla()
    }

    pub fn card(self, look: &Look) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.card_step).hsla()
    }

    pub fn mesh_colors(self, look: &Look) -> SignupMeshColors {
        SignupMeshColors {
            background: look.resolve_role(SemanticRole::Background).hsla(),
            accent_1: look.resolve_step(ScaleFamily::Color, 1).hsla(),
            accent_2: look.resolve_step(ScaleFamily::Color, 2).hsla(),
            accent_3: look.resolve_step(ScaleFamily::Color, 3).hsla(),
            accent_5: look.resolve_step(ScaleFamily::Color, 5).hsla(),
            accent_7: look.resolve_step(ScaleFamily::Color, 7).hsla(),
            accent_9: look.resolve_step(ScaleFamily::Color, 9).hsla(),
        }
    }
}

/// Logical SVG viewBox.
pub const MESH_VIEWBOX_W: f32 = 2560.0;
#[cfg(test)]
const MESH_VIEWBOX_H: f32 = 1920.0;

/// Display recipe from the web: oversized + shifted into the stage.
pub const MESH_DISPLAY_WIDTH_FRAC: f32 = 2.4;
pub const MESH_DISPLAY_LEFT_FRAC: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignupMeshCacheKey {
    pub width: u32,
    pub height: u32,
    pub color_fingerprint: u64,
}

impl SignupMeshCacheKey {
    pub fn for_look(look: &Look, width: u32, height: u32) -> Self {
        let colors = SignupStage::default().mesh_colors(look);
        Self { width, height, color_fingerprint: fingerprint_mesh_colors(&colors) }
    }
}

/// Immutable snapshot passed to the background executor; never reads a live Look.
#[derive(Clone, Copy)]
pub struct MeshRequest {
    pub key: SignupMeshCacheKey,
    pub colors: SignupMeshColors,
}
impl MeshRequest {
    pub fn for_look(look: &Look, width: u32, height: u32) -> Self {
        Self {
            key: SignupMeshCacheKey::for_look(look, width, height),
            colors: SignupStage::default().mesh_colors(look),
        }
    }
    pub fn render(self) -> Option<Arc<RenderImage>> {
        rasterize_signup_mesh_stage(&self.colors, self.key.width, self.key.height)
    }
}

/// One in-flight job plus the newest desired snapshot. Keep the previous image
/// visible until its replacement is ready; obsolete results are never published.
#[derive(Default)]
pub struct MeshCache {
    desired: Option<MeshRequest>,
    running: bool,
    failed: Option<SignupMeshCacheKey>,
    rendered: Option<(SignupMeshCacheKey, Arc<RenderImage>)>,
}
impl MeshCache {
    pub fn image(&self) -> Option<Arc<RenderImage>> {
        self.rendered.as_ref().map(|(_, image)| Arc::clone(image))
    }
    pub fn request(&mut self, request: MeshRequest) -> Option<MeshRequest> {
        if request.key.width == 0 || request.key.height == 0 {
            return None;
        }
        self.desired = Some(request);
        if self.running
            || self.failed == Some(request.key)
            || self.rendered.as_ref().is_some_and(|(key, _)| *key == request.key)
        {
            return None;
        }
        self.running = true;
        Some(request)
    }
    /// Returns the latest replacement job, if the completed job became obsolete.
    pub fn complete(&mut self, key: SignupMeshCacheKey, image: Option<Arc<RenderImage>>) -> Option<MeshRequest> {
        let next = self.desired.filter(|desired| desired.key != key);
        if let Some(next) = next {
            if !self.rendered.as_ref().is_some_and(|(cached, _)| *cached == next.key) {
                return Some(next);
            }
        } else if let Some(image) = image {
            self.rendered = Some((key, image));
            self.failed = None;
        }
        if self.desired.is_some_and(|desired| desired.key == key)
            && !self.rendered.as_ref().is_some_and(|(cached, _)| *cached == key)
        {
            self.failed = Some(key);
        }
        self.running = false;
        None
    }
}

/// Rasterize the full SVG viewBox into `width`×`height`.
#[cfg(test)]
fn rasterize_signup_mesh(colors: &SignupMeshColors, width: u32, height: u32) -> Option<Arc<RenderImage>> {
    let world = Transform::from_scale(width as f32 / MESH_VIEWBOX_W, height as f32 / MESH_VIEWBOX_H);
    rasterize_signup_mesh_with_world(colors, width, height, world)
}

/// Rasterize the mesh as seen through the web stage window
/// (`width: 240%; margin-left: 70%`), sized to the stage pixmap.
pub fn rasterize_signup_mesh_stage(colors: &SignupMeshColors, width: u32, height: u32) -> Option<Arc<RenderImage>> {
    if width == 0 || height == 0 {
        return None;
    }
    let stage_aspect = width as f32 / height as f32;
    let svg_visible_w = MESH_VIEWBOX_W / MESH_DISPLAY_WIDTH_FRAC;
    let svg_visible_h = svg_visible_w / stage_aspect;
    let svg_x0 = -MESH_DISPLAY_LEFT_FRAC / MESH_DISPLAY_WIDTH_FRAC * MESH_VIEWBOX_W;
    let svg_y0 = 0.0;
    let world = Transform::from_scale(width as f32 / svg_visible_w, height as f32 / svg_visible_h)
        .pre_concat(Transform::from_translate(-svg_x0, -svg_y0));
    rasterize_signup_mesh_with_world(colors, width, height, world)
}

// Geometry is immutable across palette changes and resizes. Preserve parse failure
// as None rather than panicking during initialization.
static MESH_PATHS: LazyLock<Option<Vec<Path>>> =
    LazyLock::new(|| MESH_LAYERS.iter().map(|layer| parse_svg_path(layer.d)).collect());

fn rasterize_signup_mesh_with_world(
    colors: &SignupMeshColors,
    width: u32,
    height: u32,
    world: Transform,
) -> Option<Arc<RenderImage>> {
    if width == 0 || height == 0 {
        return None;
    }

    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(Color::TRANSPARENT);

    for (layer, path) in MESH_LAYERS.iter().zip(MESH_PATHS.as_ref()?.iter()) {
        let stops = layer
            .stops
            .iter()
            .map(|(offset, slot)| GradientStop::new(*offset, slot_color(colors, *slot)))
            .collect::<Vec<_>>();

        let grad_tx = svg_gradient_transform(layer.tx, layer.ty, layer.rotate_deg, layer.sx, layer.sy);
        let shader_tx = world.pre_concat(grad_tx);
        let shader = RadialGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(0.0, 0.0),
            1.0,
            stops,
            SpreadMode::Pad,
            shader_tx,
        )?;

        let mut paint = Paint::default();
        paint.shader = shader;
        paint.anti_alias = true;
        pixmap.fill_path(path, &paint, FillRule::Winding, world, None);
    }

    pixmap_to_render_image(pixmap)
}

/// Convenience: resolve mesh colors from a look and rasterize for a stage-sized pixmap.
#[cfg(test)]
fn rasterize_signup_mesh_for_look(look: &Look, width: u32, height: u32) -> Option<Arc<RenderImage>> {
    let colors = SignupStage::default().mesh_colors(look);
    rasterize_signup_mesh_stage(&colors, width, height)
}

#[derive(Clone, Copy)]
enum ColorSlot {
    Background,
    Accent1,
    Accent2,
    Accent3,
    Accent5,
    Accent7,
    Accent9,
}

fn slot_color(colors: &SignupMeshColors, slot: ColorSlot) -> Color {
    let hsla = match slot {
        ColorSlot::Background => colors.background,
        ColorSlot::Accent1 => colors.accent_1,
        ColorSlot::Accent2 => colors.accent_2,
        ColorSlot::Accent3 => colors.accent_3,
        ColorSlot::Accent5 => colors.accent_5,
        ColorSlot::Accent7 => colors.accent_7,
        ColorSlot::Accent9 => colors.accent_9,
    };
    hsla_to_skia(hsla)
}

fn hsla_to_skia(color: Hsla) -> Color {
    let rgb = color.to_rgb();
    let a = color.a.clamp(0.0, 1.0);
    Color::from_rgba(rgb.r.clamp(0.0, 1.0), rgb.g.clamp(0.0, 1.0), rgb.b.clamp(0.0, 1.0), a)
        .unwrap_or(Color::TRANSPARENT)
}

fn svg_gradient_transform(tx: f32, ty: f32, rotate_deg: f32, sx: f32, sy: f32) -> Transform {
    Transform::from_translate(tx, ty)
        .pre_concat(Transform::from_rotate(rotate_deg))
        .pre_concat(Transform::from_scale(sx, sy))
}

fn fingerprint_mesh_colors(colors: &SignupMeshColors) -> u64 {
    let mut hash = 1469598103934665603u64;
    for c in [
        colors.background,
        colors.accent_1,
        colors.accent_2,
        colors.accent_3,
        colors.accent_5,
        colors.accent_7,
        colors.accent_9,
    ] {
        let rgb = c.to_rgb();
        for v in [rgb.r, rgb.g, rgb.b, c.a] {
            let bits = (v.clamp(0.0, 1.0) * 10_000.0).round() as u32 as u64;
            hash ^= bits;
            hash = hash.wrapping_mul(1099511628211u64);
        }
    }
    hash
}

fn pixmap_to_render_image(pixmap: Pixmap) -> Option<Arc<RenderImage>> {
    let width = pixmap.width();
    let height = pixmap.height();
    // tiny-skia pixmap is premultiplied RGBA; GPUI `RenderImage` expects BGRA.
    let mut raw_bytes = pixmap.data().to_vec();
    for pixel in raw_bytes.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, raw_bytes)?;
    let frame = Frame::new(image_buffer);
    Some(Arc::new(RenderImage::new(smallvec![frame])))
}

struct MeshLayer {
    d: &'static str,
    tx: f32,
    ty: f32,
    rotate_deg: f32,
    sx: f32,
    sy: f32,
    stops: &'static [(f32, ColorSlot)],
}

const MESH_LAYERS: &[MeshLayer] = &[
    MeshLayer {
        d: PATH_0,
        tx: -804.109,
        ty: -2036.8,
        rotate_deg: 64.9401,
        sx: 6436.87,
        sy: 6304.81,
        stops: &[
            (0.0, ColorSlot::Background),
            (0.0833333, ColorSlot::Accent7),
            (0.364583, ColorSlot::Accent5),
            (0.658041, ColorSlot::Background),
            (0.798521, ColorSlot::Accent9),
            (0.942708, ColorSlot::Background),
            (1.0, ColorSlot::Background),
        ],
    },
    MeshLayer {
        d: PATH_1,
        tx: 201.6,
        ty: -1080.02,
        rotate_deg: 64.9401,
        sx: 6436.87,
        sy: 6304.81,
        stops: &[
            (0.0, ColorSlot::Background),
            (0.0833333, ColorSlot::Accent2),
            (0.333803, ColorSlot::Accent1),
            (0.658041, ColorSlot::Background),
            (0.798521, ColorSlot::Accent9),
            (0.942708, ColorSlot::Background),
            (1.0, ColorSlot::Background),
        ],
    },
    MeshLayer {
        d: PATH_2,
        tx: 912.834,
        ty: -811.021,
        rotate_deg: 64.9401,
        sx: 6436.87,
        sy: 6304.81,
        stops: &[
            (0.0, ColorSlot::Background),
            (0.140625, ColorSlot::Accent3),
            (0.333803, ColorSlot::Accent7),
            (0.658041, ColorSlot::Background),
            (0.798521, ColorSlot::Accent9),
            (0.942708, ColorSlot::Background),
            (1.0, ColorSlot::Background),
        ],
    },
    MeshLayer {
        d: PATH_3,
        tx: 1711.41,
        ty: -1639.11,
        rotate_deg: 64.9401,
        sx: 6436.87,
        sy: 6304.81,
        stops: &[
            (0.0, ColorSlot::Background),
            (0.0833333, ColorSlot::Accent7),
            (0.333803, ColorSlot::Accent1),
            (0.658041, ColorSlot::Background),
            (0.798521, ColorSlot::Accent9),
            (0.942708, ColorSlot::Background),
            (1.0, ColorSlot::Background),
        ],
    },
    MeshLayer {
        d: PATH_4,
        tx: 3479.06,
        ty: -623.459,
        rotate_deg: 113.028,
        sx: 8332.26,
        sy: 4870.62,
        stops: &[
            (0.0, ColorSlot::Background),
            (0.0833333, ColorSlot::Accent7),
            (0.333803, ColorSlot::Accent1),
            (0.658041, ColorSlot::Background),
            (0.798521, ColorSlot::Accent9),
            (0.942708, ColorSlot::Background),
            (1.0, ColorSlot::Background),
        ],
    },
];

const PATH_0: &str = "M-119.809 -1055.99L859.027 -684.98C915.435 -663.6 955.626 -624.994 968.519 -579.807L1129.49 -15.6245L1860.47 -241.727C1919.02 -259.836 1985.68 -257.939 2042.09 -236.559L3020.93 134.453C3124.79 173.822 3164.97 266.777 3110.66 342.073L2850.06 703.385C2827.36 734.857 2790.34 759.666 2745.28 773.604L1467.45 1168.86L1748.58 2154.16C1758.67 2189.52 1751.28 2226.32 1727.72 2258.12L1361.75 2752.01L203.258 2312.91C146.85 2291.53 106.659 2252.92 93.7664 2207.73L-67.2076 1643.55L-798.184 1869.65C-856.73 1887.76 -923.398 1885.87 -979.806 1864.48L-2138.3 1425.38L-1787.63 925.687C-1765.05 893.507 -1727.57 868.111 -1681.77 853.942L-405.167 459.07L-686.568 -527.183C-696.491 -561.961 -689.511 -598.157 -666.811 -629.629L-406.21 -990.941C-351.902 -1066.24 -223.676 -1095.36 -119.809 -1055.99Z";
const PATH_1: &str = "M885.9 -99.2158L1864.74 271.796C1921.14 293.177 1961.34 331.783 1974.23 376.97L2135.2 941.152L2866.18 715.049C2924.72 696.94 2991.39 698.837 3047.8 720.218L4026.64 1091.23C4130.5 1130.6 4170.68 1223.55 4116.37 1298.85L3855.77 1660.16C3833.07 1691.63 3796.05 1716.44 3750.99 1730.38L2473.16 2125.63L2754.29 3110.94C2764.38 3146.29 2756.99 3183.09 2733.43 3214.9L2367.46 3708.79L1208.97 3269.68C1152.56 3248.3 1112.37 3209.7 1099.48 3164.51C816.824 2173.87 747.087 1929.46 319.141 429.593C309.218 394.815 316.198 358.619 338.898 327.147L599.499 -34.1647C653.807 -109.461 782.033 -138.585 885.9 -99.2158Z";
const PATH_2: &str = "M1597.13 169.784L2575.97 540.796C2632.38 562.177 2672.57 600.783 2685.46 645.97L2846.44 1210.15L3577.41 984.049C3635.96 965.94 3702.63 967.837 3759.03 989.218L4737.87 1360.23C4841.74 1399.6 4881.91 1492.55 4827.61 1567.85L4567 1929.16C4544.3 1960.63 4507.28 1985.44 4462.22 1999.38L3184.4 2394.63L3465.53 3379.94C3475.61 3415.29 3468.23 3452.09 3444.66 3483.9L3078.69 3977.79L1920.2 3538.68C1863.79 3517.3 1823.6 3478.7 1810.71 3433.51L1649.74 2869.33L918.759 3095.43C860.213 3113.54 793.545 3111.64 737.138 3090.26L-421.356 2651.15L-70.6875 2151.46C-48.1049 2119.28 -10.63 2093.89 35.1782 2079.72L1311.78 1684.85L1030.38 698.593C1020.45 663.815 1027.43 627.619 1050.13 596.147L1310.73 234.835C1365.04 159.539 1493.27 130.415 1597.13 169.784Z";
const PATH_3: &str = "M2395.71 -658.308L3374.55 -287.296C3430.96 -265.915 3471.15 -227.309 3484.04 -182.122L3645.01 382.06L4375.99 155.958C4434.54 137.848 4501.2 139.745 4557.61 161.126L5536.45 532.138C5640.32 571.507 5680.49 664.461 5626.18 739.757L5365.58 1101.07C5342.88 1132.54 5305.86 1157.35 5260.8 1171.29L3982.97 1566.54L4264.1 2551.84C4274.19 2587.2 4266.81 2624 4243.24 2655.81L3877.27 3149.7L2718.78 2710.59C2662.37 2689.21 2622.18 2650.6 2609.29 2605.42L2448.31 2041.24L1717.34 2267.34C1658.79 2285.45 1592.12 2283.55 1535.72 2262.17L377.222 1823.06L727.891 1323.37C750.473 1291.19 787.948 1265.8 833.756 1251.63L2110.35 856.754L1828.95 -129.498C1819.03 -164.277 1826.01 -200.472 1848.71 -231.944L2109.31 -593.257C2163.62 -668.552 2291.85 -697.677 2395.71 -658.308Z";
const PATH_4: &str = "M3059.26 767.932L3310.25 1618.16C3324.72 1667.15 3315.74 1727.88 3285.79 1783.6L2911.89 2479.3L3514.51 2558.36C3562.77 2564.69 3599.15 2596.78 3613.62 2645.77L3864.61 3496C3891.25 3586.22 3837.41 3706.98 3744.37 3765.74L3297.91 4047.66C3259.03 4072.22 3217.48 4082.97 3180.34 4078.1L2126.89 3939.89L1473.9 5154.88C1450.47 5198.48 1415.9 5235.81 1376.24 5260.35L760.412 5641.34L463.348 4635.06C448.884 4586.06 457.863 4525.33 487.81 4469.61L861.713 3773.92L259.094 3694.86C210.828 3688.53 174.448 3656.44 159.984 3607.44L-137.08 2601.17L474.823 2206.89C514.228 2181.5 556.514 2170.3 594.278 2175.25L1646.71 2313.32L2300.33 1097.17C2323.38 1054.28 2357.22 1017.43 2396.11 992.876L2842.57 710.953C2935.61 652.202 3032.62 677.712 3059.26 767.932Z";

/// Minimal SVG path subset used by the Radix mesh (`M`/`L`/`C`/`Z`).
fn parse_svg_path(d: &str) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut chars = d.chars().peekable();
    let mut cmd = 'M';
    let mut first_in_subpath = true;
    let mut last = (0.0_f32, 0.0_f32);
    let mut start = (0.0_f32, 0.0_f32);

    while chars.peek().is_some() {
        while matches!(chars.peek(), Some(c) if c.is_whitespace() || *c == ',') {
            chars.next();
        }
        let Some(&c) = chars.peek() else {
            break;
        };
        if c.is_ascii_alphabetic() {
            cmd = chars.next()?;
            if cmd == 'Z' || cmd == 'z' {
                builder.close();
                last = start;
                first_in_subpath = true;
                continue;
            }
        }

        match cmd {
            'M' | 'm' => {
                let (x, y) = read_pair(&mut chars)?;
                let (x, y) = if cmd == 'm' { (last.0 + x, last.1 + y) } else { (x, y) };
                if first_in_subpath {
                    builder.move_to(x, y);
                    start = (x, y);
                    first_in_subpath = false;
                } else {
                    builder.line_to(x, y);
                }
                last = (x, y);
                cmd = if cmd == 'M' { 'L' } else { 'l' };
            }
            'L' | 'l' => {
                let (x, y) = read_pair(&mut chars)?;
                let (x, y) = if cmd == 'l' { (last.0 + x, last.1 + y) } else { (x, y) };
                builder.line_to(x, y);
                last = (x, y);
            }
            'C' | 'c' => {
                let (x1, y1) = read_pair(&mut chars)?;
                let (x2, y2) = read_pair(&mut chars)?;
                let (x, y) = read_pair(&mut chars)?;
                let (x1, y1, x2, y2, x, y) = if cmd == 'c' {
                    (last.0 + x1, last.1 + y1, last.0 + x2, last.1 + y2, last.0 + x, last.1 + y)
                } else {
                    (x1, y1, x2, y2, x, y)
                };
                builder.cubic_to(x1, y1, x2, y2, x, y);
                last = (x, y);
            }
            'Z' | 'z' => {
                builder.close();
                last = start;
                first_in_subpath = true;
            }
            _ => return None,
        }
    }

    builder.finish()
}

fn read_pair(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(f32, f32)> {
    let x = read_number(chars)?;
    let y = read_number(chars)?;
    Some((x, y))
}

fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<f32> {
    while matches!(chars.peek(), Some(c) if c.is_whitespace() || *c == ',') {
        chars.next();
    }
    let mut buf = String::new();
    if matches!(chars.peek(), Some('+' | '-')) {
        buf.push(chars.next()?);
    }
    let mut seen_dot = false;
    let mut seen_exp = false;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            buf.push(chars.next()?);
        } else if c == '.' && !seen_dot && !seen_exp {
            seen_dot = true;
            buf.push(chars.next()?);
        } else if (c == 'e' || c == 'E') && !seen_exp {
            seen_exp = true;
            buf.push(chars.next()?);
            if matches!(chars.peek(), Some('+' | '-')) {
                buf.push(chars.next()?);
            }
        } else {
            break;
        }
    }
    if buf.is_empty() || buf == "+" || buf == "-" {
        return None;
    }
    buf.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_jobs_coalesce_and_never_publish_obsolete_results() {
        let look = Look::built_in();
        let first = MeshRequest::for_look(&look, 32, 24);
        let intermediate = MeshRequest::for_look(&look, 33, 24);
        let latest = MeshRequest::for_look(&look, 34, 24);
        let mut cache = MeshCache::default();
        assert!(cache.request(first).is_some());
        assert!(cache.request(intermediate).is_none());
        assert!(cache.request(latest).is_none());
        let next = cache.complete(first.key, first.render()).expect("latest job");
        assert_eq!(next.key, latest.key);
        assert!(cache.image().is_none(), "obsolete result was discarded");
        assert!(cache.complete(next.key, next.render()).is_none());
        let image = cache.image().expect("latest image");
        assert!(cache.request(latest).is_none(), "unchanged inputs reuse the cache");
        assert!(cache.request(first).is_some());
        assert!(Arc::ptr_eq(&image, &cache.image().expect("retained image")));
        assert!(cache.request(latest).is_none());
        assert!(cache.complete(first.key, first.render()).is_none());
        assert!(Arc::ptr_eq(&image, &cache.image().expect("return to cached image")));
    }

    #[test]
    fn mesh_cache_ignores_unrelated_look_changes_and_stops_failed_retries() {
        let look = Look::built_in();
        let first = MeshRequest::for_look(&look, 32, 24);
        look.set_classic_params(Default::default());
        assert_eq!(first.key, MeshRequest::for_look(&look, 32, 24).key);
        look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
        assert_ne!(first.key, MeshRequest::for_look(&look, 32, 24).key);
        let mut cache = MeshCache::default();
        assert!(cache.request(first).is_some());
        assert!(cache.complete(first.key, None).is_none());
        assert!(cache.request(first).is_none(), "do not retry a failed image on every frame");
        assert!(cache.request(MeshRequest::for_look(&look, 32, 24)).is_some());
    }

    /// Headless timing probe: run explicitly with --ignored --nocapture.
    #[test]
    #[ignore]
    fn profile_mesh_rasterization() {
        let colors = SignupStage::default().mesh_colors(&Look::built_in());
        for (width, height) in [(435, 500), (835, 960)] {
            let start = std::time::Instant::now();
            for _ in 0..30 {
                std::hint::black_box(rasterize_signup_mesh_stage(&colors, width, height).expect("mesh"));
            }
            eprintln!("mesh {width}x{height}: {:?} per frame", start.elapsed() / 30);
        }
    }

    #[test]
    fn signup_stage_uses_gray_fill_and_color_mesh() {
        let look = Look::built_in();
        let stage = SignupStage::default();
        assert_eq!(stage.fill(&look), look.resolve_step(ScaleFamily::Gray, 2).hsla());
        assert_eq!(stage.card(&look), look.resolve_step(ScaleFamily::Gray, 1).hsla());
        let mesh = stage.mesh_colors(&look);
        assert_eq!(mesh.accent_3, look.resolve_step(ScaleFamily::Color, 3).hsla());
        assert_ne!(mesh.accent_9.s, stage.fill(&look).s);
    }

    #[test]
    fn parses_all_mesh_paths() {
        for layer in MESH_LAYERS {
            assert!(parse_svg_path(layer.d).is_some(), "failed: {}", &layer.d[..32]);
        }
    }

    #[test]
    fn rasterize_produces_image() {
        let look = Look::built_in();
        assert!(rasterize_signup_mesh_for_look(&look, 320, 240).is_some());
        assert!(rasterize_signup_mesh(&SignupStage::default().mesh_colors(&look), 320, 240).is_some());
    }

    #[test]
    fn mesh_pixels_follow_color_scale_not_swapped_channels() {
        let look = Look::built_in();
        let colors = SignupStage::default().mesh_colors(&look);
        let accent = colors.accent_9.to_rgb();
        // Built-in indigo: blue channel dominates red.
        assert!(accent.b > accent.r);

        let mut pixmap = Pixmap::new(64, 48).expect("pixmap");
        pixmap.fill(Color::TRANSPARENT);
        let world = Transform::from_scale(64.0 / MESH_VIEWBOX_W, 48.0 / MESH_VIEWBOX_H);
        let path = parse_svg_path(PATH_0).expect("path");
        let stops = MESH_LAYERS[0]
            .stops
            .iter()
            .map(|(offset, slot)| GradientStop::new(*offset, slot_color(&colors, *slot)))
            .collect::<Vec<_>>();
        let grad_tx = svg_gradient_transform(
            MESH_LAYERS[0].tx,
            MESH_LAYERS[0].ty,
            MESH_LAYERS[0].rotate_deg,
            MESH_LAYERS[0].sx,
            MESH_LAYERS[0].sy,
        );
        let shader = RadialGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(0.0, 0.0),
            1.0,
            stops,
            SpreadMode::Pad,
            world.pre_concat(grad_tx),
        )
        .expect("shader");
        let mut paint = Paint::default();
        paint.shader = shader;
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, world, None);

        // Sample opaque-ish pixels before BGRA swap: RGBA order from tiny-skia.
        let data = pixmap.data();
        let mut best = (0u8, 0u8, 0u8);
        for pixel in data.chunks_exact(4) {
            let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
            if a > 200 && b > best.2 {
                best = (r, g, b);
            }
        }
        assert!(best.2 > best.0, "expected bluish mesh pixel, got {best:?}");
    }
}
