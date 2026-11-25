//! SDK selection controls on the design surface page (issue #383).
//!
//! - **Selector** (`selector`, which extends `button`) and **Split Button** (`popup_menu`, which extends
//!   `button`) take a theme, so the page binds their `.lss` and Design mode pins the picked states.
//! - **Autocomplete** (`autocomplete`) takes no theme of its own, but its hosted text field does: the page
//!   binds and pins the field (`textfield.lss`, shown beside the autocomplete's `.lss`).
//! - **Selection Panel** (`selection_panel`), **ComboBox** (`combobox`) and **SearchSelector**
//!   (`search_selector`) draw from the SDK's own styles and take no theme, so their pages show the live
//!   control beside its `.lss` without binding or pinning it.
//!
//! Each State matrix cell is a small control of its own, drawn in that cell's states where the control
//! takes a theme.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::context_menu::MenuItem;
use xgpui_luma::controls::popup_menu::{
    MenuSource, PopupMenuLook, PopupMenuPalette, PopupMenuTheme, PopupMenuTriggerMetrics, PopupMenuTriggerStyle,
    StyledPopupMenuTheme,
};
use xgpui_luma::controls::selector::{
    ItemsPanelSource, Selector, SelectorBuilder, SelectorEvent, SelectorItem, SelectorLook, SelectorPalette,
    SelectorTheme, SelectorTriggerStyle, SelectorVisualState, StyledSelectorTheme, ThemedSelectorTemplate,
};
use xgpui_luma::controls::split_button::{SplitButton, SplitButtonBuilder, SplitButtonEvent, ThemedSplitButtonTemplate};
use xgpui_luma::controls::{autocomplete, combobox, search_selector, selection_panel, selector_list};
use xgpui_luma::style::ThemeSource;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::theme::{InteractionState, MetricTokens, StandardBoxScale};

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

/// `state`, or the picked states when some are pinned.
fn pin(pinned: &Option<StateSet>, state: InteractionState) -> InteractionState {
    pinned.as_ref().map_or(state, StateSet::interaction)
}

// --- Selector ----------------------------------------------------------------------------------------------

struct PinnedSelectorTheme {
    base: StyledSelectorTheme,
    pinned: Option<StateSet>,
}

impl SelectorTheme for PinnedSelectorTheme {
    fn resolve(&self, style: SelectorTriggerStyle, state: InteractionState, flat: bool) -> SelectorPalette {
        self.base.resolve(style, pin(&self.pinned, state), flat)
    }

    fn metrics(&self) -> MetricTokens {
        self.base.metrics()
    }

    fn resolve_look(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        scale: &StandardBoxScale,
        flat: bool,
    ) -> SelectorLook {
        self.base.resolve_look(style, pin(&self.pinned, state), scale, flat)
    }

    fn resolve_visual_look(
        &self,
        style: SelectorTriggerStyle,
        visual: SelectorVisualState,
        scale: &StandardBoxScale,
        flat: bool,
    ) -> SelectorLook {
        match &self.pinned {
            Some(states) => self.base.resolve_look(style, states.interaction(), scale, flat),
            None => self.base.resolve_visual_look(style, visual, scale, flat),
        }
    }
}

fn selector_template(theme: Theme, pinned: Option<StateSet>) -> Arc<ThemedSelectorTemplate<SelectorItem>> {
    let panel: ItemsPanelSource = Arc::new(|| selector_list::default_selector_items_panel_look(&Default::default()));
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

    /// `SelectorBuilder` has no way to make the trigger invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
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
    fn resolve(
        &self,
        style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        self.base.resolve(style, metrics, pin(&self.pinned, state))
    }

    fn metrics(&self) -> MetricTokens {
        self.base.metrics()
    }

    fn resolve_look(
        &self,
        style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        cx: &mut App,
    ) -> PopupMenuLook {
        self.base.resolve_look(style, metrics, pin(&self.pinned, state), scale_factor, cx)
    }
}

/// The `popup_menu` control in `theme`, its trigger in the picked states when some are pinned. Split Button
/// and Popup Menu share it.
pub(super) fn popup_menu_theme(theme: Theme, pinned: Option<StateSet>) -> Arc<dyn PopupMenuTheme> {
    let menu: MenuSource = Arc::new(|| xgpui_luma::controls::floating_menu::default_floating_menu_theme().resolve());
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

    /// `SplitButtonBuilder` has no way to make the button invalid.
    fn unreachable_states(&self) -> &'static [&'static str] {
        &["invalid"]
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

// --- Controls that take no theme ---------------------------------------------------------------------------

type MakeCell = fn(SharedString, bool, &mut App) -> AnyView;
type Subscribe = fn(&AnyView, &mut Context<ControlSide>, OnSurfaceEvent) -> Subscription;
type SetEnabled = fn(&AnyView, bool, &mut App);

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
        let field = autocomplete_field(theme, pinned.cloned());
        self.control.update(cx, |control, cx| control.set_textfield_template(field, cx));
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

// --- Controls that take no theme (continued) ---------------------------------------------------------------

/// A selection control drawn from the SDK's own styles: the page shows it beside its `.lss`, unbound.
pub(super) struct UnthemedSurface {
    title: &'static str,
    view: AnyView,
    set_enabled: SetEnabled,
    subscribe: Subscribe,
    make_cell: MakeCell,
    width: f32,
    cells: Cells,
}

impl UnthemedSurface {
    fn new(
        title: &'static str,
        view: AnyView,
        (set_enabled, subscribe, make_cell): (SetEnabled, Subscribe, MakeCell),
        width: f32,
    ) -> Box<dyn SurfaceControl> {
        Box::new(Self { title, view, set_enabled, subscribe, make_cell, width, cells: Cells::default() })
    }

    pub(super) fn make_selection_panel(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        fn panel(id: impl Into<SharedString>) -> selection_panel::SelectionPanelBuilder {
            selection_panel::SelectionPanelBuilder::new(id).items(ITEMS.iter().map(|&(id, label, enabled)| {
                selection_panel::SelectionPanelItem::new(id).label(label).enabled(enabled)
            }))
        }
        let view = panel("workshop-sdk-selection-panel").spawn(cx);
        Self::new(
            "Selection Panel",
            view.into(),
            (
                |view, enabled, cx| {
                    if let Ok(panel) = view.clone().downcast::<selection_panel::SelectionPanelControl>() {
                        panel.update(cx, |panel, cx| panel.set_enabled(enabled, cx));
                    }
                },
                |view, cx, on_event| match view.clone().downcast::<selection_panel::SelectionPanelControl>() {
                    Ok(panel) => cx.subscribe(
                        &panel,
                        move |side: &mut ControlSide, _, event: &selection_panel::SelectionPanelEvent, cx| {
                            let state = match event {
                                selection_panel::SelectionPanelEvent::HoverChanged { visible_index } => {
                                    Some(("hover", visible_index.is_some()))
                                }
                                _ => None,
                            };
                            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
                        },
                    ),
                    // The view is always this control; nothing to follow otherwise.
                    Err(_) => Subscription::new(|| {}),
                },
                |id, enabled, cx| panel(id).enabled(enabled).spawn(cx).into(),
            ),
            220.,
        )
    }

    pub(super) fn make_combobox(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let view = combobox::ComboBoxBuilder::new("workshop-sdk-combobox", filter_items()).spawn(cx);
        Self::new(
            "ComboBox",
            view.into(),
            (
                |view, enabled, cx| {
                    if let Ok(control) = view.clone().downcast::<combobox::ComboBoxControl>() {
                        control.update(cx, |control, cx| control.set_enabled(enabled, cx));
                    }
                },
                |view, cx, on_event| match view.clone().downcast::<combobox::ComboBoxControl>() {
                    Ok(control) => {
                        cx.subscribe(&control, move |side: &mut ControlSide, _, event: &combobox::ComboBoxEvent, cx| {
                            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
                        })
                    }
                    // The view is always this control; nothing to follow otherwise.
                    Err(_) => Subscription::new(|| {}),
                },
                |id, enabled, cx| combobox::ComboBoxBuilder::new(id, filter_items()).enabled(enabled).spawn(cx).into(),
            ),
            260.,
        )
    }

    pub(super) fn make_search_selector(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let view =
            search_selector::SearchSelectorBuilder::new("workshop-sdk-search-selector", filter_items()).spawn(cx);
        Self::new(
            "SearchSelector",
            view.into(),
            (
                |view, enabled, cx| {
                    if let Ok(control) = view.clone().downcast::<search_selector::SearchSelectorControl>() {
                        control.update(cx, |control, cx| control.set_enabled(enabled, cx));
                    }
                },
                |view, cx, on_event| match view.clone().downcast::<search_selector::SearchSelectorControl>() {
                    Ok(control) => cx.subscribe(
                        &control,
                        move |side: &mut ControlSide, _, event: &search_selector::SearchSelectorEvent, cx| {
                            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
                        },
                    ),
                    // The view is always this control; nothing to follow otherwise.
                    Err(_) => Subscription::new(|| {}),
                },
                |id, enabled, cx| {
                    search_selector::SearchSelectorBuilder::new(id, filter_items()).enabled(enabled).spawn(cx).into()
                },
            ),
            260.,
        )
    }
}

impl SurfaceControl for UnthemedSurface {
    fn title(&self) -> &'static str {
        self.title
    }

    fn view(&self) -> AnyView {
        self.view.clone()
    }

    /// The control takes no theme; only cells are remade, so a disabled column shows.
    fn bind(&self, _theme: Theme, _pinned: Option<&StateSet>, _cx: &mut App) {
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        (self.set_enabled)(&self.view, enabled, cx);
    }

    fn template_states(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        (self.subscribe)(&self.view, cx, on_event)
    }

    /// A control of its own, disabled when `:disabled` is; no other state can be drawn.
    fn cell(&self, _theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let make = self.make_cell;
        let cell = self.cells.get(&id, || make(id.clone(), !states.has("disabled"), cx));
        div().w(px(self.width)).child(cell).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

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

    #[test]
    fn the_unthemed_selection_pages_show_their_control_and_a_matrix() {
        for id in ["selection-panel", "combobox", "search-selector"] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            assert!(cx.update(|_, cx| side.read(cx).control_as::<UnthemedSurface>().is_some()), "{id}");
            assert!(cx.debug_bounds("workshop-surface-control").is_some(), "{id} draws its control");
            side.update(cx, |side, cx| side.show_matrix(cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "{id}'s matrix draws");
        }
    }
}
