//! Tabs navigation property mappings:
//!
//! | Part              | Token                         |
//! |-------------------|-------------------------------|
//! | Inactive label    | `foreground`                  |
//! | Active label      | `foreground`                 |
//! | Active indicator  | `primary` / `ring` on focus   |
//! | Disabled label    | `muted-foreground`            |
//! | Disabled list bg  | `muted`                       |

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct TabsListColorTable {
    pub disabled_background: ResolvedColor,
}

impl TabsListColorTable {
    pub fn fallback() -> Self {
        Self { disabled_background: ResolvedColor::transparent() }
    }
}

#[derive(Clone, Debug)]
pub struct TabsItemColorTable {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

impl TabsItemColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), indicator: None }
    }
}

#[cfg(test)]
mod tests {

    use xgpui_luma::theme::{InteractionState, ThemeMode};

    #[test]
    fn embedded_tabs_preserve_geometry_across_modes_sizes_and_states() {
        // Its own look: `built_in()` is shared, and other tests change its mode while this one runs.
        let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for size in [crate::ShadcnSize::Sm, crate::ShadcnSize::Md, crate::ShadcnSize::Lg] {
                let theme = look.with_size(size).tabs_theme();
                for enabled in [true, false] {
                    let list = theme.resolve_list(enabled);
                    assert_eq!(list.padding, 0.0);
                    assert_eq!(list.gap, tokens.control_sizes.gap(size));
                    assert_eq!(list.radius, tokens.control_sizes.radius(size));
                }
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    for active in [true, false] {
                        let item = theme.resolve_item(active, state);
                        assert_eq!(item.indicator_height, 3.0);
                        assert_eq!(item.padding_x, tokens.control_sizes.padding_x(size));
                        assert_eq!(item.radius, tokens.control_sizes.radius(size));
                        assert_eq!(
                            item.height,
                            item.label_typography.line_height + tokens.control_sizes.padding_y(size) * 2.0
                        );
                    }
                }
            }
        }
    }
}
