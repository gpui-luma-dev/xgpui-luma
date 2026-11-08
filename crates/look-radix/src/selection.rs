//! Selection factories retain SDK builders, callbacks, policies, and templates.
//! Pass an explicit Look (including forks); themes resolve live colors while
//! builder-owned geometry and motion defaults are captured at construction.
use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::{autocomplete, combobox, search_selector, selection_panel};
use xgpui_luma::controls::autocomplete::{AutocompleteLook, AutocompleteTheme, SelectionItem};
use xgpui_luma::controls::selection_panel::{SelectionPanelItemLike, SelectionPanelLook};
use xgpui_luma::theme::{ControlSize, LumaTextStyle};
use crate::{Look, PopupMenuVariant, SemanticRole, TextFieldVariant, Tone};

struct AutocompleteThemeAdapter {
    look: Look,
}

impl AutocompleteTheme for AutocompleteThemeAdapter {
    fn resolve(&self, size: ControlSize) -> AutocompleteLook {
        let geometry = self.look.common_stylesheet().autocomplete.resolve_geometry(
            size.stylesheet_key(),
            xgpui_luma::theme::stylesheet::AutocompleteGeometry {
                status_font_size: 12.0,
                status_line_height: 16.0,
                ..Default::default()
            },
        );
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        AutocompleteLook {
            status_color: self.look.resolve_role(SemanticRole::Primary).hsla(),
            status_typography: LumaTextStyle {
                size: geometry.status_font_size,
                line_height: geometry.status_line_height,
                weight: gpui::FontWeight::NORMAL,
            },
            muted_text_color: muted,
            clear_icon_color: muted,
            clear_icon_hover_color: self.look.resolve_role(SemanticRole::Foreground).hsla(),
            menu: crate::floating_menu_look_for(&self.look, PopupMenuVariant::Solid, Tone::Accent, size),
        }
    }
}

/// Live Radix status, clear-affordance, and menu styling shared by search controls.
pub fn autocomplete_theme(look: &Look) -> Arc<dyn AutocompleteTheme> {
    Arc::new(AutocompleteThemeAdapter { look: look.clone() })
}

/// Radix Autocomplete using the SDK builder and its complete callback/policy API.
/// Configure `.size(ControlSize)` and other SDK options before `.spawn(cx)`.
/// Theme changes require notifying the control and owner, as with [`Look`].
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma::controls::autocomplete::SelectionItem;
/// use xgpui_luma_look_radix::{Look, autocomplete_builder};
/// fn example<M: 'static>(look: &Look, cx: &mut Context<M>) {
///     let control = autocomplete_builder(look, "fruit", [SelectionItem::new("apple", "Apple")])
///         .placeholder("Find fruit").spawn(cx);
/// }
/// ```
pub fn autocomplete_builder(
    look: &Look,
    id: impl Into<SharedString>,
    items: impl IntoIterator<Item = SelectionItem>,
) -> autocomplete::AutocompleteBuilder {
    let panel_look = look.clone();
    autocomplete::new(id, items)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().autocomplete)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move |size| {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, size)
        }))
}

/// Radix ComboBox with SDK typing policy, callback, and layered-template options.
/// Retains the explicit Look; geometry/motion defaults are captured at construction.
pub fn combobox_builder(
    look: &Look,
    id: impl Into<SharedString>,
    items: impl IntoIterator<Item = SelectionItem>,
) -> combobox::ComboBoxBuilder {
    let panel_look = look.clone();
    combobox::new(id, items)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().combobox)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move |size| {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, size)
        }))
}

/// Radix SearchSelector combining the existing selector trigger and text-field themes.
/// Returns the SDK builder so callback, focus, scroll, and template options remain available.
pub fn search_selector_builder(
    look: &Look,
    id: impl Into<SharedString>,
    items: impl IntoIterator<Item = SelectionItem>,
) -> search_selector::SearchSelectorBuilder {
    let panel_look = look.clone();
    search_selector::new(id, items)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().search_selector)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .selector_theme(crate::selector_theme(look))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move |size| {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, size)
        }))
}

/// Inline selection-panel chrome, with selector-list overrides followed by
/// independent `selection_panel` overrides using Radix size keys `1`–`3`.
/// Returns a snapshot; resolve again after changing the supplied Look.
pub fn selection_panel_look(look: &Look, size: ControlSize) -> SelectionPanelLook {
    let panel = crate::selector_items_panel_look_for(look, PopupMenuVariant::Solid, Tone::Accent, size);
    SelectionPanelLook {
        background: panel.background,
        foreground: panel.foreground,
        border: panel.border,
        border_width: panel.border_width,
        disabled_opacity: panel.disabled_opacity,
        shadow: panel.shadow,
        radius: panel.radius,
        padding: panel.padding,
        min_width: panel.min_width,
        item_disabled_foreground: panel.item_disabled_foreground,
        item_hover_background: panel.item_hover_background,
        item_hover_foreground: panel.item_hover_foreground,
        item_typography: panel.item_typography,
        item_height: panel.item_height,
        item_padding_x: panel.item_padding_x,
        item_gap: panel.item_gap,
        item_icon_size: panel.item_icon_size,
        item_radius: panel.item_radius,
    }
    .with_stylesheet(&look.common_stylesheet().selection_panel, crate::look::sdk_size_key(size))
}

/// Radix SelectionPanel with SDK typed items and embedded/popup interaction policies.
/// Custom look providers, templates, and callbacks applied afterward take precedence.
pub fn selection_panel_builder<T: SelectionPanelItemLike + 'static>(
    look: &Look,
    id: impl Into<SharedString>,
    items: impl IntoIterator<Item = T>,
) -> selection_panel::SelectionPanelBuilder<T> {
    let panel_look = look.clone();
    selection_panel::SelectionPanelBuilder::new(id)
        .items(items)
        .stylesheet(&look.common_stylesheet())
        .scrollbar_template(crate::scrollbar_template(look))
        .look_provider(Arc::new(move |size| selection_panel_look(&panel_look, size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    use crate::{Accent, Gray};

    #[test]
    fn bound_theme_tracks_mode_palette_and_keeps_forks_independent() {
        let look = Look::built_in();
        let fork = look.fork();
        let theme = autocomplete_theme(&fork);
        let initial = theme.resolve(ControlSize::Md);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            fork.set_mode(mode);
            for accent in [Accent::Red, Accent::Blue] {
                fork.set_palettes(accent, Gray::Slate);
                let resolved = theme.resolve(ControlSize::Md);
                assert_eq!(resolved.status_color, fork.resolve_role(SemanticRole::Primary).hsla());
                assert_eq!(resolved.clear_icon_color, fork.resolve_role(SemanticRole::MutedForeground).hsla());
                assert_eq!(resolved.clear_icon_hover_color, fork.resolve_role(SemanticRole::Foreground).hsla());
                assert_eq!(resolved.menu.item_hover_background, resolved.status_color);
            }
        }
        assert_eq!(autocomplete_theme(&look).resolve(ControlSize::Md).status_color, initial.status_color);
        assert_ne!(theme.resolve(ControlSize::Md).status_color, initial.status_color);
    }

    #[test]
    fn per_size_panel_overrides_and_status_typography_resolve_live() {
        let look = Look::built_in();
        let theme = autocomplete_theme(&look);
        let mut sheet = look.common_stylesheet();
        for (size, height) in [("1", 25.0), ("2", 37.0), ("3", 49.0)] {
            sheet.selector_list.sizes.custom.entry(size.into()).or_default().item_height = Some(height);
            sheet.selection_panel.sizes.custom.entry(size.into()).or_default().border_width = Some(0.0);
        }
        sheet.autocomplete.geometry.status_font_size = Some(19.0);
        look.set_common_stylesheet(sheet);
        for (size, height) in [(ControlSize::Sm, 25.0), (ControlSize::Md, 37.0), (ControlSize::Lg, 49.0)] {
            let panel = selection_panel_look(&look, size);
            assert_eq!(panel.item_height, height);
            assert_eq!(panel.border_width, 0.0);
            assert_eq!(theme.resolve(size).status_typography.size, 19.0);
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn panel_factory_keeps_sdk_scroll_policies() {
    use gpui::TestAppContext;
    use xgpui_luma::interaction::{ScrollBoundaryPolicy, WheelFocusScope, WheelScrollPolicy};
    use xgpui_luma::controls::selection_panel::SelectionPanelItem;
    let mut app = TestAppContext::single();
    let look = Look::built_in();
    let control = selection_panel_builder(&look, "policy", [SelectionPanelItem::new("a").label("A")])
        .wheel_scroll_policy(WheelScrollPolicy::PassThrough)
        .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
        .wheel_focus_scope(WheelFocusScope::Owner)
        .spawn(&mut app);
    control.read_with(&app, |panel, _| {
        let policy = panel.scroll_interaction();
        assert_eq!(policy.wheel, WheelScrollPolicy::PassThrough);
        assert_eq!(policy.boundary, ScrollBoundaryPolicy::Chain);
        assert_eq!(policy.focus_scope, WheelFocusScope::Owner);
    });
}
