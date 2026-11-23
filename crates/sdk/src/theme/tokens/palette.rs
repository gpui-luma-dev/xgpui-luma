use gpui::Hsla;
use crate::color::{ColorValue, GamutMapping, gpui_bridge::SrgbRenderCache};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

/// Semantic source colors; the default component type retains Palette color spaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LumaPalette<C = ColorValue> {
    pub app: AppPalette<C>,
    pub shell: ShellPalette<C>,
    pub surface: SurfacePalette<C>,
    pub state: StatePalette<C>,
    pub form: FormPalette<C>,
    pub focus: FocusPalette<C>,
    pub border: BorderPalette<C>,
    pub navigation: NavigationPalette<C>,
    pub data: DataPalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub border: C,
    pub hover_background: C,
    pub active_background: C,
    pub close_hover_background: C,
    pub close_active_background: C,
    pub close_foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub muted_foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfacePalette<C = ColorValue> {
    pub panel: SurfaceWithBorderPalette<C>,
    pub floating: SurfaceWithBorderPalette<C>,
    pub subtle: SurfaceTonePalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceWithBorderPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub border: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceTonePalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatePalette<C = ColorValue> {
    pub hover: StateTonePalette<C>,
    pub pressed: StateBackgroundPalette<C>,
    pub selected: StateTonePalette<C>,
    pub disabled: StateTonePalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateTonePalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateBackgroundPalette<C = ColorValue> {
    pub background: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormPalette<C = ColorValue> {
    pub input: FormInputPalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormInputPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub border: C,
    pub invalid_border: C,
    pub placeholder: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FocusPalette<C = ColorValue> {
    pub ring: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BorderPalette<C = ColorValue> {
    pub default: C,
    pub strong: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub muted_foreground: C,
    pub hover_background: C,
    pub selected_background: C,
    pub selected_foreground: C,
    pub border: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataPalette<C = ColorValue> {
    /// Positive completion/status affordances.
    pub positive: C,
    pub accent_1: C,
    pub accent_2: C,
    pub accent_3: C,
    pub accent_4: C,
    pub accent_5: C,
}

impl Default for LumaPalette<ColorValue> {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaPalette<ColorValue> {
    pub fn light() -> Self {
        Self::light_with(|c| ColorValue::srgb(c[0], c[1], c[2], c[3]))
    }
    pub fn dark() -> Self {
        Self::dark_with(|c| ColorValue::srgb(c[0], c[1], c[2], c[3]))
    }

    /// Validate and retain the source palette, with an immutable sRGB preview.
    pub fn snapshot_srgb(self, policy: GamutMapping) -> Result<SrgbPalette> {
        let mut cache = SrgbRenderCache::default();
        let preview = self.clone().map_colors(&mut |source| cache.resolve_hsla(source, policy))?;
        Ok(SrgbPalette { source: Arc::new(self), preview: Arc::new(preview), policy })
    }
}

impl<C> LumaPalette<C> {
    fn light_with(color: impl Fn([f32; 4]) -> C) -> Self {
        // The focus ring is red in both modes, so focus is easy to see while states are worked on.
        let focus_ring = color([1.0, 0.0, 0.0, 1.0]);
        // Bundled colors have zero saturation. Keep the original percentage
        // arithmetic so source components retain their exact f32 values.
        let color = |lightness: f32, alpha| {
            let component = lightness / 100.0;
            color([component, component, component, alpha])
        };
        Self {
            app: AppPalette {
                background: color(98.04, 1.0),
                foreground: color(11.18, 1.0),
                muted_foreground: color(46.86, 1.0),
            },
            shell: ShellPalette {
                background: color(11.0, 1.0),
                foreground: color(100.0, 1.0),
                border: color(100.0, 0.1),
                hover_background: color(100.0, 0.12),
                active_background: color(100.0, 0.2),
                close_hover_background: color(30.0, 1.0),
                close_active_background: color(40.0, 1.0),
                close_foreground: color(100.0, 1.0),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: color(100.0, 1.0),
                    foreground: color(11.18, 1.0),
                    border: color(83.92, 1.0),
                },
                floating: SurfaceWithBorderPalette {
                    background: color(100.0, 1.0),
                    foreground: color(11.18, 1.0),
                    border: color(83.92, 1.0),
                },
                subtle: SurfaceTonePalette { background: color(96.08, 1.0), foreground: color(26.67, 1.0) },
            },
            state: StatePalette {
                hover: StateTonePalette { background: color(91.37, 1.0), foreground: color(11.18, 1.0) },
                pressed: StateBackgroundPalette { background: color(83.92, 1.0) },
                selected: StateTonePalette { background: color(12.0, 1.0), foreground: color(100.0, 1.0) },
                disabled: StateTonePalette { background: color(96.08, 1.0), foreground: color(65.1, 1.0) },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: color(100.0, 1.0),
                    foreground: color(11.18, 1.0),
                    border: color(65.1, 1.0),
                    invalid_border: color(20.0, 1.0),
                    placeholder: color(65.1, 1.0),
                },
            },
            focus: FocusPalette { ring: focus_ring },
            border: BorderPalette { default: color(83.92, 1.0), strong: color(46.86, 1.0) },
            navigation: NavigationPalette {
                background: color(100.0, 1.0),
                foreground: color(11.18, 1.0),
                muted_foreground: color(46.86, 1.0),
                hover_background: color(96.08, 1.0),
                selected_background: color(12.0, 1.0),
                selected_foreground: color(100.0, 1.0),
                border: color(83.92, 1.0),
            },
            data: DataPalette {
                positive: color(30.0, 1.0),
                accent_1: color(20.0, 1.0),
                accent_2: color(35.0, 1.0),
                accent_3: color(45.0, 1.0),
                accent_4: color(55.0, 1.0),
                accent_5: color(65.0, 1.0),
            },
        }
    }

    fn dark_with(color: impl Fn([f32; 4]) -> C) -> Self {
        let focus_ring = color([1.0, 0.0, 0.0, 1.0]);
        let color = |lightness: f32, alpha| {
            let component = lightness / 100.0;
            color([component, component, component, alpha])
        };
        Self {
            app: AppPalette {
                background: color(8.43, 1.0),
                foreground: color(98.04, 1.0),
                muted_foreground: color(65.1, 1.0),
            },
            shell: ShellPalette {
                background: color(11.0, 1.0),
                foreground: color(100.0, 1.0),
                border: color(100.0, 0.1),
                hover_background: color(100.0, 0.12),
                active_background: color(100.0, 0.2),
                close_hover_background: color(30.0, 1.0),
                close_active_background: color(40.0, 1.0),
                close_foreground: color(100.0, 1.0),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: color(10.98, 1.0),
                    foreground: color(98.04, 1.0),
                    border: color(26.67, 1.0),
                },
                floating: SurfaceWithBorderPalette {
                    background: color(10.98, 1.0),
                    foreground: color(98.04, 1.0),
                    border: color(26.67, 1.0),
                },
                subtle: SurfaceTonePalette { background: color(17.45, 1.0), foreground: color(83.92, 1.0) },
            },
            state: StatePalette {
                hover: StateTonePalette { background: color(26.67, 1.0), foreground: color(98.04, 1.0) },
                pressed: StateBackgroundPalette { background: color(34.51, 1.0) },
                selected: StateTonePalette { background: color(90.0, 1.0), foreground: color(10.0, 1.0) },
                disabled: StateTonePalette { background: color(17.45, 1.0), foreground: color(46.86, 1.0) },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: color(11.18, 1.0),
                    foreground: color(98.04, 1.0),
                    border: color(34.51, 1.0),
                    invalid_border: color(85.0, 1.0),
                    placeholder: color(46.86, 1.0),
                },
            },
            focus: FocusPalette { ring: focus_ring },
            border: BorderPalette { default: color(26.67, 1.0), strong: color(65.1, 1.0) },
            navigation: NavigationPalette {
                background: color(10.98, 1.0),
                foreground: color(98.04, 1.0),
                muted_foreground: color(65.1, 1.0),
                hover_background: color(17.45, 1.0),
                selected_background: color(90.0, 1.0),
                selected_foreground: color(10.0, 1.0),
                border: color(26.67, 1.0),
            },
            data: DataPalette {
                positive: color(75.0, 1.0),
                accent_1: color(90.0, 1.0),
                accent_2: color(75.0, 1.0),
                accent_3: color(65.0, 1.0),
                accent_4: color(55.0, 1.0),
                accent_5: color(45.0, 1.0),
            },
        }
    }
}

impl<C> LumaPalette<C> {
    fn map_colors<D>(self, map: &mut impl FnMut(C) -> Result<D>) -> Result<LumaPalette<D>> {
        Ok(LumaPalette {
            shell: ShellPalette {
                background: map(self.shell.background).with_context(|| "palette.shell.background")?,
                foreground: map(self.shell.foreground).with_context(|| "palette.shell.foreground")?,
                border: map(self.shell.border).with_context(|| "palette.shell.border")?,
                hover_background: map(self.shell.hover_background).with_context(|| "palette.shell.hover_background")?,
                active_background: map(self.shell.active_background)
                    .with_context(|| "palette.shell.active_background")?,
                close_hover_background: map(self.shell.close_hover_background)
                    .with_context(|| "palette.shell.close_hover_background")?,
                close_active_background: map(self.shell.close_active_background)
                    .with_context(|| "palette.shell.close_active_background")?,
                close_foreground: map(self.shell.close_foreground).with_context(|| "palette.shell.close_foreground")?,
            },
            app: AppPalette {
                background: map(self.app.background).with_context(|| "palette.app.background")?,
                foreground: map(self.app.foreground).with_context(|| "palette.app.foreground")?,
                muted_foreground: map(self.app.muted_foreground).with_context(|| "palette.app.muted_foreground")?,
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: map(self.surface.panel.background)
                        .with_context(|| "palette.surface.panel.background")?,
                    foreground: map(self.surface.panel.foreground)
                        .with_context(|| "palette.surface.panel.foreground")?,
                    border: map(self.surface.panel.border).with_context(|| "palette.surface.panel.border")?,
                },
                floating: SurfaceWithBorderPalette {
                    background: map(self.surface.floating.background)
                        .with_context(|| "palette.surface.floating.background")?,
                    foreground: map(self.surface.floating.foreground)
                        .with_context(|| "palette.surface.floating.foreground")?,
                    border: map(self.surface.floating.border).with_context(|| "palette.surface.floating.border")?,
                },
                subtle: SurfaceTonePalette {
                    background: map(self.surface.subtle.background)
                        .with_context(|| "palette.surface.subtle.background")?,
                    foreground: map(self.surface.subtle.foreground)
                        .with_context(|| "palette.surface.subtle.foreground")?,
                },
            },
            state: StatePalette {
                hover: StateTonePalette {
                    background: map(self.state.hover.background).with_context(|| "palette.state.hover.background")?,
                    foreground: map(self.state.hover.foreground).with_context(|| "palette.state.hover.foreground")?,
                },
                pressed: StateBackgroundPalette {
                    background: map(self.state.pressed.background)
                        .with_context(|| "palette.state.pressed.background")?,
                },
                selected: StateTonePalette {
                    background: map(self.state.selected.background)
                        .with_context(|| "palette.state.selected.background")?,
                    foreground: map(self.state.selected.foreground)
                        .with_context(|| "palette.state.selected.foreground")?,
                },
                disabled: StateTonePalette {
                    background: map(self.state.disabled.background)
                        .with_context(|| "palette.state.disabled.background")?,
                    foreground: map(self.state.disabled.foreground)
                        .with_context(|| "palette.state.disabled.foreground")?,
                },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: map(self.form.input.background).with_context(|| "palette.form.input.background")?,
                    foreground: map(self.form.input.foreground).with_context(|| "palette.form.input.foreground")?,
                    border: map(self.form.input.border).with_context(|| "palette.form.input.border")?,
                    invalid_border: map(self.form.input.invalid_border)
                        .with_context(|| "palette.form.input.invalid_border")?,
                    placeholder: map(self.form.input.placeholder).with_context(|| "palette.form.input.placeholder")?,
                },
            },
            focus: FocusPalette { ring: map(self.focus.ring).with_context(|| "palette.focus.ring")? },
            border: BorderPalette {
                default: map(self.border.default).with_context(|| "palette.border.default")?,
                strong: map(self.border.strong).with_context(|| "palette.border.strong")?,
            },
            navigation: NavigationPalette {
                background: map(self.navigation.background).with_context(|| "palette.navigation.background")?,
                foreground: map(self.navigation.foreground).with_context(|| "palette.navigation.foreground")?,
                muted_foreground: map(self.navigation.muted_foreground)
                    .with_context(|| "palette.navigation.muted_foreground")?,
                hover_background: map(self.navigation.hover_background)
                    .with_context(|| "palette.navigation.hover_background")?,
                selected_background: map(self.navigation.selected_background)
                    .with_context(|| "palette.navigation.selected_background")?,
                selected_foreground: map(self.navigation.selected_foreground)
                    .with_context(|| "palette.navigation.selected_foreground")?,
                border: map(self.navigation.border).with_context(|| "palette.navigation.border")?,
            },
            data: DataPalette {
                positive: map(self.data.positive).with_context(|| "palette.data.positive")?,
                accent_1: map(self.data.accent_1).with_context(|| "palette.data.accent_1")?,
                accent_2: map(self.data.accent_2).with_context(|| "palette.data.accent_2")?,
                accent_3: map(self.data.accent_3).with_context(|| "palette.data.accent_3")?,
                accent_4: map(self.data.accent_4).with_context(|| "palette.data.accent_4")?,
                accent_5: map(self.data.accent_5).with_context(|| "palette.data.accent_5")?,
            },
        })
    }
}

/// Immutable source palette and precomputed projection for the current sRGB backend.
/// Field access reads the preview; `source()` returns retained ColorValue fields.
/// Rebuild from a modified source to change colors or mapping policy.
#[derive(Clone, Debug)]
pub struct SrgbPalette {
    source: Arc<LumaPalette>,
    preview: Arc<LumaPalette<Hsla>>,
    policy: GamutMapping,
}

impl SrgbPalette {
    pub fn source(&self) -> &LumaPalette {
        &self.source
    }
    pub fn mapping(&self) -> GamutMapping {
        self.policy
    }

    pub(crate) fn light() -> Self {
        static PALETTE: OnceLock<SrgbPalette> = OnceLock::new();
        PALETTE
            .get_or_init(|| Self {
                source: Arc::new(LumaPalette::light()),
                preview: Arc::new(LumaPalette::light_with(|c| {
                    gpui::Rgba { r: c[0], g: c[1], b: c[2], a: c[3] }.into()
                })),
                policy: GamutMapping::CssLocalMinde,
            })
            .clone()
    }
    pub(crate) fn dark() -> Self {
        static PALETTE: OnceLock<SrgbPalette> = OnceLock::new();
        PALETTE
            .get_or_init(|| Self {
                source: Arc::new(LumaPalette::dark()),
                preview: Arc::new(LumaPalette::dark_with(|c| gpui::Rgba { r: c[0], g: c[1], b: c[2], a: c[3] }.into())),
                policy: GamutMapping::CssLocalMinde,
            })
            .clone()
    }
}

impl Deref for SrgbPalette {
    type Target = LumaPalette<Hsla>;
    fn deref(&self) -> &Self::Target {
        &self.preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_palette_persistence_preserves_wide_gamut_and_extended_components() {
        let mut source = LumaPalette::light();
        source.app.background = ColorValue::display_p3(1.0, 0.0, 0.0, 0.37);
        source.state.selected.background = ColorValue::oklch(0.7, 0.4, 725.0, 0.6);
        source.data.accent_5 = ColorValue::linear_srgb(-0.25, 2.0, 0.5, 1.0);
        let stored = toml::to_string(&source).unwrap();
        let restored: LumaPalette = toml::from_str(&stored).unwrap();
        assert_eq!(restored, source);
        let mapped = restored.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(mapped.source(), &source);
        assert_eq!(mapped.app.background.a, 0.37);
        assert!(!mapped.source().app.background.is_in_gamut(crate::color::Gamut::Srgb).unwrap());
    }

    #[test]
    fn rebuilding_source_or_policy_keeps_old_snapshots_independent() {
        let mut source = LumaPalette::light();
        source.app.background = ColorValue::display_p3(1.0, 0.0, 0.0, 0.5);
        let clipped = source.clone().snapshot_srgb(GamutMapping::Clip).unwrap();
        let mapped = source.clone().snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(clipped.source(), mapped.source());
        assert_ne!(clipped.app.background, mapped.app.background);
        source.app.background = ColorValue::srgb(0.0, 0.0, 0.0, 1.0);
        let changed = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_ne!(changed.source(), mapped.source());
        assert_eq!(mapped.source().app.background, ColorValue::display_p3(1.0, 0.0, 0.0, 0.5));
    }

    #[test]
    fn snapshot_errors_identify_the_invalid_source_field() {
        let mut source = LumaPalette::dark();
        source.navigation.selected_foreground = ColorValue::srgb(0.0, 0.0, 0.0, f32::NAN);
        let error = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap_err();
        assert!(error.to_string().contains("palette.navigation.selected_foreground"));
    }

    #[test]
    fn structural_previews_match_the_bridge_for_every_field() {
        for snapshot in [SrgbPalette::light(), SrgbPalette::dark()] {
            let projected = snapshot.source().clone().snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
            assert_eq!(*snapshot, *projected);
        }
    }
}
