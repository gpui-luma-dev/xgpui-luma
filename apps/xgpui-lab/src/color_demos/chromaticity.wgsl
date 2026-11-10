// CIE locus geometry is generated from the attributed CSV before compilation.
// Color conversion and fixed-L/h chroma reduction come from xgpui-color.
struct Parameters {
    simulated: f32,
    luminance: f32,
    reduce_chroma: f32,
    show_fill: f32,
    source_x: f32,
    source_y: f32,
    source_visible: f32,
    overlays: f32,
    hatch: f32,
    uv: f32,
    grid_r: f32,
    grid_g: f32,
    grid_b: f32,
    locus_r: f32,
    locus_g: f32,
    locus_b: f32,
    reference_r: f32,
    reference_g: f32,
    reference_b: f32,
    reference_halo_r: f32,
    reference_halo_g: f32,
    reference_halo_b: f32,
    subject_r: f32,
    subject_g: f32,
    subject_b: f32,
    subject_halo_r: f32,
    subject_halo_g: f32,
    subject_halo_b: f32,
}
fn project(xy: vec2<f32>, uv: f32) -> vec2<f32> {
    if uv < 0.5 { return xy; }
    let d = max(-2.0 * xy.x + 12.0 * xy.y + 3.0, 0.000001);
    return vec2(4.0 * xy.x, 9.0 * xy.y) / d;
}
fn unproject(uv: vec2<f32>, enabled: f32) -> vec2<f32> {
    if enabled < 0.5 { return uv; }
    let d = 6.0 * uv.x - 16.0 * uv.y + 12.0;
    return vec2(9.0 * uv.x, 4.0 * uv.y) / max(d, 0.000001);
}
fn segment_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let edge = b - a;
    return length(p - a - edge * clamp(dot(p - a, edge) / max(dot(edge, edge), 1e-20), 0.0, 1.0));
}
fn locus_distance(p: vec2<f32>, uv: f32) -> f32 {
    var nearest = 1e10;
    var inside = false;
    var previous = project(LOCUS[LOCUS_COUNT - 1u], uv);
    for (var i = 0u; i < LOCUS_COUNT; i++) {
        let current = project(LOCUS[i], uv);
        nearest = min(nearest, segment_distance(p, previous, current));
        if (current.y > p.y) != (previous.y > p.y) {
            let crossing = (previous.x - current.x) * (p.y - current.y) / (previous.y - current.y) + current.x;
            if p.x < crossing { inside = !inside; }
        }
        previous = current;
    }
    return select(nearest, -nearest, inside);
}
fn triangle_distance(p: vec2<f32>, index: u32, uv: f32) -> f32 {
    let a = project(PRIMARIES[index * 3u], uv);
    let b = project(PRIMARIES[index * 3u + 1u], uv);
    let c = project(PRIMARIES[index * 3u + 2u], uv);
    return min(segment_distance(p, a, b), min(segment_distance(p, b, c), segment_distance(p, c, a)));
}
fn stroke_coverage(distance: f32, width: f32, aa: f32) -> f32 {
    return 1.0 - smoothstep(0.5 * width - 0.5 * aa, 0.5 * width + 0.5 * aa, distance);
}
fn cross_distance(pixel: vec2<f32>, center: vec2<f32>) -> f32 {
    return min(segment_distance(pixel, center - vec2(5.0, 0.0), center + vec2(5.0, 0.0)),
        segment_distance(pixel, center - vec2(0.0, 5.0), center + vec2(0.0, 5.0)));
}
// The display order puts Adobe RGB before Rec.2020; xgpui-color's IDs do not.
fn color_space(index: f32) -> u32 {
    if index == 2.0 { return 3u; }
    if index == 3.0 { return 2u; }
    return u32(index);
}
fn chart_rgb(xy: vec2<f32>, p: Parameters) -> vec3<f32> {
    let y = max(xy.y, 0.00001);
    let xyz = vec3(xy.x * p.luminance / y, p.luminance, (1.0 - xy.x - xy.y) * p.luminance / y);
    let space = color_space(p.simulated);
    let linear = gamut_from_xyz(xyz, space);
    let mapped = gamut_map(linear, space, u32(p.reduce_chroma));
    let srgb = clamp(gamut_convert(mapped, space, 0u), vec3(0.0), vec3(1.0));
    return gamut_encode(srgb, 0u);
}
// Straight-alpha source-over keeps antialiased edges correct on any enclosure.
fn over(base: vec4<f32>, rgb: vec3<f32>, coverage: f32) -> vec4<f32> {
    let alpha = coverage + base.a * (1.0 - coverage);
    let premultiplied = rgb * coverage + base.rgb * base.a * (1.0 - coverage);
    return vec4(premultiplied / max(alpha, 0.000001), alpha);
}
// A 2 px cross over a 4 px halo, so a marker reads on any fill or background.
fn marker(base: vec4<f32>, distance: f32, rgb: vec3<f32>, halo: vec3<f32>, aa: f32) -> vec4<f32> {
    return over(over(base, halo, stroke_coverage(distance, 4.0, aa)), rgb, stroke_coverage(distance, 2.0, aa));
}
fn shade(input: ShaderInput, p: Parameters) -> vec4<f32> {
    let extent = select(vec2(0.8, 0.9), vec2(0.65), p.uv > 0.5);
    let scale = max(1.0, min((input.size.x - 48.0) / extent.x, (input.size.y - 48.0) / extent.y));
    let left = (input.size.x - scale * extent.x) * 0.5;
    let bottom = input.size.y - 24.0;
    let coordinate = vec2((input.position.x - left) / scale, (bottom - input.position.y) / scale);
    let xy = unproject(coordinate, p.uv);
    // Distances are in logical pixels. Coverage spans one physical pixel at
    // every backing scale; no low-resolution texture or cell tessellation.
    let aa = 1.0 / max(input.scale_factor, 1.0);
    let distance = locus_distance(coordinate, p.uv) * scale;
    let coverage = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, distance);
    var color = vec4(0.0);
    if p.show_fill > 0.5 && coverage > 0.0 {
        let fill = chart_rgb(xy, p);
        color = vec4(fill, coverage);
        if p.overlays > 0.5 && p.hatch > 0.5 {
            let y = max(xy.y, 0.00001);
            let xyz = vec3(xy.x * p.luminance / y, p.luminance, (1.0 - xy.x - xy.y) * p.luminance / y);
            let linear = gamut_from_xyz(xyz, color_space(p.simulated));
            if !gamut_in_bounds(linear) {
                let hatch_distance = abs(fract((input.position.x + input.position.y) / 8.0) - 0.5) * 8.0;
                let hatch = stroke_coverage(hatch_distance, 0.85, aa);
                color = vec4(mix(color.rgb, vec3(16.0, 24.0, 35.0) / 255.0, hatch * 0.55), color.a);
            }
        }
    }
    if p.overlays > 0.5 {
        let plot_distance = max(abs(coordinate.x - extent.x * 0.5) - extent.x * 0.5, abs(coordinate.y - extent.y * 0.5) - extent.y * 0.5) * scale;
        let plot_coverage = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, plot_distance);
        let grid = abs(coordinate / vec2(0.1) - round(coordinate / vec2(0.1))) * vec2(0.1) * scale;
        color = over(color, vec3(p.grid_r, p.grid_g, p.grid_b),
            stroke_coverage(min(grid.x, grid.y), 1.0, aa) * plot_coverage);
        color = over(color, vec3(p.locus_r, p.locus_g, p.locus_b), stroke_coverage(abs(distance), 1.5, aa));
        for (var i = 0u; i < 4u; i++) {
            color = over(color, OUTLINE_COLORS[i], stroke_coverage(triangle_distance(coordinate, i, p.uv) * scale, 2.0, aa));
        }
        let white_coordinate = project(vec2(0.3127, 0.3290), p.uv);
        let white = vec2(left + white_coordinate.x * scale, bottom - white_coordinate.y * scale);
        color = marker(color, cross_distance(input.position, white), vec3(p.reference_r, p.reference_g, p.reference_b),
            vec3(p.reference_halo_r, p.reference_halo_g, p.reference_halo_b), aa);
        if p.source_visible > 0.5 {
            let source_coordinate = project(vec2(p.source_x, p.source_y), p.uv);
            let selected = vec2(left + source_coordinate.x * scale, bottom - source_coordinate.y * scale);
            color = marker(color, cross_distance(input.position, selected), vec3(p.subject_r, p.subject_g, p.subject_b),
                vec3(p.subject_halo_r, p.subject_halo_g, p.subject_halo_b), aa);
        }
    }
    // Preserve rounded enclosure clipping while leaving empty regions transparent.
    let corner = abs(input.position - input.size * 0.5) - (input.size * 0.5 - vec2(6.0));
    let corner_distance = length(max(corner, vec2(0.0))) + min(max(corner.x, corner.y), 0.0) - 6.0;
    return vec4(color.rgb, color.a * (1.0 - smoothstep(-0.5 * aa, 0.5 * aa, corner_distance)));
}
