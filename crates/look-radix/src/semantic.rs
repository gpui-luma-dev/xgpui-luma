//! Semantic role → scale family + step mappings for the Radix look.

use xgpui_luma::theme::ThemeMode;

use crate::scale::{ModeScales, ScaleFamily, ScaleStep};

/// Semantic slots resolved through the Radix color and gray scales.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SemanticRole {
    Background,
    Surface,
    Border,
    Foreground,
    Primary,
    PrimaryForeground,
    Soft,
    SoftForeground,
    Focus,
    Destructive,
    DestructiveForeground,
    MutedForeground,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticMapping {
    pub family: ScaleFamily,
    pub step: ScaleStep,
}

impl SemanticRole {
    /// Maps a semantic role to a scale step for the active mode.
    pub fn mapping(self, mode: ThemeMode) -> SemanticMapping {
        match self {
            Self::Background => SemanticMapping { family: ScaleFamily::Gray, step: 1 },
            Self::Surface => SemanticMapping { family: ScaleFamily::Gray, step: 2 },
            Self::Border => SemanticMapping { family: ScaleFamily::Gray, step: 6 },
            Self::Foreground => SemanticMapping { family: ScaleFamily::Gray, step: 12 },
            Self::MutedForeground => SemanticMapping { family: ScaleFamily::Gray, step: 11 },
            Self::Primary => SemanticMapping { family: ScaleFamily::Color, step: 9 },
            Self::PrimaryForeground => match mode {
                ThemeMode::Light => SemanticMapping { family: ScaleFamily::Color, step: 1 },
                ThemeMode::Dark => SemanticMapping { family: ScaleFamily::Gray, step: 12 },
            },
            Self::Soft => SemanticMapping { family: ScaleFamily::Color, step: 3 },
            Self::SoftForeground => SemanticMapping { family: ScaleFamily::Color, step: 11 },
            Self::Focus => SemanticMapping { family: ScaleFamily::Color, step: 8 },
            Self::Destructive => SemanticMapping { family: ScaleFamily::Destructive, step: 9 },
            Self::DestructiveForeground => match mode {
                ThemeMode::Light => SemanticMapping { family: ScaleFamily::Destructive, step: 1 },
                ThemeMode::Dark => SemanticMapping { family: ScaleFamily::Gray, step: 12 },
            },
        }
    }

    pub fn resolve(self, scales: ModeScales, mode: ThemeMode) -> xgpui_luma::theme::provenance::ResolvedColor {
        let map = self.mapping(mode);
        scales.resolved(map.family, map.step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{Accent, Gray, scale_pair};

    #[test]
    fn primary_uses_color_step_nine() {
        let map = SemanticRole::Primary.mapping(ThemeMode::Light);
        assert_eq!(map.family, ScaleFamily::Color);
        assert_eq!(map.step, 9);
    }

    #[test]
    fn light_and_dark_share_border_step() {
        assert_eq!(SemanticRole::Border.mapping(ThemeMode::Light), SemanticRole::Border.mapping(ThemeMode::Dark));
        let scales = scale_pair(Accent::Indigo, Gray::Auto);
        let light = SemanticRole::Border.resolve(scales.for_mode(ThemeMode::Light), ThemeMode::Light);
        let dark = SemanticRole::Border.resolve(scales.for_mode(ThemeMode::Dark), ThemeMode::Dark);
        assert_ne!(light.hsla().l, dark.hsla().l);
    }
}
