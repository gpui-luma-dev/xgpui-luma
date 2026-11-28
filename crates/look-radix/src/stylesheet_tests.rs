//! Resolved-output baselines and customization boundaries for migrated families.
// Some baselines read the compatibility Looks, which build from the resolved styles (#433).
#![allow(deprecated)] // Compatibility paths and parity tests still read the deprecated Looks (#433).
use xgpui_luma::style::PartRead;
use super::*;
use xgpui_luma::theme::{StandardBoxScale, ThemeMode};
use xgpui_luma::controls::{textfield::TextFieldState, textarea::TextAreaState, toolbar::ToolbarVariant};

#[test]
fn size_and_state_baselines_survive_shared_geometry_migration() {
    let look = Look::built_in();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        for (size, height, padding, gap, font, line) in [
            (ButtonSize::One, 24.0, 8.0, 4.0, 12.0, 16.0),
            (ButtonSize::Two, 32.0, 12.0, 8.0, 14.0, 20.0),
            (ButtonSize::Three, 40.0, 16.0, 12.0, 16.0, 24.0),
            (ButtonSize::Four, 48.0, 24.0, 12.0, 18.0, 26.0),
        ] {
            for radius in Radius::ALL {
                let box_ = crate::button_layout::button_box_for(size, radius);
                assert_eq!(
                    (box_.height, box_.padding_x, box_.gap, box_.typography.size, box_.typography.line_height),
                    (height, padding, gap, font, line)
                );
                assert_eq!(box_.radius, resolve_button_radius(size, radius));
            }
        }
        for (sdk, font, line, progress_height) in [
            (RadixSize::One, 12.5, 18.0, 4.0),
            (RadixSize::Two, 14.0, 20.0, 6.0),
            (RadixSize::Three, 16.0, 22.0, 8.0),
        ] {
            let scale = StandardBoxScale::compute(&look.metrics_for(sdk), 2.0);
            use xgpui_luma::style::PartRead;
            for bits in 0..16 {
                let hovered = bits & 1 != 0;
                let focused = bits & 2 != 0;
                let invalid = bits & 4 != 0;
                let enabled = bits & 8 == 0;
                for variant in TextFieldVariant::ALL {
                    let field_style = textfield_theme_with(&look.with_size(sdk), variant)
                        .resolve_style(TextFieldState { hovered, focused, invalid, ..Default::default() }, enabled);
                    let field = field_style.root();
                    assert_eq!(
                        (
                            field.height().unwrap_or(0.0),
                            field.number("padding_x"),
                            field.number("padding_y"),
                            field.radius().unwrap_or(0.0)
                        ),
                        (scale.height, scale.padding_x, scale.padding_y, scale.radius)
                    );
                    let text = field.typography(14.0, 20.0);
                    assert_eq!((text.size, text.line_height), (font, line));
                    let area_style = textarea_theme_with(&look.with_size(sdk), variant)
                        .resolve_style(TextAreaState { hovered, focused, invalid, ..Default::default() }, enabled);
                    let area = area_style.root();
                    assert_eq!(
                        (
                            area.height().unwrap_or(0.0),
                            area.number("padding_x"),
                            area.number("padding_y"),
                            area.radius().unwrap_or(0.0)
                        ),
                        (scale.height, scale.padding_x, scale.padding_y, scale.radius)
                    );
                    let text = area.typography(14.0, 20.0);
                    assert_eq!((text.size, text.line_height), (14.0, 20.0));
                }
            }
            for variant in ProgressVariant::ALL {
                for enabled in [true, false] {
                    assert_eq!(
                        progress_theme_with(&look.with_size(sdk), variant, Paint::accent())
                            .resolve(enabled)
                            .track_height,
                        progress_height
                    );
                }
            }
        }
        let menu =
            xgpui_luma::controls::floating_menu::MenuStyle::resolve(&crate::popup_menu::floating_menu_binding_for(
                &look,
                PopupMenuVariant::Solid,
                Tone::Accent,
                crate::RadixSize::Two,
            ));
        let (panel, item) = (menu.panel(), menu.item());
        assert_eq!(
            (
                panel.number("min_width"),
                item.number("height"),
                item.number("padding_x"),
                item.typography(14.0, 20.0).size
            ),
            (180.0, 32.0 * 0.9, 9.0, 14.0)
        );
        let tooltip = xgpui_luma::controls::tooltip::resolve_tooltip(&tooltip_theme(&look).style());
        let tooltip = tooltip.root();
        let text = tooltip.typography(14.0, 20.0);
        assert_eq!(
            (
                tooltip.number("padding_x"),
                tooltip.number("padding_y"),
                tooltip.number("max_width"),
                text.size,
                text.line_height
            ),
            (8.0, 4.0, 260.0, 12.0, 16.0)
        );
    }
}

#[test]
fn live_edits_zero_and_local_overrides_keep_forks_isolated() {
    let look = Look::built_in();
    let fork = look.fork();
    let explicit_toolbar = toolbar_theme_with(&fork, ToolbarStyle { gap: 17.0, ..Default::default() });
    let card = Card::new(&fork).variant(CardVariant::Ghost);
    let explicit_card = Card::new(&fork).style_override(|style| style.padding = 21.0);
    let avatar = Avatar::new(&fork, "AB").size(AvatarSize::Nine).radius(Radius::Full);
    let explicit_avatar = Avatar::new(&fork, "A").style_override(|style| style.diameter = 33.0);
    let mut config = fork.common_stylesheet();
    config
        .row_limits
        .0
        .insert("pager".into(), xgpui_luma::theme::stylesheet::RowLimits { min: None, max: Some(57) });
    fork.set_common_stylesheet(config.clone());
    assert_ne!(look.common_stylesheet(), config);
    let _scale = StandardBoxScale::compute(&fork.metrics(), 1.0);
    assert_eq!(explicit_toolbar.resolve(true, ToolbarVariant::Outline).gap, 17.0);
    // The card is drawn from `.lss`; `[common.card]` no longer reaches it.
    assert_eq!((card.resolve_style().padding, card.resolve_style().margin), (12.0, -12.0));
    assert_eq!(explicit_card.resolve_style().padding, 21.0);
    // The avatar is drawn from `.lss` too: size 9, a full circle, the two-letter size.
    assert_eq!(
        (avatar.resolve_style().diameter, avatar.resolve_style().radius, avatar.resolve_style().font_size),
        (160.0, 80.0, 60.0)
    );
    assert_eq!(explicit_avatar.resolve_style().diameter, 33.0);
    assert_eq!(button_box_for(ButtonSize::Three, Radius::Full).height, 40.0);
}

#[test]
fn toggle_size_four_is_not_collapsed_into_sdk_lg() {
    use xgpui_luma::style::lss::{AttrValue, ControlId, StyleKey, Value};
    let look = Look::built_in();
    let config = crate::style::config();
    let key = StyleKey::new(ControlId::new("toggle"))
        .attr_opt(config.attribute("variant").unwrap(), Some(AttrValue::new("soft")))
        .attr_opt(config.attribute("size").unwrap(), Some(AttrValue::new("4")));
    let resolved = look.style_theme().resolve(&key);
    let root = resolved.root();
    let padding_x = root.value("padding_x").and_then(Value::as_number);
    assert_eq!((root.height(), padding_x, root.font_size()), (Some(48.0), Some(24.0), Some(18.0)));
}

#[test]
fn slide_panel_uses_captured_common_visual_defaults() {
    use xgpui_luma::controls::slide_panel::{SlidePanelSizeConfig, SlidePanelTopAnchor};
    let look = Look::built_in().fork();
    let panel = look.slide_panel_state(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(0.0, 0.0, 900.0));
    assert_eq!(panel.geometry(), xgpui_luma::controls::slide_panel::SlidePanelGeometry::sdk());
    // The drawer's resize handle is the look's, at its size, as in Resizable Panels (#360).
    assert_eq!(panel.resize_handle_metrics(), crate::resizable_panels_theme(&look).resize_handle_metrics());
}
