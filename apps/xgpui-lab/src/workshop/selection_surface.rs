//! SDK selection controls on the design surface page (issue #383).
//!
//! - **Selector** (`selector`, which extends `button`) and **Split Button** (`popup_menu`, which extends
//!   `button`) take a theme, so the page binds their `.lss` and Design mode pins the picked states.
//! - **Autocomplete** (`autocomplete`) and **ComboBox** (`combobox`) take no theme of their own, but their
//!   hosted text field does: the page binds and pins the field (`textfield.lss`, shown beside their `.lss`).
//! - **SearchSelector** (`search_selector`): the page binds and pins its trigger (`selector.lss`) and binds
//!   its popup's search field (`textfield.lss`).
//! - **Selection Panel** (`selection_panel`, which extends `floating_menu`): the page binds its look and pins
//!   `:disabled`; a row's `:hover` is per-item state (#397 item 18).
//!
//! Each State matrix cell is a small control of its own, drawn in that cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::context_menu::MenuItem;
use xgpui_luma::controls::popup_menu::{PopupMenuTheme, StyledPopupMenuTheme};
use xgpui_luma::controls::selector::{
    Selector, SelectorBuilder, SelectorEvent, SelectorItem, SelectorTheme, StyledSelectorTheme, ThemedSelectorTemplate,
};
use xgpui_luma::controls::split_button::{SplitButton, SplitButtonBuilder, SplitButtonEvent, ThemedSplitButtonTemplate};
use xgpui_luma::controls::floating_menu::floating_menu_source;
use xgpui_luma::controls::{autocomplete, combobox, search_selector, selection_panel, selector_list};
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::theme::MetricTokens;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use super::textfield_surface::{PinnedTextFieldTemplate, STATES as TEXTFIELD_STATES, template as textfield_template};
use crate::theme::Look;
use xgpui_luma::controls::textfield::TextFieldTemplate;

/// The options: id, label (with ascenders and descenders) and whether it can be chosen.
const ITEMS: [(&str, &str, bool); 5] = [
    ("typography", "Typography", true),
    ("display", "Display", true),
    ("legacy", "Legacy", false),
    ("spacing", "Spacing", true),
    ("quality", "Quality", true),
];

fn filter_items() -> Vec<autocomplete::SelectionItem> {
    ITEMS
        .iter()
        .map(|&(id, label, enabled)| autocomplete::SelectionItem::new(id, label).enabled(enabled))
        .collect()
}

/// Matrix cells, made with the theme last bound and cleared when the page binds again.
#[derive(Default)]
struct Cells(RefCell<HashMap<SharedString, AnyView>>);

impl Cells {
    fn get(&self, id: &SharedString, make: impl FnOnce() -> AnyView) -> AnyView {
        self.0.borrow_mut().entry(id.clone()).or_insert_with(make).clone()
    }

    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// --- Selector ----------------------------------------------------------------------------------------------

struct PinnedSelectorTheme {
    base: StyledSelectorTheme,
    pinned: Option<StateSet>,
}

impl SelectorTheme for PinnedSelectorTheme {
    /// The base style with the pinned states set in its key (#433), so Design mode draws them however the
    /// pointer moves.
    fn style(&self) -> StyleBinding {
        let base = self.base.style();
        let Some(pinned) = &self.pinned else {
            return base;
        };
        ["hover", "pressed", "focused", "disabled", "invalid"]
            .into_iter()
            .fold(base, |binding, name| binding.state(name, pinned.has(name)))
    }

    fn menu(&self) -> xgpui_luma::style::StyleBinding {
        self.base.menu()
    }

    fn metrics(&self) -> MetricTokens {
        self.base.metrics()
    }
}

fn selector_template(theme: Theme, pinned: Option<StateSet>) -> Arc<ThemedSelectorTemplate<SelectorItem>> {
    let panel = selector_list::selector_items_panel_source(ThemeSource::Fixed(theme.clone()));
    let base = StyledSelectorTheme::new(ThemeSource::Fixed(theme), panel);
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(PinnedSelectorTheme { base, pinned }),
        selector_list::default_selector_items_template(),
    ))
}

fn selector(id: impl Into<SharedString>) -> SelectorBuilder<SelectorItem> {
    SelectorBuilder::new(id)
        .label("Pick a style")
        .items(ITEMS.iter().map(|&(id, label, enabled)| SelectorItem::new(id).label(label).enabled(enabled)))
}

pub(super) struct SelectorSurface {
    selector: Entity<Selector>,
    cells: Cells,
}

impl SelectorSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { selector: selector("workshop-sdk-selector").spawn(cx), cells: Cells::default() })
    }

    /// The SDK selector, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn selector(&self) -> Entity<Selector> {
        self.selector.clone()
    }
}

impl SurfaceControl for SelectorSurface {
    fn title(&self) -> &'static str {
        "Selector"
    }

    fn view(&self) -> AnyView {
        self.selector.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = selector_template(theme, pinned.cloned());
        self.selector.update(cx, |selector, cx| selector.set_template(template, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.selector.update(cx, |selector, cx| selector.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["focused"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.selector, move |side: &mut ControlSide, _, event: &SelectorEvent, cx| {
            let state = match event {
                SelectorEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            selector(id.clone())
                .template(selector_template(theme.clone(), Some(states.clone())))
                .spawn(cx)
                .into()
        });
        div().w(px(200.)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Split Button ------------------------------------------------------------------------------------------

struct PinnedPopupMenuTheme {
    base: StyledPopupMenuTheme,
    pinned: Option<StateSet>,
}

impl PopupMenuTheme for PinnedPopupMenuTheme {
    /// The base style with the pinned states set in its key (#433), so Design mode draws them however the
    /// pointer moves.
    fn style(&self) -> StyleBinding {
        let base = self.base.style();
        let Some(pinned) = &self.pinned else {
            return base;
        };
        ["hover", "pressed", "focused", "disabled", "invalid"]
            .into_iter()
            .fold(base, |binding, name| binding.state(name, pinned.has(name)))
    }

    fn menu(&self) -> StyleBinding {
        self.base.menu()
    }

    fn metrics(&self) -> MetricTokens {
        self.base.metrics()
    }
}

/// The `popup_menu` control in `theme`, its trigger in the picked states when some are pinned. Split Button
/// and Popup Menu share it.
pub(super) fn popup_menu_theme(theme: Theme, pinned: Option<StateSet>) -> Arc<dyn PopupMenuTheme> {
    let menu = floating_menu_source(ThemeSource::Fixed(theme.clone()));
    let base = StyledPopupMenuTheme::new(ThemeSource::Fixed(theme), menu);
    Arc::new(PinnedPopupMenuTheme { base, pinned })
}

fn split_button_template(theme: Theme, pinned: Option<StateSet>) -> Arc<ThemedSplitButtonTemplate> {
    Arc::new(ThemedSplitButtonTemplate::new(popup_menu_theme(theme, pinned)))
}

fn split_button(id: impl Into<SharedString>) -> SplitButtonBuilder {
    SplitButtonBuilder::new(id)
        .label("Apply styling")
        .items(ITEMS.iter().map(|&(id, label, enabled)| MenuItem::new(id).label(label).enabled(enabled)))
}

pub(super) struct SplitButtonSurface {
    button: Entity<SplitButton>,
    cells: Cells,
}

impl SplitButtonSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { button: split_button("workshop-sdk-split-button").spawn(cx), cells: Cells::default() })
    }

    /// The SDK split button, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn button(&self) -> Entity<SplitButton> {
        self.button.clone()
    }
}

impl SurfaceControl for SplitButtonSurface {
    fn title(&self) -> &'static str {
        "Split Button"
    }

    fn view(&self) -> AnyView {
        self.button.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = split_button_template(theme, pinned.cloned());
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["focused"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.button, move |side: &mut ControlSide, _, event: &SplitButtonEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, SplitButtonEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                SplitButtonEvent::HoverChanged { hovered } => Some(("hover", *hovered)),
                SplitButtonEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            split_button(id.clone())
                .template(split_button_template(theme.clone(), Some(states.clone())))
                .spawn(cx)
                .into()
        });
        div().child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Autocomplete ------------------------------------------------------------------------------------------

/// The SDK autocomplete template without its status line: the line (search status, matches, last event)
/// is left out of the drawing and kept in `status` for the page's lower panel.
struct StatusAside {
    base: Arc<dyn autocomplete::AutocompleteTemplate>,
    status: Arc<Mutex<SharedString>>,
}

impl StatusAside {
    fn wrap(status: Arc<Mutex<SharedString>>) -> Arc<dyn autocomplete::AutocompleteTemplate> {
        Arc::new(Self { base: Arc::new(autocomplete::DefaultAutocompleteTemplate::new()), status })
    }
}

impl autocomplete::AutocompleteTemplate for StatusAside {
    fn render(
        &self,
        mut model: autocomplete::AutocompleteRenderModel,
        handlers: autocomplete::AutocompleteTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        if let Ok(mut status) = self.status.lock() {
            *status = format!("{} · {}", model.status_label, model.status_detail).into();
        }
        model.status_label = SharedString::default();
        model.status_detail = SharedString::default();
        self.base.render(model, handlers, window, cx)
    }
}

// --- Autocomplete ------------------------------------------------------------------------------------------

/// Autocomplete's own `.lss` has only its status line, clear button and layout; its states are the hosted
/// text field's (`textfield.lss`, shown beside it). The page binds and pins that field's template.
pub(super) struct AutocompleteSurface {
    control: Entity<autocomplete::AutocompleteControl>,
    cells: Cells,
    /// The status text the control would draw above itself, shown on the lower panel instead.
    status: Arc<Mutex<SharedString>>,
}

/// The hosted text field's template for the `textfield` control in `theme`, drawn in `pinned` when set.
fn autocomplete_field(theme: Theme, pinned: Option<StateSet>) -> Arc<dyn TextFieldTemplate> {
    Arc::new(PinnedTextFieldTemplate { base: textfield_template(theme), pinned })
}

/// The `autocomplete` control in `theme`, its popup the theme's floating menu.
fn autocomplete_theme(theme: Theme) -> Arc<dyn autocomplete::AutocompleteTheme> {
    let binding = xgpui_luma::style::StyleBinding::new("autocomplete", ThemeSource::Fixed(theme.clone()));
    Arc::new(autocomplete::StyledAutocompleteTheme::new(binding, floating_menu_source(ThemeSource::Fixed(theme))))
}

impl AutocompleteSurface {
    const WIDTH: f32 = 260.;

    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let status = Arc::new(Mutex::new(SharedString::default()));
        let control = autocomplete::AutocompleteBuilder::new("workshop-sdk-autocomplete", filter_items())
            .template(StatusAside::wrap(status.clone()))
            .debug_status(true)
            .spawn(cx);
        Box::new(Self { control, cells: Cells::default(), status })
    }
}

impl SurfaceControl for AutocompleteSurface {
    fn title(&self) -> &'static str {
        "Autocomplete TextBox"
    }

    fn view(&self) -> AnyView {
        self.control.clone().into()
    }

    /// The hosted field's `textfield.lss` draws its states.
    fn composed(&self) -> &'static [&'static str] {
        &["textfield"]
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.cells.clear();
        let field = autocomplete_field(theme.clone(), pinned.cloned());
        self.control.update(cx, |control, cx| {
            control.set_autocomplete_theme(autocomplete_theme(theme.clone()), cx);
            control.set_popup_menu(selector_list::selector_items_panel_source(ThemeSource::Fixed(theme)), cx);
            control.set_textfield_template(field, cx);
        });
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.control.update(cx, |control, cx| control.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        TEXTFIELD_STATES.to_vec()
    }

    fn status(&self) -> Option<SharedString> {
        self.status.lock().ok().map(|status| status.clone())
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.control, move |side: &mut ControlSide, _, event: &autocomplete::AutocompleteEvent, cx| {
            let state = match event {
                autocomplete::AutocompleteEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// An autocomplete of its own, its field drawn in the cell's states.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            autocomplete::AutocompleteBuilder::new(id.clone(), filter_items())
                .autocomplete_theme(autocomplete_theme(theme.clone()))
                .popup_menu(selector_list::selector_items_panel_source(ThemeSource::Fixed(theme.clone())))
                .textfield_template(autocomplete_field(theme.clone(), Some(states.clone())))
                .enabled(!states.has("disabled"))
                .spawn(cx)
                .into()
        });
        div().w(px(Self::WIDTH)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- ComboBox ----------------------------------------------------------------------------------------------

/// ComboBox's own `.lss` has only the open affordance's layout; its states are the hosted text field's
/// (`textfield.lss`, shown beside it). The page binds and pins that field's template, like Autocomplete's.
pub(super) struct ComboBoxSurface {
    control: Entity<combobox::ComboBoxControl>,
    cells: Cells,
}

impl ComboBoxSurface {
    const WIDTH: f32 = 260.;

    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let control = combobox::ComboBoxBuilder::new("workshop-sdk-combobox", filter_items()).spawn(cx);
        Box::new(Self { control, cells: Cells::default() })
    }
}

impl SurfaceControl for ComboBoxSurface {
    fn title(&self) -> &'static str {
        "ComboBox"
    }

    fn view(&self) -> AnyView {
        self.control.clone().into()
    }

    /// The hosted field's `textfield.lss` draws its states.
    fn composed(&self) -> &'static [&'static str] {
        &["textfield"]
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.cells.clear();
        let field = autocomplete_field(theme, pinned.cloned());
        self.control.update(cx, |control, cx| control.set_textfield_template(field, cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.control.update(cx, |control, cx| control.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        TEXTFIELD_STATES.to_vec()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.control, move |side: &mut ControlSide, _, event: &combobox::ComboBoxEvent, cx| {
            let state = match event {
                combobox::ComboBoxEvent::FocusChanged { focused } => Some(("focused", *focused)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// A combobox of its own, its field drawn in the cell's states.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            combobox::ComboBoxBuilder::new(id.clone(), filter_items())
                .textfield_template(autocomplete_field(theme.clone(), Some(states.clone())))
                .enabled(!states.has("disabled"))
                .spawn(cx)
                .into()
        });
        div().w(px(Self::WIDTH)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- SearchSelector ----------------------------------------------------------------------------------------

/// The trigger states SearchSelector draws through its selector theme (`selector.lss`).
const SEARCH_SELECTOR_STATES: [&str; 5] = ["hover", "pressed", "focused", "invalid", "disabled"];

/// SearchSelector's own `.lss` has only layout; its trigger is a selector (`selector.lss`) and its popup's
/// search field a text field (`textfield.lss`), both shown beside it. The page binds and pins the trigger and
/// binds the search field.
pub(super) struct SearchSelectorSurface {
    control: Entity<search_selector::SearchSelectorControl>,
    cells: Cells,
}

/// Draws the SDK template with the picked states in the trigger's state (Design mode), or as the control is.
/// The theme colors the trigger; the template decides from this state whether the focus ring shows.
struct PinnedSearchSelectorTemplate {
    pinned: Option<StateSet>,
}

impl search_selector::SearchSelectorTemplate for PinnedSearchSelectorTemplate {
    fn render(
        &self,
        mut model: search_selector::SearchSelectorRenderModel,
        handlers: search_selector::SearchSelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        if let Some(states) = &self.pinned {
            let state = &mut model.trigger_state;
            state.hovered = states.has("hover");
            state.focused = states.has("focused");
            state.focus_visible = state.focused;
            state.invalid = states.has("invalid");
            model.enabled = !states.has("disabled");
        }
        search_selector::default_search_selector_template().render(model, handlers, window, cx)
    }
}

/// The trigger's theme for the `selector` control in `theme`, drawn in `pinned` when set.
fn search_selector_trigger(theme: Theme, pinned: Option<StateSet>) -> Arc<dyn SelectorTheme> {
    let panel = selector_list::selector_items_panel_source(ThemeSource::Fixed(theme.clone()));
    Arc::new(PinnedSelectorTheme { base: StyledSelectorTheme::new(ThemeSource::Fixed(theme), panel), pinned })
}

impl SearchSelectorSurface {
    const WIDTH: f32 = 260.;

    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let control =
            search_selector::SearchSelectorBuilder::new("workshop-sdk-search-selector", filter_items()).spawn(cx);
        Box::new(Self { control, cells: Cells::default() })
    }
}

impl SurfaceControl for SearchSelectorSurface {
    fn title(&self) -> &'static str {
        "SearchSelector"
    }

    fn view(&self) -> AnyView {
        self.control.clone().into()
    }

    /// The trigger's `selector.lss` draws its states; the popup's search field reads `textfield.lss`.
    fn composed(&self) -> &'static [&'static str] {
        &["selector", "textfield"]
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.cells.clear();
        let trigger = search_selector_trigger(theme.clone(), pinned.cloned());
        let field = autocomplete_field(theme.clone(), None);
        self.control.update(cx, |control, cx| {
            control.set_selector_theme(trigger, cx);
            control.set_popup_menu(selector_list::selector_items_panel_source(ThemeSource::Fixed(theme)), cx);
            control.set_textfield_template(field, cx);
            control.set_template(Arc::new(PinnedSearchSelectorTemplate { pinned: pinned.cloned() }), cx);
        });
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.control.update(cx, |control, cx| control.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        SEARCH_SELECTOR_STATES.to_vec()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(
            &self.control,
            move |side: &mut ControlSide, _, event: &search_selector::SearchSelectorEvent, cx| {
                let state = match event {
                    search_selector::SearchSelectorEvent::FocusChanged { focused } => Some(("focused", *focused)),
                    _ => None,
                };
                on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
            },
        )
    }

    /// A search selector of its own, its trigger drawn in the cell's states.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            search_selector::SearchSelectorBuilder::new(id.clone(), filter_items())
                .selector_theme(search_selector_trigger(theme.clone(), Some(states.clone())))
                .popup_menu(selector_list::selector_items_panel_source(ThemeSource::Fixed(theme.clone())))
                .template(Arc::new(PinnedSearchSelectorTemplate { pinned: Some(states.clone()) }))
                .textfield_template(autocomplete_field(theme.clone(), None))
                .enabled(!states.has("disabled"))
                .spawn(cx)
                .into()
        });
        div().w(px(Self::WIDTH)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- SelectionPanel ----------------------------------------------------------------------------------------

/// The panel's style: the `selection_panel` control in `theme` (the floating menu's surface and rows).
fn selection_panel_menu(theme: Theme) -> xgpui_luma::controls::popup_menu::MenuSource {
    let binding = xgpui_luma::style::StyleBinding::new("selection_panel", ThemeSource::Fixed(theme));
    Arc::new(move || binding.clone())
}

/// Draws the SDK panel template enabled or disabled as Design mode picks it (Design disables the live panel's
/// input, which would otherwise draw its rows disabled), or as the panel is.
struct PinnedSelectionPanelTemplate {
    base: Arc<dyn selection_panel::SelectionPanelTemplate<selection_panel::SelectionPanelItem>>,
    pinned: Option<StateSet>,
}

impl selection_panel::SelectionPanelTemplate<selection_panel::SelectionPanelItem> for PinnedSelectionPanelTemplate {
    fn render(
        &self,
        model: &selection_panel::SelectionPanelRenderModel<'_, selection_panel::SelectionPanelItem>,
        handlers: selection_panel::SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let Some(states) = &self.pinned else {
            return self.base.render(model, handlers, cx);
        };
        let pinned = selection_panel::SelectionPanelRenderModel {
            panel_id: model.panel_id,
            control_id: model.control_id,
            items: model.items,
            visible_indices: model.visible_indices,
            selected_source_index: model.selected_source_index,
            active_visible_index: model.active_visible_index,
            hovered_visible_index: model.hovered_visible_index,
            pressed_visible_index: model.pressed_visible_index,
            open: model.open,
            enabled: !states.has("disabled"),
            focus: model.focus,
            item_template: model.item_template,
            menu: model.menu.clone(),
            show_selection_marker: model.show_selection_marker,
            icons: model.icons,
            show_panel_chrome: model.show_panel_chrome,
        };
        self.base.render(&pinned, handlers, cx)
    }
}

fn selection_panel_template(
    pinned: Option<StateSet>,
) -> Arc<dyn selection_panel::SelectionPanelTemplate<selection_panel::SelectionPanelItem>> {
    Arc::new(PinnedSelectionPanelTemplate { base: selection_panel::default_selection_panel_template(), pinned })
}

fn selection_panel_builder(id: impl Into<SharedString>) -> selection_panel::SelectionPanelBuilder {
    selection_panel::SelectionPanelBuilder::new(id).items(
        ITEMS
            .iter()
            .map(|&(id, label, enabled)| selection_panel::SelectionPanelItem::new(id).label(label).enabled(enabled)),
    )
}

/// SelectionPanel extends the floating menu (`selection_panel.lss`): the page binds its look to the theme
/// and pins `:disabled`. Pinning a row's `:hover` is per-item state (#397 item 18).
pub(super) struct SelectionPanelSurface {
    control: Entity<selection_panel::SelectionPanelControl>,
    cells: Cells,
}

impl SelectionPanelSurface {
    const WIDTH: f32 = 220.;

    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let control = selection_panel_builder("workshop-sdk-selection-panel").spawn(cx);
        Box::new(Self { control, cells: Cells::default() })
    }
}

impl SurfaceControl for SelectionPanelSurface {
    fn title(&self) -> &'static str {
        "Selection Panel"
    }

    fn view(&self) -> AnyView {
        self.control.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        self.cells.clear();
        let (menu, template) = (selection_panel_menu(theme), selection_panel_template(pinned.cloned()));
        self.control.update(cx, |control, cx| {
            control.set_menu(menu, cx);
            control.set_template(template, cx);
        });
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.control.update(cx, |control, cx| control.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(
            &self.control,
            move |side: &mut ControlSide, _, event: &selection_panel::SelectionPanelEvent, cx| {
                let state = match event {
                    selection_panel::SelectionPanelEvent::HoverChanged { visible_index } => {
                        Some(("hover", visible_index.is_some()))
                    }
                    _ => None,
                };
                on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
            },
        )
    }

    /// A panel of its own in the theme, enabled or disabled as the cell's states are.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let cell = self.cells.get(&id, || {
            let control = selection_panel_builder(id.clone())
                .template(selection_panel_template(Some(states.clone())))
                .enabled(!states.has("disabled"))
                .spawn(cx);
            let menu = selection_panel_menu(theme.clone());
            control.update(cx, |control, cx| control.set_menu(menu, cx));
            control.into()
        });
        div().w(px(Self::WIDTH)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
#[allow(deprecated)]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use xgpui_luma::controls::selector::SelectorTriggerStyle;
    use xgpui_luma::theme::InteractionState;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    /// The pages open the bound theme's floating menu, not the SDK's (#397).
    #[test]
    fn popups_open_the_bound_themes_menu() {
        let dark = xgpui_luma::style::dark_theme();
        let want = xgpui_luma::controls::floating_menu::styled_floating_menu_look(
            &xgpui_luma::style::StyleBinding::new("floating_menu", ThemeSource::Fixed(dark.clone())),
        );
        let sdk = xgpui_luma::controls::floating_menu::styled_floating_menu_look(
            &xgpui_luma::controls::floating_menu::default_floating_menu_theme().style(),
        );
        assert_ne!(want.background, sdk.background, "the dark menu differs from the SDK's");

        let popup = popup_menu_theme(dark.clone(), None);
        assert_eq!(
            xgpui_luma::controls::floating_menu::styled_floating_menu_look(&popup.menu()).background,
            want.background,
            "popup menu and split button"
        );
        let selector = search_selector_trigger(dark.clone(), None).menu();
        assert_eq!(
            xgpui_luma::controls::floating_menu::styled_floating_menu_look(&selector).background,
            want.background,
            "selector"
        );
        assert_eq!(
            xgpui_luma::controls::floating_menu::styled_floating_menu_look(&autocomplete_theme(dark).menu()).background,
            want.background,
            "autocomplete"
        );
    }

    fn pin_and_show_matrix(side: &Entity<ControlSide>, state: &str, cx: &mut gpui::VisualTestContext) {
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction(state, cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has(state), "{pinned:?}");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw the control");
    }

    #[test]
    fn the_selector_page_skips_disabled_items_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "popup-selector");
        let side = cx.update(|_, cx| page.read(cx).side());
        let selector = cx.update(|_, cx| side.read(cx).control_as::<SelectorSurface>().expect("a selector").selector());
        cx.update(|window, cx| selector.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space home down down enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(selector.read(cx).selected_id().map(|id| id.as_ref()), Some("spacing")));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Change"), "choosing is logged: {text}");
        pin_and_show_matrix(&side, "hover", cx);
    }

    #[test]
    fn the_split_button_page_acts_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "split-button");
        let side = cx.update(|_, cx| page.read(cx).side());
        let button =
            cx.update(|_, cx| side.read(cx).control_as::<SplitButtonSurface>().expect("a split button").button());
        cx.update(|window, cx| button.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space down end enter");
        cx.run_until_parked();
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("ActionClick") && text.contains("quality"), "acting and choosing are logged: {text}");
        pin_and_show_matrix(&side, "pressed", cx);
    }

    #[test]
    fn the_autocomplete_page_pins_its_text_field_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "autocomplete-textfield");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<AutocompleteSurface>().is_some()));
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.toggle_state("focused", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("focused"), "{pinned:?}");
        pin_and_show_matrix(&side, "hover", cx);
    }

    /// Autocomplete and SearchSelector open their popup in the bound theme's floating menu: switched to dark,
    /// the open popup paints the dark menu's fill, not the SDK's (#397).
    #[test]
    fn the_autocomplete_and_search_selector_popups_follow_the_bound_theme() {
        for (id, typed) in [("autocomplete-textfield", Some("a")), ("search-selector", None)] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            side.update(cx, |side, cx| side.choose_mode("dark", cx));
            cx.run_until_parked();
            let theme = cx.update(|_, cx| side.read(cx).bound_theme());
            let menu = xgpui_luma::controls::floating_menu::styled_floating_menu_look;
            let want = menu(&selector_list::selector_items_panel_source(ThemeSource::Fixed(theme))()).background;
            let sdk = menu(&xgpui_luma::style::StyleBinding::sdk("floating_menu")).background;
            assert_ne!(want, sdk, "{id}: the dark menu differs from the SDK's");

            let at = cx.debug_bounds("workshop-surface-control").expect("the control").center();
            cx.simulate_click(at, Default::default());
            cx.run_until_parked();
            if let Some(text) = typed {
                cx.simulate_input(text);
                cx.run_until_parked();
            }
            let painted = cx.update(|window, _| {
                window.painted_quads().iter().any(|quad| quad.background == gpui::Background::from(want))
            });
            assert!(painted, "{id}: the open popup paints the dark menu's fill");
        }
    }

    /// A disabled field or trigger paints its `:disabled` fill opaque, not faded over what's behind it (#397).
    #[test]
    fn disabled_fills_are_opaque() {
        for (id, control) in [("combobox", "textfield"), ("popup-menu", "popup_menu"), ("split-button", "popup_menu")] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            let theme = cx.update(|_, cx| side.read(cx).bound_theme());
            let key = theme.key_named(control, &[], &["disabled"]).expect("key");
            let fill = theme.resolve(&key).root().background().expect("a disabled fill");
            let fill = gpui::Background::from(gpui::Hsla::from(fill));
            let painted = cx.update(|window, _| window.painted_quads().iter().any(|quad| quad.background == fill));
            assert!(painted, "{id}: a disabled cell paints its fill at full opacity");
        }
    }

    /// Design mode pins `state` with the key's switch (or the interaction picker for an exclusive state).
    fn pin(side: &Entity<ControlSide>, state: &str, exclusive: bool, cx: &mut gpui::VisualTestContext) -> StateSet {
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| {
            if exclusive {
                side.choose_interaction(state, cx)
            } else {
                side.toggle_state(state, cx)
            }
        });
        cx.run_until_parked();
        cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states")
    }

    /// ComboBox offers its field's states; pinning `:invalid` gives the field a different border (#397).
    #[test]
    fn the_combobox_page_pins_its_field_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "combobox");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<ComboBoxSurface>().is_some()));
        let pinned = pin(&side, "invalid", false, cx);
        assert!(pinned.has("invalid"), "{pinned:?}");
        let theme = xgpui_luma::style::default_theme();
        // Resolved the way the field draws (`resolve_look_with_scale`).
        let (pinned_border, plain_border) = cx.update(|_, cx| {
            let border = |pinned: Option<StateSet>, cx: &mut App| {
                autocomplete_field(theme.clone(), pinned)
                    .resolve_look_with_scale(Default::default(), true, 1.0, cx)
                    .border
            };
            (border(Some(pinned.clone()), cx), border(None, cx))
        });
        assert_ne!(pinned_border, plain_border, "the pinned field draws the invalid border");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw comboboxes");
    }

    /// SearchSelector offers its trigger's states; pinning `:hover` gives the trigger a different background
    /// (#397).
    #[test]
    fn the_search_selector_page_pins_its_trigger_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "search-selector");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<SearchSelectorSurface>().is_some()));
        let (columns, _) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert!(columns.iter().any(|column| column == ":hover"), "the trigger's states are offered: {columns:?}");
        let pinned = pin(&side, "hover", true, cx);
        assert!(pinned.has("hover"), "{pinned:?}");
        let theme = xgpui_luma::style::default_theme();
        let background = |pinned: Option<StateSet>| {
            use xgpui_luma::style::PartRead;
            xgpui_luma::controls::selector::resolve_selector_trigger(
                &search_selector_trigger(theme.clone(), pinned).style(),
                SelectorTriggerStyle::default(),
                Default::default(),
                1.0,
            )
            .root()
            .color("background")
        };
        assert_ne!(background(Some(pinned)), background(None), "the pinned trigger draws its hover background");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw search selectors");
    }

    /// Pinning `:focused` on the SearchSelector page paints the trigger's focus ring on the stage (#397).
    #[test]
    fn the_search_selector_page_paints_a_pinned_focus_ring() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "search-selector");
        let side = cx.update(|_, cx| page.read(cx).side());
        let ring = {
            use xgpui_luma::style::PartRead;
            xgpui_luma::controls::selector::resolve_selector_trigger(
                &search_selector_trigger(xgpui_luma::style::default_theme(), None).style(),
                SelectorTriggerStyle::default(),
                xgpui_luma::controls::selector::SelectorVisualState {
                    interaction: InteractionState { focused: true, ..Default::default() },
                    ..Default::default()
                },
                1.0,
            )
            .root()
            .color("focus_border")
        };
        let ring_painted = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, _| window.painted_quads().iter().any(|quad| quad.border_color == ring))
        };
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        cx.run_until_parked();
        assert!(!ring_painted(cx), "no ring with nothing pinned");
        let pinned = pin(&side, "focused", false, cx);
        assert!(pinned.has("focused"), "{pinned:?}");
        assert!(ring_painted(cx), "the pinned focus ring is painted");
    }

    /// Selection Panel offers `:disabled`; pinned or not, the live panel draws enabled or disabled as picked,
    /// though Design mode disables its input (#397).
    #[test]
    fn the_selection_panel_page_binds_its_look_and_pins_disabled() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "selection-panel");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<SelectionPanelSurface>().is_some()));
        let pinned = pin(&side, "disabled", true, cx);
        assert!(pinned.has("disabled"), "{pinned:?}");
        // The style the page binds is the theme's floating-menu rows, not left at a default.
        let menu = (selection_panel_menu(xgpui_luma::style::default_theme()))();
        let rows = xgpui_luma::controls::floating_menu::MenuStyle::resolve(&menu).row_colors();
        assert!(rows.hover_background.a > 0.0, "the theme's row hover is bound");
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw panels");
    }
}
