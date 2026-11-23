use gpui::{BoxShadow, Hsla, point, px};
use crate::color::{ColorValue, GamutMapping, gpui_bridge::SrgbRenderCache};
use anyhow::{Context as _, Result};
use serde::{Serialize, Deserialize};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LumaElevation<C = ColorValue> {
    pub none: LumaShadow<C>,
    pub drag_preview: LumaShadow<C>,
    pub control: LumaShadow<C>,
    pub thumb: LumaShadow<C>,
    pub menu: LumaShadow<C>,
    pub popover: LumaShadow<C>,
    pub panel: LumaShadow<C>,
    pub dialog: LumaShadow<C>,
    /// Nonmodal overlay-window elevation, independent of dialog elevation.
    pub overlay_window: LumaShadow<C>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LumaShadow<C = ColorValue> {
    pub layers: Vec<LumaShadowLayer<C>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LumaShadowLayer<C = ColorValue> {
    pub color: C,
    pub inset: bool,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_finite")]
    pub offset_x: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_finite")]
    pub offset_y: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub blur: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_finite")]
    pub spread: f32,
}

impl Default for LumaElevation {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaElevation<ColorValue> {
    /// The SDK's light shadows, from `theme/palette.lss`.
    pub fn light() -> Self {
        super::from_lss::sdk(super::ThemeMode::Light).elevation.clone()
    }
    /// The SDK's dark shadows, from `theme/palette.lss` (`mode dark`).
    pub fn dark() -> Self {
        super::from_lss::sdk(super::ThemeMode::Dark).elevation.clone()
    }

    /// Prepare current GPUI previews once while retaining original shadow colors.
    pub fn snapshot_srgb(self, policy: GamutMapping) -> Result<SrgbElevation> {
        let mut cache = SrgbRenderCache::default();
        let resolve = |shadow: &LumaShadow, name: &str, cache: &mut SrgbRenderCache| -> Result<LumaShadow<Hsla>> {
            let layers = shadow
                .layers
                .iter()
                .enumerate()
                .map(|(index, layer)| {
                    anyhow::ensure!(
                        [layer.offset_x, layer.offset_y, layer.blur, layer.spread].iter().all(|v| v.is_finite())
                            && layer.blur >= 0.0,
                        "invalid shadow geometry at {name}[{index}]"
                    );
                    Ok(LumaShadowLayer::new(
                        cache.resolve_hsla(layer.color, policy).with_context(|| format!("shadow {name}[{index}]"))?,
                        layer.offset_x,
                        layer.offset_y,
                        layer.blur,
                        layer.spread,
                        layer.inset,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(LumaShadow::new(layers))
        };
        let preview = LumaElevation {
            none: resolve(&self.none, "none", &mut cache)?,
            drag_preview: resolve(&self.drag_preview, "drag_preview", &mut cache)?,
            control: resolve(&self.control, "control", &mut cache)?,
            thumb: resolve(&self.thumb, "thumb", &mut cache)?,
            menu: resolve(&self.menu, "menu", &mut cache)?,
            popover: resolve(&self.popover, "popover", &mut cache)?,
            panel: resolve(&self.panel, "panel", &mut cache)?,
            dialog: resolve(&self.dialog, "dialog", &mut cache)?,
            overlay_window: resolve(&self.overlay_window, "overlay_window", &mut cache)?,
        };
        Ok(SrgbElevation { source: Arc::new(self), preview: Arc::new(preview), policy })
    }
}

impl<C> Default for LumaShadow<C> {
    fn default() -> Self {
        Self { layers: Vec::new() }
    }
}

impl<C> LumaShadow<C> {
    pub fn new(layers: Vec<LumaShadowLayer<C>>) -> Self {
        Self { layers }
    }

    /// Construct an explicit shadow; `inset` selects inner versus outer painting.
    pub fn single(color: C, offset_x: f32, offset_y: f32, blur: f32, spread: f32, inset: bool) -> Self {
        Self::new(vec![LumaShadowLayer::new(color, offset_x, offset_y, blur, spread, inset)])
    }
}

impl LumaShadow<Hsla> {
    pub fn to_box_shadows(&self) -> Vec<BoxShadow> {
        self.layers.iter().map(LumaShadowLayer::to_box_shadow).collect()
    }
}

impl<C> LumaShadowLayer<C> {
    /// Construct one explicit layer, including its inner/outer painting mode.
    pub fn new(color: C, offset_x: f32, offset_y: f32, blur: f32, spread: f32, inset: bool) -> Self {
        Self { color, inset, offset_x, offset_y, blur, spread }
    }
}

impl LumaShadowLayer<Hsla> {
    pub fn to_box_shadow(&self) -> BoxShadow {
        BoxShadow {
            color: self.color,
            offset: point(px(self.offset_x), px(self.offset_y)),
            blur_radius: px(self.blur),
            spread_radius: px(self.spread),
            inset: self.inset,
        }
    }
}

/// Immutable source shadows and their current sRGB rendering snapshot.
#[derive(Clone, Debug)]
pub struct SrgbElevation {
    source: Arc<LumaElevation>,
    preview: Arc<LumaElevation<Hsla>>,
    policy: GamutMapping,
}
impl SrgbElevation {
    pub fn source(&self) -> &LumaElevation {
        &self.source
    }
    pub fn mapping(&self) -> GamutMapping {
        self.policy
    }
    pub fn light() -> Self {
        static ELEVATION: OnceLock<SrgbElevation> = OnceLock::new();
        ELEVATION.get_or_init(|| Self::sdk(LumaElevation::light())).clone()
    }
    pub fn dark() -> Self {
        static ELEVATION: OnceLock<SrgbElevation> = OnceLock::new();
        ELEVATION.get_or_init(|| Self::sdk(LumaElevation::dark())).clone()
    }

    /// The SDK shadows' preview. Their colors come from the built-in `.lss`, checked by tests.
    fn sdk(source: LumaElevation) -> Self {
        source
            .snapshot_srgb(GamutMapping::CssLocalMinde)
            .unwrap_or_else(|e| panic!("the SDK shadows map to sRGB: {e:#}"))
    }
}

impl Deref for SrgbElevation {
    type Target = LumaElevation<Hsla>;
    fn deref(&self) -> &Self::Target {
        &self.preview
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_retains_extended_p3_shadow_source() {
        let mut source = LumaElevation::light();
        let color = ColorValue::display_p3(1.15, -0.1, 0.2, 0.234567);
        source.menu.layers[0].color = color;
        let snapshot = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(snapshot.source().menu.layers[0].color, color);
        assert_eq!(snapshot.menu.layers[0].color.a, color.alpha());
        let stored = toml::to_string(snapshot.source()).unwrap();
        let restored: LumaElevation = toml::from_str(&stored).unwrap();
        assert_eq!(restored, *snapshot.source());
    }
    #[test]
    fn invalid_shadow_color_is_rejected_with_role() {
        let mut source = LumaElevation::light();
        source.thumb.layers[0].color = ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0);
        assert!(source.snapshot_srgb(GamutMapping::Clip).unwrap_err().to_string().contains("thumb[0]"));
    }
}
