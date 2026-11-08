use std::sync::Arc;

use gpui::{
    AnyElement, App, Entity, FontWeight, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px, svg,
};
use xgpui_luma::controls::autocomplete::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplateHandlers, AutocompleteRenderModel,
    AutocompleteTemplateHandlers, default_autocomplete_items_template, default_autocomplete_template,
};
use xgpui_luma::controls::combobox::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, ComboBoxPanelRenderModel,
    ComboBoxPanelTemplate, ComboBoxRenderModel, ComboBoxTemplateHandlers, SelectionItem as ComboBoxSelectionItem,
    default_combobox_items_template, default_combobox_panel_template, default_combobox_template,
};
use xgpui_luma::controls::search_selector::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, SearchSelectorRenderModel,
    SearchSelectorTemplateHandlers, SelectionItem as SearchSelectorSelectionItem,
    default_search_selector_items_template, default_search_selector_panel_template, default_search_selector_template,
};
use xgpui_luma::controls::selector::{
    ControlFocusState, SelectorIcons, SelectorItem, SelectorPath, SelectorPlacement, SelectorRenderModel,
    SelectorTemplateHandlers, SelectorVisualState,
};
use xgpui_luma::motion::overlay_presence::OverlayPresence;
use xgpui_luma::controls::selector_list::{
    SelectorItem as SelectorPanelItem, SelectorItemsPanelLook, SelectorItemsRenderModel, SelectorItemsTemplateHandlers,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, default_selector_items_template,
};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::textfield::{TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTheme};
use xgpui_luma::controls::textfield::TextFieldLook;
use xgpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle, StandardBoxScale};
use xgpui_luma_look_shadcn::stylesheet::{embedded_stylesheet, resolve_button_metrics_rule};
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnSize};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::preview_handlers::{input_noop_click, input_noop_hover, input_textfield_handlers};
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::shared::shadow_matrix::{ShadowPreviewShape, render_shadow_token_matrix};

type SelectorPreviewScrollWheelHandler = Arc<dyn Fn(&gpui::ScrollEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy)]
struct SelectorTemplateStateSample {
    id: &'static str,
    label: &'static str,
    textfield_state: TextFieldState,
    textfield_enabled: bool,
    selector_state: InteractionState,
    selector_focus: ControlFocusState,
    selector_enabled: bool,
    selector_open: bool,
    selector_selected: bool,
}

#[derive(Clone, Copy)]
enum SelectorTemplateControl {
    Autocomplete,
    ComboBox,
    Selector,
    SearchSelector,
}

impl SelectorTemplateControl {
    fn header(self) -> &'static str {
        match self {
            Self::Autocomplete => "Autocomplete",
            Self::ComboBox => "ComboBox",
            Self::Selector => "Selector",
            Self::SearchSelector => "SearchSelector",
        }
    }
}

const SELECTOR_TRIGGER_WIDTH: f32 = 168.0;
const SELECTOR_CELL_WIDTH: f32 = 180.0;

struct SelectorPreviewTextfieldRequest<'a> {
    look: &'a Arc<ShadcnLook>,
    template: &'a Arc<dyn TextFieldTemplate>,
    theme: &'a Arc<dyn TextFieldTheme>,
    id: &'a SharedString,
    placeholder: &'a SharedString,
    value: &'a SharedString,
    state: &'a SelectorTemplateStateSample,
    size: ControlSize,
}

struct SelectorPopupHandlers {
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
}

struct ComboBoxPopupTemplates<'a> {
    items: &'a Arc<dyn ComboBoxItemsTemplate>,
    panel: &'a Arc<dyn ComboBoxPanelTemplate>,
}

struct SearchSelectorPopupTemplates<'a> {
    geometry: xgpui_luma::theme::stylesheet::SearchSelectorGeometry,
    items: &'a Arc<dyn SearchSelectorItemsTemplate>,
    panel: &'a Arc<dyn SearchSelectorPanelTemplate>,
}

fn render_vertical_state_rail(label: &'static str, state_id: &str, label_color: gpui::Hsla) -> AnyElement {
    div()
        .id(format!("luma-studio-choice-template-state-rail-{label}"))
        .w(px(28.0))
        .min_h(px(38.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(svg().path(state_label_asset_path(state_id)).w(px(20.0)).h(px(38.0)).text_color(label_color))
        .child(div().w(px(1.0)).h_full().bg(label_color))
        .into_any_element()
}

fn state_label_asset_path(state_id: &str) -> &'static str {
    match state_id {
        "default" => "assets/labels/default-label.svg",
        "hover" => "assets/labels/hover-label.svg",
        "focused" => "assets/labels/focused-label.svg",
        "open" => "assets/labels/open-label.svg",
        "invalid" => "assets/labels/invalid-label.svg",
        "pressed" => "assets/labels/pressed-label.svg",
        "disabled" => "assets/labels/disabled-label.svg",
        _ => "assets/labels/default-label.svg",
    }
}

pub(crate) fn render_selector_templates_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Selectors",
        "Default, hover, focused, open, pressed, selected, and disabled trigger states with Sm / Md / Lg sizing across selector panels.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_selector_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, scroll_wheel, window, cx),
    )
}

fn selector_preview_typography_for_size(
    look: &Arc<ShadcnLook>,
    base: LumaTextStyle,
    size: ControlSize,
) -> LumaTextStyle {
    let mut typography = base;
    if let Some(rule) = embedded_stylesheet().button.metrics_for_size(size) {
        let tokens = look.mode_tokens();
        let metrics = resolve_button_metrics_rule(rule, &tokens.metrics, size);
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }
    typography
}

fn apply_selector_preview_textfield_size(
    look: &Arc<ShadcnLook>,
    mut textfield_look: TextFieldLook,
    size: ControlSize,
) -> TextFieldLook {
    textfield_look.typography = selector_preview_typography_for_size(look, textfield_look.typography, size);
    textfield_look
}

fn render_selector_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "sizes" => render_selector_sizes_body(&look, scroll_wheel, window, cx),
        "shadows" => render_shadow_token_matrix(&look, ShadowPreviewShape::Selector),
        _ => render_selector_template_preview_body(&look, scroll_wheel, window, cx),
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

fn selector_template_controls() -> [SelectorTemplateControl; 4] {
    [
        SelectorTemplateControl::Autocomplete,
        SelectorTemplateControl::ComboBox,
        SelectorTemplateControl::Selector,
        SelectorTemplateControl::SearchSelector,
    ]
}

fn render_selector_template_preview_body(
    look: &Arc<ShadcnLook>,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let states = selector_template_state_samples();
    let controls = selector_template_controls();

    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(10.0))
        .child(render_selector_header_row(&controls, chrome.muted_text, px(28.0)))
        .children(states.iter().map(|state| {
            render_selector_state_row(look, state, &controls, chrome.muted_text, scroll_wheel.clone(), window, cx)
        }))
        .into_any_element()
}

fn render_selector_header_row(
    controls: &[SelectorTemplateControl],
    label_color: gpui::Hsla,
    rail_width: gpui::Pixels,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(rail_width))
        .children(controls.iter().map(|control| {
            div()
                .w(px(SELECTOR_CELL_WIDTH))
                .flex()
                .justify_center()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(control.header())
        }))
        .into_any_element()
}

fn render_selector_state_row(
    look: &Arc<ShadcnLook>,
    state: &SelectorTemplateStateSample,
    controls: &[SelectorTemplateControl],
    label_color: gpui::Hsla,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .gap(px(10.0))
        .child(render_vertical_state_rail(state.label, state.id, label_color))
        .child(div().flex().items_start().gap(px(10.0)).children(controls.iter().map(|control| {
            render_selector_control_cell(
                look,
                *control,
                state,
                ControlSize::Md,
                state.id == "pressed" || state.selector_open,
                scroll_wheel.clone(),
                window,
                cx,
            )
        })))
        .into_any_element()
}

fn render_selector_sizes_body(
    look: &Arc<ShadcnLook>,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let controls = selector_template_controls();
    let sizes = [(ControlSize::Sm, "Sm"), (ControlSize::Md, "Md"), (ControlSize::Lg, "Lg")];
    let state = SelectorTemplateStateSample {
        id: "size",
        label: "Size",
        textfield_state: TextFieldState {
            hovered: true,
            focused: true,
            focus_visible: true,
            ..TextFieldState::default()
        },
        textfield_enabled: true,
        selector_state: InteractionState { hovered: true, focused: true, pressed: true, ..InteractionState::default() },
        selector_focus: ControlFocusState { focused: true, focus_visible: true },
        selector_enabled: true,
        selector_open: false,
        selector_selected: true,
    };

    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(10.0))
        .child(render_selector_header_row(&controls, chrome.muted_text, px(36.0)))
        .children(sizes.into_iter().map(|(size, label)| {
            div()
                .flex()
                .items_start()
                .gap(px(10.0))
                .child(
                    div()
                        .w(px(36.0))
                        .pt(px(8.0))
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.muted_text)
                        .child(label),
                )
                .child(div().flex().items_start().gap(px(10.0)).children(controls.iter().map(|control| {
                    render_selector_control_cell(look, *control, &state, size, true, scroll_wheel.clone(), window, cx)
                })))
        }))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_selector_control_cell(
    look: &Arc<ShadcnLook>,
    control: SelectorTemplateControl,
    state: &SelectorTemplateStateSample,
    size: ControlSize,
    show_popup: bool,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-selector-controls-template-{}-{}-{:?}",
        state.id,
        control.header(),
        size
    ));

    let trigger = match control {
        SelectorTemplateControl::Autocomplete => render_selector_autocomplete_trigger(
            look,
            &id,
            "Type to filter...",
            state,
            size,
            scroll_wheel.clone(),
            window,
            cx,
        ),
        SelectorTemplateControl::ComboBox => {
            render_selector_combobox_trigger(look, &id, "Select state", state, size, scroll_wheel.clone(), window, cx)
        }
        SelectorTemplateControl::SearchSelector => render_selector_search_selector_trigger(
            look,
            &id,
            "Choose a state...",
            state,
            size,
            scroll_wheel.clone(),
            window,
            cx,
        ),
        SelectorTemplateControl::Selector => render_selector_selector_trigger(look, &id, state, size, window, cx),
    };

    let popup = if show_popup {
        Some(render_selector_popup_preview(
            look,
            &id,
            control,
            state.selector_selected,
            size,
            scroll_wheel.clone(),
            window,
            cx,
        ))
    } else {
        None
    };

    div()
        .w(px(SELECTOR_CELL_WIDTH))
        .flex()
        .flex_col()
        .items_center()
        .justify_start()
        .gap(px(6.0))
        .child(trigger)
        .when_some(popup, |root, popup| root.child(popup))
        .on_scroll(move |event, window, cx| scroll_wheel(event, window, cx))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_selector_autocomplete_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    size: ControlSize,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let textfield_template = look.primary_textfield_template();
    let textfield_theme = look.primary_textfield_theme();
    let autocomplete_template = default_autocomplete_template();
    let value = SharedString::from(if state.selector_selected { "California" } else { "" });
    let placeholder = SharedString::from(placeholder);
    let status_theme = look.autocomplete_theme().resolve(size);
    let popup_look = look.selector_items_panel_look(shadcn_size(size));

    let model = AutocompleteRenderModel {
        icons: look.stylesheet().common.icons.clone(),
        geometry: look.stylesheet().common.autocomplete.resolve_geometry(
            size.stylesheet_key(),
            xgpui_luma::theme::stylesheet::AutocompleteGeometry {
                status_font_size: status_theme.status_typography.size,
                status_line_height: status_theme.status_typography.line_height,
                ..Default::default()
            },
        ),
        status_typography: status_theme.status_typography,
        id: id.clone(),
        textfield: render_selector_preview_textfield(
            SelectorPreviewTextfieldRequest {
                look,
                template: &textfield_template,
                theme: &textfield_theme,
                id,
                placeholder: &placeholder,
                value: &value,
                state,
                size,
            },
            window,
            cx,
        ),
        query_is_empty: !state.selector_selected,
        popup_width: px(SELECTOR_TRIGGER_WIDTH),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: status_theme.status_color,
        muted_text_color: status_theme.muted_text_color,
        popup_bounds: None,
        popup_look,
        popup_content: None,
        presence: OverlayPresence::new(state.selector_open, true),
    };

    div()
        .w(px(SELECTOR_TRIGGER_WIDTH))
        .child(autocomplete_template.render(
            model,
            AutocompleteTemplateHandlers {
                scroll_wheel: Box::new(move |event, window, cx| scroll_wheel(event, window, cx)),
                ..Default::default()
            },
            window,
            cx,
        ))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_selector_combobox_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    size: ControlSize,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let textfield_template = look.primary_textfield_template();
    let textfield_theme = look.primary_textfield_theme();
    let combobox_template = default_combobox_template();
    let value = SharedString::from(if state.selector_selected { "California" } else { "" });
    let placeholder = SharedString::from(placeholder);
    let status_theme = look.autocomplete_theme().resolve(size);
    let popup_look = look.selector_items_panel_look(shadcn_size(size));
    let popup_bounds = (state.id == "pressed" || state.selector_open)
        .then(|| gpui::Bounds::new(gpui::point(px(0.0), px(0.0)), gpui::size(px(SELECTOR_TRIGGER_WIDTH), px(32.0))));
    let popup_open = popup_bounds.is_some();

    let model = ComboBoxRenderModel {
        icons: look.stylesheet().common.icons.clone(),
        geometry: look.stylesheet().common.combobox.resolve_geometry(size.stylesheet_key(), Default::default()),
        textfield: render_selector_preview_textfield(
            SelectorPreviewTextfieldRequest {
                look,
                template: &textfield_template,
                theme: &textfield_theme,
                id,
                placeholder: &placeholder,
                value: &value,
                state,
                size,
            },
            window,
            cx,
        ),
        query_is_empty: !state.selector_selected,
        show_down_arrow: true,
        show_clear_button: true,
        full_width: true,
        minimum_trigger_width: px(SELECTOR_TRIGGER_WIDTH),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: status_theme.status_color,
        muted_text_color: status_theme.muted_text_color,
        popup_bounds,
        popup_look,
        popup_content: None,
        presence: OverlayPresence::new(popup_open, true),
    };

    div()
        .w(px(SELECTOR_TRIGGER_WIDTH))
        .child(combobox_template.render(
            model,
            ComboBoxTemplateHandlers {
                scroll_wheel: Box::new(move |event, window, cx| scroll_wheel(event, window, cx)),
                ..Default::default()
            },
            window,
            cx,
        ))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_selector_search_selector_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    size: ControlSize,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let search_selector_template = default_search_selector_template();
    let selector_theme = look.selector_theme();
    let interaction = InteractionState {
        hovered: state.textfield_state.hovered,
        focused: state.textfield_state.focused || state.textfield_state.focus_visible,
        disabled: !state.textfield_enabled,
        ..InteractionState::default()
    };
    let trigger_look = selector_theme.resolve_visual_look(
        Default::default(),
        SelectorVisualState {
            interaction,
            open: state.selector_open,
            selected: state.selector_selected,
            invalid: state.selector_state.invalid,
        },
        size,
        &StandardBoxScale::compute(size, &selector_theme.metrics(), window.scale_factor()),
        false,
    );
    let model = SearchSelectorRenderModel {
        icons: look.stylesheet().common.icons.clone(),
        geometry: look.stylesheet().common.search_selector.resolve_geometry(size.stylesheet_key(), Default::default()),
        id: id.clone(),
        trigger_label: if state.selector_selected {
            SharedString::from("California")
        } else {
            SharedString::from(placeholder)
        },
        trigger_label_is_placeholder: !state.selector_selected,
        trigger_state: state.textfield_state,
        trigger_look,
        trigger_typography_override: Some(selector_preview_typography_for_size(
            look,
            look.mode_tokens().typography.text.body,
            size,
        )),
        enabled: state.textfield_enabled,
        size,
        full_width: true,
        minimum_trigger_width: px(SELECTOR_TRIGGER_WIDTH),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: look.chrome().muted_text,
        muted_text_color: look.chrome().muted_text,
        popup_content: None,
        presence: OverlayPresence::new(state.selector_open, true),
        disclosure_progress: if state.selector_open { 1.0 } else { 0.0 },
    };

    div()
        .w(px(SELECTOR_TRIGGER_WIDTH))
        .child(search_selector_template.render(
            model,
            SearchSelectorTemplateHandlers {
                scroll_wheel: Box::new(move |event, window, cx| scroll_wheel(event, window, cx)),
                ..Default::default()
            },
            window,
            cx,
        ))
        .into_any_element()
}

fn render_selector_selector_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    state: &SelectorTemplateStateSample,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selector_template = look.selector_template();
    let label = SharedString::from("Select state");
    let items = selector_trigger_items().into_iter().collect::<Vec<_>>();
    let model = SelectorRenderModel {
        popup_scroll: None,
        id,
        label: &label,
        selected_index: state.selector_selected.then_some(1),
        items: &items,
        // The open sample renders its panel inline in `render_selector_control_cell`.
        // Keep the trigger's visual state open without asking the SDK template to
        // create a second anchored overlay with no trigger bounds.
        open: false,
        trigger_bounds: None,
        placement: SelectorPlacement::BelowStart,
        opening_mode: Default::default(),
        popup_geometry: None,
        initialize_popup_scroll: false,
        active_path: None,
        enabled: state.selector_enabled,
        size,
        trigger_style: Default::default(),
        icons: &SelectorIcons::default(),
        without_elevation: false,
        item_template: None,
        panel_template: None,
        focus: state.selector_focus,
        state: state.selector_state,
        visual_state: xgpui_luma::controls::selector::SelectorVisualState {
            interaction: state.selector_state,
            open: state.selector_open,
            selected: state.selector_selected,
            invalid: state.selector_state.invalid,
        },
        presence: OverlayPresence::new(false, true),
    };

    selector_template
        .render(&model, SelectorTemplateHandlers::default(), window, cx)
        .w(px(SELECTOR_TRIGGER_WIDTH))
        .into_any_element()
}

fn render_selector_preview_textfield(
    request: SelectorPreviewTextfieldRequest<'_>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let SelectorPreviewTextfieldRequest { look, template, theme, id, placeholder, value, state, size } = request;
    let character_offsets = selector_textfield_character_offsets(
        look,
        value.as_ref(),
        theme.clone(),
        state.textfield_state,
        state.textfield_enabled,
        size,
        window,
    );

    let look =
        selector_preview_textfield_look(look, theme, state.textfield_state, state.textfield_enabled, size, window);
    let text_model = TextFieldRenderModel {
        id,
        placeholder,
        value,
        prefix_icon: None,
        size,
        enabled: state.textfield_enabled,
        full_width: true,
        state: state.textfield_state,
        marked_range: None,
        caret_visible: false,
        horizontal_scroll: 0.0,
        character_offsets,
        look,
    };

    template.render(&text_model, input_textfield_handlers(), window, cx).into_any_element()
}

fn render_selector_popup_preview(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    control: SelectorTemplateControl,
    selected: bool,
    size: ControlSize,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let popup_look = look.selector_items_panel_look(shadcn_size(size));
    let popup_id = SharedString::from(format!("{id}-popup-preview"));
    let items = selector_popup_items_for_control(control);
    let item_hovers = (0..items.len())
        .map(|_| Box::new(input_noop_hover) as SelectorPanelHoverHandler)
        .collect::<Vec<_>>();
    let item_clicks = (0..items.len())
        .map(|_| Box::new(input_noop_click) as SelectorPanelClickHandler)
        .collect::<Vec<_>>();

    let rows: AnyElement = match control {
        SelectorTemplateControl::Autocomplete => default_autocomplete_items_template()
            .render(
                &AutocompleteItemsRenderModel {
                    id: &popup_id,
                    items: &items,
                    look: popup_look.clone(),
                    highlighted_index: Some(0),
                },
                AutocompleteItemsTemplateHandlers { item_hovers, item_clicks },
            )
            .on_scroll({
                let scroll_wheel = scroll_wheel.clone();
                move |event, window, cx| scroll_wheel(event, window, cx)
            })
            .into_any_element(),
        SelectorTemplateControl::ComboBox => render_selector_combobox_popup_preview_from_templates(
            &popup_id,
            &items,
            popup_look.clone(),
            ComboBoxPopupTemplates {
                items: &default_combobox_items_template(),
                panel: &default_combobox_panel_template(),
            },
            SelectorPopupHandlers { item_hovers, item_clicks },
            scroll_wheel.clone(),
            cx,
        ),
        SelectorTemplateControl::Selector => default_selector_items_template()
            .render(
                &SelectorItemsRenderModel {
                    scroll_handle: None,
                    menu_id: &popup_id,
                    selector_id: id,
                    items: &items,
                    selected_index: selected.then_some(1),
                    active_path: Some(SelectorPath::Item(0)),
                    open: true,
                    enabled: true,
                    focus: ControlFocusState { focused: true, focus_visible: true },
                    item_template: None,
                    look: popup_look.clone(),
                    max_height: px(240.0),
                    scrolling: true,
                    selection_icon: &SelectorIcons::default().selected,
                },
                SelectorItemsTemplateHandlers { item_hovers, item_clicks, ..Default::default() },
                cx,
            )
            .on_scroll({
                let scroll_wheel = scroll_wheel.clone();
                move |event, window, cx| scroll_wheel(event, window, cx)
            })
            .into_any_element(),
        SelectorTemplateControl::SearchSelector => {
            let search_id = SharedString::from(format!("{id}-popup-search-preview"));
            let search_placeholder = SharedString::from("Selection search");
            let search_value = SharedString::from("");
            let search_offsets = selector_textfield_character_offsets(
                look,
                search_value.as_ref(),
                look.input_textfield_theme(),
                TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
                true,
                size,
                window,
            );
            let search_state = TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() };
            let search_look =
                selector_preview_textfield_look(look, &look.input_textfield_theme(), search_state, true, size, window);
            let search_model = TextFieldRenderModel {
                id: &search_id,
                placeholder: &search_placeholder,
                value: &search_value,
                prefix_icon: None,
                size,
                enabled: true,
                full_width: true,
                state: search_state,
                marked_range: None,
                caret_visible: false,
                horizontal_scroll: 0.0,
                character_offsets: search_offsets,
                look: search_look,
            };

            let search_content = look
                .input_textfield_template()
                .render(&search_model, input_textfield_handlers(), window, cx)
                .on_scroll({
                    let scroll_wheel = scroll_wheel.clone();
                    move |event, window, cx| scroll_wheel(event, window, cx)
                })
                .into_any_element();

            render_selector_search_selector_popup_preview_from_templates(
                &popup_id,
                &items,
                popup_look.clone(),
                SearchSelectorPopupTemplates {
                    geometry: look
                        .stylesheet()
                        .common
                        .search_selector
                        .resolve_geometry(size.stylesheet_key(), Default::default()),
                    items: &default_search_selector_items_template(),
                    panel: &default_search_selector_panel_template(),
                },
                search_content,
                SelectorPopupHandlers { item_hovers, item_clicks },
                scroll_wheel.clone(),
                cx,
            )
        }
    };

    div()
        .w(px(popup_look.min_width))
        .on_scroll(move |event, window, cx| scroll_wheel(event, window, cx))
        .child(rows)
        .into_any_element()
}

fn selector_template_state_samples() -> [SelectorTemplateStateSample; 7] {
    [
        SelectorTemplateStateSample {
            id: "default",
            label: "Default",
            textfield_state: TextFieldState::default(),
            textfield_enabled: true,
            selector_state: InteractionState::default(),
            selector_focus: ControlFocusState::default(),
            selector_enabled: true,
            selector_open: false,
            selector_selected: false,
        },
        SelectorTemplateStateSample {
            id: "hover",
            label: "Hover",
            textfield_state: TextFieldState { hovered: true, ..TextFieldState::default() },
            textfield_enabled: true,
            selector_state: InteractionState { hovered: true, ..InteractionState::default() },
            selector_focus: ControlFocusState::default(),
            selector_enabled: true,
            selector_open: false,
            selector_selected: false,
        },
        SelectorTemplateStateSample {
            id: "focused",
            label: "Focused",
            textfield_state: TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
            textfield_enabled: true,
            selector_state: InteractionState { focused: true, ..InteractionState::default() },
            selector_focus: ControlFocusState { focused: true, focus_visible: true },
            selector_enabled: true,
            selector_open: false,
            selector_selected: true,
        },
        SelectorTemplateStateSample {
            id: "open",
            label: "Open",
            textfield_state: TextFieldState {
                hovered: true,
                focused: true,
                focus_visible: true,
                ..TextFieldState::default()
            },
            textfield_enabled: true,
            selector_state: InteractionState { focused: true, ..InteractionState::default() },
            selector_focus: ControlFocusState { focused: true, focus_visible: true },
            selector_enabled: true,
            selector_open: true,
            selector_selected: true,
        },
        SelectorTemplateStateSample {
            id: "invalid",
            label: "Invalid",
            textfield_state: TextFieldState { invalid: true, ..TextFieldState::default() },
            textfield_enabled: true,
            selector_state: InteractionState { invalid: true, ..InteractionState::default() },
            selector_focus: ControlFocusState::default(),
            selector_enabled: true,
            selector_open: false,
            selector_selected: false,
        },
        SelectorTemplateStateSample {
            id: "pressed",
            label: "Pressed",
            textfield_state: TextFieldState {
                hovered: true,
                focused: true,
                focus_visible: true,
                ..TextFieldState::default()
            },
            textfield_enabled: true,
            selector_state: InteractionState {
                hovered: true,
                focused: true,
                pressed: true,
                ..InteractionState::default()
            },
            selector_focus: ControlFocusState { focused: true, focus_visible: true },
            selector_enabled: true,
            selector_open: false,
            selector_selected: true,
        },
        SelectorTemplateStateSample {
            id: "disabled",
            label: "Disabled",
            textfield_state: TextFieldState::default(),
            textfield_enabled: false,
            selector_state: InteractionState { disabled: true, ..InteractionState::default() },
            selector_focus: ControlFocusState::default(),
            selector_enabled: false,
            selector_open: false,
            selector_selected: false,
        },
    ]
}

fn selector_trigger_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

fn selector_popup_items_for_control(control: SelectorTemplateControl) -> Vec<SelectorPanelItem> {
    match control {
        SelectorTemplateControl::Autocomplete => vec![
            SelectorPanelItem::new("autocomplete-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("autocomplete-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("autocomplete-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::ComboBox => vec![
            SelectorPanelItem::new("combobox-preview-item-1").label("California"),
            SelectorPanelItem::new("combobox-preview-item-2").label("Colorado"),
            SelectorPanelItem::new("combobox-preview-item-3").label("Connecticut"),
        ],
        SelectorTemplateControl::Selector => vec![
            SelectorPanelItem::new("selector-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("selector-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("selector-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::SearchSelector => vec![
            SelectorPanelItem::new("search-selector-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("search-selector-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("search-selector-preview-item-3").label("Arizona"),
        ],
    }
}

fn selector_textfield_character_offsets(
    look: &Arc<ShadcnLook>,
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &mut Window,
) -> Vec<f32> {
    let look = selector_preview_textfield_look(look, &theme, state, enabled, size, window);
    let value_shared = SharedString::from(value.to_string());
    let run = TextRun {
        len: value_shared.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = look.typography.weight;
            font
        },
        color: look.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(value_shared, px(look.typography.size), &[run], None);
    let chars = value.chars().count();

    let mut offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = selector_char_to_byte_offset(value, char_offset);
        offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    offsets
}

fn selector_char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    if char_offset == 0 {
        return 0;
    }

    text.char_indices().nth(char_offset).map_or(text.len(), |(byte, _)| byte)
}

fn selector_preview_textfield_look(
    look: &Arc<ShadcnLook>,
    theme: &Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &Window,
) -> xgpui_luma::controls::textfield::TextFieldLook {
    let scale = StandardBoxScale::compute(size, &theme.metrics(), window.scale_factor());
    apply_selector_preview_textfield_size(look, theme.resolve_look(state, enabled, size, &scale), size)
}

fn shadcn_size(size: ControlSize) -> ShadcnSize {
    match size {
        ControlSize::Sm => ShadcnSize::Sm,
        ControlSize::Md => ShadcnSize::Md,
        ControlSize::Lg => ShadcnSize::Lg,
    }
}

fn render_selector_combobox_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    look: SelectorItemsPanelLook,
    templates: ComboBoxPopupTemplates<'_>,
    handlers: SelectorPopupHandlers,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    cx: &mut App,
) -> AnyElement {
    let SelectorPopupHandlers { item_hovers, item_clicks } = handlers;
    let combobox_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            ComboBoxSelectionItem::new(format!("combobox-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..combobox_items.len()).collect::<Vec<_>>();

    let list = templates
        .items
        .render(
            &ComboBoxItemsRenderModel {
                menu_id: popup_id,
                combobox_id: popup_id,
                items: &combobox_items,
                visible_indices: &visible_indices,
                selected_source_index: None,
                active_visible_index: Some(0),
                open: true,
                enabled: true,
                item_template: None,
                look: look.clone(),
            },
            ComboBoxItemsTemplateHandlers { item_hovers, item_clicks },
            cx,
        )
        .on_scroll({
            let scroll_wheel = scroll_wheel.clone();
            move |event, window, cx| scroll_wheel(event, window, cx)
        });

    templates.panel.render(
        ComboBoxPanelRenderModel {
            id: popup_id,
            items: &combobox_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_look: look,
            list_content: list.into_any_element(),
        },
        cx,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_selector_search_selector_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    look: SelectorItemsPanelLook,
    templates: SearchSelectorPopupTemplates<'_>,
    search_content: AnyElement,
    handlers: SelectorPopupHandlers,
    scroll_wheel: SelectorPreviewScrollWheelHandler,
    cx: &mut App,
) -> AnyElement {
    let SelectorPopupHandlers { item_hovers, item_clicks } = handlers;
    let search_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            SearchSelectorSelectionItem::new(format!("search-selector-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..search_items.len()).collect::<Vec<_>>();

    let rows = templates
        .items
        .render(
            &SearchSelectorItemsRenderModel {
                trailing_icon_size: templates.geometry.trailing_icon_size,
                menu_id: popup_id,
                search_selector_id: popup_id,
                items: &search_items,
                visible_indices: &visible_indices,
                selected_source_index: None,
                active_visible_index: Some(0),
                open: true,
                enabled: true,
                item_template: None,
                look: look.clone(),
            },
            SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks },
            cx,
        )
        .on_scroll({
            let scroll_wheel = scroll_wheel.clone();
            move |event, window, cx| scroll_wheel(event, window, cx)
        })
        .into_any_element();

    templates.panel.render(
        SearchSelectorPanelRenderModel {
            geometry: templates.geometry,
            id: popup_id,
            items: &search_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_look: look,
            search_content: Some(search_content),
            list_content: rows,
        },
        cx,
    )
}
