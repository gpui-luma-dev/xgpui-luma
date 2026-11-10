//! Paints lab colors at the window's real output. When the window presents wide-gamut
//! SDR, colors are drawn as linear Display P3 through xgpui-gamut's instanced
//! `WidePainter`, as the xgpui-gamut labs do; otherwise they are sRGB previews.
use std::sync::Arc;

use gpui::{AnyElement, IntoElement, ParentElement, Pixels, Rgba, Styled, canvas, div, px};
use xgpui_color::{ColorInterpolation, GamutMapping, LinearRgb, LinearRgba, RgbSpace};
use xgpui_gamut::{
    WideColor,
    instanced::{WideGradient, WidePainter},
};
use xgpui_shader::{Gradient, GradientStop, painted};

/// The most stops the shader gradient takes; denser bars draw one band per sample.
const MAX_STOPS: usize = 32;

/// Paints one band of a P3 bar.
type BandPaint = Box<dyn Fn(&WidePainter, &mut gpui::Window, gpui::Bounds<Pixels>) -> anyhow::Result<()>>;

/// Where lab colors go: linear Display P3 on a wide window, or an sRGB preview.
pub(super) struct Display {
    painter: Option<Arc<WidePainter>>,
}

impl Display {
    /// `painter` is `Some` only when the window's effective presentation is wide-gamut SDR.
    pub fn new(painter: Option<Arc<WidePainter>>) -> Self {
        Self { painter }
    }

    pub fn wide(&self) -> bool {
        self.painter.is_some()
    }

    /// What the swatches show, for captions.
    pub fn label(&self) -> &'static str {
        if self.wide() {
            "Display P3 output"
        } else {
            "sRGB preview"
        }
    }

    /// A fill of one color. Colors outside the display gamut are fitted with `policy`, for display only.
    pub fn fill(
        &self,
        color: LinearRgb,
        alpha: f64,
        policy: GamutMapping,
        radius: Pixels,
    ) -> anyhow::Result<AnyElement> {
        let Some(painter) = self.painter.clone() else {
            let rgba = srgb(color, alpha, policy)?;
            return Ok(div().size_full().rounded(radius).bg(rgba).into_any_element());
        };
        let wide = WideColor::linear_p3(p3(color, alpha, policy)?)?;
        Ok(canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                if let Err(error) = painter.paint_color(window, bounds, radius, &wide) {
                    eprintln!("Color lab P3 fill: {error:#}");
                }
            },
        )
        .size_full()
        .into_any_element())
    }

    /// A bar of CPU samples, fitted into the display gamut with `policy`. Up to 32 samples
    /// are interpolated between; denser bars draw every sample as its own band.
    pub fn samples(&self, colors: &[LinearRgba], policy: GamutMapping, height: Pixels) -> anyhow::Result<AnyElement> {
        let Some(painter) = self.painter.clone() else {
            let colors = colors
                .iter()
                .map(|color| srgb(color.rgb(), color.alpha(), policy).map(|c| [c.r, c.g, c.b, c.a]))
                .collect::<anyhow::Result<Vec<_>>>()?;
            return srgb_bar(&colors, height);
        };
        let colors = colors
            .iter()
            .map(|color| p3(color.rgb(), color.alpha(), policy))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let band = |paint: BandPaint| {
            let painter = painter.clone();
            div().flex_1().min_w_0().h_full().child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        if let Err(error) = paint(&painter, window, bounds) {
                            eprintln!("Color lab P3 bar: {error:#}");
                        }
                    },
                )
                .size_full(),
            )
        };
        let bands = if colors.len() <= MAX_STOPS {
            // One linear segment between each pair of neighboring samples, like the sRGB shader gradient.
            colors
                .windows(2)
                .map(|pair| {
                    let gradient = WideGradient::linear(
                        [0., 0.],
                        [1., 0.],
                        [linear_p3(pair[0])?, linear_p3(pair[1])?],
                        ColorInterpolation::LinearRgb,
                        GamutMapping::Clip,
                    )?;
                    Ok(band(Box::new(move |painter, window, bounds| {
                        painter.paint_gradient(window, bounds, px(0.), &gradient)
                    })))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            colors
                .into_iter()
                .map(|color| {
                    let wide = WideColor::linear_p3(color)?;
                    Ok(band(Box::new(move |painter, window, bounds| {
                        painter.paint_color(window, bounds, px(0.), &wide)
                    })))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        };
        Ok(div().flex().w_full().h(height).children(bands).into_any_element())
    }
}

/// A bar of encoded sRGB samples (straight alpha), for references that target sRGB.
pub(super) fn srgb_bar(colors: &[[f32; 4]], height: Pixels) -> anyhow::Result<AnyElement> {
    // Same display path as extensions-web/color.rs: shader interpolation between CPU-mapped samples.
    if colors.len() <= MAX_STOPS {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, color)| GradientStop::new(i as f32 / (colors.len() - 1) as f32, *color))
            .collect();
        let paint = Arc::new(Gradient::linear(90., stops)?);
        return Ok(painted().w_full().h(height).rounded_md().bg_paint(paint).into_any_element());
    }
    // The shader producer accepts at most 32 stops. Preserve every CPU sample
    // at higher resolutions instead of silently truncating the reference.
    Ok(div()
        .flex()
        .w_full()
        .h(height)
        .rounded_md()
        .overflow_hidden()
        .children(colors.iter().map(|&[r, g, b, a]| div().flex_1().min_w_0().h_full().bg(Rgba { r, g, b, a })))
        .into_any_element())
}

/// Encoded sRGB with straight alpha, fitted with `policy`.
fn srgb(color: LinearRgb, alpha: f64, policy: GamutMapping) -> anyhow::Result<Rgba> {
    let [r, g, b] = color.map_gamut(RgbSpace::Srgb, policy)?.encode()?.components();
    Ok(Rgba { r: r as f32, g: g as f32, b: b as f32, a: alpha as f32 })
}

/// Linear Display P3 with straight alpha, fitted with `policy` and clamped against rounding.
fn p3(color: LinearRgb, alpha: f64, policy: GamutMapping) -> anyhow::Result<[f32; 4]> {
    let [r, g, b] = color.map_gamut(RgbSpace::DisplayP3, policy)?.components();
    Ok([r, g, b, alpha].map(|channel| channel.clamp(0., 1.) as f32))
}

fn linear_p3([r, g, b, a]: [f32; 4]) -> anyhow::Result<LinearRgba> {
    Ok(LinearRgba::new(LinearRgb::new(RgbSpace::DisplayP3, [r as f64, g as f64, b as f64])?, a as f64)?)
}

#[cfg(test)]
mod tests {
    use super::{p3, srgb};
    use xgpui_color::{GamutMapping, LinearRgb, RgbSpace};

    #[::core::prelude::v1::test]
    fn wide_output_keeps_p3_colors_that_srgb_previews_must_fit() {
        let p3_red = LinearRgb::new(RgbSpace::DisplayP3, [1., 0., 0.]).unwrap();
        // Drawn as P3, the most saturated P3 red is unchanged.
        assert_eq!(p3(p3_red, 1., GamutMapping::Clip).unwrap(), [1., 0., 0., 1.]);
        // The sRGB preview has to clip it to sRGB red: the difference the wide path shows.
        let preview = srgb(p3_red, 1., GamutMapping::Clip).unwrap();
        assert_eq!([preview.r, preview.g, preview.b], [1., 0., 0.]);
        // A Rec2020 green outside P3 is fitted into displayable P3 channels.
        let rec2020_green = LinearRgb::new(RgbSpace::Rec2020, [0., 1., 0.]).unwrap();
        for policy in [GamutMapping::Clip, GamutMapping::ReduceChroma] {
            let fitted = p3(rec2020_green, 1., policy).unwrap();
            assert!(fitted.iter().all(|channel| (0. ..=1.).contains(channel)));
        }
    }
}
