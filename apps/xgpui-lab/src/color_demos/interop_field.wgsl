// wgpu wrapper: shared shade math emits encoded sRGB or linear source RGB.
struct ShaderInput { position: vec2<f32>, size: vec2<f32>, scale_factor: f32 }
struct Uniforms { parameters: Parameters, viewport: vec4<f32> }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@vertex fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}
@fragment fn fragment(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = uniforms.parameters;
    let input = ShaderInput(pixel.xy / uniforms.viewport.z, uniforms.viewport.xy, uniforms.viewport.z);
    let color = shade(input, p);
    var linear = color.rgb;
    var source = u32(p.source_space);
    if source == 0u { linear = gamut_decode(linear, 0u); }
    // Shared shade already converts Adobe RGB to linear Rec2020.
    if source == 3u { source = 2u; }
    // Extended linear P3 retains components outside [0,1].
    return vec4(gamut_convert(linear, source, 1u) * color.a, color.a);
}
