//! Tabs navigation property mappings:
//!
//! | Part              | Token                         |
//! |-------------------|-------------------------------|
//! | Inactive label    | `foreground`                  |
//! | Active label      | `foreground`                 |
//! | Active indicator  | `primary` / `ring` on focus   |
//! | Disabled label    | `muted-foreground`            |
//! | Disabled list bg  | `muted`                       |

#[cfg(test)]
mod tests {

    use xgpui_luma::theme::{InteractionState, ThemeMode};

    /// A tab list in `enabled`, read from the theme's resolved style (#433).
    struct ListProbe {
        padding: f32,
        radius: f32,
        gap: f32,
    }

    /// A tab, `active` or not, in `state`, read from the theme's resolved style (#433).
    struct ItemProbe {
        radius: f32,
        padding_x: f32,
        height: f32,
        indicator_height: f32,
        line_height: f32,
    }

    fn tab_list(theme: &std::sync::Arc<dyn xgpui_luma::controls::tabs::TabsTheme>, enabled: bool) -> ListProbe {
        use xgpui_luma::controls::tabs::resolve_tabs_list;
        use xgpui_luma::style::PartRead;
        let resolved = resolve_tabs_list(&theme.style(), enabled);
        let list = resolved.part(xgpui_luma::style::lss::PartId::new("list"));
        ListProbe { padding: list.number("padding"), radius: list.number("radius"), gap: list.number("gap") }
    }

    fn tab_item(
        theme: &std::sync::Arc<dyn xgpui_luma::controls::tabs::TabsTheme>,
        active: bool,
        state: InteractionState,
    ) -> ItemProbe {
        use xgpui_luma::controls::tabs::resolve_tab;
        use xgpui_luma::style::PartRead;
        let resolved = resolve_tab(&theme.item_style(active, state), active, state);
        let (root, item) = (resolved.root(), resolved.part(xgpui_luma::style::lss::PartId::new("item")));
        ItemProbe {
            radius: item.number("radius"),
            padding_x: item.number("padding_x"),
            height: item.number("height"),
            indicator_height: root.number("indicator_height"),
            line_height: item.typography(14.0, 20.0).line_height,
        }
    }

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
                    let list = tab_list(&theme, enabled);
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
                        let item = tab_item(&theme, active, state);
                        assert_eq!(item.indicator_height, 3.0);
                        assert_eq!(item.padding_x, tokens.control_sizes.padding_x(size));
                        assert_eq!(item.radius, tokens.control_sizes.radius(size));
                        assert_eq!(item.height, item.line_height + tokens.control_sizes.padding_y(size) * 2.0);
                    }
                }
            }
        }
    }
}
