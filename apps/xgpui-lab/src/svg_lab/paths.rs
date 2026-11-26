//! Path fixtures and comparison panels adapted from media-lab/src/paths.rs.
use anyhow::Result;
use crate::theme::ColorVizLookExt;
use gpui::{prelude::*, *};
use xgpui_curve::geom::Affine;
use xgpui_curve::gpui_paint::stroke_path;
use xgpui_curve::measure::{MeasuredPath, PathEffect};
use xgpui_curve::svg_path::parse;

pub fn samples() -> Result<Vec<MeasuredPath>> {
    let contours: [&[&str]; 3] = [
        &["M20 100 C60 -30 140 210 190 85 Q250 -20 300 100"],
        &["M160 150 C120 115 20 60 70 30 C105 5 145 20 160 50 C175 20 215 5 250 30 C300 60 200 115 160 150 Z"],
        &["M20 45 L110 45 L110 120 L20 120 Z", "M170 120 Q170 5 230 45 C300 -5 320 120 250 135"],
    ];
    contours.into_iter().map(|parts| Ok(MeasuredPath::new(&parse(&parts.join(" "))?, 0.05)?)).collect()
}
pub(super) fn effect(trim: f32, dash: usize, phase: f32) -> Result<PathEffect> {
    let e = PathEffect::new().trim(0. ..f64::from(trim))?;
    let pattern: &[f64] = match dash {
        0 => return Ok(e),
        1 => &[16., 8.],
        2 => &[4., 7.],
        _ => &[18., 5., 3.],
    };
    Ok(e.dash(pattern, f64::from(phase))?)
}
/// Applies `effect`, then tessellates the stroke in fixture coordinates. `None` when nothing is visible.
pub(super) fn stroke(path: &MeasuredPath, effect: &PathEffect, width: f32) -> Result<Option<Path<Pixels>>> {
    let visible = path.apply(effect)?;
    if visible.elements().is_empty() {
        return Ok(None);
    }
    let stroked = stroke_path(&visible, Affine::IDENTITY, width);
    stroked.map(Some).ok_or_else(|| anyhow::anyhow!("Stroke tessellation failed"))
}
fn place(mut path: Path<Pixels>, offset: Point<Pixels>, scale: f32) -> Path<Pixels> {
    for v in &mut path.vertices {
        v.xy_position = offset + v.xy_position * scale;
    }
    path.bounds.origin = offset + path.bounds.origin * scale;
    path.bounds.size = path.bounds.size.map(|v| v * scale);
    path
}
pub(super) fn panel(
    look: &xgpui_luma_look_radix::Look,
    path: &MeasuredPath,
    effect: PathEffect,
    width: f32,
    color: u32,
    height: f32,
) -> Div {
    let built = stroke(path, &effect, width);
    let error = built.as_ref().err().map(ToString::to_string);
    let built = built.ok().flatten();
    div()
        .w_full()
        .h(px(height))
        .debug_selector(|| "svg-path-panel".into())
        .rounded_lg()
        .bg(look.chrome().panel_background)
        .overflow_hidden()
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, (), window, _| {
                    if let Some(path) = &built {
                        let scale = (f32::from(bounds.size.width) / 340.).min(f32::from(bounds.size.height) / 180.);
                        let offset = bounds.origin
                            + point(
                                (bounds.size.width - px(320. * scale)) * 0.5,
                                (bounds.size.height - px(160. * scale)) * 0.5,
                            );
                        window.with_content_mask(Some(ContentMask { bounds }), |window| {
                            window.paint_path(place(path.clone(), offset, scale), rgb(color))
                        });
                    }
                },
            )
            .size_full(),
        )
        .children(error.map(|e| {
            div()
                .text_sm()
                .text_color(look.resolve_role(xgpui_luma_look_radix::SemanticRole::Destructive).hsla())
                .child(e)
        }))
}
