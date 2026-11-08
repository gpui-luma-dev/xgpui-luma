//! Radix `data-accent-color` override: paint from the accent scale or lock to gray.
//!
//! Radix Themes resolves every chromatic step against the accent scale unless a subtree
//! sets `data-accent-color="gray"`, which forces the neutral scale no matter what the root
//! accent is. `Tone` is that switch, so a control can be previewed or placed as either.

use gpui::Hsla;
use xgpui_luma::theme::ThemeMode;

use crate::look::Look;
use crate::scale::{ScaleFamily, ScaleStep};
use crate::semantic::SemanticRole;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Follows the theme's accent scale.
    #[default]
    Accent,
    /// Locked to the neutral scale, whatever the accent is.
    Gray,
}

impl Tone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accent => "accent",
            Self::Gray => "gray",
        }
    }

    /// The 12-step scale this tone paints from.
    pub fn family(self) -> ScaleFamily {
        match self {
            Self::Accent => ScaleFamily::Color,
            Self::Gray => ScaleFamily::Gray,
        }
    }

    pub fn step(self, look: &Look, step: ScaleStep) -> Hsla {
        look.resolve_step(self.family(), step).hsla()
    }

    /// Foreground that sits on the tone's solid step 9, Radix's `*-contrast`.
    pub fn contrast(self, look: &Look) -> Hsla {
        match self {
            Self::Accent => look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
            Self::Gray => match look.mode() {
                ThemeMode::Light => look.resolve_step(ScaleFamily::Gray, 1).hsla(),
                ThemeMode::Dark => look.resolve_step(ScaleFamily::Gray, 12).hsla(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{Accent, Gray};

    #[test]
    fn gray_tone_ignores_the_accent_scale() {
        let look = Look::built_in();

        assert_eq!(Tone::Gray.step(&look, 9), look.resolve_step(ScaleFamily::Gray, 9).hsla());
        assert_ne!(Tone::Accent.step(&look, 9), Tone::Gray.step(&look, 9));
    }

    #[test]
    fn contrast_stays_legible_on_a_solid_face() {
        let look = Look::built_in();

        for tone in [Tone::Accent, Tone::Gray] {
            assert!(tone.contrast(&look).l > tone.step(&look, 9).l);
        }
    }

    #[test]
    fn bright_accents_use_dark_contrast_on_solid_faces() {
        for accent in [Accent::Lime, Accent::Mint, Accent::Sky, Accent::Yellow, Accent::Amber] {
            let look = Look::from_palettes(accent, Gray::Auto, Default::default(), ThemeMode::Light);
            let contrast = Tone::Accent.contrast(&look);
            assert!(contrast.l < Tone::Accent.step(&look, 9).l, "{accent:?} light solid needs a darker contrast label");

            look.set_mode(ThemeMode::Dark);
            let contrast = Tone::Accent.contrast(&look);
            assert!(contrast.l < Tone::Accent.step(&look, 9).l, "{accent:?} dark solid needs a darker contrast label");
        }
    }
}
