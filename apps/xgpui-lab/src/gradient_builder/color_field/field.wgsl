// Freeform field: same encoded-channel interpolation as field.metal and ColorField::sample.
struct Point { geometry: vec4<f32>, color: vec4<f32>, hsl: vec4<f32>, shape: vec4<f32> };
struct Parameters {
    background: vec4<f32>, background_hsl: vec4<f32>, options: vec4<f32>, shape: vec4<f32>,
    points: array<Point, 16>,
};
@group(0) @binding(0) var<uniform> p: Parameters;

@vertex fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}
fn hsl_rgb(hsl: vec3<f32>) -> vec3<f32> {
    let k = fract(hsl.x + vec3<f32>(0.0, 2.0/3.0, 1.0/3.0));
    let hue = clamp(abs(k * 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    let chroma = (1.0 - abs(2.0 * hsl.z - 1.0)) * hsl.y;
    return (hue - 0.5) * chroma + hsl.z;
}
@fragment fn fragment(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pixel.xy / p.shape.zw;
    let normalized = p.options.y > 0.5;
    let hsl = p.options.z > 0.5;
    var rgb = p.background.rgb;
    var hs = p.background_hsl.xyz;
    var alpha = p.background.a;
    var total = p.options.w;
    var associated_weight = total * alpha;
    var sum = select(rgb, hs, hsl) * associated_weight;
    for (var i = 0u; i < u32(p.options.x); i++) {
        let f = p.points[i];
        let d = uv - f.geometry.xy;
        let x = f.geometry.z * d.x + f.geometry.w * d.y;
        let y = -f.geometry.w * d.x + f.geometry.z * d.y;
        let distance = select(x*x/f.shape.y + y*y*f.shape.y, dot(d,d), f.shape.y == 1.0);
        let weight = exp(-pow(max(0.0, distance*5.0/(f.shape.x*f.shape.x)), p.shape.x));
        let a = weight * f.color.a;
        if (!normalized) {
            if (hsl) { hs += (f.hsl.xyz - hs)*a; rgb = hsl_rgb(hs); }
            else { rgb += (f.color.rgb - rgb)*a; }
            alpha = a + alpha*(1.0-a);
        } else {
            total += weight;
            associated_weight += a;
            sum += select(f.color.rgb, f.hsl.xyz, hsl)*a;
        }
    }
    if (normalized) {
        if (total <= 1e-12) { rgb = p.background.rgb; alpha = p.background.a; }
        else if (associated_weight <= 1e-12) { rgb = vec3<f32>(0.0); alpha = 0.0; }
        else { rgb = sum/associated_weight; if (hsl) { rgb = hsl_rgb(rgb); } alpha = associated_weight/total; }
    }
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    alpha = clamp(alpha, 0.0, 1.0);
    // BGRA8 slots use straight encoded sRGB. RGBA16F slots use associated linear P3.
    // For wide output the editor has already encoded the inputs in P3 primaries.
    if (p.shape.y > 0.5) {
        rgb = select(pow((rgb+0.055)/1.055, vec3<f32>(2.4)), rgb/12.92, rgb <= vec3<f32>(0.04045));
        rgb *= alpha;
    }
    return vec4<f32>(rgb, alpha);
}
