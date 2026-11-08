//! Named Radix palettes: the accent and gray slots a theme is built from.
//!
//! A Radix theme is two named palettes plus an appearance: an accent (26 choices, the 25
//! chromatics plus gray) and a gray (6 neutrals, or `Auto` to take the accent's own
//! recommended pairing). Both resolve to real `@radix-ui/colors` steps, so `indigo` here is
//! Radix's indigo rather than an approximation of it.

use xgpui_luma::theme::ThemeMode;

use crate::colors::family_source_steps;
#[cfg(test)]
use crate::colors::family_steps;
use crate::scale::{CUSTOM_PALETTE, ColorScale, ModeScales, ScalePair};

/// Family backing [`crate::ScaleFamily::Destructive`]; Radix builds error states from red.
const DESTRUCTIVE_PALETTE: &str = "red";

/// Accent palettes, in the order Radix's theme panel lists them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    Gray,
    Gold,
    Bronze,
    Brown,
    Yellow,
    Amber,
    Orange,
    Tomato,
    Red,
    Ruby,
    Crimson,
    Pink,
    Plum,
    Purple,
    Violet,
    Iris,
    #[default]
    Indigo,
    Blue,
    Cyan,
    Teal,
    Jade,
    Green,
    Grass,
    Lime,
    Mint,
    Sky,
}

impl Accent {
    pub const ALL: [Self; 26] = [
        Self::Gray,
        Self::Gold,
        Self::Bronze,
        Self::Brown,
        Self::Yellow,
        Self::Amber,
        Self::Orange,
        Self::Tomato,
        Self::Red,
        Self::Ruby,
        Self::Crimson,
        Self::Pink,
        Self::Plum,
        Self::Purple,
        Self::Violet,
        Self::Iris,
        Self::Indigo,
        Self::Blue,
        Self::Cyan,
        Self::Teal,
        Self::Jade,
        Self::Green,
        Self::Grass,
        Self::Lime,
        Self::Mint,
        Self::Sky,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gray => "gray",
            Self::Gold => "gold",
            Self::Bronze => "bronze",
            Self::Brown => "brown",
            Self::Yellow => "yellow",
            Self::Amber => "amber",
            Self::Orange => "orange",
            Self::Tomato => "tomato",
            Self::Red => "red",
            Self::Ruby => "ruby",
            Self::Crimson => "crimson",
            Self::Pink => "pink",
            Self::Plum => "plum",
            Self::Purple => "purple",
            Self::Violet => "violet",
            Self::Iris => "iris",
            Self::Indigo => "indigo",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::Teal => "teal",
            Self::Jade => "jade",
            Self::Green => "green",
            Self::Grass => "grass",
            Self::Lime => "lime",
            Self::Mint => "mint",
            Self::Sky => "sky",
        }
    }

    /// The neutral Radix pairs with this accent, mirroring `getMatchingGrayColor`.
    pub fn matching_gray(self) -> Gray {
        match self {
            Self::Tomato
            | Self::Red
            | Self::Ruby
            | Self::Crimson
            | Self::Pink
            | Self::Plum
            | Self::Purple
            | Self::Violet => Gray::Mauve,
            Self::Iris | Self::Indigo | Self::Blue | Self::Sky | Self::Cyan => Gray::Slate,
            Self::Teal | Self::Jade | Self::Mint | Self::Green => Gray::Sage,
            Self::Grass | Self::Lime => Gray::Olive,
            Self::Yellow | Self::Amber | Self::Orange | Self::Brown => Gray::Sand,
            Self::Gray | Self::Gold | Self::Bronze => Gray::Gray,
        }
    }

    /// Accents whose solid step 9 stays bright enough that Radix uses a dark `*-contrast`.
    pub fn uses_dark_solid_contrast(self) -> bool {
        matches!(self, Self::Lime | Self::Mint | Self::Sky | Self::Yellow | Self::Amber)
    }
}

/// Gray palettes, plus `Auto` for the accent's recommended pairing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gray {
    #[default]
    Auto,
    Gray,
    Mauve,
    Slate,
    Sage,
    Olive,
    Sand,
}

impl Gray {
    pub const ALL: [Self; 7] = [Self::Auto, Self::Gray, Self::Mauve, Self::Slate, Self::Sage, Self::Olive, Self::Sand];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Gray => "gray",
            Self::Mauve => "mauve",
            Self::Slate => "slate",
            Self::Sage => "sage",
            Self::Olive => "olive",
            Self::Sand => "sand",
        }
    }

    /// Resolves `Auto` against the accent; every other choice is already concrete.
    pub fn resolve(self, accent: Accent) -> Self {
        match self {
            Self::Auto => accent.matching_gray(),
            other => other,
        }
    }
}

/// One theme slot: a named Radix palette, or a scale hand-seeded away from any of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteSlot {
    Named(&'static str),
    Custom,
}

impl PaletteSlot {
    pub fn label(self) -> &'static str {
        match self {
            Self::Named(palette) => palette,
            Self::Custom => CUSTOM_PALETTE,
        }
    }
}

/// The palettes a look is currently painting from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalettes {
    pub accent: PaletteSlot,
    pub gray: PaletteSlot,
}

impl ThemePalettes {
    pub fn named(accent: Accent, gray: Gray) -> Self {
        Self { accent: PaletteSlot::Named(accent.as_str()), gray: PaletteSlot::Named(gray.resolve(accent).as_str()) }
    }
}

impl Default for ThemePalettes {
    fn default() -> Self {
        Self::named(Accent::default(), Gray::default())
    }
}

/// Builds both modes' scales from the named accent and gray.
pub fn scale_pair(accent: Accent, gray: Gray) -> ScalePair {
    let gray = gray.resolve(accent);

    ScalePair { light: mode_scales(accent, gray, ThemeMode::Light), dark: mode_scales(accent, gray, ThemeMode::Dark) }
}

fn mode_scales(accent: Accent, gray: Gray, mode: ThemeMode) -> ModeScales {
    ModeScales {
        gray: named_scale(gray.as_str(), mode),
        color: named_scale(accent.as_str(), mode),
        destructive: named_scale(DESTRUCTIVE_PALETTE, mode),
    }
}

/// Missing or invalid catalog families use a flat black fallback. Catalog tests
/// validate all named families, so bundled enum selections take the parsed path.
fn named_scale(palette: &'static str, mode: ThemeMode) -> ColorScale {
    family_source_steps(mode, palette)
        .ok()
        .flatten()
        .and_then(|steps| ColorScale::named(palette, steps, xgpui_luma::color::GamutMapping::CssLocalMinde).ok())
        .unwrap_or_else(ColorScale::black)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleFamily;

    #[test]
    fn every_accent_and_gray_name_exists_in_the_catalog() {
        for accent in Accent::ALL {
            assert!(family_steps(ThemeMode::Light, accent.as_str()).is_some(), "{}", accent.as_str());
            assert!(family_steps(ThemeMode::Dark, accent.as_str()).is_some(), "{}", accent.as_str());
        }
        for gray in Gray::ALL {
            let resolved = gray.resolve(Accent::Indigo);
            assert!(family_steps(ThemeMode::Light, resolved.as_str()).is_some(), "{}", resolved.as_str());
        }
    }

    #[test]
    fn auto_gray_follows_radix_pairing() {
        assert_eq!(Gray::Auto.resolve(Accent::Indigo), Gray::Slate);
        assert_eq!(Gray::Auto.resolve(Accent::Ruby), Gray::Mauve);
        assert_eq!(Gray::Auto.resolve(Accent::Lime), Gray::Olive);
        assert_eq!(Gray::Auto.resolve(Accent::Gold), Gray::Gray);
        assert_eq!(Gray::Sand.resolve(Accent::Indigo), Gray::Sand);
    }

    #[test]
    fn indigo_slate_carries_real_radix_steps_and_provenance() {
        let scales = scale_pair(Accent::Indigo, Gray::Auto).for_mode(ThemeMode::Light);
        let accent_9 = scales.resolved(ScaleFamily::Color, 9);
        let gray_9 = scales.resolved(ScaleFamily::Gray, 9);

        assert_eq!(accent_9.hsla(), crate::colors::parse_color("#3e63dd"));
        assert_eq!(gray_9.hsla(), crate::colors::parse_color("#8b8d98"));
        match accent_9.source {
            xgpui_luma::theme::provenance::ColorSource::ScaleStep { family, step } => {
                assert_eq!(family, "indigo");
                assert_eq!(step, 9);
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn palette_labels_report_the_resolved_names() {
        let palettes = ThemePalettes::named(Accent::Indigo, Gray::Auto);

        assert_eq!(palettes.accent.label(), "indigo");
        assert_eq!(palettes.gray.label(), "slate");
        assert_eq!(PaletteSlot::Custom.label(), "custom");
    }
}
