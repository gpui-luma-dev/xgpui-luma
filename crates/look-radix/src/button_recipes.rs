//! Look-owned state recipes use the SDK's semantic/interaction types.
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::{button::ClassicButtonParams, semantic::SemanticRole};

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
    focus_border: Option<Color>,
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
                    focus_border: Some(Color::None),
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
                Rule { focus_border: Some(Color::Semantic(SemanticRole::Focus)), ..Default::default() },
                Rule {
                    variants: vec!["page".into()],
                    background: Some(Color::Transparent),
                    foreground: Some(Color::Semantic(SemanticRole::MutedForeground)),
                    border: Some(Color::None),
                    focus_border: Some(Color::None),
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
                    focus_border: Some(Color::None),
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
