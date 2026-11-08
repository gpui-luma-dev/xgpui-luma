//! Shared demo assets for Luma apps (Studio, shells).
//!
//! Embeds tweakcn CSS packs and fonts referenced by those themes. Not part of the
//! SDK or look runtime — production apps should supply their own CSS and fonts.

pub mod color_picker;
mod built_in;
mod fonts;

pub use built_in::{BuiltInTheme, built_in_theme, built_in_themes};
pub use fonts::{GEIST_MONO_FAMILY, JETBRAINS_MONO_FAMILY, RAJDHANI_FAMILY, register, register_all};
use xgpui_luma_look_shadcn::ShadcnLook;

/// Canonical fallback theme CSS bundled by the shadcn look crate.
pub use xgpui_luma_look_shadcn::FALLBACK_CSS;

/// Independently mutable look parsed from [`FALLBACK_CSS`].
///
/// Parse afresh so app edits do not mutate the shared [`ShadcnLook::built_in`] fallback.
pub fn fallback_look() -> ShadcnLook {
    ShadcnLook::from_css_str(FALLBACK_CSS).expect("embedded shadcn fallback CSS should parse")
}

/// Look parsed from a bundled tweakcn theme id.
pub fn built_in_look(theme_id: &str) -> anyhow::Result<ShadcnLook> {
    let Some(theme) = built_in_theme(theme_id) else {
        anyhow::bail!("unknown built-in shadcn theme `{theme_id}`");
    };
    ShadcnLook::from_css_str(theme.css)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn bundled_themes_parse_source_colors_and_build_previews() {
        for theme in built_in_themes() {
            let look =
                ShadcnLook::from_css_str(theme.css).unwrap_or_else(|error| panic!("theme {}: {error}", theme.id));
            for tokens in [look.light_tokens(), look.dark_tokens()] {
                for token in xgpui_luma_look_shadcn::ShadcnToken::ALL {
                    if tokens.catalog.get(token.css_name()).is_some() {
                        tokens.catalog.source_color(token.css_name()).unwrap().validate().unwrap();
                    }
                }
            }
        }
    }

    #[test]
    fn fallback_look_matches_bundled_fallback_in_both_modes() {
        let look = fallback_look();
        let fallback = ShadcnLook::built_in();
        assert_eq!(look.light_tokens().catalog, fallback.light_tokens().catalog);
        assert_eq!(look.dark_tokens().catalog, fallback.dark_tokens().catalog);
    }

    #[test]
    fn fallback_look_edits_do_not_change_other_instances_or_fallback() {
        let edited = fallback_look();
        let untouched = fallback_look();
        let fallback = ShadcnLook::built_in();
        let original_light = untouched.light_tokens();
        let original_dark = untouched.dark_tokens();
        let fallback_light = fallback.light_tokens();
        let fallback_dark = fallback.dark_tokens();

        edited
            .apply_token_overrides(&HashMap::from([(String::from("primary"), String::from("#ff00ff"))]))
            .expect("valid primary override");

        assert_ne!(edited.light_tokens().catalog, original_light.catalog);
        assert_ne!(edited.dark_tokens().catalog, original_dark.catalog);
        assert_eq!(untouched.light_tokens().catalog, original_light.catalog);
        assert_eq!(untouched.dark_tokens().catalog, original_dark.catalog);
        assert_eq!(fallback.light_tokens().catalog, fallback_light.catalog);
        assert_eq!(fallback.dark_tokens().catalog, fallback_dark.catalog);
    }
}
