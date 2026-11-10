//! Rendering implementations ported from XGPUI e0a7833b6e6ae234eb6c19d2988c085e9524daeb.
use super::{BACKGROUND_POINTS, FIELDS, FOLD_CRESTS, RIBBONS, edge_y};
use gpui::*;
use xgpui_shader::{
    WindowShaderExt,
    instanced::{InstanceField, ShaderInstance, ShaderProgram},
};
#[cfg(target_os = "windows")]
mod dx11;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod hlsl;
#[cfg(target_os = "macos")]
mod metal;
mod native;
#[cfg(any(target_os = "linux", feature = "waves-wgpu"))]
mod wgpu_producer;
pub(super) const NATIVE_BACKEND: &str = native::BACKEND;
const STEPS: usize = 256;
thread_local! {
    static WAVES_PROGRAM: Result<ShaderProgram<WavesParams>, String> = ShaderProgram::new(&waves_source()).map_err(|error| format!("{error:#}"));
}
fn ribbon_motion(phase: f32, index: usize, x: f32) -> f32 {
    0.012 * ((phase + x * 5. + index as f32 * 0.7).sin() - (x * 5. + index as f32 * 0.7).sin())
}

/// WGSL constants generated from the Rust composition data, followed by the
/// single-pass shader, so both renderers share one source of truth.
fn waves_source() -> String {
    wgsl_constants() + include_str!("renderers/pastel_waves.wgsl")
}

/// The same constants followed by the native wgpu/D3D11 producer.
fn producer_source() -> String {
    wgsl_constants() + include_str!("renderers/producer.wgsl")
}

fn wgsl_constants() -> String {
    fn floats(values: impl IntoIterator<Item = f32>) -> String {
        values.into_iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(",")
    }
    fn vectors<const N: usize>(values: impl IntoIterator<Item = [f32; N]>) -> String {
        values.into_iter().map(|v| format!("vec{}<f32>({})", N, floats(v))).collect::<Vec<_>>().join(",")
    }
    let edges: Vec<f32> = RIBBONS.iter().flat_map(|r| r.top.into_iter().chain(r.bottom)).collect();
    let styles = RIBBONS.iter().map(|r| [r.color[0], r.color[1], r.color[2], r.alpha]);
    let geometry = FIELDS.iter().map(|&(center, radii, _, _)| [center[0], center[1], radii[0], radii[1]]);
    let tints = FIELDS.iter().map(|&(_, _, color, alpha)| [color[0], color[1], color[2], alpha]);
    let positions = BACKGROUND_POINTS.iter().map(|&(position, _)| position);
    let colors = BACKGROUND_POINTS.iter().map(|&(_, color)| [color[0], color[1], color[2]]);
    let ribbons = RIBBONS.len();
    format!(
        "const RIB_EDGES: array<f32,{}> = array<f32,{}>({});\n\
         const RIB_STYLE: array<vec4<f32>,{ribbons}> = array<vec4<f32>,{ribbons}>({});\n\
         const FOLD_CRESTS: array<f32,4> = array<f32,4>({});\n\
         const FIELD_GEOM: array<vec4<f32>,9> = array<vec4<f32>,9>({});\n\
         const FIELD_TINT: array<vec4<f32>,9> = array<vec4<f32>,9>({});\n\
         const BG_POS: array<vec2<f32>,5> = array<vec2<f32>,5>({});\n\
         const BG_COLOR: array<vec3<f32>,5> = array<vec3<f32>,5>({});\n",
        edges.len(),
        edges.len(),
        floats(edges),
        vectors(styles),
        floats(FOLD_CRESTS),
        vectors(geometry),
        vectors(tints),
        vectors(positions),
        vectors(colors),
    )
}

/// Normalized y bounds (min, max) of each ribbon's fill path, the bounds its
/// mesh brush is evaluated in.
fn fill_bounds(phase: f32) -> [f32; 18] {
    let mut bounds = [0f32; 18];
    for (index, ribbon) in RIBBONS.iter().enumerate() {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for step in 0..=STEPS {
            let x = step as f32 / STEPS as f32;
            let drift = ribbon_motion(phase, index, x);
            for y in [edge_y(ribbon.top, x), edge_y(ribbon.bottom, x)] {
                lo = lo.min(y + drift);
                hi = hi.max(y + drift);
            }
        }
        bounds[index * 2] = lo;
        bounds[index * 2 + 1] = hi;
    }
    bounds
}

/// `Parameters` of pastel_waves.wgsl: eight `vec4<f32>` slots, `data0`..`data7`,
/// packed by `waves_instance`.
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct WavesParams {
    data: [[f32; 4]; 8],
}
impl ShaderInstance for WavesParams {
    const FIELDS: &'static [InstanceField] = &{
        const NAMES: [&str; 8] = ["data0", "data1", "data2", "data3", "data4", "data5", "data6", "data7"];
        let mut fields = [InstanceField { name: "", offset: 0, components: 4 }; 8];
        let mut i = 0;
        while i < 8 {
            fields[i].name = NAMES[i];
            fields[i].offset = i * 16;
            i += 1;
        }
        fields
    };
}

/// Phase, grain and the per-ribbon fill bounds the mesh fill is evaluated in.
fn waves_instance(phase: f32, grain: bool) -> WavesParams {
    let mut values = [0f32; 32];
    values[0] = phase;
    values[1] = if grain { 1. } else { 0. };
    values[4..22].copy_from_slice(&fill_bounds(phase));
    WavesParams { data: std::array::from_fn(|i| std::array::from_fn(|j| values[i * 4 + j])) }
}

/// Metal constants generated from the Rust composition data, followed by the
/// native shader source.
#[cfg(target_os = "macos")]
fn metal_source() -> String {
    fn floats(values: impl IntoIterator<Item = f32>) -> String {
        values.into_iter().map(|v| format!("{v:?}f")).collect::<Vec<_>>().join(",")
    }
    fn vectors<const N: usize>(values: impl IntoIterator<Item = [f32; N]>) -> String {
        values.into_iter().map(|v| format!("float{}({})", N, floats(v))).collect::<Vec<_>>().join(",")
    }
    let edges: Vec<f32> = RIBBONS.iter().flat_map(|r| r.top.into_iter().chain(r.bottom)).collect();
    let styles = RIBBONS.iter().map(|r| [r.color[0], r.color[1], r.color[2], r.alpha]);
    let geometry = FIELDS.iter().map(|&(center, radii, _, _)| [center[0], center[1], radii[0], radii[1]]);
    let tints = FIELDS.iter().map(|&(_, _, color, alpha)| [color[0], color[1], color[2], alpha]);
    let positions = BACKGROUND_POINTS.iter().map(|&(position, _)| position);
    let colors = BACKGROUND_POINTS.iter().map(|&(_, color)| [color[0], color[1], color[2]]);
    let ribbons = RIBBONS.len();
    format!(
        "#include <metal_stdlib>\nusing namespace metal;\n\
         constant float RIB_EDGES[{}] = {{{}}};\n\
         constant float4 RIB_STYLE[{ribbons}] = {{{}}};\n\
         constant float FOLD_CRESTS[4] = {{{}}};\n\
         constant float4 FIELD_GEOM[9] = {{{}}};\n\
         constant float4 FIELD_TINT[9] = {{{}}};\n\
         constant float2 BG_POS[5] = {{{}}};\n\
         constant float3 BG_COLOR[5] = {{{}}};\n{}",
        edges.len(),
        floats(edges),
        vectors(styles),
        floats(FOLD_CRESTS),
        vectors(geometry),
        vectors(tints),
        vectors(positions),
        vectors(colors),
        include_str!("renderers/pastel_waves.metal"),
    )
}

pub(super) fn paint_shader(bounds: Bounds<Pixels>, phase: f32, grain: bool, window: &mut Window) -> Result<(), String> {
    WAVES_PROGRAM.with(|program| {
        let program = program.as_ref().map_err(Clone::clone)?;
        window
            .paint_custom_shader(bounds, program, waves_instance(phase, grain))
            .map_err(|error| format!("{error:#}"))
    })
}
/// One native producer snapshot per frame, composited over the full bounds.
pub(super) fn paint_native(
    bounds: Bounds<Pixels>,
    phase: f32,
    grain: bool,
    window: &mut Window,
    status: std::sync::Arc<std::sync::Mutex<super::RenderStatus>>,
) {
    let content: std::sync::Arc<dyn CustomContent> = std::sync::Arc::new(ReportingNative {
        status,
        content: native::PastelWavesNative {
            phase,
            grain,
            scale: window.scale_factor(),
            fill_bounds: fill_bounds(phase),
        },
    });
    window.paint_custom(CustomGeometry::RoundedRect { bounds, corner_radii: Corners::default() }, content);
}

/// Native preparation runs after element painting; publish its result for the
/// next UI frame and switch to retained paths after a preparation failure.
#[derive(Debug)]
struct ReportingNative {
    status: std::sync::Arc<std::sync::Mutex<super::RenderStatus>>,
    content: native::PastelWavesNative,
}
impl CustomContent for ReportingNative {
    fn prepare(
        &self,
        backend: &dyn std::any::Any,
        extent: Size<DevicePixels>,
    ) -> anyhow::Result<Box<dyn std::any::Any>> {
        let result = self.content.prepare(backend, extent);
        if let Ok(mut status) = self.status.lock() {
            match &result {
                Ok(_) => {
                    status.effective = Some(super::Renderer::Native);
                    status.error = None;
                }
                Err(error) => {
                    status.effective = None;
                    status.error = Some(format!("{error:#}"));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn portable_and_producer_shaders_validate() -> anyhow::Result<()> {
        let _program = ShaderProgram::<WavesParams>::new(&waves_source())?;
        let source = producer_source();
        let module = naga::front::wgsl::parse_str(&source)
            .map_err(|error| anyhow::anyhow!("{}", error.emit_to_string(&source)))?;
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty())
            .validate(&module)?;
        assert_eq!(module.entry_points.len(), 3);
        Ok(())
    }
    #[::core::prelude::v1::test]
    fn fill_bounds_cover_every_sample() {
        for phase in [0., 0.4, 1.8, std::f32::consts::PI] {
            let bounds = fill_bounds(phase);
            for (index, ribbon) in RIBBONS.iter().enumerate() {
                for step in 0..=STEPS {
                    let x = step as f32 / STEPS as f32;
                    for y in [edge_y(ribbon.top, x), edge_y(ribbon.bottom, x)] {
                        let y = y + ribbon_motion(phase, index, x);
                        assert!(y >= bounds[index * 2] && y <= bounds[index * 2 + 1]);
                    }
                }
            }
        }
    }
    #[::core::prelude::v1::test]
    fn native_failure_is_reported_without_a_panic() {
        let status: std::sync::Arc<std::sync::Mutex<super::super::RenderStatus>> = Default::default();
        let content = ReportingNative {
            status: std::sync::Arc::clone(&status),
            content: native::PastelWavesNative { phase: 0., grain: true, scale: 1., fill_bounds: fill_bounds(0.) },
        };
        assert!(content.prepare(&(), size(DevicePixels(100), DevicePixels(100))).is_err());
        let status = status.lock().unwrap();
        assert!(status.error.as_deref().unwrap().contains("no native producer"));
        assert_eq!(status.effective, None);
    }
}
