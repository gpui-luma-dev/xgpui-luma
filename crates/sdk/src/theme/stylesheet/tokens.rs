//! Native vanilla token defaults and field-by-field external token overrides.
use serde::{Deserialize, Deserializer, Serialize};
use crate::theme::{LumaElevation, LumaPalette, LumaThemeMode, LumaTypography, MetricTokens, ThemeMode};

/// Complete shared tokens. Sparse authored tables inherit the SDK baseline.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TokenStylesheet {
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub palette: PaletteStylesheet,
    pub elevation: ElevationStylesheet,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PaletteStylesheet {
    pub light: LumaPalette,
    pub dark: LumaPalette,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ElevationStylesheet {
    pub light: LumaElevation,
    pub dark: LumaElevation,
}

impl Default for TokenStylesheet {
    fn default() -> Self {
        Self {
            metrics: MetricTokens::default(),
            typography: LumaTypography::default(),
            palette: PaletteStylesheet { light: LumaPalette::light(), dark: LumaPalette::dark() },
            elevation: ElevationStylesheet { light: LumaElevation::light(), dark: LumaElevation::dark() },
        }
    }
}

impl TokenStylesheet {
    /// Independent source snapshot. Programmatic edits can be applied before
    /// projecting colors through `ThemeTokens::from_source`.
    pub fn source(&self, mode: ThemeMode) -> LumaThemeMode {
        let (palette, elevation) = match mode {
            ThemeMode::Light => (&self.palette.light, &self.elevation.light),
            ThemeMode::Dark => (&self.palette.dark, &self.elevation.dark),
        };
        LumaThemeMode::new(palette.clone(), self.metrics, self.typography.clone(), elevation.clone())
    }
}

impl<'de> Deserialize<'de> for TokenStylesheet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Complete {
            metrics: MetricTokens,
            typography: LumaTypography,
            palette: PaletteStylesheet,
            elevation: ElevationStylesheet,
        }
        let authored = toml::Value::deserialize(deserializer)?;
        let mut tokens = toml::Value::try_from(Self::default()).map_err(serde::de::Error::custom)?;
        overlay(&mut tokens, authored);
        super::math::normalize(&mut tokens).map_err(serde::de::Error::custom)?;
        for group in ["palette", "elevation"] {
            super::colors::normalize(&mut tokens[group]).map_err(serde::de::Error::custom)?;
        }
        let tokens: Complete = tokens.try_into().map_err(serde::de::Error::custom)?;
        Ok(Self {
            metrics: tokens.metrics,
            typography: tokens.typography,
            palette: tokens.palette,
            elevation: tokens.elevation,
        })
    }
}

/// Tables inherit recursively; values and arrays replace their base, including
/// zero, false and empty arrays. Motion transitions and tagged source colors
/// replace complete values rather than inheriting their individual components.
pub(super) fn overlay(base: &mut toml::Value, authored: toml::Value) {
    if base.get("kind").is_some()
        || authored.get("kind").is_some()
        || base.get("space").is_some()
        || authored.get("space").is_some()
    {
        *base = authored;
        return;
    }
    match (base, authored) {
        (toml::Value::Table(base), toml::Value::Table(authored)) => {
            // A later layer's general geometry outranks earlier size/variant
            // values. Specific values authored in this layer still apply last.
            if let Some(toml::Value::Table(geometry)) = authored.get("geometry") {
                for axis in ["sizes", "variants"] {
                    if let Some(toml::Value::Table(variants)) = base.get_mut(axis) {
                        for (_, variant) in variants.iter_mut() {
                            if let toml::Value::Table(variant) = variant {
                                for key in geometry.keys() {
                                    variant.remove(key);
                                }
                            }
                        }
                    }
                }
            }
            for (key, value) in authored {
                if let Some(existing) = base.get_mut(&key) {
                    overlay(existing, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, authored) => *base = authored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{color::GamutMapping, theme::stylesheet::CommonStylesheet};

    #[test]
    fn native_defaults_produce_both_mode_snapshots() {
        let native = TokenStylesheet::default();
        assert_eq!(native.metrics.control.height, 36.0);
        assert_eq!(native.typography.text.label.weight, gpui::FontWeight::MEDIUM);
        assert_eq!(native.elevation.dark.menu.layers.len(), 2);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for mapping in [GamutMapping::Clip, GamutMapping::CssLocalMinde] {
                let source = native.source(mode);
                let actual = crate::theme::ThemeTokens::from_source(source.clone(), mapping).unwrap();
                assert_eq!(actual.palette.source(), &source.palette);
                assert_eq!(actual.elevation.source(), &source.elevation);
            }
        }
        let shell = &CommonStylesheet::sdk_default().shell.geometry;
        assert_eq!(crate::shell::TITLE_BAR_HEIGHT.as_f32(), shell.height.unwrap());
        let padding = if cfg!(target_os = "macos") {
            shell.macos_left_padding
        } else {
            shell.left_padding
        };
        assert_eq!(crate::shell::TITLE_BAR_LEFT_PADDING.as_f32(), padding.unwrap());
    }

    #[test]
    fn native_tokens_round_trip_through_the_external_format_adapter() {
        let native = TokenStylesheet::default();
        let mut edited = native.clone();
        edited.metrics.control.padding_x = 0.0;
        edited.palette.light.app.background = crate::color::ColorValue::display_p3(1.15, -0.1, 0.2, 0.37);
        edited.elevation.dark.menu.layers[0].color = crate::color::ColorValue::oklch(0.7, 0.4, 725.0, 0.25);
        for tokens in [native, edited] {
            let stored = toml::to_string(&tokens).unwrap();
            let restored: TokenStylesheet = toml::from_str(&stored).unwrap();
            assert_eq!(restored, tokens);
        }
    }

    #[test]
    fn authored_hsla_overrides_palette_and_shadow_colors() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.tokens.palette.light.app]\nforeground = 'hsla(120, 100%, 50%, 0.5)'\n[common.tokens.elevation.light.menu]\nlayers = [{ color = 'hsla(240, 100%, 50%, 0.25)', offset_x = 0, offset_y = 1, blur = 2, spread = 0, inset = false }]",
            "[common.tokens.palette.light.app]\nforeground = 'hsla(0, 100%, 50%, 0)'",
        ]).unwrap();
        assert_eq!(sheet.tokens.palette.light.app.foreground, crate::color::ColorValue::srgb(1.0, 0.0, 0.0, 0.0));
        assert_eq!(
            sheet.tokens.elevation.light.menu.layers[0].color,
            crate::color::ColorValue::srgb(0.0, 0.0, 1.0, 0.25)
        );
        assert_eq!(sheet.tokens.palette.dark, LumaPalette::dark());
    }

    #[test]
    fn sparse_token_tables_inherit_and_zero_survives() {
        let sheet = CommonStylesheet::parse("[common.tokens.metrics.control]\npadding_x = 0").unwrap();
        assert_eq!(sheet.tokens.metrics.control.padding_x, 0.0);
        assert_eq!(sheet.tokens.metrics.control.height, 36.0);
        assert_eq!(sheet.tokens.typography, LumaTypography::default());
        assert_eq!(sheet.tokens.palette.light, LumaPalette::light());
        assert_eq!(sheet.tokens.elevation.dark, LumaElevation::dark());
    }

    #[test]
    fn token_validation_rejects_unknown_fields_and_invalid_values() {
        for source in [
            "[common.tokens]\nunknown = 1",
            "[common.tokens.metrics.control]\nheight = -1",
            "[common.tokens.metrics.control]\nheight = nan",
            "[common.tokens.metrics.control]\nheight = 1e100",
            "[common.tokens.metrics.control]\nheigth = 20",
            "[common.tokens.typography.text.label]\nweight = 1001",
            "[common.tokens.typography.text.label]\nline_height = -1",
            "[common.tokens.palette.light.app]\nbackground = 'hsla(0, 0%, 0%, 2)'",
            "[common.tokens.palette.light.app]\nunknown = 1",
            "[common.tokens.palette.light.app.background]\nspace = 'display-p3'",
            "[common.tokens.palette.light.app.background]\ncomponents = [1, 0, 0, 1]",
            "[common.tokens.palette.light.app.background]",
            "[common.tokens.elevation.dark.menu]\nlayers = [{ color = 'hsla(0, 0%, 0%, 1)', offset_x = 0, offset_y = 0, blur = -1, spread = 0, inset = false }]",
        ] {
            assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
        }
    }

    #[test]
    fn layer_order_outranks_specificity_in_an_earlier_layer() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.progress.sizes.1]\ntrack_height = 9",
            "[common.progress.geometry]\ntrack_height = 0",
        ])
        .unwrap();
        let geometry = sheet.progress.resolve_geometry_for("1", Default::default());
        assert_eq!(geometry.track_height, 0.0);
        assert!(matches!(sheet.progress.inspect_geometry_for("1", Default::default()).track_height.source,
            crate::theme::provenance::MetricSource::Authored { key }
                if key == "common.progress.geometry.track_height"));
        assert_eq!(geometry.size, 64.0);
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.progress.geometry]\ntrack_height = 0\n[common.progress.sizes.1]\ntrack_height = 7",
        ])
        .unwrap();
        assert_eq!(sheet.progress.resolve_geometry_for("1", Default::default()).track_height, 7.0);
    }

    #[test]
    fn layers_keep_motion_atomic_and_empty_shadow_arrays_explicit() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.motion.transition]\nkind = 'instant'\n[common.tokens.elevation.light.menu]\nlayers = []",
        ])
        .unwrap();
        assert!(sheet.tokens.elevation.light.menu.layers.is_empty());
        assert!(CommonStylesheet::resolve_layers(&["[common.motion.transition]\nkind = 'timed'",]).is_err());
        // Generic layer merging must not treat false as an absent value.
        let mut value: toml::Value = toml::from_str("enabled = true").unwrap();
        overlay(&mut value, toml::from_str("enabled = false").unwrap());
        assert_eq!(value["enabled"].as_bool(), Some(false));
    }
}
