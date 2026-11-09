use gpui::RenderImage;
use xgpui_luma::color::ColorValue;

#[derive(Clone, Copy, Default)]
pub struct ImageAdjustments {
    pub saturation: f32,
    pub vibrance: f32,
}

impl ImageAdjustments {
    pub fn is_neutral(self) -> bool {
        self.saturation == 0.0 && self.vibrance == 0.0
    }

    /// Adjust a source color before it is encoded for any renderer (CPU,
    /// shaders, Display P3). Works on HSL over encoded Display P3, unclamped,
    /// so wide colors are neither clipped nor capped at sRGB saturation.
    /// Vibrance preferentially affects muted colors; neutral gray stays gray.
    /// Saturation never rises above 100% unless the source already did.
    /// Neutral settings return the source unchanged (space and components).
    pub fn adjust_color(self, color: ColorValue) -> ColorValue {
        use crate::color_stop::hsl::{hsl_to_rgb, rgb_to_hsl};
        if self.is_neutral() {
            return color;
        }
        let Ok(p3) = color.to_display_p3_unclamped() else {
            return color;
        };
        // Extended-range HSL over the P3-encoded values (no clipping).
        let [hue, original, lightness] = rgb_to_hsl([p3.red, p3.green, p3.blue]);
        let muted = original.clamp(0.0, 1.0);
        let mut saturation = original + self.vibrance.clamp(-100.0, 100.0) / 100.0 * muted * (1.0 - muted);
        saturation *= 1.0 + self.saturation.clamp(-100.0, 100.0) / 100.0;
        let [r, g, b] = hsl_to_rgb([hue, saturation.clamp(0.0, original.max(1.0)), lightness]);
        ColorValue::display_p3(r, g, b, p3.alpha)
    }
}

/// Save a GPUI premultiplied BGRA preview as a straight-alpha RGBA PNG.
pub fn save_preview_png(source: &RenderImage, path: &std::path::Path) -> anyhow::Result<()> {
    let size = source.size(0);
    let mut bytes = source.as_bytes(0).ok_or_else(|| anyhow::anyhow!("missing preview pixels"))?.to_vec();
    for pixel in bytes.as_chunks_mut::<4>().0 {
        let alpha = pixel[3] as f32 / 255.0;
        pixel.swap(0, 2);
        for channel in &mut pixel[..3] {
            *channel = if alpha > 0.0 {
                (*channel as f32 / alpha).round().clamp(0.0, 255.0) as u8
            } else {
                0
            };
        }
    }
    let buffer =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(size.width.0.try_into()?, size.height.0.try_into()?, bytes)
            .ok_or_else(|| anyhow::anyhow!("invalid preview dimensions"))?;
    buffer.save_with_format(path, image::ImageFormat::Png)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::color::Gamut;

    #[test]
    fn png_export_writes_straight_rgba_pixels() {
        let buffer = image::ImageBuffer::from_raw(1, 1, vec![0, 0, 128, 128]).unwrap();
        let source = RenderImage::new(smallvec::smallvec![image::Frame::new(buffer)]);
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("color-viz-export-{}-{nonce}.png", std::process::id()));
        save_preview_png(&source, &path).unwrap();
        let output = image::open(&path).unwrap().into_rgba8();
        std::fs::remove_file(path).unwrap();
        assert_eq!(output.dimensions(), (1, 1));
        assert_eq!(output.get_pixel(0, 0).0, [255, 0, 0, 128]);
    }

    fn p3(color: ColorValue) -> [f32; 4] {
        let c = color.to_display_p3_unclamped().unwrap();
        [c.red, c.green, c.blue, c.alpha]
    }

    #[test]
    fn neutral_settings_keep_the_source_exactly() {
        let color = ColorValue::srgb(0.2, 0.4, 0.7, 0.3);
        assert_eq!(ImageAdjustments::default().adjust_color(color), color);
    }

    #[test]
    fn saturation_zeroes_chroma_and_preserves_alpha() {
        let color = ColorValue::srgb(0.6, 0.3, 0.1, 0.5);
        let [r, g, b, a] = p3(ImageAdjustments { saturation: -100.0, vibrance: 100.0 }.adjust_color(color));
        assert!((r - g).abs() < 1e-5 && (g - b).abs() < 1e-5, "{r} {g} {b}");
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn vibrance_boosts_muted_colors_and_preserves_saturated_colors() {
        let settings = ImageAdjustments { saturation: 0.0, vibrance: 100.0 };
        let chroma = |c: ColorValue| c.to_oklcha_unclamped().unwrap().chroma;
        let muted = ColorValue::display_p3(0.5, 0.4, 0.45, 1.0);
        assert!(chroma(settings.adjust_color(muted)) > chroma(muted) + 0.01);
        let saturated = ColorValue::display_p3(1.0, 0.0, 0.0, 1.0);
        let out = p3(settings.adjust_color(saturated));
        assert!((out[0] - 1.0).abs() < 1e-5 && out[1].abs() < 1e-5 && out[2].abs() < 1e-5);
    }

    #[test]
    fn saturation_reaches_beyond_srgb_without_clipping() {
        // +100% saturation of sRGB red lands on P3 red, outside sRGB.
        let boosted =
            ImageAdjustments { saturation: 100.0, vibrance: 0.0 }.adjust_color(ColorValue::srgb(1.0, 0.0, 0.0, 1.0));
        assert!(!boosted.is_in_gamut(Gamut::Srgb).unwrap());
        assert!(boosted.is_in_gamut(Gamut::DisplayP3).unwrap());
        // Already-wide colors are not pulled back to 100% by a boost.
        let wide = ColorValue::oklch(0.65, 0.4, 145.0, 1.0);
        let boosted = ImageAdjustments { saturation: 10.0, vibrance: 50.0 }.adjust_color(wide);
        assert!(!boosted.is_in_gamut(Gamut::DisplayP3).unwrap());
    }
}
