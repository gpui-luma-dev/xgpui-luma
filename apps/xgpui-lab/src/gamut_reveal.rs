//! Generated ring/X pixels shared by the standalone and current lab pages.
use std::sync::Arc;
use gpui::{prelude::*, *};
use xgpui_color::{GamutMapping, LinearRgb, RgbSpace};
use xgpui_media::{
    ImageJob, MediaImage, MediaLoader,
    pipeline::{ImageColor, ImagePrimaries},
};

pub(crate) struct Reveal {
    jobs: Vec<ImageJob>,
    images: Vec<Option<MediaImage>>,
    pub(crate) error: Option<String>,
}
fn symbol(x: f64, y: f64) -> bool {
    let radius = x.hypot(y);
    (0.66..0.75).contains(&radius) || (x.abs() < 0.44 && y.abs() < 0.44 && (x.abs() - y.abs()).abs() < 0.09)
}
fn colors() -> anyhow::Result<([f64; 3], [f64; 3])> {
    let background = [1., 0., 0.]; // Linear Display P3 red, outside sRGB.
    let foreground = LinearRgb::new(RgbSpace::Srgb, [1., 0., 0.])?.convert(RgbSpace::DisplayP3)?.components(); // Same sRGB red expressed in P3; no reinterpretation.
    Ok((background, foreground))
}
impl Reveal {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let (background, foreground) = colors()?;
        let side = 384u32;
        let mut native = Vec::with_capacity((side * side) as usize);
        let mut mask = Vec::with_capacity(native.capacity());
        for y in 0..side {
            for x in 0..side {
                let coverage = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
                    .into_iter()
                    .filter(|(dx, dy)| {
                        symbol(
                            2. * (f64::from(x) + dx) / f64::from(side) - 1.,
                            2. * (f64::from(y) + dy) / f64::from(side) - 1.,
                        )
                    })
                    .count() as f64
                    / 4.;
                let rgb: [f64; 3] = std::array::from_fn(|c| background[c] * (1. - coverage) + foreground[c] * coverage);
                native.push([rgb[0] as f32, rgb[1] as f32, rgb[2] as f32, 1.]);
                let srgb = LinearRgb::new(RgbSpace::DisplayP3, rgb)?.convert(RgbSpace::Srgb)?.components();
                let outside = srgb.into_iter().any(|v| !(-1e-10..=1. + 1e-10).contains(&v));
                mask.push(if outside { [1., 0., 1., 1.] } else { [1., 0., 0., 1.] });
            }
        }
        let loader = MediaLoader::default();
        Ok(Self {
            jobs: vec![
                loader.load_pixel_buffer(
                    side,
                    side,
                    native,
                    ImageColor::LINEAR_P3.into(),
                    ImagePrimaries::DisplayP3,
                    GamutMapping::Clip,
                )?,
                loader.load_pixel_buffer(
                    side,
                    side,
                    mask,
                    ImageColor::SRGB.into(),
                    ImagePrimaries::DisplayP3,
                    GamutMapping::Clip,
                )?,
            ],
            images: vec![None, None],
            error: None,
        })
    }
    pub(crate) fn poll(&mut self, window: &mut Window) {
        for (job, image) in self.jobs.iter_mut().zip(&mut self.images) {
            if let Some(ready) = job.try_take() {
                *image = Some(ready);
            }
            match job.status() {
                xgpui_media::LoadStatus::Loading => window.request_animation_frame(),
                xgpui_media::LoadStatus::Failed(error) => self.error = Some(error),
                _ => {}
            }
        }
        for image in self.images.iter().flatten() {
            if let xgpui_media::UploadStatus::Failed(error) = image.upload_status() {
                self.error = Some(error);
            }
        }
    }
    pub(crate) fn picture(&self, selection: usize, wide: bool) -> impl IntoElement {
        let image = self.images[usize::from(selection == 2)].clone();
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                if let Some(image) = &image {
                    wake_for_upload(image, bounds, window);
                    let geometry = CustomGeometry::RoundedRect { bounds, corner_radii: Default::default() };
                    if selection == 1 && wide {
                        if let Ok(image) = image.wide(None) {
                            window.paint_custom(geometry, Arc::new(image));
                        }
                    } else {
                        window.paint_custom(geometry, Arc::new(image.clone()));
                    }
                }
            },
        )
        .size_full()
    }
}
fn wake_for_upload(image: &MediaImage, bounds: Bounds<Pixels>, window: &mut Window) {
    if !bounds.intersect(&window.content_mask().bounds).is_empty()
        && !matches!(image.upload_status(), xgpui_media::UploadStatus::Ready | xgpui_media::UploadStatus::Failed(_))
    {
        window.request_animation_frame();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::v1::test;
    use xgpui_media::pipeline;
    fn half(bits: u16) -> f64 {
        let sign = if bits & 0x8000 == 0 { 1. } else { -1. };
        let exponent = (bits >> 10) & 31;
        let fraction = f64::from(bits & 1023);
        sign * if exponent == 0 {
            fraction * 2f64.powi(-24)
        } else {
            (1. + fraction / 1024.) * 2f64.powi(i32::from(exponent) - 15)
        }
    }
    #[test]
    fn native_colors_differ_but_srgb_clipping_hides_the_symbol() {
        let (background, foreground) = colors().unwrap();
        assert!((background[0] - foreground[0]).abs() > 0.15);
        for color in [background, foreground] {
            let preview = LinearRgb::new(RgbSpace::DisplayP3, color)
                .unwrap()
                .map_gamut(RgbSpace::Srgb, GamutMapping::Clip)
                .unwrap()
                .components();
            for (value, expected) in preview.into_iter().zip([1., 0., 0.]) {
                assert!((value - expected).abs() < 1e-12);
            }
        }
        // Check the actual half-float storage boundary, not just f64 conversion.
        let pixels: Vec<[f32; 4]> =
            [background, foreground].into_iter().map(|v| [v[0] as f32, v[1] as f32, v[2] as f32, 1.]).collect();
        let packed = pipeline::pixels_rgba16f_data(
            2,
            1,
            pixels,
            ImageColor::LINEAR_P3.into(),
            ImagePrimaries::DisplayP3,
            GamutMapping::Clip,
        )
        .unwrap();
        let data = &packed.levels()[0];
        for pixel in data.as_chunks::<8>().0 {
            let rgb = std::array::from_fn(|i| half(u16::from_le_bytes([pixel[2 * i], pixel[2 * i + 1]])));
            let preview = LinearRgb::new(RgbSpace::DisplayP3, rgb)
                .unwrap()
                .map_gamut(RgbSpace::Srgb, GamutMapping::Clip)
                .unwrap()
                .components();
            for (value, expected) in preview.into_iter().zip([1., 0., 0.]) {
                assert!((value - expected).abs() < 0.001);
            }
        }
        let bg = LinearRgb::new(RgbSpace::DisplayP3, background)
            .unwrap()
            .convert(RgbSpace::Srgb)
            .unwrap()
            .components();
        let fg = LinearRgb::new(RgbSpace::DisplayP3, foreground)
            .unwrap()
            .convert(RgbSpace::Srgb)
            .unwrap()
            .components();
        assert!(bg[0] > 1.2 && bg[1] < 0.);
        assert!(fg.into_iter().all(|v| (-1e-10..=1. + 1e-10).contains(&v)));
        assert!(symbol(0.7, 0.));
        assert!(symbol(0., 0.));
        assert!(!symbol(0.9, 0.));
    }
}
