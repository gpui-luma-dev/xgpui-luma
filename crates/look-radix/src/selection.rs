//! Selection factories retain SDK builders, callbacks, policies, and templates.
//! Pass an explicit Look (including forks); themes resolve live colors while
//! builder-owned geometry and motion defaults are captured at construction.
use std::sync::Arc;

use gpui::SharedString;
use xgpui_luma::controls::{autocomplete, combobox, search_selector, selection_panel};
use xgpui_luma::controls::autocomplete::{AutocompleteTheme, SelectionItem};
use xgpui_luma::controls::selection_panel::{SelectionPanelItemLike, SelectionPanelLook};
use crate::{Look, PopupMenuVariant, TextFieldVariant, Tone};

/// Live Radix status, clear-affordance, and menu styling shared by search controls.
/// Drawn from `.lss` (`style/autocomplete.lss`); the popup is the look's solid accent floating menu.
pub fn autocomplete_theme(look: &Look) -> Arc<dyn AutocompleteTheme> {
    let size = look.size();
    let menu_look = look.clone();
    let binding = xgpui_luma::style::StyleBinding::new("autocomplete", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(autocomplete::StyledAutocompleteTheme::new(
        binding,
        Arc::new(move || crate::floating_menu_look_for(&menu_look, PopupMenuVariant::Solid, Tone::Accent, size)),
    ))
}

/// Radix Autocomplete using the SDK builder and its complete callback/policy API.
/// Configure `.size(RadixSize)` and other SDK options before `.spawn(cx)`.
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
        .stylesheet(&look.sized_stylesheet(look.size()))
        .motion_defaults(look.motion_defaults().autocomplete)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move || {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, panel_look.size())
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
        .stylesheet(&look.sized_stylesheet(look.size()))
        .motion_defaults(look.motion_defaults().combobox)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move || {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, panel_look.size())
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
        .stylesheet(&look.sized_stylesheet(look.size()))
        .motion_defaults(look.motion_defaults().search_selector)
        .textfield_template(crate::textfield_template(look, TextFieldVariant::Surface))
        .selector_theme(crate::selector_theme(look))
        .autocomplete_theme(autocomplete_theme(look))
        .scrollbar_template(crate::scrollbar_template(look))
        .popup_look_provider(Arc::new(move || {
            crate::selector_items_panel_look_for(&panel_look, PopupMenuVariant::Solid, Tone::Accent, panel_look.size())
        }))
}

/// Inline selection-panel chrome, with selector-list overrides followed by
/// independent `selection_panel` overrides using Radix size keys `1`–`3`.
/// Returns a snapshot; resolve again after changing the supplied Look.
pub fn selection_panel_look(look: &Look, size: crate::RadixSize) -> SelectionPanelLook {
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
    .with_stylesheet(&look.sized_stylesheet(size).selection_panel)
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
        .stylesheet(&look.sized_stylesheet(look.size()))
        .scrollbar_template(crate::scrollbar_template(look))
        .look_provider(Arc::new(move || selection_panel_look(&panel_look, panel_look.size())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SemanticRole;
    use xgpui_luma::theme::ThemeMode;
    use crate::{Accent, Gray};

    #[test]
    fn bound_theme_tracks_mode_palette_and_keeps_forks_independent() {
        let look = Look::built_in();
        let fork = look.fork();
        let theme = autocomplete_theme(&fork);
        let initial = theme.resolve();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            fork.set_mode(mode);
            for accent in [Accent::Red, Accent::Blue] {
                fork.set_palettes(accent, Gray::Slate);
                let resolved = theme.resolve();
                crate::control_group::assert_close(
                    resolved.status_color,
                    fork.resolve_role(SemanticRole::Primary).hsla(),
                );
                crate::control_group::assert_close(
                    resolved.clear_icon_color,
                    fork.resolve_role(SemanticRole::MutedForeground).hsla(),
                );
                crate::control_group::assert_close(
                    resolved.clear_icon_hover_color,
                    fork.resolve_role(SemanticRole::Foreground).hsla(),
                );
                crate::control_group::assert_close(resolved.menu.item_hover_background, resolved.status_color);
            }
        }
        assert_eq!(autocomplete_theme(&look).resolve().status_color, initial.status_color);
        assert_ne!(theme.resolve().status_color, initial.status_color);
    }

    #[test]
    fn per_size_panel_overrides_resolve_live() {
        let look = Look::built_in();
        let mut sheet = look.common_stylesheet();
        for (size, height) in [("1", 25.0), ("2", 37.0), ("3", 49.0)] {
            sheet.selector_list.sizes.get_or_default(size).item_height = Some(height);
            sheet.selection_panel.sizes.get_or_default(size).border_width = Some(0.0);
        }
        look.set_common_stylesheet(sheet);
        for (size, height) in
            [(crate::RadixSize::One, 25.0), (crate::RadixSize::Two, 37.0), (crate::RadixSize::Three, 49.0)]
        {
            let panel = selection_panel_look(&look, size);
            assert_eq!(panel.item_height, height);
            assert_eq!(panel.border_width, 0.0);
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
