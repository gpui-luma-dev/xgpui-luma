//! Forced-state SDK template previews, with palette and size tabs like other controls.
use std::sync::Arc;
use gpui::{AnyElement, App, Entity, Hsla, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::{autocomplete, combobox, search_selector, selection_panel, scrollbar, selector};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::textfield::TextFieldState;
use xgpui_luma::infra::{icon::SelectionStatusIcons, state::ControlFocusState, value::ControlRange};
use xgpui_luma::motion::overlay_presence::OverlayPresence;
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale};
use xgpui_luma_look_radix::{self as radix, Accent, Gray, Look, PopupMenuVariant, Tone};
use super::{preview_handlers, states};
use super::matrix_grid::{centered, column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Autocomplete,
    ComboBox,
    SearchSelector,
    SelectionPanel,
    Scrollbar,
}

#[derive(Clone, Copy)]
struct Variant {
    label: &'static str,
    ghost: bool,
    orientation: scrollbar::ScrollbarOrientation,
}
fn variants(kind: Kind) -> Vec<Variant> {
    use scrollbar::ScrollbarOrientation::{Horizontal, Vertical};
    let variant = |label, ghost, orientation| Variant { label, ghost, orientation };
    match kind {
        Kind::Scrollbar => vec![
            variant("Ghost · Horizontal", true, Horizontal),
            variant("Soft · Horizontal", false, Horizontal),
            variant("Ghost · Vertical", true, Vertical),
            variant("Soft · Vertical", false, Vertical),
        ],
        Kind::SearchSelector => vec![variant("Outline", false, Horizontal), variant("Ghost", true, Horizontal)],
        _ => vec![variant("Surface", false, Horizontal)],
    }
}

fn samples(kind: Kind) -> Vec<states::StateSample> {
    let mut samples = states::samples().to_vec();
    if matches!(kind, Kind::Autocomplete | Kind::ComboBox | Kind::SearchSelector) {
        samples.insert(
            4,
            states::StateSample {
                id: "invalid",
                label: "Invalid",
                icon: "exclamation-triangle",
                state: InteractionState { invalid: true, ..Default::default() },
            },
        );
    }
    samples
}

pub(super) fn tabbed(
    kind: Kind,
    look: &Arc<Look>,
    tabs: Entity<Tabs>,
    colors: [Hsla; 3],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let [fg, muted, border] = colors;
    let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
    // Preview-only typography experiment; keep control geometry and other sections intact.
    let compact_look = (kind == Kind::Autocomplete && active.as_ref() == "template-preview").then(|| {
        let fork = look.fork();
        let mut sheet = fork.common_stylesheet();
        let field = sheet.textfield.sizes.custom.entry("2".into()).or_default();
        field.font_size = Some(12.0);
        field.line_height = Some(16.0);
        let items = sheet.selector_list.sizes.custom.entry("2".into()).or_default();
        items.font_size = Some(12.0);
        items.line_height = Some(16.0);
        fork.set_common_stylesheet(sheet);
        Arc::new(fork)
    });
    let look = compact_look.as_ref().unwrap_or(look);
    let rows = variants(kind);
    let samples = samples(kind);
    let mut grid = fixed_grid(
        equal_data_columns(
            if compact_selection(kind) { 104.0 } else { 144.0 },
            preview_width(kind),
            match active.as_ref() {
                "colors" => rows.len(),
                "all-sizes" => 3,
                _ => samples.len(),
            },
        ),
        if compact_selection(kind) { 12.0 } else { 20.0 },
        20.0,
    )
    .child(empty_corner(), 0, 0);
    match active.as_ref() {
        "colors" => {
            for (col, variant) in rows.iter().enumerate() {
                grid = grid.child(column_header(variant.label, muted), 0, col + 1);
            }
            // Scrollbars use the neutral scale, so compare gray palettes instead of accents.
            let palettes: Vec<_> = if kind == Kind::Scrollbar {
                Gray::ALL
                    .iter()
                    .filter(|gray| **gray != Gray::Auto)
                    .map(|gray| (gray.as_str(), None, Some(*gray)))
                    .collect()
            } else {
                Accent::ALL.iter().map(|accent| (accent.as_str(), Some(*accent), None)).collect()
            };
            for (row, (label, accent, gray)) in palettes.into_iter().enumerate() {
                let fork = look.fork();
                fork.set_palettes(accent.unwrap_or(Accent::Blue), gray.unwrap_or(Gray::Auto));
                grid = grid.child(row_label(super::palettes::title_case(label), fg), row + 1, 0);
                for (col, variant) in rows.iter().enumerate() {
                    grid = grid.child(
                        sample(
                            &fork,
                            Preview {
                                kind,
                                variant: *variant,
                                size: ControlSize::Md,
                                state: InteractionState {
                                    hovered: true,
                                    focused: kind != Kind::Scrollbar,
                                    ..Default::default()
                                },
                                id: format!("color-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
        "all-sizes" => {
            let sizes = [(ControlSize::Sm, "Size 1"), (ControlSize::Md, "Size 2"), (ControlSize::Lg, "Size 3")];
            for (col, (_, label)) in sizes.iter().enumerate() {
                grid = grid.child(column_header(*label, muted), 0, col + 1);
            }
            for (row, variant) in rows.iter().enumerate() {
                grid = grid.child(row_label(variant.label, fg), row + 1, 0);
                for (col, (size, _)) in sizes.iter().enumerate() {
                    grid = grid.child(
                        sample(
                            look,
                            Preview {
                                kind,
                                variant: *variant,
                                size: *size,
                                state: InteractionState { focused: kind != Kind::Scrollbar, ..Default::default() },
                                id: format!("size-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
        _ => {
            for (col, state) in samples.iter().enumerate() {
                grid = grid.child(states::header_cell(state, muted), 0, col + 1);
            }
            for (row, variant) in rows.iter().enumerate() {
                grid = grid.child(row_label(variant.label, fg), row + 1, 0);
                for (col, state) in samples.iter().enumerate() {
                    grid = grid.child(
                        sample(
                            look,
                            Preview {
                                kind,
                                variant: *variant,
                                size: ControlSize::Md,
                                state: state.state,
                                id: format!("state-{row}-{col}"),
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
        }
    }
    preview_tabbed(tabs, border, grid.into_any_element())
}

fn compact_selection(kind: Kind) -> bool {
    matches!(kind, Kind::Autocomplete | Kind::ComboBox | Kind::SearchSelector)
}

fn preview_width(kind: Kind) -> f32 {
    if compact_selection(kind) { 140.0 } else { 200.0 }
}

struct Preview {
    kind: Kind,
    variant: Variant,
    size: ControlSize,
    state: InteractionState,
    id: String,
}
fn sample(look: &Look, preview: Preview, window: &mut Window, cx: &mut App) -> AnyElement {
    let Preview { kind, variant, size, state, id } = preview;
    let kind_id = match kind {
        Kind::Autocomplete => "autocomplete",
        Kind::ComboBox => "combobox",
        Kind::SearchSelector => "search-selector",
        Kind::SelectionPanel => "selection-panel",
        Kind::Scrollbar => "scrollbar",
    };
    let id = SharedString::from(format!("guide-{kind_id}-{id}"));
    if kind == Kind::Scrollbar {
        let model = scrollbar::ScrollbarRenderModel {
            id: &id,
            orientation: variant.orientation,
            range: ControlRange::from(0..100),
            step: 1.0,
            page_step: 20.0,
            value: 40.0,
            percentage: 0.4,
            thumb_fraction: 0.3,
            length: Some(140.0),
            size,
            style: if variant.ghost {
                scrollbar::ScrollbarStyle::Ghost
            } else {
                scrollbar::ScrollbarStyle::Soft
            },
            enabled: !state.disabled,
            state,
        };
        return centered(radix::scrollbar_template(look).render(
            &model,
            preview_handlers::scrollbar_handlers(),
            window,
            cx,
        ));
    }
    if kind == Kind::SelectionPanel {
        return panel(look, &id, size, state, cx);
    }
    let chrome = radix::autocomplete_theme(look).resolve(size);
    let field_state = TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focused,
        invalid: state.invalid,
        ..Default::default()
    };
    let sheet = look.common_stylesheet();
    let popup_look = radix::selector_items_panel_look_for(look, PopupMenuVariant::Solid, Tone::Accent, size);
    let trigger = match kind {
        Kind::Autocomplete => autocomplete::default_autocomplete_template()
            .render(
                autocomplete::AutocompleteRenderModel {
                    icons: sheet.icons.clone(),
                    geometry: sheet.autocomplete.resolve_geometry(size.stylesheet_key(), Default::default()),
                    id: id.clone(),
                    textfield: super::textfields::selection_field(
                        look,
                        &id,
                        size,
                        field_state,
                        !state.disabled,
                        window,
                        cx,
                    ),
                    query_is_empty: false,
                    popup_width: px(preview_width(kind)),
                    status_label: "Searching".into(),
                    status_detail: "3 options".into(),
                    status_color: chrome.status_color,
                    status_typography: chrome.status_typography,
                    muted_text_color: chrome.muted_text_color,
                    popup_bounds: None,
                    popup_look,
                    popup_content: None,
                    presence: OverlayPresence::new(false, false),
                },
                Default::default(),
                window,
                cx,
            )
            .into_any_element(),
        Kind::ComboBox => combobox::default_combobox_template().render(
            combobox::ComboBoxRenderModel {
                icons: sheet.icons.clone(),
                geometry: sheet.combobox.resolve_geometry(size.stylesheet_key(), Default::default()),
                textfield: super::textfields::selection_field(
                    look,
                    &id,
                    size,
                    field_state,
                    !state.disabled,
                    window,
                    cx,
                ),
                query_is_empty: false,
                show_down_arrow: true,
                show_clear_button: true,
                full_width: true,
                minimum_trigger_width: px(preview_width(kind)),
                status_label: "".into(),
                status_detail: "".into(),
                status_color: chrome.status_color,
                muted_text_color: chrome.muted_text_color,
                popup_bounds: None,
                popup_look,
                popup_content: None,
                presence: OverlayPresence::new(false, false),
            },
            Default::default(),
            window,
            cx,
        ),
        Kind::SearchSelector => {
            let theme = radix::selector_theme(look);
            let trigger_style = if variant.ghost {
                selector::SelectorTriggerStyle::Ghost
            } else {
                selector::SelectorTriggerStyle::Outline
            };
            let trigger_look = theme.resolve_visual_look(
                trigger_style,
                selector::SelectorVisualState {
                    interaction: state,
                    open: state.focused,
                    selected: true,
                    invalid: state.invalid,
                },
                size,
                &StandardBoxScale::compute(size, &theme.metrics(), window.scale_factor()),
                false,
            );
            search_selector::default_search_selector_template()
                .render(
                    search_selector::SearchSelectorRenderModel {
                        icons: sheet.icons.clone(),
                        geometry: sheet.search_selector.resolve_geometry(size.stylesheet_key(), Default::default()),
                        id: id.clone(),
                        trigger_label: "Option one".into(),
                        trigger_label_is_placeholder: false,
                        trigger_state: field_state,
                        trigger_look,
                        trigger_typography_override: None,
                        enabled: !state.disabled,
                        size,
                        full_width: true,
                        minimum_trigger_width: px(preview_width(kind)),
                        status_label: "".into(),
                        status_detail: "".into(),
                        status_color: chrome.status_color,
                        muted_text_color: chrome.muted_text_color,
                        popup_content: None,
                        presence: OverlayPresence::new(false, false),
                        disclosure_progress: 0.0,
                    },
                    Default::default(),
                    window,
                    cx,
                )
                .into_any_element()
        }
        _ => unreachable!(),
    };
    // Inline expansion keeps forced popup previews inside their matrix cells.
    let selector = id.to_string();
    div()
        .debug_selector(move || selector.clone())
        .w(px(preview_width(kind)))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(trigger)
        .when(state.focused, |root| {
            root.child(expanded_panel(kind, look, &SharedString::from(format!("{id}-popup")), size, window, cx))
        })
        .into_any_element()
}

fn panel(look: &Look, id: &SharedString, size: ControlSize, state: InteractionState, cx: &mut App) -> AnyElement {
    let items = [
        selection_panel::SelectionPanelItem::new("one").label("Option one"),
        selection_panel::SelectionPanelItem::new("two").label("Option two"),
        selection_panel::SelectionPanelItem::new("disabled").label("Unavailable").enabled(false),
    ];
    let icons = SelectionStatusIcons::default();
    let model = selection_panel::SelectionPanelRenderModel {
        panel_id: id,
        control_id: id,
        items: &items,
        visible_indices: &[0, 1, 2],
        selected_source_index: Some(0),
        active_visible_index: state.focused.then_some(1),
        hovered_visible_index: state.hovered.then_some(1),
        pressed_visible_index: state.pressed.then_some(1),
        open: true,
        enabled: !state.disabled,
        focus: ControlFocusState { focused: state.focused, focus_visible: state.focused },
        item_template: None,
        look: radix::selection_panel_look(look, size),
        show_selection_marker: true,
        icons: &icons,
        show_panel_chrome: true,
    };
    div()
        .w(px(200.0))
        .child(selection_panel::default_selection_panel_template().render(
            &model,
            preview_handlers::selection_panel_handlers(items.len()),
            cx,
        ))
        .into_any_element()
}

/// Use the controls' real item/panel templates with inline placement for matrix cells.
fn expanded_panel(
    kind: Kind,
    look: &Look,
    id: &SharedString,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let items = [
        autocomplete::SelectionItem::new("one", "Option one"),
        autocomplete::SelectionItem::new("two", "Option two"),
        autocomplete::SelectionItem::new("disabled", "Unavailable").enabled(false),
    ];
    let indices = [0, 1, 2];
    let mut panel_look = radix::selector_items_panel_look_for(look, PopupMenuVariant::Solid, Tone::Accent, size);
    if compact_selection(kind) {
        panel_look.min_width = preview_width(kind);
    }
    let sheet = look.common_stylesheet();
    if kind == Kind::SearchSelector {
        let geometry = sheet.search_selector.resolve_geometry(size.stylesheet_key(), Default::default());
        let list = search_selector::default_search_selector_items_template().render(
            &search_selector::SearchSelectorItemsRenderModel {
                trailing_icon_size: geometry.trailing_icon_size,
                menu_id: id,
                search_selector_id: id,
                items: &items,
                visible_indices: &indices,
                selected_source_index: Some(0),
                active_visible_index: Some(1),
                open: true,
                enabled: true,
                item_template: None,
                look: panel_look.clone(),
            },
            search_selector::SearchSelectorItemsTemplateHandlers {
                item_hovers: (0..items.len())
                    .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _)
                    .collect(),
                item_clicks: (0..items.len())
                    .map(|_| Box::new(|_: &gpui::ClickEvent, _: &mut Window, _: &mut App| {}) as _)
                    .collect(),
            },
            cx,
        );
        let field_id = SharedString::from(format!("{id}-search-input"));
        let field =
            super::textfields::selection_field(look, &field_id, size, TextFieldState::default(), true, window, cx);
        return search_selector::default_search_selector_panel_template().render(
            search_selector::SearchSelectorPanelRenderModel {
                id,
                geometry,
                items: &items,
                visible_indices: &indices,
                selected_source_index: Some(0),
                active_visible_index: Some(1),
                open: true,
                enabled: true,
                item_template: None,
                popup_bounds: None,
                popup_look: panel_look,
                search_content: Some(field),
                list_content: list.into_any_element(),
            },
            cx,
        );
    }
    let list = if kind == Kind::Autocomplete {
        let selector_items: Vec<_> = items
            .iter()
            .map(|item| selector::SelectorItem::new(item.id.clone()).label(item.label.clone()).enabled(item.enabled))
            .collect();
        autocomplete::default_autocomplete_items_template()
            .render(
                &autocomplete::AutocompleteItemsRenderModel {
                    id,
                    items: &selector_items,
                    look: panel_look.clone(),
                    highlighted_index: Some(1),
                },
                autocomplete::AutocompleteItemsTemplateHandlers {
                    item_hovers: (0..items.len())
                        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _)
                        .collect(),
                    item_clicks: (0..items.len())
                        .map(|_| Box::new(|_: &gpui::ClickEvent, _: &mut Window, _: &mut App| {}) as _)
                        .collect(),
                },
            )
            .into_any_element()
    } else {
        combobox::default_combobox_items_template()
            .render(
                &combobox::ComboBoxItemsRenderModel {
                    menu_id: id,
                    combobox_id: id,
                    items: &items,
                    visible_indices: &indices,
                    selected_source_index: Some(0),
                    active_visible_index: Some(1),
                    open: true,
                    enabled: true,
                    item_template: None,
                    look: panel_look.clone(),
                },
                combobox::ComboBoxItemsTemplateHandlers {
                    item_hovers: (0..items.len())
                        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as _)
                        .collect(),
                    item_clicks: (0..items.len())
                        .map(|_| Box::new(|_: &gpui::ClickEvent, _: &mut Window, _: &mut App| {}) as _)
                        .collect(),
                },
                cx,
            )
            .into_any_element()
    };
    combobox::default_combobox_panel_template().render(
        combobox::ComboBoxPanelRenderModel {
            id,
            items: &items,
            visible_indices: &indices,
            selected_source_index: Some(0),
            active_visible_index: Some(1),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_look: panel_look,
            list_content: list,
        },
        cx,
    )
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext};
    use radix::SemanticRole;
    struct Harness {
        look: Arc<Look>,
        tabs: Entity<Tabs>,
        kind: Kind,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let colors = [SemanticRole::Foreground, SemanticRole::MutedForeground, SemanticRole::Border]
                .map(|role| self.look.resolve_role(role).hsla());
            tabbed(self.kind, &self.look, self.tabs.clone(), colors, window, cx)
        }
    }
    #[test]
    fn all_autocomplete_states_fit_with_compact_widths() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            let tabs = super::super::state::preview_navigation("autocomplete", &look, cx);
            Harness { look, tabs, kind: Kind::Autocomplete }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(600.0)));
        cx.run_until_parked();
        for selector in [
            "guide-autocomplete-state-0-0",
            "guide-autocomplete-state-0-1",
            "guide-autocomplete-state-0-2",
            "guide-autocomplete-state-0-3",
            "guide-autocomplete-state-0-4",
            "guide-autocomplete-state-0-5",
        ] {
            let bounds = cx.debug_bounds(selector).unwrap();
            assert_eq!(bounds.size.width, px(140.0));
            assert!(bounds.left() >= px(0.0));
            assert!(bounds.right() <= px(1040.0));
        }
    }
    #[test]
    fn combobox_and_search_selector_states_fit_with_compact_widths() {
        for (kind, selectors) in [
            (Kind::ComboBox, ["guide-combobox-state-0-0", "guide-combobox-state-0-2", "guide-combobox-state-0-5"]),
            (
                Kind::SearchSelector,
                [
                    "guide-search-selector-state-0-0",
                    "guide-search-selector-state-0-2",
                    "guide-search-selector-state-1-5",
                ],
            ),
        ] {
            let mut app = TestAppContext::single();
            let (_, cx) = app.add_window_view(|_, cx| {
                let look = Arc::new(Look::built_in());
                let tabs = super::super::state::preview_navigation("compact-selection", &look, cx);
                Harness { look, tabs, kind }
            });
            cx.simulate_resize(gpui::size(px(1040.0), px(800.0)));
            cx.run_until_parked();
            for selector in selectors {
                let bounds = cx.debug_bounds(selector).unwrap();
                assert_eq!(bounds.size.width, px(140.0));
                assert!(bounds.left() >= px(0.0));
                assert!(bounds.right() <= px(1040.0));
            }
        }
    }
}
