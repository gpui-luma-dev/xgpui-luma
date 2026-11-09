//! App-owned Display P3 preview using the public typed shader API.
//! Stops arrive as source colors and are encoded to bounded Display P3 here;
//! sRGB stops keep their meaning (they are converted, not relabeled).
use super::*;
use super::super::encode::encode_rgba;
use gpui::{AnyElement, IntoElement, Window, canvas, div, prelude::*};
use xgpui_shader::instanced::{InstanceField, ShaderInstance, ShaderProgram, WindowShaderExt};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Parameters {
    angle: f32,
}
impl ShaderInstance for Parameters {
    const FIELDS: &'static [InstanceField] = &[InstanceField { name: "angle", offset: 0, components: 1 }];
}
#[derive(Default)]
pub(super) struct Cache(Option<(String, Result<ShaderProgram<Parameters>, String>)>);

pub(super) fn available(window: &Window) -> bool {
    let status = window.color_status();
    status.effective() == gpui::WindowColorMode::WideGamutSdr
        && status.supports_wide_gamut_sdr()
        && status.display_wide_gamut() == Some(true)
}

fn source(kind: GradientType, stops: &[(f32, ColorValue)]) -> String {
    let position = match kind {
        GradientType::Linear => {
            "let t = clamp(0.5 + dot(input.uv - vec2<f32>(0.5), vec2<f32>(sin(p.angle), -cos(p.angle))), 0.0, 1.0);"
        }
        GradientType::Radial => "let t = clamp(length(input.uv - vec2<f32>(0.5)) / 0.70710678118, 0.0, 1.0);",
        GradientType::Conic => {
            "let d = (input.uv - vec2<f32>(0.5)) * input.size; let turn = (atan2(d.x, -d.y) - p.angle) / 6.28318530718; let t = fract(turn + 2.0);"
        }
    };
    // Premultiplied encoded P3; the shader decodes after interpolation.
    let color = |c: ColorValue| {
        let [r, g, b, a] = encode_rgba(c, OutputSpace::DisplayP3);
        format!("vec4<f32>({:?}, {:?}, {:?}, {:?})", r * a, g * a, b * a, a)
    };
    let sorted = super::super::paint::sorted_stops(stops);
    let mut body = format!("var c = {};", color(sorted[0].1));
    for pair in sorted.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        body.push_str(&format!(
            "if (t >= {:?}) {{ c = mix({}, {}, clamp((t - {:?}) / {:?}, 0.0, 1.0)); }}",
            a.0,
            color(a.1),
            color(b.1),
            a.0,
            (b.0 - a.0).max(0.000001)
        ));
    }
    format!("struct Parameters {{ angle: f32, }}
fn decode(v: vec3<f32>) -> vec3<f32> {{ return select(pow((v + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)), v / 12.92, v <= vec3<f32>(0.04045)); }}
fn shade(input: ShaderInput, p: Parameters) -> vec4<f32> {{ {position} {body} return vec4<f32>(decode(c.rgb / max(c.a, 0.000001)), c.a); }}")
}
impl Cache {
    pub(super) fn element(
        &mut self,
        kind: GradientType,
        stops: &[(f32, ColorValue)],
        angle: f32,
        neutral: bool,
    ) -> AnyElement {
        if !neutral {
            return super::linear_shader::preview_notice(
                "P3 preview requires neutral Saturation/Vibrance adjustments.",
            );
        }
        if stops.len() < 2 {
            return div().into_any_element();
        }
        let source = source(kind, stops);
        if self.0.as_ref().is_none_or(|(old, _)| old != &source) {
            let program = ShaderProgram::new_linear_p3(&source).map_err(|error| error.to_string());
            self.0 = Some((source, program));
        }
        match self.0.as_ref().unwrap().1.clone() {
            Ok(program) => canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    if let Err(error) =
                        window.paint_custom_shader(bounds, &program, Parameters { angle: angle.to_radians() })
                    {
                        tracing::error!("P3 gradient: {error:#}");
                    }
                },
            )
            .size_full()
            .into_any_element(),
            Err(error) => super::linear_shader::preview_notice(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p3_programs_compile_for_all_gradient_types_and_eight_stops() {
        let stops: Vec<_> =
            (0..8).map(|i| (i as f32 / 7., srgb_source(gpui::hsla(i as f32 / 8., 1., 0.5, 0.7)))).collect();
        for kind in [GradientType::Linear, GradientType::Radial, GradientType::Conic] {
            ShaderProgram::<Parameters>::new_linear_p3(&source(kind, &stops)).unwrap();
        }
    }

    #[test]
    fn p3_stops_reach_the_shader_unclipped() {
        let stops =
            [(0.0, ColorValue::display_p3(1.0, 0.0, 0.0, 1.0)), (1.0, ColorValue::display_p3(0.0, 1.0, 0.0, 0.5))];
        let wgsl = source(GradientType::Linear, &stops);
        // Exact P3 primaries (premultiplied), not their sRGB gamut-mapped previews.
        assert!(wgsl.contains("vec4<f32>(1.0, 0.0, 0.0, 1.0)"), "{wgsl}");
        assert!(wgsl.contains("vec4<f32>(0.0, 0.5, 0.0, 0.5)"), "{wgsl}");
        ShaderProgram::<Parameters>::new_linear_p3(&wgsl).unwrap();
        // sRGB red is converted into P3 coordinates rather than relabeled as P3 red.
        let srgb = source(GradientType::Linear, &[(0.0, ColorValue::srgb(1.0, 0.0, 0.0, 1.0)); 2]);
        assert!(!srgb.contains("vec4<f32>(1.0, 0.0, 0.0, 1.0)"), "{srgb}");
    }
}
