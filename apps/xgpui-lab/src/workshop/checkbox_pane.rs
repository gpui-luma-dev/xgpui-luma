//! Checkbox work pane (issue #291): the SDK checkbox and its state matrix, the style key built from what
//! the checkbox's `.lss` declares, and the control's events, beside the `.lss` inspector in a resizable
//! split.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, Render, WeakEntity, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::checkbox::{self, CheckboxData, CheckboxEvent, StyledCheckboxTemplate};
use xgpui_luma::controls::resizable_panels::{ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation};
use xgpui_luma::controls::slider::Slider;
use xgpui_luma::controls::switch::Switch;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::{ControlId, Source, StateId, StyleKey, Stylesheet, Theme};
use xgpui_luma::theme::InteractionState;

use super::event_stream::ControlEventStream;
use super::lss_inspector::{self, LssInspector, TAB_ROW_HEIGHT};
use super::catalog::COMPONENT_CATALOG;
use super::pinned_template::{PinnedCheckbox, PinnedCheckboxTemplate};
use super::sample_layout::centered_ruled_background;
use super::pane_type;
use crate::screen::{LabMetrics, text};
use crate::theme::{ColorVizLookExt, Look, TypographyExt};

const MIN_SIDE_WIDTH: f32 = 360.;
/// Initial height of the lower panel (states and events) under the stage.
const LOWER_HEIGHT: f32 = 400.;
const MIN_STAGE_HEIGHT: f32 = 160.;
/// The lower panel never gets shorter than this, so the states strip and the events stay on screen.
const MIN_LOWER_HEIGHT: f32 = 400.;
/// The workshop canvas's inner corner radius (`CANVAS_RADIUS` less its border); the pane's outer
/// corners match it so nothing square shows past the rounded canvas.
const CANVAS_INNER_RADIUS: f32 = 9.;
const STRIP_LABEL_WIDTH: f32 = 78.;
/// States the template reads that the live checkbox can never reach: `CheckboxBuilder` has no way to
/// make it invalid.
const UNREACHABLE_STATES: &[&str] = &["invalid"];
/// The stage zoom's range and step: actual size up to five times.
const ZOOM_RANGE: std::ops::Range<f32> = 1.0..5.0;
const ZOOM_STEP: f32 = 0.25;
const ZOOM_SLIDER_WIDTH: f32 = 140.;
/// The stage grid's square at actual size; it scales with the zoom, like the checkbox.
const STAGE_GRID_SPACING: f32 = 10.;

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Control,
    Matrix,
}

/// The work pane: the control side and the inspector, half each, with a draggable divider between them.
pub(super) struct CheckboxPane {
    look: Arc<Look>,
    theme: Theme,
    control: ControlId,
    side: Entity<ControlSide>,
    inspector: Entity<LssInspector>,
    split: Entity<ResizablePanels>,
    /// Whether the left corners are rounded: the shell's sidebar is hidden, so the pane meets the
    /// canvas's rounded left edge.
    left_rounded: bool,
}

impl CheckboxPane {
    /// `control` is the checkbox's `.lss` control in `theme`.
    pub(super) fn new(look: Arc<Look>, theme: Theme, control: ControlId, cx: &mut Context<Self>) -> Self {
        let inspector = cx.new(|cx| LssInspector::new(look.clone(), theme.clone(), control, cx));
        let side = cx.new(|cx| ControlSide::new(look.clone(), theme.clone(), control, inspector.clone(), cx));
        let (side_view, inspector_view) = (side.clone(), inspector.clone());
        let split = xgpui_luma_look_radix::resizable_panels_builder(&look, "workshop-checkbox-split")
            .show_border(false)
            .panel(ResizablePanelSpec::new_render(move || side_view.clone()).weight(1.).min(px(MIN_SIDE_WIDTH)))
            .panel(ResizablePanelSpec::new_render(move || inspector_view.clone()).weight(1.).min(px(MIN_SIDE_WIDTH)))
            .spawn(cx);
        Self { look, theme, control, side, inspector, split, left_rounded: false }
    }

    pub(super) fn set_left_rounded(&mut self, rounded: bool, cx: &mut Context<Self>) {
        self.left_rounded = rounded;
        cx.notify();
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.side.update(cx, |side, cx| side.sync_look(look.clone(), cx));
        self.inspector.update(cx, |inspector, cx| inspector.sync_look(look, cx));
        cx.notify();
    }

    /// The page title in the lab screens' text roles: crumb (Workshop · group), title, the `.lss` control
    /// with its file and layer, and the control's doc comment as the lede.
    fn title(&self) -> AnyElement {
        let (look, chrome) = (self.look.as_ref(), self.look.chrome());
        let inspection = self.theme.inspect(&StyleKey::new(self.control), None);
        let source = lss_inspector::control_source(&inspection)
            .map_or_else(String::new, |(file, layer)| format!("{file} · layer {layer}"));
        let group = COMPONENT_CATALOG
            .iter()
            .find(|g| g.entries.iter().any(|e| e.id == "checkbox"))
            .map_or("", |g| g.label);
        let metrics = LabMetrics::STANDARD;
        div()
            .debug_selector(|| "workshop-checkbox-title".into())
            .flex_none()
            .flex()
            .flex_col()
            .gap(metrics.header_gap)
            .px(px(18.))
            .py(px(14.))
            .border_b_1()
            .border_color(chrome.border)
            .child(text(look, pane_type::CRUMB, format!("Workshop · {group}")))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_baseline()
                    .gap(px(12.))
                    .child(text(look, pane_type::TITLE, "Checkbox"))
                    .child(text(look, pane_type::CONTROL, self.control.to_string()))
                    .child(text(look, pane_type::CONTROL, source).text_color(chrome.muted_text)),
            )
            .children(inspection.doc.map(|d| text(look, pane_type::LEDE, d)))
            .into_any_element()
    }

    /// The pane look's light/dark, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn look_mode(&self) -> xgpui_luma::theme::ThemeMode {
        self.look.mode()
    }

    /// The control side, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn side(&self) -> Entity<ControlSide> {
        self.side.clone()
    }

    /// The inspector, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn inspector(&self) -> Entity<LssInspector> {
        self.inspector.clone()
    }
}

impl Render for CheckboxPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        div()
            .id("workshop-checkbox-pane")
            .debug_selector(|| "workshop-checkbox-pane".into())
            .size_full()
            .rounded_tr(px(CANVAS_INNER_RADIUS))
            .rounded_br(px(CANVAS_INNER_RADIUS))
            .when(self.left_rounded, |d| d.rounded_tl(px(CANVAS_INNER_RADIUS)).rounded_bl(px(CANVAS_INNER_RADIUS)))
            .bg(chrome.content_background)
            .text_color(chrome.body_text)
            .typography_style(pane_type::body(&self.look))
            .flex()
            .flex_col()
            .child(self.title())
            .child(div().flex_1().min_h_0().child(self.split.clone()))
    }
}

/// One state of the style key and its switch.
struct KeyState {
    state: StateId,
    /// No rule in the file uses it; the template sets it in its key (and may draw it, e.g. the focus
    /// ring).
    template_only: bool,
    /// The live control can reach it (see [`UNREACHABLE_STATES`]).
    reachable: bool,
    switch: Switch,
}

/// The control and its state matrix above, the style key and the control's events below, with a
/// draggable divider between them.
pub(super) struct ControlSide {
    look: Arc<Look>,
    /// The inspected theme in the mode chosen with `modes`.
    theme: Theme,
    /// The stylesheet's declared modes (e.g. light, dark), one selected.
    /// A Radix toggle group (one on).
    modes: Option<ControlGroup<ControlGroupItem>>,
    /// `theme`'s stylesheet plus a probe control reading `palette.app.background`, for the stage.
    stage_probe: Option<Stylesheet>,
    /// The mode's app background, painted behind the control.
    stage_background: Option<Hsla>,
    control: ControlId,
    checkbox: checkbox::Checkbox,
    /// The live checkbox's zoom on the Control stage, a visual aid: the checkbox draws from `theme` with
    /// every length scaled by it. The matrix and the inspector stay at actual size.
    zoom: f32,
    zoom_slider: Slider,
    /// The states the live checkbox draws while the key is pinned (`follow` off), like a designer's
    /// state picker; `None` while it follows the control and draws as it is.
    pinned: Option<PinnedCheckbox>,
    stage: Stage,
    /// Radix tabs: Control · State matrix.
    stage_tabs: Entity<xgpui_luma::controls::tabs::Tabs>,
    /// The key's states, in config order: those the file's rules use, then any the template sets
    /// that no rule uses.
    states: Vec<KeyState>,
    /// Whether the key follows the checkbox (checked, hover, enabled); pressed has no event.
    follow: bool,
    follow_switch: Switch,
    reset: Entity<Button>,
    key: StyleKey,
    inspector: Entity<LssInspector>,
    events: Entity<ControlEventStream>,
    /// Stage above, states and events below.
    split: Entity<ResizablePanels>,
}

impl ControlSide {
    fn new(
        look: Arc<Look>,
        theme: Theme,
        control: ControlId,
        inspector: Entity<LssInspector>,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = theme.stylesheet().config().clone();

        let checkbox = checkbox::new("workshop-sdk-checkbox").content(|_, _| div().child("Label")).spawn(cx);
        cx.subscribe(&checkbox, |this: &mut Self, _, event: &CheckboxEvent, cx| {
            let line = format!("{event:?}");
            this.events.update(cx, |events, cx| events.append_line(&line, cx));
            if !this.follow {
                return;
            }
            match event {
                CheckboxEvent::Change { checked } => this.set_state("checked", *checked, cx),
                CheckboxEvent::HoverChanged { hovered } => this.set_state("hover", *hovered, cx),
                CheckboxEvent::EnabledChanged { enabled } => this.set_state("disabled", !enabled, cx),
                CheckboxEvent::FocusChanged { focused } => this.set_state("focused", *focused, cx),
                _ => {}
            }
        })
        .detach();

        let stage_tabs = xgpui_luma_look_radix::Tabs::new("workshop-checkbox-stage")
            .look(look.as_ref())
            .items([
                xgpui_luma::controls::tabs::TabsItem::new("control").label("Control"),
                xgpui_luma::controls::tabs::TabsItem::new("matrix").label("State matrix"),
            ])
            .active("control")
            .on_value_changed(cx, |this: &mut Self, id, _, cx| {
                this.stage = if id.as_ref() == "matrix" {
                    Stage::Matrix
                } else {
                    Stage::Control
                };
                cx.notify();
            })
            .spawn(cx);
        // States the file's rules use, plus states the template sets with no rule (e.g. `focused`).
        let rule_states = theme.stylesheet().axes(control).states;
        let template_states: Vec<StateId> = checkbox::default_checkbox_template()
            .style_states()
            .into_iter()
            .filter_map(|n| config.state(n))
            .collect();
        let states = config
            .state_names()
            .filter_map(|name| config.state(name).map(|state| (state, name.to_string())))
            .filter(|(state, _)| rule_states.contains(state) || template_states.contains(state))
            .map(|(state, name)| {
                // A state the live checkbox can never reach is shown but never settable.
                let reachable = !UNREACHABLE_STATES.contains(&name.as_str());
                let switch = xgpui_luma_look_radix::Switch::new(format!("workshop-checkbox-state-{name}"))
                    .look(look.as_ref())
                    .label(format!(":{name}"))
                    .checked(false)
                    .enabled(reachable)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        // Ignore the switch catching up with a followed change; a person's toggle pins the key.
                        if this.key.active_states().contains(state) == on {
                            return;
                        }
                        this.set_follow(false, cx);
                        this.apply(this.key.set_state(state, on), cx);
                    })
                    .spawn(cx);
                KeyState { template_only: !rule_states.contains(&state), reachable, state, switch }
            })
            .collect();
        let zoom_slider = xgpui_luma_look_radix::Slider::new("workshop-checkbox-zoom")
            .look(look.as_ref())
            .size(xgpui_luma_look_radix::SliderSize::One)
            .range(ZOOM_RANGE)
            .step(ZOOM_STEP)
            .value(ZOOM_RANGE.start)
            .on_value_changed(cx, |this: &mut Self, _, zoom, cx| this.set_zoom(zoom, cx))
            .spawn(cx);
        let follow_switch = xgpui_luma_look_radix::Switch::new("workshop-checkbox-follow")
            .look(look.as_ref())
            .label("Track states")
            .checked(true)
            .on_value_changed(cx, |this: &mut Self, on, cx| this.set_follow(on, cx))
            .spawn(cx);
        let reset = xgpui_luma_look_radix::Button::new("workshop-checkbox-reset")
            .look(look.as_ref())
            .outline()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .label("Reset")
            .on_click(cx, |this: &mut Self, cx| this.apply(this.followed_key(cx), cx))
            .spawn(cx);
        let events = cx.new(|cx| {
            ControlEventStream::new(cx, look.clone(), "workshop-checkbox-events", "The checkbox's SDK events.")
                .without_card()
        });
        let side = cx.weak_entity();
        let stage_slot = cx.new(|_| SideSlot { side: side.clone(), slot: Slot::Stage });
        let lower_slot = cx.new(|_| SideSlot { side, slot: Slot::Lower });
        let split = xgpui_luma_look_radix::resizable_panels_builder(&look, "workshop-checkbox-side-split")
            .orientation(ResizablePanelsOrientation::Vertical)
            .show_border(false)
            .panel(ResizablePanelSpec::new_render(move || stage_slot.clone()).weight(1.).min(px(MIN_STAGE_HEIGHT)))
            .panel(
                ResizablePanelSpec::new_render(move || lower_slot.clone())
                    .size(px(LOWER_HEIGHT))
                    .min(px(MIN_LOWER_HEIGHT)),
            )
            .spawn(cx);
        let mode_names: Vec<String> = theme.stylesheet().modes();
        let modes = (!mode_names.is_empty()).then(|| {
            let group =
                xgpui_luma_look_radix::control_group_builder(look.as_ref(), "workshop-checkbox-mode", look.size())
                    .horizontal()
                    .single_required()
                    .items(mode_names.iter().map(|m| ControlGroupItem::new(m.clone()).label(m.clone())))
                    .selected_ids([theme.mode().to_string()])
                    .spawn(cx);
            cx.subscribe(&group, |this: &mut Self, _, event: &ControlGroupEvent, cx| {
                if let ControlGroupEvent::Change { selected_ids, .. } = event
                    && let Some(mode) = selected_ids.first()
                {
                    this.set_mode(mode, cx);
                }
            })
            .detach();
            group
        });
        let stage_probe = theme
            .stylesheet()
            .with_source(
                "app",
                Source::new("workshop/stage.lss", "workshop_stage { background = palette.app.background }"),
            )
            .ok();
        let stage_background = stage_probe.as_ref().and_then(|probe| app_background(&theme, probe));
        Self {
            look,
            theme,
            modes,
            stage_probe,
            stage_background,
            control,
            checkbox,
            zoom: ZOOM_RANGE.start,
            zoom_slider,
            pinned: None,
            stage: Stage::Control,
            stage_tabs,
            states,
            follow: true,
            follow_switch,
            reset,
            key: StyleKey::new(control),
            inspector,
            events,
            split,
        }
    }

    /// Inspect in another declared mode: the live checkbox, the matrix, the inspector and the stage
    /// background all use the same stylesheet bound in `mode`.
    fn set_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        let Ok(theme) = self.theme.with_mode(mode) else {
            return;
        };
        self.inspector.update(cx, |inspector, cx| inspector.set_theme(theme.clone(), cx));
        self.stage_background = self.stage_probe.as_ref().and_then(|probe| app_background(&theme, probe));
        self.theme = theme;
        self.bind_checkbox(cx);
        cx.notify();
    }

    /// Draw the stage checkbox `zoom` times its actual size.
    fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        let zoom = zoom.clamp(ZOOM_RANGE.start, ZOOM_RANGE.end);
        if zoom == self.zoom {
            return;
        }
        self.zoom = zoom;
        self.bind_checkbox(cx);
        cx.notify();
    }

    /// Give the live checkbox `theme` scaled by the zoom, drawing the pinned states if any. Only the
    /// template changes; the control and its data are untouched.
    fn bind_checkbox(&mut self, cx: &mut Context<Self>) {
        let theme = self.theme.with_length_scale(self.zoom).unwrap_or_else(|_| self.theme.clone());
        let template = PinnedCheckboxTemplate::new(Arc::new(StyledCheckboxTemplate::new(theme)), self.pinned);
        self.checkbox.update(cx, |checkbox, cx| checkbox.set_template(Arc::new(template), cx));
    }

    /// Pin the live checkbox to the key's states while the key is pinned; let it draw as it is while
    /// the key follows it.
    fn sync_pinned(&mut self, cx: &mut Context<Self>) {
        let pinned = (!self.follow).then(|| {
            let states: Vec<StateId> =
                self.states.iter().map(|s| s.state).filter(|s| self.key.active_states().contains(*s)).collect();
            let (state, checked) = self.template_state(&states);
            PinnedCheckbox { state, checked }
        });
        if pinned != self.pinned {
            self.pinned = pinned;
            self.bind_checkbox(cx);
        }
    }

    fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }

    /// Set one state by name in the key; a state the checkbox does not declare is ignored.
    fn set_state(&mut self, name: &str, on: bool, cx: &mut Context<Self>) {
        if let Some(state) = self.theme.stylesheet().config().state(name)
            && self.states.iter().any(|s| s.state == state)
        {
            self.apply(self.key.set_state(state, on), cx);
        }
    }

    /// Use `key` for the matrix highlight, the state switches and the inspector.
    fn apply(&mut self, key: StyleKey, cx: &mut Context<Self>) {
        self.key = key;
        for state in &self.states {
            let on = key.active_states().contains(state.state);
            if *state.switch.read(cx).data() != on {
                state.switch.update(cx, |switch, cx| switch.set_data(on, cx));
            }
        }
        self.inspector.update(cx, |inspector, cx| inspector.set_key(key, cx));
        self.sync_pinned(cx);
        cx.notify();
    }

    fn set_follow(&mut self, follow: bool, cx: &mut Context<Self>) {
        self.follow = follow;
        if *self.follow_switch.read(cx).data() != follow {
            self.follow_switch.update(cx, |switch, cx| switch.set_data(follow, cx));
        }
        if follow {
            self.apply(self.followed_key(cx), cx);
        }
        self.sync_pinned(cx);
        cx.notify();
    }

    /// The key for the checkbox as it is: only `checked`, since hover, press and focus are transient.
    fn followed_key(&self, cx: &Context<Self>) -> StyleKey {
        let base = StyleKey::new(self.control);
        match self.theme.stylesheet().config().state("checked") {
            Some(checked) if self.follow => base.set_state(checked, *self.checkbox.read(cx).data()),
            _ => base,
        }
    }

    /// The zoom slider (Control stage only) and the stage tabs, right-aligned in a row as tall as the
    /// inspector's tab row.
    fn tab_row(&self) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .debug_selector(|| "workshop-checkbox-tab-row".into())
            .flex_none()
            .h(px(TAB_ROW_HEIGHT))
            .flex()
            .items_end()
            .justify_end()
            .px(px(12.))
            .border_b_1()
            .border_color(chrome.border)
            .when(self.stage == Stage::Control, |row| {
                row.child(
                    div()
                        .debug_selector(|| "workshop-checkbox-zoom".into())
                        .flex_1()
                        .self_center()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(div().font_weight(pane_type::LABEL_WEIGHT).text_color(chrome.muted_text).child("ZOOM"))
                        .child(div().w(px(ZOOM_SLIDER_WIDTH)).child(self.zoom_slider.clone()))
                        .child(div().font_family(pane_type::MONO).child(format!("{}×", self.zoom))),
                )
            })
            .child(self.stage_tabs.clone())
            .into_any_element()
    }

    fn control_stage(&self) -> AnyElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(div().debug_selector(|| "workshop-checkbox-control".into()).child(self.checkbox.clone()))
            .into_any_element()
    }

    /// The matrix axes, from the key's states. Columns are the config's exclusive group as one choice
    /// (none, then weakest to strongest), since only its strongest active state counts; rows are every
    /// combination of the other states.
    fn matrix_axes(&self) -> (Vec<Option<StateId>>, Vec<Vec<StateId>>) {
        let config = self.theme.stylesheet().config();
        let used: Vec<StateId> = self.states.iter().map(|s| s.state).collect();
        let mut exclusive: Vec<StateId> =
            config.exclusive_states().filter_map(|n| config.state(n)).filter(|s| used.contains(s)).collect();
        exclusive.reverse();
        let columns = std::iter::once(None).chain(exclusive.iter().copied().map(Some)).collect();
        let rest: Vec<StateId> = used.into_iter().filter(|s| !exclusive.contains(s)).collect();
        let rows = (0u32..1 << rest.len())
            .map(|mask| rest.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, s)| *s).collect())
            .collect();
        (columns, rows)
    }

    /// The template's interaction state and checked data for a set of states.
    fn template_state(&self, states: &[StateId]) -> (InteractionState, bool) {
        let config = self.theme.stylesheet().config();
        let has = |name: &str| config.state(name).is_some_and(|s| states.contains(&s));
        let interaction = InteractionState {
            hovered: has("hover"),
            pressed: has("pressed"),
            focused: has("focused"),
            disabled: has("disabled"),
            invalid: has("invalid"),
        };
        (interaction, has("checked"))
    }

    fn states_label(&self, states: &[StateId], empty: &str) -> String {
        let config = self.theme.stylesheet().config();
        if states.is_empty() {
            empty.to_string()
        } else {
            states.iter().map(|s| format!(":{}", config.state_name(*s))).collect()
        }
    }

    /// The SDK checkbox template rendered for each state combination, drawn bare (no cell chrome). The
    /// current key's row and column labels are highlighted.
    fn matrix(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();
        let config = self.theme.stylesheet().config();
        let accent = self.look.resolve_step(xgpui_luma_look_radix::ScaleFamily::Color, 11).hsla();
        let template = StyledCheckboxTemplate::new(self.theme.clone());
        let label = |text: String, current: bool| {
            div()
                .font_family(pane_type::MONO)
                .text_color(if current { accent } else { chrome.muted_text })
                .when(current, |d| d.font_weight(pane_type::LABEL_WEIGHT))
                .child(text)
        };
        let (columns, rows) = self.matrix_axes();
        let active = self.key.active_states();
        // The key's column: its strongest active member of the exclusive group.
        let current_column = config
            .exclusive_states()
            .filter_map(|n| config.state(n))
            .find(|s| active.contains(*s) && columns.contains(&Some(*s)));
        let rest: Vec<StateId> = rows.last().cloned().unwrap_or_default();
        let current_row = |row: &[StateId]| rest.iter().all(|s| active.contains(*s) == row.contains(s));
        let mut grid =
            div().debug_selector(|| "workshop-checkbox-matrix".into()).p(px(20.)).flex().flex_col().gap(px(10.));
        grid = grid.child(div().flex().gap(px(10.)).child(div().w(px(150.))).children(columns.iter().map(|column| {
            let text = self.states_label(&column.iter().copied().collect::<Vec<_>>(), "—");
            div().flex_1().child(label(text, *column == current_column))
        })));
        for (r, row_states) in rows.iter().enumerate() {
            let mut row =
                div().flex().items_center().gap(px(10.)).child(
                    div().w(px(150.)).child(label(self.states_label(row_states, "base"), current_row(row_states))),
                );
            for (c, column) in columns.iter().enumerate() {
                let cell: Vec<StateId> = row_states.iter().copied().chain(*column).collect();
                let (state, checked) = self.template_state(&cell);
                let model = ButtonRenderModel {
                    id: format!("workshop-checkbox-matrix-{r}-{c}").into(),
                    data: CheckboxData::new(checked),
                    state,
                    content: Arc::new(|_, _| div().child("Label").into_any_element()),
                    ..Default::default()
                };
                row = row
                    .child(div().flex_1().h(px(48.)).flex().items_center().child(template.render(&model, window, cx)));
            }
            grid = grid.child(row);
        }
        grid.into_any_element()
    }

    fn key_strip(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let axes = self.theme.stylesheet().axes(self.control);
        let config = self.theme.stylesheet().config();
        let attributes = if axes.attributes.is_empty() {
            "none — the file has no scopes".to_string()
        } else {
            axes.attributes
                .iter()
                .map(|(attr, values)| {
                    let values: Vec<&str> = values.iter().map(|v| v.name()).collect();
                    format!("{} {}", config.attribute_name(*attr), values.join(" · "))
                })
                .collect::<Vec<_>>()
                .join("   ")
        };
        let heading = |text: &'static str| {
            div()
                .w(px(STRIP_LABEL_WIDTH))
                .flex_shrink_0()
                .font_weight(pane_type::LABEL_WEIGHT)
                .text_color(chrome.muted_text)
                .child(text.to_uppercase())
        };
        div()
            .debug_selector(|| "workshop-checkbox-key".into())
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(18.))
            .py(px(10.))
            .border_t_1()
            .border_color(chrome.border)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(14.))
                    .child(heading("States"))
                    .children(self.states.iter().filter(|s| !s.template_only).map(|s| s.switch.clone()))
                    .child(div().flex_1())
                    .child(self.follow_switch.clone()),
            )
            .when(self.states.iter().any(|s| s.template_only), |strip| {
                strip.child(
                    div()
                        .debug_selector(|| "workshop-checkbox-template-states".into())
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(14.))
                        .child(heading("Template"))
                        .children(self.states.iter().filter(|s| s.template_only).map(|s| {
                            let selector = format!("workshop-checkbox-state-{}", config.state_name(s.state));
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(div().debug_selector(move || selector.clone()).child(s.switch.clone()))
                                .when(!s.reachable, |d| {
                                    d.child(div().text_color(chrome.muted_text).child("not settable on checkbox"))
                                })
                        }))
                        .child(
                            div().text_color(chrome.muted_text).child("no rule uses these; the template draws them"),
                        ),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(14.))
                    .child(heading("Attributes"))
                    .child(div().text_color(chrome.muted_text).child(attributes))
                    .child(heading("Modes"))
                    .child(match &self.modes {
                        Some(modes) => modes.clone().into_any_element(),
                        None => div().text_color(chrome.muted_text).child("none").into_any_element(),
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(chrome.border)
                            .font_family(pane_type::MONO)
                            .child(lss_inspector::key_line(&self.theme, self.control, &self.key)),
                    )
                    .child(self.reset.clone()),
            )
            .into_any_element()
    }

    /// The states the live checkbox is pinned to, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn pinned(&self) -> Option<PinnedCheckbox> {
        self.pinned
    }

    /// Turn "Track states" on or off as its switch would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn track_states(&mut self, follow: bool, cx: &mut Context<Self>) {
        self.set_follow(follow, cx);
    }

    /// Zoom the stage checkbox as the slider would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn zoom_to(&mut self, zoom: f32, cx: &mut Context<Self>) {
        self.set_zoom(zoom, cx);
    }

    /// The SDK checkbox, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn checkbox(&self) -> checkbox::Checkbox {
        self.checkbox.clone()
    }

    /// Whether a state is on in the key, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn state_on(&self, name: &str) -> bool {
        self.theme.stylesheet().config().state(name).is_some_and(|s| self.key.active_states().contains(s))
    }

    /// Toggle a state switch as a person would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle_state(&mut self, name: &str, cx: &mut Context<Self>) {
        let config = self.theme.stylesheet().config().clone();
        if let Some(state) = self.states.iter().find(|s| config.state_name(s.state) == name).map(|s| s.state) {
            let on = !self.key.active_states().contains(state);
            self.set_follow(false, cx);
            self.apply(self.key.set_state(state, on), cx);
        }
    }

    /// The matrix's column and row labels, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn matrix_labels(&self) -> (Vec<String>, Vec<String>) {
        let (columns, rows) = self.matrix_axes();
        (
            columns.iter().map(|c| self.states_label(&c.iter().copied().collect::<Vec<_>>(), "—")).collect(),
            rows.iter().map(|r| self.states_label(r, "base")).collect(),
        )
    }

    /// The template-only states and whether each is reachable, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn template_only_states(&self) -> Vec<(String, bool)> {
        let config = self.theme.stylesheet().config();
        self.states
            .iter()
            .filter(|s| s.template_only)
            .map(|s| (config.state_name(s.state).to_string(), s.reachable))
            .collect()
    }

    /// Choose a mode as the picker would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn choose_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        self.set_mode(mode, cx);
    }

    /// The bound mode and stage background, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn mode_and_stage(&self) -> (String, Option<Hsla>) {
        (self.theme.mode().to_string(), self.stage_background)
    }

    /// Show the state matrix, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn show_matrix(&self, cx: &mut gpui::App) {
        lss_inspector::choose_tab(&self.stage_tabs, "matrix", cx);
    }
}

impl ControlSide {
    fn stage_slot(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        // The grid zooms with the checkbox; the matrix is always at actual size.
        let (stage, zoom) = match self.stage {
            Stage::Control => (self.control_stage(), self.zoom),
            Stage::Matrix => (self.matrix(window, cx), 1.),
        };
        div()
            .id("workshop-checkbox-stage")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .when_some(self.stage_background, |stage, background| stage.bg(background))
            .child(centered_ruled_background(STAGE_GRID_SPACING * zoom))
            .child(div().relative().size_full().child(stage))
            .into_any_element()
    }

    fn lower_slot(&self) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .id("workshop-checkbox-lower")
            .size_full()
            .overflow_y_scroll()
            .child(self.key_strip())
            .child(div().px(px(18.)).py(px(10.)).border_t_1().border_color(chrome.border).child(self.events.clone()))
            .into_any_element()
    }
}

impl Render for ControlSide {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        div()
            .size_full()
            .min_w_0()
            .flex()
            .flex_col()
            .text_color(chrome.body_text)
            .typography_style(pane_type::body(&self.look))
            .child(self.tab_row())
            .child(div().flex_1().min_h_0().child(self.split.clone()))
    }
}

#[derive(Clone, Copy)]
enum Slot {
    Stage,
    Lower,
}

/// One panel of the control side's split. Split panels render views, so each panel is a view that
/// draws its part of the control side.
struct SideSlot {
    side: WeakEntity<ControlSide>,
    slot: Slot,
}

impl Render for SideSlot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let slot = self.slot;
        let content = self
            .side
            .update(cx, |side, cx| match slot {
                Slot::Stage => side.stage_slot(window, cx),
                Slot::Lower => side.lower_slot(),
            })
            .ok();
        div().size_full().children(content)
    }
}

/// `palette.app.background` in `theme`'s mode, read through the stage probe control.
fn app_background(theme: &Theme, probe: &Stylesheet) -> Option<Hsla> {
    let theme = theme.with_stylesheet(probe).ok()?;
    let stage = probe.control("workshop_stage")?;
    theme.resolve(&StyleKey::new(stage)).root().background().map(Hsla::from)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext, VisualTestContext};
    use crate::workshop::{Sample, Workshop};

    fn open(app: &mut TestAppContext) -> (Entity<Workshop>, Entity<CheckboxPane>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |v, cx| v.select("checkbox", cx));
        cx.run_until_parked();
        let pane = cx.update(|_, cx| match &workshop.read(cx).samples["checkbox"] {
            Sample::Checkbox(v) => v.clone(),
            _ => panic!("missing checkbox pane"),
        });
        (workshop, pane, cx)
    }

    fn winner_states(pane: &Entity<CheckboxPane>, part: &str, property: &str, cx: &mut VisualTestContext) -> String {
        cx.update(|_, cx| {
            let inspection = pane.read(cx).inspector().read(cx).inspection();
            let part = inspection.parts.iter().find(|p| p.part.name() == part).expect("part");
            let row = part.rows.iter().find(|r| r.property == property).expect("row");
            row.explanation.winner.as_ref().map(|w| w.states.clone()).unwrap_or_default()
        })
    }

    #[test]
    fn checkbox_pane_shows_control_key_events_and_inspector() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        for id in ["workshop-checkbox-pane", "workshop-checkbox-control", "workshop-checkbox-key", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no second inspector column");
        let (pane_bounds, title) = (
            cx.debug_bounds("workshop-checkbox-pane").unwrap(),
            cx.debug_bounds("workshop-checkbox-title").unwrap(),
        );
        assert_eq!(title.size.width, pane_bounds.size.width, "title spans both sides");
        let (left, right) = (
            cx.debug_bounds("workshop-checkbox-tab-row").unwrap(),
            cx.debug_bounds("lss-inspector-tab-row").unwrap(),
        );
        assert_eq!((left.top(), left.size.height), (right.top(), right.size.height), "tab rows line up");

        let inspector = cx.update(|_, cx| pane.read(cx).inspector());
        inspector.update(cx, |inspector, cx| inspector.select_part("indicator", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("style-inspector-part-indicator").is_some(), "indicator section rendered");
        assert!(cx.debug_bounds("style-inspector-row-indicator-background").is_some(), "background row rendered");
        assert!(cx.debug_bounds("style-inspector-box-indicator").is_some(), "indicator box diagram rendered");

        inspector.update(cx, |inspector, cx| inspector.expand("indicator", "background", cx));
        cx.run_until_parked();
        inspector.update(cx, |inspector, cx| inspector.show_source(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("lss-inspector-source").is_some(), "source view rendered");
        inspector.update(cx, |inspector, cx| inspector.show_palette(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("lss-inspector-palette").is_some(), "palette view rendered");
        let (file, used) = cx.update(|_, cx| inspector.read(cx).palette_lines_used());
        assert!(file.ends_with("palette.lss"), "the default (light) palette: {file}");
        assert!(
            !used.is_empty() && used.iter().all(|l| l.contains("= gray(")),
            "used lines are color tokens: {used:?}"
        );

        let side = cx.update(|_, cx| pane.read(cx).side());
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-matrix").is_some(), "state matrix rendered");
    }

    #[test]
    fn checkbox_pane_splits_start_half_and_half_with_a_resizable_lower_panel() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        cx.update(|_, cx| {
            let sizes = pane.read(cx).split.read(cx).panel_sizes_px();
            assert_eq!(sizes.len(), 2);
            assert!((sizes[0] - sizes[1]).abs() < 2., "control and inspector start equal: {sizes:?}");
            let side = pane.read(cx).side.read(cx);
            let sizes = side.split.read(cx).panel_sizes_px();
            assert!((sizes[1] - LOWER_HEIGHT).abs() < 2., "lower panel starts at its height: {sizes:?}");
        });
        assert!(cx.debug_bounds("workshop-checkbox-key").is_some(), "states in the lower panel");
    }

    #[gpui::test]
    fn zooming_scales_the_stage_checkbox_only() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        let inspector = cx.update(|_, cx| pane.read(cx).inspector());
        let inspected_height = |cx: &mut VisualTestContext| {
            cx.update(|_, cx| {
                let inspection = inspector.read(cx).inspection();
                let root = inspection.parts.iter().find(|p| p.part.name() == "root").expect("root");
                root.rows.iter().find(|r| r.property == "height").expect("height").explanation.value.clone()
            })
        };
        assert!(cx.debug_bounds("workshop-checkbox-zoom").is_some(), "the Control stage offers zoom");
        let actual = cx.debug_bounds("workshop-checkbox-control").expect("control").size;
        let actual_height = inspected_height(cx);

        side.update(cx, |side, cx| side.zoom_to(2.0, cx));
        cx.run_until_parked();
        let zoomed = cx.debug_bounds("workshop-checkbox-control").expect("control").size;
        assert!((zoomed.height - actual.height * 2.0).abs() <= px(1.), "{actual:?} → {zoomed:?}");
        assert!(zoomed.width > actual.width * 1.5, "the label grows too: {actual:?} → {zoomed:?}");
        assert_eq!(inspected_height(cx), actual_height, "the inspector shows actual-size values");

        side.update(cx, |side, cx| side.choose_mode("dark", cx));
        cx.run_until_parked();
        let dark = cx.debug_bounds("workshop-checkbox-control").expect("control").size;
        assert_eq!(dark.height, zoomed.height, "a mode change keeps the zoom");

        side.update(cx, |side, cx| side.zoom_to(9.0, cx));
        cx.run_until_parked();
        let most = cx.debug_bounds("workshop-checkbox-control").expect("control").size;
        assert!((most.height - actual.height * 5.0).abs() <= px(1.), "zoom stops at 5×: {most:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-zoom").is_none(), "the matrix is not zoomed");
    }

    #[test]
    fn choosing_a_mode_rebinds_the_control_matrix_inspector_and_stage() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        let inspector = cx.update(|_, cx| pane.read(cx).inspector());
        let (mode, light_stage) = cx.update(|_, cx| side.read(cx).mode_and_stage());
        assert_eq!(mode, "light");
        assert!(light_stage.is_some(), "the stage paints the mode's app background");

        side.update(cx, |side, cx| side.choose_mode("dark", cx));
        side.update(cx, |side, cx| side.toggle_state("checked", cx));
        cx.run_until_parked();
        let (mode, dark_stage) = cx.update(|_, cx| side.read(cx).mode_and_stage());
        assert_eq!(mode, "dark");
        assert_ne!(dark_stage, light_stage, "the stage follows the mode");
        assert_eq!(cx.update(|_, cx| inspector.read(cx).mode()), "dark");

        // :checked indicator background is the dark palette's, as the SDK's dark theme resolves it.
        let expected = {
            let dark = xgpui_luma::style::dark_theme();
            let key = dark.key_named("checkbox", &[], &["checked"]).expect("key");
            dark.resolve(&key).part(xgpui_luma::style::lss::PartId::new("indicator")).background()
        };
        let inspected = cx.update(|_, cx| {
            let inspection = inspector.read(cx).inspection();
            let part = inspection.parts.iter().find(|p| p.part.name() == "indicator").expect("part");
            let row = part.rows.iter().find(|r| r.property == "background").expect("row");
            match &row.explanation.value {
                Some(xgpui_luma::style::lss::Value::Color(c)) => Some(*c),
                _ => None,
            }
        });
        assert_eq!(inspected, expected);
        let (_, used) = cx.update(|_, cx| inspector.read(cx).palette_lines_used());
        assert!(used.iter().any(|l| l == "background = gray(90, 1)"), "the mode dark block's line is used: {used:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-matrix").is_some(), "the matrix draws in dark");
    }

    #[test]
    fn property_rows_are_one_line_columns_under_a_header() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let inspector = cx.update(|_, cx| pane.read(cx).inspector());
        assert!(cx.debug_bounds("lss-inspector-list-header").is_some(), "column header");
        // Names start at the Property column, whatever their length (the button must not centre them).
        let header = cx.debug_bounds("lss-inspector-header-property").expect("header").left();
        for id in [
            "lss-inspector-name-gap",
            "lss-inspector-name-disabled_opacity",
            "lss-inspector-name-indicator_scale",
        ] {
            assert_eq!(cx.debug_bounds(id).expect(id).left(), header, "{id} starts at the Property column");
        }
        let column = |id: &'static str, cx: &mut VisualTestContext| cx.debug_bounds(id).expect(id);
        let (value, rule) = (column("lss-inspector-header-value", cx), column("lss-inspector-header-rule", cx));
        assert_eq!(column("lss-inspector-value-height", cx).left(), value.left(), "values line up with Value");
        assert_eq!(column("lss-inspector-rule-height", cx).right(), rule.right(), "rules line up with Rule");
        // A literal shows no expression; a token reference shows it; the base rule has no selector.
        let summary = |part: &str, property: &'static str, cx: &mut VisualTestContext| {
            cx.update(|_, cx| inspector.read(cx).row_summary(part, property)).expect("row rendered")
        };
        assert_eq!(summary("root", "indicator_scale", cx), ("0.5".into(), String::new(), None));
        assert_eq!(summary("root", "height", cx), ("36px".into(), "metrics.control.control_height".into(), None));

        let side = cx.update(|_, cx| pane.read(cx).side());
        side.update(cx, |side, cx| side.toggle_state("checked", cx));
        inspector.update(cx, |inspector, cx| inspector.select_part("indicator", cx));
        cx.run_until_parked();
        let (_, expression, selector) = summary("indicator", "background", cx);
        assert_eq!(expression, "palette.state.selected.background");
        assert_eq!(selector.as_deref(), Some(":checked"), "a state rule shows its selector");
    }

    #[test]
    fn the_pane_chrome_follows_the_app_light_and_dark() {
        let mut app = TestAppContext::single();
        let (workshop, pane, cx) = open(&mut app);
        for mode in [xgpui_luma::theme::ThemeMode::Dark, xgpui_luma::theme::ThemeMode::Light] {
            workshop.update(cx, |v, cx| v.sync_theme(mode, cx));
            cx.run_until_parked();
            assert_eq!(cx.update(|_, cx| pane.read(cx).look_mode()), mode);
        }
    }

    #[test]
    fn picked_parts_each_get_an_accordion_section() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let (side, inspector) = cx.update(|_, cx| (pane.read(cx).side(), pane.read(cx).inspector()));
        assert!(cx.debug_bounds("style-inspector-part-root").is_some(), "root is picked at first");
        assert!(cx.debug_bounds("style-inspector-part-indicator").is_none());

        // Working on the checked state: indicator and glyph together, one section each.
        side.update(cx, |side, cx| side.toggle_state("checked", cx));
        inspector.update(cx, |inspector, cx| inspector.pick_parts(&["indicator", "glyph"], cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("style-inspector-part-root").is_none(), "root unpicked");
        assert!(cx.debug_bounds("style-inspector-row-indicator-background").is_some());
        assert!(cx.debug_bounds("style-inspector-row-glyph-foreground").is_some());
    }

    #[test]
    fn toggling_parts_turns_each_on_without_turning_others_off() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let inspector = cx.update(|_, cx| pane.read(cx).inspector());
        // No modifier keys: Right moves to the next part, Space toggles it, as a plain click does.
        let picker = cx.update(|_, cx| inspector.read(cx).part_picker_focus(cx));
        cx.update(|window, cx| picker.focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("right space right space");
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| inspector.read(cx).picked()), [0, 1, 2], "root, indicator and glyph");
        assert!(cx.debug_bounds("style-inspector-row-indicator-background").is_some());
        assert!(cx.debug_bounds("style-inspector-row-glyph-foreground").is_some());
        // Toggling a picked part turns just that one off.
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| inspector.read(cx).picked()), [0, 1], "glyph off, the others stay");
    }

    #[test]
    fn the_lower_panel_minimum_fits_the_whole_states_strip() {
        let mut app = TestAppContext::single();
        let (_, _pane, cx) = open(&mut app);
        let key = cx.debug_bounds("workshop-checkbox-key").expect("states strip");
        // The strip plus room for the events heading below it.
        assert!(f32::from(key.size.height) + 40. <= MIN_LOWER_HEIGHT, "states strip is {:?}", key.size.height);
    }

    #[test]
    fn checkbox_pane_rounds_its_left_corners_only_while_the_sidebar_is_hidden() {
        let mut app = TestAppContext::single();
        let (workshop, pane, cx) = open(&mut app);
        assert!(!cx.update(|_, cx| pane.read(cx).left_rounded), "sidebar shown: square");
        workshop.update(cx, |v, cx| v.set_sidebar_hidden(true, cx));
        assert!(cx.update(|_, cx| pane.read(cx).left_rounded), "sidebar hidden: rounded");
        workshop.update(cx, |v, cx| v.set_sidebar_hidden(false, cx));
        assert!(!cx.update(|_, cx| pane.read(cx).left_rounded), "sidebar shown again: square");
    }

    #[test]
    fn checkbox_pane_offers_template_states_and_builds_matrix_axes_from_the_states() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        // `focused` and `invalid`: set by the template, used by no rule; a checkbox can't be invalid.
        assert_eq!(
            cx.update(|_, cx| side.read(cx).template_only_states()),
            [("focused".to_string(), true), ("invalid".to_string(), false)]
        );
        assert!(cx.debug_bounds("workshop-checkbox-template-states").is_some(), "template row rendered");
        let invalid = cx.debug_bounds("workshop-checkbox-state-invalid").expect("invalid switch");
        let key_before = cx.update(|_, cx| side.read(cx).key);
        cx.simulate_click(invalid.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).key), key_before, "the :invalid switch is disabled");
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        // The exclusive group (hover < pressed < disabled) is one axis; the rest combine.
        assert_eq!(columns, ["—", ":hover", ":pressed", ":disabled"]);
        assert_eq!(
            rows,
            [
                "base",
                ":focused",
                ":checked",
                ":focused:checked",
                ":invalid",
                ":focused:invalid",
                ":checked:invalid",
                ":focused:checked:invalid"
            ]
        );
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-matrix").is_some(), "matrix renders every cell");
    }

    #[test]
    fn checkbox_key_tracks_focus_and_the_inspector_notes_no_rule_uses_it() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        let checkbox = cx.update(|_, cx| side.read(cx).checkbox());
        cx.update(|window, cx| checkbox.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        assert!(cx.update(|_, cx| side.read(cx).state_on("focused")), "tracks :focused");
        let note = cx.update(|_, cx| pane.read(cx).inspector().read(cx).unused_states_note());
        assert_eq!(note.as_deref(), Some("no rule uses :focused"));
        assert_eq!(winner_states(&pane, "indicator", "background", cx), "", "focus changes no value");
    }

    #[test]
    fn checkbox_key_follows_the_control_until_a_state_is_pinned() {
        let mut app = TestAppContext::single();
        let (workshop, pane, cx) = open(&mut app);
        assert_eq!(winner_states(&pane, "indicator", "background", cx), "");

        let checkbox = cx.update(|_, cx| pane.read(cx).side().read(cx).checkbox());
        cx.update(|window, cx| checkbox.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert!(cx.update(|_, cx| pane.read(cx).side().read(cx).state_on("checked")), "follows :checked");
        assert_eq!(winner_states(&pane, "indicator", "background", cx), ":checked");

        // Pinning :disabled stops following; unchecking the control no longer changes the key.
        let side = cx.update(|_, cx| pane.read(cx).side());
        side.update(cx, |side, cx| side.toggle_state("disabled", cx));
        cx.run_until_parked();
        assert_eq!(winner_states(&pane, "indicator", "background", cx), ":disabled");
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert!(cx.update(|_, cx| pane.read(cx).side().read(cx).state_on("checked")), "pinned key keeps :checked");

        // Retained across navigation.
        workshop.update(cx, |v, cx| v.select("pager", cx));
        workshop.update(cx, |v, cx| v.select("checkbox", cx));
        cx.run_until_parked();
        assert!(cx.update(|_, cx| pane.read(cx).side().read(cx).state_on("disabled")), "retained");
    }

    #[gpui::test]
    fn a_pinned_state_draws_until_tracking_resumes() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), None, "tracking: drawn as it is");

        // Pinning :focused draws the focus ring without keyboard focus, and keeps drawing it while
        // focus is elsewhere.
        side.update(cx, |side, cx| side.toggle_state("focused", cx));
        cx.run_until_parked();
        let focused =
            PinnedCheckbox { state: InteractionState { focused: true, ..Default::default() }, checked: false };
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), Some(focused));
        let checkbox = cx.update(|_, cx| side.read(cx).checkbox());
        cx.update(|window, cx| checkbox.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.update(|window, cx| window.blur(cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), Some(focused), "focus moving away keeps the pin");

        // More pinned states combine; zooming keeps them.
        side.update(cx, |side, cx| side.toggle_state("checked", cx));
        side.update(cx, |side, cx| side.zoom_to(2.0, cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), Some(PinnedCheckbox { checked: true, ..focused }));

        side.update(cx, |side, cx| side.track_states(true, cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), None, "tracking again: drawn as it is");
    }
}
