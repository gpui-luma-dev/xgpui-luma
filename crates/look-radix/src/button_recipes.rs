//! Look-owned state recipes use the SDK's semantic/interaction types.
use std::sync::OnceLock;
use gpui::Hsla;
use xgpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};
use serde::{Deserialize, Serialize};
use crate::{
    button::{ButtonVariant, Paint, ClassicButtonParams},
    look::Look,
    semantic::SemanticRole,
    scale::ScaleFamily,
};
use xgpui_luma::controls::button_family::ButtonFamilyRole;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "String")]
enum Color {
    Tone(u8),
    Gray(u8),
    Contrast,
    Transparent,
    None,
    Background,
    Semantic(SemanticRole),
}
impl Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            Self::Tone(step) => format!("tone.{step}"),
            Self::Gray(step) => format!("gray.{step}"),
            Self::Contrast => "contrast".into(),
            Self::Transparent => "transparent".into(),
            Self::None => "none".into(),
            Self::Background => "background".into(),
            Self::Semantic(role) => match role {
                SemanticRole::Background => "role.background",
                SemanticRole::Surface => "role.surface",
                SemanticRole::Foreground => "role.foreground",
                SemanticRole::PrimaryForeground => "role.primary_foreground",
                SemanticRole::MutedForeground => "role.muted_foreground",
                SemanticRole::Border => "role.border",
                SemanticRole::Focus => "role.focus",
                _ => return Err(serde::ser::Error::custom("unsupported recipe semantic role")),
            }
            .into(),
        };
        serializer.serialize_str(&value)
    }
}
impl TryFrom<String> for Color {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        for (prefix, gray) in [("tone.", false), ("gray.", true)] {
            if let Some(step) = value.strip_prefix(prefix) {
                let step: u8 = step.parse().map_err(|_| "invalid scale step")?;
                if !(1..=12).contains(&step) {
                    return Err("scale step must be 1..=12".into());
                }
                return Ok(if gray { Self::Gray(step) } else { Self::Tone(step) });
            }
        }
        Ok(match value.as_str() {
            "contrast" => Self::Contrast,
            "transparent" => Self::Transparent,
            "none" => Self::None,
            "background" => Self::Background,
            "role.background" => Self::Semantic(SemanticRole::Background),
            "role.surface" => Self::Semantic(SemanticRole::Surface),
            "role.foreground" => Self::Semantic(SemanticRole::Foreground),
            "role.primary_foreground" => Self::Semantic(SemanticRole::PrimaryForeground),
            "role.muted_foreground" => Self::Semantic(SemanticRole::MutedForeground),
            "role.border" => Self::Semantic(SemanticRole::Border),
            "role.focus" => Self::Semantic(SemanticRole::Focus),
            _ => return Err(format!("unknown recipe color {value}")),
        })
    }
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    #[serde(default)]
    variants: Vec<String>,
    selected: Option<bool>,
    high_contrast: Option<bool>,
    layer: Option<String>,
    focused: Option<bool>,
    enabled: Option<bool>,
    #[serde(default)]
    classic_edge: bool,
    mode: Option<String>,
    background: Option<Color>,
    foreground: Option<Color>,
    border: Option<Color>,
    #[serde(default)]
    background_delta: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipes {
    pub classic: ClassicButtonParams,
    rules: Vec<Rule>,
}
pub(crate) fn embedded() -> &'static Recipes {
    static RECIPES: OnceLock<Recipes> = OnceLock::new();
    RECIPES.get_or_init(Recipes::native_default)
}

impl Recipes {
    /// Native bundled button definitions, retaining the ordered overrides.
    fn native_default() -> Self {
        let mut classic = ClassicButtonParams::zero();
        classic.pressed_rim_lightness_scale = 0.5;
        classic.pressed_rim_alpha_scale = 0.6;
        classic.lightness_min = 0.02;
        classic.lightness_max = 0.98;
        classic.radius = 4.0;
        classic.fill_top_delta = 0.0;
        classic.fill_bottom_delta = 0.067;
        classic.border_delta = -0.022;
        classic.bevel_darken = 0.045;
        classic.bevel_offset = 1.0;
        classic.bevel_alpha = 1.0;
        classic.rim_lighten = 0.3;
        classic.rim_offset = 1.0;
        classic.rim_alpha = 0.95;
        classic.fade_offset = 4.0;
        classic.fade_blur = 4.0;
        classic.fade_spread = -2.0;
        classic.fade_alpha = 0.35;
        classic.drop_offset = 1.0;
        classic.drop_blur = 2.0;
        classic.drop_alpha = 0.08;
        Self {
            classic,
            rules: vec![
                Rule {
                    background: Some(Color::Tone(9)),
                    foreground: Some(Color::Contrast),
                    border: Some(Color::Tone(9)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["soft".into()],
                    selected: Some(false),
                    background: Some(Color::Tone(3)),
                    foreground: Some(Color::Tone(11)),
                    border: Some(Color::Tone(3)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["surface".into()],
                    selected: Some(false),
                    background: Some(Color::Tone(2)),
                    foreground: Some(Color::Tone(11)),
                    border: Some(Color::Tone(7)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["outline".into()],
                    selected: Some(false),
                    background: Some(Color::Transparent),
                    foreground: Some(Color::Semantic(SemanticRole::Foreground)),
                    border: Some(Color::Semantic(SemanticRole::Border)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["ghost".into(), "ghost_quiet".into()],
                    selected: Some(false),
                    background: Some(Color::Transparent),
                    foreground: Some(Color::Semantic(SemanticRole::Foreground)),
                    border: Some(Color::None),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["classic".into(), "solid".into()],
                    selected: Some(false),
                    high_contrast: Some(true),
                    background: Some(Color::Tone(12)),
                    foreground: Some(Color::Gray(1)),
                    border: Some(Color::Tone(12)),
                    ..Default::default()
                },
                Rule {
                    variants: vec![
                        "soft".into(),
                        "surface".into(),
                        "outline".into(),
                        "ghost".into(),
                        "ghost_quiet".into(),
                    ],
                    selected: Some(false),
                    high_contrast: Some(true),
                    foreground: Some(Color::Tone(12)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["outline".into()],
                    selected: Some(false),
                    high_contrast: Some(true),
                    border: Some(Color::Tone(11)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["solid".into(), "classic".into()],
                    layer: Some("hovered".into()),
                    high_contrast: Some(false),
                    background: Some(Color::Tone(10)),
                    ..Default::default()
                },
                Rule {
                    selected: Some(true),
                    layer: Some("hovered".into()),
                    high_contrast: Some(false),
                    background: Some(Color::Tone(10)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["solid".into(), "classic".into()],
                    layer: Some("hovered".into()),
                    high_contrast: Some(true),
                    background: Some(Color::Tone(12)),
                    background_delta: 0.04,
                    ..Default::default()
                },
                Rule {
                    selected: Some(true),
                    layer: Some("hovered".into()),
                    high_contrast: Some(true),
                    background: Some(Color::Tone(12)),
                    background_delta: 0.04,
                    ..Default::default()
                },
                Rule {
                    variants: vec![
                        "soft".into(),
                        "surface".into(),
                        "outline".into(),
                        "ghost".into(),
                        "ghost_quiet".into(),
                    ],
                    selected: Some(false),
                    layer: Some("hovered".into()),
                    background: Some(Color::Tone(3)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["solid".into(), "classic".into()],
                    layer: Some("pressed".into()),
                    high_contrast: Some(false),
                    background: Some(Color::Tone(10)),
                    ..Default::default()
                },
                Rule {
                    selected: Some(true),
                    layer: Some("pressed".into()),
                    high_contrast: Some(false),
                    background: Some(Color::Tone(10)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["solid".into(), "classic".into()],
                    layer: Some("pressed".into()),
                    high_contrast: Some(true),
                    background: Some(Color::Tone(12)),
                    background_delta: -0.03,
                    ..Default::default()
                },
                Rule {
                    selected: Some(true),
                    layer: Some("pressed".into()),
                    high_contrast: Some(true),
                    background: Some(Color::Tone(12)),
                    background_delta: -0.03,
                    ..Default::default()
                },
                Rule {
                    variants: vec![
                        "soft".into(),
                        "surface".into(),
                        "outline".into(),
                        "ghost".into(),
                        "ghost_quiet".into(),
                    ],
                    selected: Some(false),
                    layer: Some("pressed".into()),
                    background: Some(Color::Tone(4)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["surface".into()],
                    selected: Some(false),
                    layer: Some("hovered".into()),
                    border: Some(Color::Tone(8)),
                    ..Default::default()
                },
                Rule {
                    layer: Some("disabled".into()),
                    background: Some(Color::Semantic(SemanticRole::Surface)),
                    foreground: Some(Color::Semantic(SemanticRole::MutedForeground)),
                    border: Some(Color::Semantic(SemanticRole::Border)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["classic".into()],
                    enabled: Some(true),
                    border: Some(Color::Background),
                    classic_edge: true,
                    ..Default::default()
                },
                Rule { focused: Some(true), border: Some(Color::Semantic(SemanticRole::Focus)), ..Default::default() },
                Rule {
                    variants: vec!["page".into()],
                    background: Some(Color::Transparent),
                    foreground: Some(Color::Semantic(SemanticRole::MutedForeground)),
                    border: Some(Color::None),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["page".into()],
                    selected: Some(true),
                    mode: Some("light".into()),
                    background: Some(Color::Semantic(SemanticRole::Foreground)),
                    foreground: Some(Color::Semantic(SemanticRole::Background)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["page".into()],
                    selected: Some(true),
                    mode: Some("dark".into()),
                    background: Some(Color::Semantic(SemanticRole::PrimaryForeground)),
                    foreground: Some(Color::Semantic(SemanticRole::Background)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["page".into()],
                    layer: Some("disabled".into()),
                    foreground: Some(Color::Semantic(SemanticRole::MutedForeground)),
                    ..Default::default()
                },
                Rule {
                    variants: vec!["ghost_quiet".into()],
                    background: Some(Color::Transparent),
                    border: Some(Color::None),
                    ..Default::default()
                },
            ],
        }
    }
}
pub(crate) fn parse(source: &str) -> anyhow::Result<Recipes> {
    #[derive(Deserialize)]
    struct Document {
        button_recipes: Recipes,
    }
    let config: Document = toml::from_str(source)?;
    let params = config.button_recipes.classic;
    anyhow::ensure!(
        params.lightness_min.is_finite()
            && params.lightness_max.is_finite()
            && 0.0 <= params.lightness_min
            && params.lightness_min <= params.lightness_max
            && params.lightness_max <= 1.0,
        "invalid Classic lightness bounds"
    );
    anyhow::ensure!(
        [params.pressed_rim_alpha_scale, params.pressed_rim_lightness_scale]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
        "invalid pressed rim scale"
    );
    anyhow::ensure!(
        [
            params.radius,
            params.fill_top_delta,
            params.fill_bottom_delta,
            params.border_delta,
            params.bevel_darken,
            params.bevel_offset,
            params.bevel_alpha,
            params.rim_lighten,
            params.rim_offset,
            params.rim_alpha,
            params.fade_offset,
            params.fade_blur,
            params.fade_spread,
            params.fade_alpha,
            params.drop_offset,
            params.drop_blur,
            params.drop_alpha
        ]
        .iter()
        .all(|v| v.is_finite()),
        "nonfinite Classic parameter"
    );
    anyhow::ensure!(
        [params.radius, params.fade_blur, params.drop_blur].iter().all(|v| *v >= 0.0),
        "negative Classic extent"
    );
    anyhow::ensure!(
        [params.bevel_alpha, params.rim_alpha, params.fade_alpha, params.drop_alpha]
            .iter()
            .all(|v| (0.0..=1.0).contains(v)),
        "Classic alpha must be 0..=1"
    );
    for rule in &config.button_recipes.rules {
        anyhow::ensure!(
            rule.variants.iter().all(|variant| VARIANTS.contains(&variant.as_str())),
            "unknown button variant"
        );
        anyhow::ensure!(
            rule.layer
                .as_ref()
                .is_none_or(|v| ["default", "hovered", "pressed", "disabled"].contains(&v.as_str())),
            "unknown interaction layer"
        );
        anyhow::ensure!(rule.mode.as_ref().is_none_or(|v| ["light", "dark"].contains(&v.as_str())), "unknown mode");
        anyhow::ensure!(rule.background_delta.is_finite(), "nonfinite lightness delta");
    }
    Ok(config.button_recipes)
}
const VARIANTS: &[&str] = &["classic", "solid", "soft", "surface", "outline", "ghost", "ghost_quiet", "page"];
fn variant_name(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Classic => "classic",
        ButtonVariant::Solid => "solid",
        ButtonVariant::Soft => "soft",
        ButtonVariant::Surface => "surface",
        ButtonVariant::Outline => "outline",
        ButtonVariant::Ghost => "ghost",
        ButtonVariant::GhostQuiet => "ghost_quiet",
        ButtonVariant::Page => "page",
    }
}
pub(crate) struct Colors {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Option<Hsla>,
}
pub(crate) fn resolve(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> Colors {
    let recipes = look.button_recipes();
    let layer = match state.layer() {
        InteractionLayer::Default => "default",
        InteractionLayer::Hovered => "hovered",
        InteractionLayer::Pressed => "pressed",
        InteractionLayer::Disabled => "disabled",
    };
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    let transparent = gpui::transparent_black();
    let mut colors = Colors { background: transparent, foreground: transparent, border: None };
    for rule in &recipes.rules {
        if (!rule.variants.is_empty() && !rule.variants.iter().any(|v| v == variant_name(variant)))
            || rule.selected.is_some_and(|v| v != selected)
            || rule.high_contrast.is_some_and(|v| v != paint.high_contrast)
            || rule.focused.is_some_and(|v| v != (state.focused && !state.disabled))
            || rule.enabled.is_some_and(|v| v == state.disabled)
            || rule.layer.as_ref().is_some_and(|v| v != layer)
            || rule.mode.as_ref().is_some_and(|v| {
                v != if look.mode() == ThemeMode::Light {
                    "light"
                } else {
                    "dark"
                }
            })
        {
            continue;
        }
        let resolve = |color: &Color, background: Hsla| -> Option<Hsla> {
            Some(match color {
                Color::Tone(step) => paint.tone.step(look, *step),
                Color::Gray(step) => look.resolve_step(ScaleFamily::Gray, *step).hsla(),
                Color::Contrast => paint.tone.contrast(look),
                Color::Transparent => transparent,
                Color::None => return None,
                Color::Background => background,
                Color::Semantic(role) => look.resolve_role(*role).hsla(),
            })
        };
        if let Some(color) = &rule.background {
            colors.background = resolve(color, colors.background).unwrap_or(transparent);
            if rule.background_delta != 0.0 {
                colors.background.l = (colors.background.l + rule.background_delta)
                    .clamp(recipes.classic.lightness_min, recipes.classic.lightness_max);
            }
        }
        if let Some(color) = &rule.foreground {
            colors.foreground = resolve(color, colors.background).unwrap_or(transparent);
        }
        if let Some(color) = &rule.border {
            colors.border = resolve(color, colors.background);
            if rule.classic_edge {
                let params = look.classic_params();
                colors.border = colors
                    .border
                    .map(|color| super::button::shift_lightness_with_params(color, params.border_delta, params));
            }
        }
    }

    colors
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::controls::button_family::ButtonFamilyPalette;
    use xgpui_luma::theme::LumaTextStyle;
    use gpui::{SharedString, FontWeight};
    use crate::button::shift_lightness;
    use crate::tone::Tone;
    fn transparent() -> Hsla {
        gpui::transparent_black()
    }
    fn legacy_palette(
        look: &Look,
        variant: ButtonVariant,
        paint: Paint,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> ButtonFamilyPalette {
        let layer = state.layer();
        let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
        // Quiet ghosts share Ghost foregrounds; their face stays transparent in every state.
        let quiet = matches!(variant, ButtonVariant::GhostQuiet);
        let page = matches!(variant, ButtonVariant::Page);
        let variant = if quiet { ButtonVariant::Ghost } else { variant };
        let high_contrast = paint.high_contrast;
        let tone = paint.tone;
        let step = |n| tone.step(look, n);

        // Page / selected chrome stay on semantic roles (app chrome, not Radix color prop).
        // Border stays None in every state: the SDK paints a focus ring from `look.border`,
        // so any opaque edge on the selected pill becomes a ring when focused.
        let (mut background, mut foreground, mut border) = if page {
            if selected {
                match look.mode() {
                    xgpui_luma::theme::ThemeMode::Light => (
                        look.resolve_role(SemanticRole::Foreground).hsla(),
                        look.resolve_role(SemanticRole::Background).hsla(),
                        None,
                    ),
                    xgpui_luma::theme::ThemeMode::Dark => (
                        look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
                        look.resolve_role(SemanticRole::Background).hsla(),
                        None,
                    ),
                }
            } else {
                (
                    look.resolve_role(SemanticRole::Background).hsla(),
                    look.resolve_role(SemanticRole::MutedForeground).hsla(),
                    None,
                )
            }
        } else if selected {
            (step(9), tone.contrast(look), Some(step(9)))
        } else {
            match variant {
                ButtonVariant::Classic | ButtonVariant::Solid => {
                    if high_contrast {
                        // Radix: accent-12 on gray-1.
                        (step(12), solid_high_contrast_label(look), Some(step(12)))
                    } else {
                        (step(9), tone.contrast(look), Some(step(9)))
                    }
                }
                ButtonVariant::Soft => {
                    let fg = if high_contrast { step(12) } else { step(11) };
                    (step(3), fg, Some(step(3)))
                }
                ButtonVariant::Surface => {
                    let fg = if high_contrast { step(12) } else { step(11) };
                    (step(2), fg, Some(step(7)))
                }
                ButtonVariant::Outline => {
                    let fg = if high_contrast {
                        step(12)
                    } else {
                        look.resolve_role(SemanticRole::Foreground).hsla()
                    };
                    let edge = if high_contrast {
                        step(11)
                    } else {
                        look.resolve_role(SemanticRole::Border).hsla()
                    };
                    (transparent(), fg, Some(edge))
                }
                ButtonVariant::Ghost | ButtonVariant::GhostQuiet => {
                    let fg = if high_contrast {
                        step(12)
                    } else {
                        look.resolve_role(SemanticRole::Foreground).hsla()
                    };
                    (transparent(), fg, None)
                }
                ButtonVariant::Page => unreachable!(),
            }
        };

        if matches!(variant, ButtonVariant::Ghost | ButtonVariant::Outline | ButtonVariant::Page) && !selected {
            background = transparent();
            // Page and Ghost are borderless chrome; Outline keeps its edge.
            if matches!(variant, ButtonVariant::Ghost | ButtonVariant::Page) {
                border = None;
            }
        }

        match layer {
            InteractionLayer::Disabled if !page => {
                background = look.resolve_role(SemanticRole::Surface).hsla();
                foreground = look.resolve_role(SemanticRole::MutedForeground).hsla();
                border = Some(look.resolve_role(SemanticRole::Border).hsla());
            }
            InteractionLayer::Disabled => {
                foreground = look.resolve_role(SemanticRole::MutedForeground).hsla();
            }
            // Page toggles are binary off|on — ignore hover / press / focus paint.
            InteractionLayer::Pressed | InteractionLayer::Hovered if page => {}
            InteractionLayer::Pressed => {
                background = shift_for_press(look, variant, tone, high_contrast, selected, background);
            }
            InteractionLayer::Hovered => {
                background = shift_for_hover(look, variant, tone, high_contrast, selected, background);
                if matches!(variant, ButtonVariant::Surface) && !selected {
                    border = Some(step(8));
                }
            }
            InteractionLayer::Default => {}
        }

        // Classic's edge tracks whatever the face settled on, so hover and press keep a
        // consistent outline instead of snapping back to the resting accent.
        if matches!(variant, ButtonVariant::Classic) && !matches!(layer, InteractionLayer::Disabled) {
            border = Some(shift_lightness(background, look.classic_params().border_delta));
        }

        if state.focused && !state.disabled && !quiet && !page {
            border = Some(look.resolve_role(SemanticRole::Focus).hsla());
        }

        if quiet {
            background = transparent();
            border = None;
        }

        ButtonFamilyPalette {
            background,
            foreground,
            muted_foreground: foreground,
            border,
            typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::MEDIUM },
            font_family: SharedString::from("System UI"),
        }
    }

    /// Solid high-contrast labels always use gray-1, matching Radix `color: var(--gray-1)`.
    fn solid_high_contrast_label(look: &Look) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, 1).hsla()
    }

    /// Recipes that tint toward accent 3/4 on hover and press instead of the solid steps.
    fn is_tinted(variant: ButtonVariant) -> bool {
        matches!(
            variant,
            ButtonVariant::Soft | ButtonVariant::Surface | ButtonVariant::Ghost | ButtonVariant::Outline
        )
    }

    fn shift_for_hover(
        look: &Look,
        variant: ButtonVariant,
        tone: Tone,
        high_contrast: bool,
        selected: bool,
        base: Hsla,
    ) -> Hsla {
        if selected || matches!(variant, ButtonVariant::Solid | ButtonVariant::Classic) {
            // HC solids stay on step 12; Radix uses a CSS filter we approximate with a lighten.
            return if high_contrast {
                shift_lightness(tone.step(look, 12), 0.04)
            } else {
                tone.step(look, 10)
            };
        }
        if is_tinted(variant) {
            return tone.step(look, 3);
        }
        let _ = look;
        base
    }

    fn shift_for_press(
        look: &Look,
        variant: ButtonVariant,
        tone: Tone,
        high_contrast: bool,
        selected: bool,
        base: Hsla,
    ) -> Hsla {
        if selected || matches!(variant, ButtonVariant::Solid | ButtonVariant::Classic) {
            return if high_contrast {
                shift_lightness(tone.step(look, 12), -0.03)
            } else {
                tone.step(look, 10)
            };
        }
        if is_tinted(variant) {
            return tone.step(look, 4);
        }
        let _ = look;
        base
    }

    #[test]
    fn authored_recipes_preserve_every_legacy_variant_state_and_mode() {
        let document = toml::Value::Table(toml::map::Map::from_iter([(
            "button_recipes".into(),
            toml::Value::try_from(embedded()).unwrap(),
        )]));
        assert_eq!(parse(&toml::to_string(&document).unwrap()).unwrap(), *embedded());
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [
                ButtonVariant::Classic,
                ButtonVariant::Solid,
                ButtonVariant::Soft,
                ButtonVariant::Surface,
                ButtonVariant::Outline,
                ButtonVariant::Ghost,
                ButtonVariant::GhostQuiet,
                ButtonVariant::Page,
            ] {
                for selected in [false, true] {
                    for paint in
                        [Paint::accent(), Paint::accent().high_contrast(), Paint::gray(), Paint::gray().high_contrast()]
                    {
                        for bits in 0..16 {
                            let state = InteractionState {
                                hovered: bits & 1 != 0,
                                pressed: bits & 2 != 0,
                                focused: bits & 4 != 0,
                                disabled: bits & 8 != 0,
                                ..Default::default()
                            };
                            let role = ButtonFamilyRole::Toggle { selected };
                            let old = legacy_palette(&look, variant, paint, role, state);
                            let new = resolve(&look, variant, paint, role, state);
                            assert_eq!(
                                (new.background, new.foreground, new.border),
                                (old.background, old.foreground, old.border),
                                "{mode:?} {variant:?} {paint:?} {selected} {state:?}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn overrides_validate_before_mutating_and_forks_are_independent() {
        let look = Look::built_in();
        assert!(look.set_button_recipes("[[button_recipes.rules]]\nbackground='tone.13'").is_err());
        let fork = look.fork();
        fork.set_button_recipes(
            "[[button_recipes.rules]]\nbackground='transparent'\nforeground='role.foreground'\nborder='none'",
        )
        .unwrap();
        let state = InteractionState::default();
        let original = resolve(&look, ButtonVariant::Solid, Paint::accent(), ButtonFamilyRole::Text, state);
        let changed = resolve(&fork, ButtonVariant::Solid, Paint::accent(), ButtonFamilyRole::Text, state);
        assert!(original.background.a > 0.0);
        assert_eq!(changed.background.a, 0.0);
        assert!(changed.border.is_none());
        assert!(
            resolve(&fork, ButtonVariant::Classic, Paint::accent(), ButtonFamilyRole::Text, state)
                .border
                .is_none()
        );
        let tuned = look.fork();
        tuned.set_button_recipes("[button_recipes.classic]\nradius=0").unwrap();
        assert_eq!(tuned.classic_params().radius, 0.0);
        tuned.set_classic_params(ClassicButtonParams { radius: 7.0, ..Default::default() });
        tuned.set_button_recipes("[button_recipes.classic]\nradius=12").unwrap();
        assert_eq!(tuned.classic_params().radius, 7.0);
    }
}
