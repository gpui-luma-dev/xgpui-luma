//! Checkbox work pane (issue #291): the SDK checkbox and its state matrix, the style key built from what
//! the checkbox's `.lss` declares, and the control's events, beside the `.lss` inspector in a resizable
//! split.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, Render, WeakEntity, Window, div, prelude::*, px, svg};
use xgpui_luma::controls::button::{Button, ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::control_group::{
    ControlGroup, ControlGroupEvent, ControlGroupItem, ControlGroupItemLike, ControlGroupItemRenderModel,
    make_control_group_item_template,
};
use xgpui_luma::controls::checkbox::{self, CheckboxData, CheckboxEvent, StyledCheckboxTemplate};
use xgpui_luma::controls::resizable_panels::{ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation};
use xgpui_luma::controls::slider::Slider;
use xgpui_luma::controls::button::ButtonContentContext;
use xgpui_luma::controls::switch::{Switch, SwitchData};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::{ControlId, Source, StateId, StyleKey, Stylesheet, Theme};
use xgpui_luma::theme::InteractionState;

use super::event_stream::ControlEventStream;
use super::lss_inspector::{self, LssInspector, TAB_ROW_HEIGHT};
use super::catalog::COMPONENT_CATALOG;
use super::pinned_template::{PinnedCheckbox, PinnedCheckboxTemplate};
use super::sample_layout::centered_ruled_background;
use super::state_icons::state_icon;
use super::pane_type;
use crate::screen::{LabMetrics, text};
use crate::theme::{ColorVizLookExt, Look, TypographyExt};

const MIN_SIDE_WIDTH: f32 = 360.;
/// Initial height of the lower panel (states or events) under the stage.
const LOWER_HEIGHT: f32 = 300.;
const MIN_STAGE_HEIGHT: f32 = 160.;
/// The lower panel never gets shorter than this, so the state picker stays on screen.
const MIN_LOWER_HEIGHT: f32 = 240.;
/// The workshop canvas's inner corner radius (`CANVAS_RADIUS` less its border); the pane's outer
/// corners match it so nothing square shows past the rounded canvas.
const CANVAS_INNER_RADIUS: f32 = 9.;
const STRIP_LABEL_WIDTH: f32 = 96.;
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

/// How the person works with the live checkbox, like a designer's live and static previews.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum WorkMode {
    /// The checkbox is live: real pointer and keyboard input, the key follows it, events are logged.
    Test,
    /// The checkbox ignores input and draws the states picked below the stage; events are paused.
    Design,
}

impl WorkMode {
    fn id(self) -> &'static str {
        match self {
            WorkMode::Test => "test",
            WorkMode::Design => "design",
        }
    }
}

/// The stylesheet modes the light/dark switch moves between.
const LIGHT: &str = "light";
const DARK: &str = "dark";

/// The light/dark switch's track (vertical; on, the thumb is at the top): a moon in the empty end
/// below the thumb when dark, a sun in the empty end above it when light, in the track's text color.
///
/// The switch doesn't measure track content and its thumb travel ignores the track border (#311), so
/// the icon is placed by hand: centered in the shorter empty end the thumb leaves (track length less
/// both borders, the thumb and its padding), and small enough to stay clear of the rounded end.
fn mode_track_content(model: &ButtonContentContext<SwitchData>, _: &mut gpui::App) -> AnyElement {
    let dark = model.data.checked;
    let icon = if dark {
        lucide_svg_static::Icon::MoonStar
    } else {
        lucide_svg_static::Icon::Sun
    };
    // `look.height` is the track's outer length along the thumb's travel; `look.icon_size` is the thumb.
    let thumb = model.look.icon_size;
    let empty = (model.look.height - 2. * MODE_TRACK_BORDER - thumb - MODE_TRACK_PADDING).max(0.);
    let size = (empty - 4.).min(thumb * 0.6).max(0.);
    div()
        .size_full()
        .flex()
        .flex_col()
        .when(dark, |track| track.justify_end())
        .when(!dark, |track| track.justify_start())
        .child(
            div()
                .debug_selector(|| "workshop-checkbox-mode-icon".into())
                .flex_none()
                .h(px(empty))
                .flex()
                .items_center()
                .justify_center()
                .child(svg().size(px(size)).path(icon.asset_path()).text_color(model.look.foreground)),
        )
        .into_any_element()
}

/// The Radix switch's track border and `track_padding` (see #311).
const MODE_TRACK_BORDER: f32 = 1.;
const MODE_TRACK_PADDING: f32 = 1.;

/// The interaction picker's item for no state of the exclusive group.
const NO_INTERACTION: &str = "none";

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
    /// The inspected theme in the mode chosen with `mode_switch`.
    theme: Theme,
    /// Light/dark on the stage's top-right: a vertical Radix switch (size 2), on (thumb up) for dark,
    /// with a sun or moon in its track. Only when the stylesheet declares both modes.
    mode_switch: Option<Switch>,
    /// `theme`'s stylesheet plus a probe control reading the stage's colors (see [`STAGE_PROBE`]).
    stage_probe: Option<Stylesheet>,
    /// The mode's stage background (see [`stage_color`]), painted behind the control and its grid.
    stage_background: Option<Hsla>,
    /// The mode's text colors on the stage, for the matrix labels (see [`stage_text`]).
    stage_text: Option<StageText>,
    control: ControlId,
    checkbox: checkbox::Checkbox,
    /// The live checkbox's zoom on the Control stage, a visual aid: the checkbox draws from `theme` with
    /// every length scaled by it. The matrix and the inspector stay at actual size.
    zoom: f32,
    zoom_slider: Slider,
    /// The states the live checkbox draws in Design mode; `None` in Test mode, where it draws as it is.
    pinned: Option<PinnedCheckbox>,
    stage: Stage,
    /// Radix tabs: Control · State matrix.
    stage_tabs: Entity<xgpui_luma::controls::tabs::Tabs>,
    /// The key's states, in config order: those the file's rules use, then any the template sets
    /// that no rule uses.
    states: Vec<KeyState>,
    /// Test (live, the key follows the checkbox) or Design (the key is picked and the checkbox draws it).
    work_mode: WorkMode,
    /// Radix toggle group: Test · Design.
    work_mode_picker: ControlGroup<ControlGroupItem>,
    /// The config's exclusive states the key uses, weakest first (e.g. hover, pressed, disabled): only
    /// one counts, so Design mode picks one with `interaction`.
    exclusive: Vec<StateId>,
    /// Radix toggle group: none or one of `exclusive`.
    interaction: Option<ControlGroup<ControlGroupItem>>,
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
            // Design mode pauses the log. Enabling is the pane's own doing (Design disables input), not
            // something the person did to the checkbox.
            if this.work_mode == WorkMode::Design || matches!(event, CheckboxEvent::EnabledChanged { .. }) {
                return;
            }
            let line = format!("{event:?}");
            this.events.update(cx, |events, cx| events.append_line(&line, cx));
            match event {
                CheckboxEvent::Change { checked } => this.set_state("checked", *checked, cx),
                CheckboxEvent::HoverChanged { hovered } => this.set_state("hover", *hovered, cx),
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
        let states: Vec<KeyState> = config
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
                        // Ignore the switch catching up with the key; a person's toggle (Design mode) sets it.
                        if this.key.active_states().contains(state) == on {
                            return;
                        }
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
        let work_mode_picker = xgpui_luma_look_radix::control_group_builder(
            look.as_ref(),
            "workshop-checkbox-work-mode",
            xgpui_luma_look_radix::RadixSize::One,
        )
        .horizontal()
        .single_required()
        .items(
            [WorkMode::Test, WorkMode::Design]
                .map(|m| ControlGroupItem::new(m.id()).label(if m == WorkMode::Test { "Test" } else { "Design" })),
        )
        .selected_ids([WorkMode::Test.id()])
        .spawn(cx);
        cx.subscribe(&work_mode_picker, |this: &mut Self, _, event: &ControlGroupEvent, cx| {
            if let ControlGroupEvent::Change { selected_ids, .. } = event
                && let Some(id) = selected_ids.first()
            {
                let mode = if id.as_ref() == WorkMode::Design.id() {
                    WorkMode::Design
                } else {
                    WorkMode::Test
                };
                this.set_work_mode(mode, cx);
            }
        })
        .detach();
        let key_states: Vec<StateId> = states.iter().map(|s| s.state).collect();
        let mut exclusive: Vec<StateId> = config
            .exclusive_states()
            .filter_map(|n| config.state(n))
            .filter(|s| key_states.contains(s))
            .collect();
        exclusive.reverse();
        let interaction = (!exclusive.is_empty()).then(|| {
            let items =
                std::iter::once(ControlGroupItem::new(NO_INTERACTION).label("None")).chain(exclusive.iter().map(|s| {
                    let name = config.state_name(*s);
                    ControlGroupItem::new(name.to_string()).label(format!(":{name}"))
                }));
            let group = xgpui_luma_look_radix::control_group_builder(
                look.as_ref(),
                "workshop-checkbox-interaction",
                look.size(),
            )
            .horizontal()
            .single_required()
            .items(items)
            // Each item: its state's icon (the default icon for none), then its label.
            .item_template(make_control_group_item_template(
                |item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _: &mut Window, _: &mut gpui::App| {
                    let id = item.item.id().as_ref();
                    let state = (id != NO_INTERACTION).then_some(id);
                    let color = item.foreground.unwrap_or_else(gpui::transparent_black);
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .children(state_icon(state, color))
                        .child(item.item.label().clone())
                },
            ))
            .selected_ids([NO_INTERACTION])
            .spawn(cx);
            cx.subscribe(&group, |this: &mut Self, _, event: &ControlGroupEvent, cx| {
                if let ControlGroupEvent::Change { selected_ids, .. } = event
                    && let Some(id) = selected_ids.first()
                {
                    this.pick_interaction(id, cx);
                }
            })
            .detach();
            group
        });
        let reset = xgpui_luma_look_radix::Button::new("workshop-checkbox-reset")
            .look(look.as_ref())
            .outline()
            .size(xgpui_luma_look_radix::ButtonSize::One)
            .label("Reset")
            .on_click(cx, |this: &mut Self, cx| this.apply(StyleKey::new(this.control), cx))
            .spawn(cx);
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-checkbox-events",
                "The checkbox's SDK events, with seconds since the first.",
            )
            .without_card()
            .with_times()
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
        let has_mode = |name: &str| mode_names.iter().any(|m| m == name);
        let mode_switch = (has_mode(LIGHT) && has_mode(DARK)).then(|| {
            let switch = xgpui_luma_look_radix::Switch::new("workshop-checkbox-mode")
                .look(look.as_ref())
                .size(xgpui_luma_look_radix::SwitchSize::Two)
                .vertical()
                .checked(theme.mode() == DARK)
                .on_value_changed(cx, |this: &mut Self, dark, cx| this.set_mode(if dark { DARK } else { LIGHT }, cx))
                .spawn(cx);
            switch.update(cx, |switch, cx| switch.set_switch_track_content(mode_track_content, cx));
            switch
        });
        let stage_probe = theme.stylesheet().with_source("app", Source::new("workshop/stage.lss", STAGE_PROBE)).ok();
        let stage_background = stage_probe.as_ref().and_then(|probe| stage_color(&theme, probe));
        let stage_text = stage_probe.as_ref().and_then(|probe| stage_text(&theme, probe));
        Self {
            look,
            theme,
            mode_switch,
            stage_probe,
            stage_background,
            stage_text,
            control,
            checkbox,
            zoom: ZOOM_RANGE.start,
            zoom_slider,
            pinned: None,
            stage: Stage::Control,
            stage_tabs,
            states,
            work_mode: WorkMode::Test,
            work_mode_picker,
            exclusive,
            interaction,
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
        if let Some(switch) = &self.mode_switch
            && *switch.read(cx).data() != (mode == DARK)
        {
            switch.update(cx, |switch, cx| switch.set_data(mode == DARK, cx));
        }
        self.stage_background = self.stage_probe.as_ref().and_then(|probe| stage_color(&theme, probe));
        self.stage_text = self.stage_probe.as_ref().and_then(|probe| stage_text(&theme, probe));
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

    /// In Design mode the live checkbox draws the key's states; in Test mode it draws as it is.
    fn sync_pinned(&mut self, cx: &mut Context<Self>) {
        let pinned = (self.work_mode == WorkMode::Design).then(|| {
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
        if let Some(group) = &self.interaction {
            let id = self.interaction_id();
            if group.read(cx).selected_id().map(|s| s.as_ref()) != Some(id.as_str()) {
                group.update(cx, |group, cx| group.set_selected_ids([id], cx));
            }
        }
        self.inspector.update(cx, |inspector, cx| inspector.set_key(key, cx));
        self.sync_pinned(cx);
        cx.notify();
    }

    /// The interaction picker's item for the key: its strongest exclusive state, or none.
    fn interaction_id(&self) -> String {
        let config = self.theme.stylesheet().config();
        self.exclusive
            .iter()
            .rev()
            .find(|s| self.key.active_states().contains(**s))
            .map_or_else(|| NO_INTERACTION.to_string(), |s| config.state_name(*s).to_string())
    }

    /// Pick one exclusive state (or none) for the key, as the interaction picker does.
    fn pick_interaction(&mut self, id: &str, cx: &mut Context<Self>) {
        if id == self.interaction_id() {
            return;
        }
        let config = self.theme.stylesheet().config().clone();
        let key = self.exclusive.iter().fold(self.key, |key, s| key.set_state(*s, config.state_name(*s) == id));
        self.apply(key, cx);
    }

    /// Design keeps the key as it is (it becomes the picked states) and stops the checkbox taking input;
    /// Test drops the picks and goes live, with the key following the checkbox again.
    fn set_work_mode(&mut self, mode: WorkMode, cx: &mut Context<Self>) {
        if mode == self.work_mode {
            return;
        }
        self.work_mode = mode;
        if self.work_mode_picker.read(cx).selected_id().map(|s| s.as_ref()) != Some(mode.id()) {
            self.work_mode_picker.update(cx, |picker, cx| picker.set_selected_ids([mode.id()], cx));
        }
        self.checkbox.update(cx, |checkbox, cx| checkbox.set_enabled(mode == WorkMode::Test, cx));
        match mode {
            WorkMode::Test => self.apply(self.followed_key(cx), cx),
            WorkMode::Design => self.sync_pinned(cx),
        }
        cx.notify();
    }

    /// The key for the checkbox as it is: only `checked`, since hover, press and focus are transient.
    fn followed_key(&self, cx: &Context<Self>) -> StyleKey {
        let base = StyleKey::new(self.control);
        match self.theme.stylesheet().config().state("checked") {
            Some(checked) => base.set_state(checked, *self.checkbox.read(cx).data()),
            None => base,
        }
    }

    /// The stage toolbar, as tall as the inspector's tab row: Test · Design, the zoom (Control stage
    /// only) and the stage tabs.
    fn tab_row(&self) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .debug_selector(|| "workshop-checkbox-tab-row".into())
            .flex_none()
            .h(px(TAB_ROW_HEIGHT))
            .flex()
            .items_end()
            .gap(px(16.))
            .px(px(12.))
            .border_b_1()
            .border_color(chrome.border)
            .child(
                div()
                    .debug_selector(|| "workshop-checkbox-work-mode".into())
                    .self_center()
                    .child(self.work_mode_picker.clone()),
            )
            .child(div().flex_1().self_center().when(self.stage == Stage::Control, |slot| {
                slot.child(
                    div()
                        .debug_selector(|| "workshop-checkbox-zoom".into())
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(10.))
                        .child(div().font_weight(pane_type::LABEL_WEIGHT).text_color(chrome.muted_text).child("ZOOM"))
                        .child(div().w(px(ZOOM_SLIDER_WIDTH)).child(self.zoom_slider.clone()))
                        .child(div().w(px(36.)).font_family(pane_type::MONO).child(format!("{}×", self.zoom))),
                )
            }))
            .child(self.stage_tabs.clone())
            .into_any_element()
    }

    /// The live checkbox centered on the stage, with a tag saying whether it is live or drawing picked
    /// states.
    fn control_stage(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let tag = match self.work_mode {
            WorkMode::Test => "Live",
            WorkMode::Design => "Picked states · not live",
        };
        div()
            .relative()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .debug_selector(|| "workshop-checkbox-stage-tag".into())
                    .absolute()
                    .top(px(10.))
                    .left(px(12.))
                    .px(px(8.))
                    .py(px(2.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.content_background)
                    .text_color(chrome.muted_text)
                    .child(tag),
            )
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
        // The labels sit on the stage, so they take its mode's text colors, not the app's; the key's row
        // and column are in full-strength text and bold.
        let text = self.stage_text.unwrap_or(StageText { strong: chrome.body_text, muted: chrome.muted_text });
        let template = StyledCheckboxTemplate::new(self.theme.clone());
        // A header: each state's icon (the default icon for none), then the states as text.
        let label = |states: &[StateId], empty: &str, current: bool| {
            let color = if current { text.strong } else { text.muted };
            let icons: Vec<AnyElement> = if states.is_empty() {
                state_icon(None, color).into_iter().collect()
            } else {
                states.iter().filter_map(|s| state_icon(Some(config.state_name(*s)), color)).collect()
            };
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .font_family(pane_type::MONO)
                .text_color(color)
                .when(current, |d| d.font_weight(pane_type::LABEL_WEIGHT))
                .children(icons)
                .child(self.states_label(states, empty))
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
            let states: Vec<StateId> = column.iter().copied().collect();
            div().flex_1().child(label(&states, "—", *column == current_column))
        })));
        for (r, row_states) in rows.iter().enumerate() {
            let mut row = div().flex().items_center().gap(px(10.)).child(div().w(px(150.)).child(label(
                row_states,
                "base",
                current_row(row_states),
            )));
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

    /// Design mode's lower panel: the states the checkbox draws, grouped like a designer's state
    /// picker. One interaction state (the exclusive group), any of the other states, and the states the
    /// checkbox can never reach, shown but not settable.
    fn states_panel(&self, cx: &gpui::App) -> AnyElement {
        let chrome = self.look.chrome();
        let config = self.theme.stylesheet().config();
        let heading = |text: &'static str| {
            div()
                .w(px(STRIP_LABEL_WIDTH))
                .flex_shrink_0()
                .font_weight(pane_type::LABEL_WEIGHT)
                .text_color(chrome.muted_text)
                .child(text.to_uppercase())
        };
        let switch = |s: &KeyState| {
            let name = config.state_name(s.state);
            let selector = format!("workshop-checkbox-state-{name}");
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .children(state_icon(Some(name), chrome.muted_text))
                .child(div().debug_selector(move || selector.clone()).child(s.switch.clone()))
        };
        let others: Vec<&KeyState> =
            self.states.iter().filter(|s| s.reachable && !self.exclusive.contains(&s.state)).collect();
        let unreachable: Vec<&KeyState> = self.states.iter().filter(|s| !s.reachable).collect();
        let row = || div().flex().flex_wrap().items_center().gap(px(14.)).min_h(px(28.));
        div()
            .debug_selector(|| "workshop-checkbox-states".into())
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(18.))
            .py(px(12.))
            .child(
                row()
                    .child(div().font_weight(pane_type::LABEL_WEIGHT).child("States"))
                    .child(div().text_color(chrome.muted_text).child("pick what the checkbox draws"))
                    .child(div().flex_1())
                    .child(self.reset.clone()),
            )
            .children(self.interaction.as_ref().map(|group| row().child(heading("Interaction")).child(group.clone())))
            .when(!others.is_empty(), |panel| {
                // States no rule uses still change the drawing when the template reads them (the focus ring).
                let template_only: Vec<String> = others
                    .iter()
                    .filter(|s| s.template_only)
                    .map(|s| format!(":{}", config.state_name(s.state)))
                    .collect();
                panel.child(row().child(heading("States")).children(others.iter().map(|s| switch(s))).when(
                    !template_only.is_empty(),
                    |row| {
                        row.child(
                            div()
                                .text_color(chrome.muted_text)
                                .child(format!("{}: no rule uses it; the template draws it", template_only.join(" "))),
                        )
                    },
                ))
            })
            .when(!unreachable.is_empty(), |panel| {
                panel.child(
                    row()
                        .child(heading("Not settable"))
                        .children(unreachable.iter().map(|s| switch(s)))
                        .child(div().text_color(chrome.muted_text).child("a checkbox can't reach these")),
                )
            })
            .child(
                div()
                    .pt(px(4.))
                    .text_color(chrome.muted_text)
                    .child(format!("Events paused in Design · {} logged in Test", self.events.read(cx).count())),
            )
            .into_any_element()
    }

    /// Test mode's lower panel: the key's live states, then the checkbox's events.
    fn events_panel(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let config = self.theme.stylesheet().config();
        let live: Vec<&str> = self
            .states
            .iter()
            .filter(|s| self.key.active_states().contains(s.state))
            .map(|s| config.state_name(s.state))
            .collect();
        let tag = |name: &str| {
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .px(px(8.))
                .py(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(chrome.border)
                .font_family(pane_type::MONO)
                .text_color(chrome.muted_text)
                .children(state_icon(Some(name), chrome.muted_text))
                .child(format!(":{name}"))
        };
        div()
            .debug_selector(|| "workshop-checkbox-live".into())
            .flex_none()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(10.))
                    .px(px(18.))
                    .py(px(10.))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(
                        div()
                            .w(px(STRIP_LABEL_WIDTH))
                            .font_weight(pane_type::LABEL_WEIGHT)
                            .text_color(chrome.muted_text)
                            .child("LIVE STATES"),
                    )
                    .when(live.is_empty(), |row| row.child(div().text_color(chrome.muted_text).child("none")))
                    .children(live.into_iter().map(tag)),
            )
            .child(div().px(px(18.)).py(px(10.)).child(self.events.clone()))
            .into_any_element()
    }

    /// The states the live checkbox is pinned to, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn pinned(&self) -> Option<PinnedCheckbox> {
        self.pinned
    }

    /// Choose Test or Design as the picker would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn choose_work_mode(&mut self, mode: WorkMode, cx: &mut Context<Self>) {
        self.set_work_mode(mode, cx);
    }

    /// Test or Design, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn work_mode(&self) -> WorkMode {
        self.work_mode
    }

    /// Pick an interaction state (or `"none"`) as the picker would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn choose_interaction(&mut self, id: &str, cx: &mut Context<Self>) {
        self.pick_interaction(id, cx);
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

    /// Toggle a state in Design mode as a person would (switching to Design first), for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle_state(&mut self, name: &str, cx: &mut Context<Self>) {
        let config = self.theme.stylesheet().config().clone();
        if let Some(state) = self.states.iter().find(|s| config.state_name(s.state) == name).map(|s| s.state) {
            self.set_work_mode(WorkMode::Design, cx);
            let on = !self.key.active_states().contains(state);
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

    /// The stage's text colors, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn stage_text_colors(&self) -> Option<(Hsla, Hsla)> {
        self.stage_text.map(|t| (t.strong, t.muted))
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
            .relative()
            .size_full()
            .child(
                div()
                    .id("workshop-checkbox-stage")
                    .debug_selector(|| "workshop-checkbox-stage".into())
                    .relative()
                    .size_full()
                    .overflow_y_scroll()
                    .when_some(self.stage_background, |stage, background| stage.bg(background))
                    .child(centered_ruled_background(STAGE_GRID_SPACING * zoom, self.stage_background))
                    .child(div().relative().size_full().child(stage)),
            )
            // Light/dark stays in the top-right corner while the matrix scrolls under it.
            .children(self.mode_switch.clone().map(|switch| {
                div()
                    .debug_selector(|| "workshop-checkbox-modes".into())
                    .absolute()
                    .top(px(10.))
                    .right(px(12.))
                    .child(switch)
            }))
            .into_any_element()
    }

    /// The lower panel for the work mode: the state picker in Design, the events in Test.
    fn lower_slot(&self, cx: &gpui::App) -> AnyElement {
        let chrome = self.look.chrome();
        div()
            .id("workshop-checkbox-lower")
            .size_full()
            .overflow_y_scroll()
            .border_t_1()
            .border_color(chrome.border)
            .child(match self.work_mode {
                WorkMode::Design => self.states_panel(cx),
                WorkMode::Test => self.events_panel(),
            })
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
                Slot::Lower => side.lower_slot(cx),
            })
            .ok();
        div().size_full().children(content)
    }
}

/// The stage's background in `theme`'s mode: `palette.surface.subtle.background`, a step off the app
/// background (lighter in dark mode), read through the stage probe control.
fn stage_color(theme: &Theme, probe: &Stylesheet) -> Option<Hsla> {
    let theme = theme.with_stylesheet(probe).ok()?;
    let stage = probe.control("workshop_stage")?;
    theme.resolve(&StyleKey::new(stage)).root().background().map(Hsla::from)
}

/// The stage probe: the stage's background, and the text colors that read on it in each mode.
const STAGE_PROBE: &str = "workshop_stage {
  background = palette.surface.subtle.background
  foreground = palette.app.foreground
  muted: color = palette.app.muted_foreground
}";

/// Text colors on the stage, from its mode.
#[derive(Clone, Copy, Debug, PartialEq)]
struct StageText {
    /// Full-strength text: `palette.app.foreground`.
    strong: Hsla,
    /// Secondary text: `palette.app.muted_foreground`.
    muted: Hsla,
}

/// The stage's text colors in `theme`'s mode, read through the stage probe control.
fn stage_text(theme: &Theme, probe: &Stylesheet) -> Option<StageText> {
    let theme = theme.with_stylesheet(probe).ok()?;
    let stage = probe.control("workshop_stage")?;
    let resolved = theme.resolve(&StyleKey::new(stage));
    let root = resolved.root();
    let strong = root.foreground().map(Hsla::from)?;
    let muted = root.value("muted").and_then(|v| v.as_color()).map(Hsla::from)?;
    Some(StageText { strong, muted })
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
        for id in [
            "workshop-checkbox-pane",
            "workshop-checkbox-modes",
            "workshop-checkbox-work-mode",
            "workshop-checkbox-control",
            "workshop-checkbox-live",
            "style-inspector",
            "lss-inspector-attributes",
        ] {
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
        assert!(cx.debug_bounds("workshop-checkbox-live").is_some(), "Test mode: events in the lower panel");
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
        assert!(light_stage.is_some(), "the stage paints the mode's subtle surface");

        // The light/dark switch sits in the stage's top-right corner; turning it on binds dark.
        let (stage, corner) = (
            cx.debug_bounds("workshop-checkbox-stage").expect("stage"),
            cx.debug_bounds("workshop-checkbox-modes").expect("light/dark switch"),
        );
        assert!(corner.right() <= stage.right() && corner.right() > stage.right() - px(40.), "right edge");
        assert!(corner.top() >= stage.top() && corner.top() < stage.top() + px(20.), "top edge");
        // Radix size 2, vertical: a 20×35 track with 2px focus-ring room around it.
        assert_eq!((corner.size.width, corner.size.height), (px(24.), px(39.)), "size 2, vertical");
        // Light: the thumb (18px) rests at the bottom; the sun's area is the empty top end, 14px (35 less
        // two 1px borders, the thumb and 1px padding), clear of the thumb.
        let inner_top = corner.top() + px(2. + 1.);
        let sun = cx.debug_bounds("workshop-checkbox-mode-icon").expect("sun");
        assert_eq!(sun.size.height, px(14.), "the empty end: {sun:?}");
        assert!(sun.top() >= inner_top - px(0.5) && sun.bottom() <= inner_top + px(14.5), "sun up top: {sun:?}");
        let switch = cx.update(|_, cx| side.read(cx).mode_switch.clone()).expect("switch");
        assert!(!cx.update(|_, cx| *switch.read(cx).data()), "light: off");
        cx.simulate_click(corner.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).mode_and_stage()).0, "dark", "the switch binds dark");
        // Dark: the thumb rests at the top (1px padding, 18px), so the moon's area starts below it.
        let moon = cx.debug_bounds("workshop-checkbox-mode-icon").expect("moon");
        assert!(
            moon.top() >= inner_top + px(19. - 0.5) && moon.bottom() <= inner_top + px(33.5),
            "moon below the thumb: {moon:?}"
        );
        side.update(cx, |side, cx| side.choose_mode("light", cx));
        cx.run_until_parked();
        assert!(!cx.update(|_, cx| *switch.read(cx).data()), "the switch follows the mode");

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
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        cx.run_until_parked();
        let states = cx.debug_bounds("workshop-checkbox-states").expect("state picker");
        assert!(f32::from(states.size.height) <= MIN_LOWER_HEIGHT, "state picker is {:?}", states.size.height);
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
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-states").is_some(), "Design mode: the state picker");
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

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Test, cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), None, "tracking again: drawn as it is");
    }

    #[gpui::test]
    fn test_is_live_and_design_draws_picked_states() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        let checkbox = cx.update(|_, cx| side.read(cx).checkbox());
        let control = || "workshop-checkbox-control";
        let logged = |cx: &mut VisualTestContext| cx.update(|_, cx| side.read(cx).events.read(cx).count());

        // Test: live. Clicking checks the checkbox, the key follows and the event is logged.
        assert_eq!(cx.update(|_, cx| side.read(cx).work_mode()), WorkMode::Test);
        assert!(cx.debug_bounds("workshop-checkbox-live").is_some());
        assert!(cx.debug_bounds("workshop-checkbox-states").is_none());
        let at = cx.debug_bounds(control()).expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *checkbox.read(cx).data()), "clicked in Test");
        assert!(cx.update(|_, cx| side.read(cx).state_on("checked")));
        let in_test = logged(cx);
        assert!(in_test > 0, "events logged in Test");
        let text = cx.update(|_, cx| side.read(cx).events.read(cx).text(cx));
        assert!(text.starts_with("[1] 0.000s "), "the first event is at 0s: {text}");
        assert!(
            text.lines().all(|l| l.split(' ').nth(1).is_some_and(|t| t.ends_with('s'))),
            "every line is timed: {text}"
        );

        // Design: the picker shows, the checkbox ignores input and the log pauses.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-states").is_some());
        assert!(cx.debug_bounds("workshop-checkbox-live").is_none());
        let at = cx.debug_bounds(control()).expect("control").center();
        cx.simulate_click(at, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(cx.update(|_, cx| *checkbox.read(cx).data()), "Design: the click is ignored");
        assert_eq!(logged(cx), in_test, "Design: no new events");

        // One interaction state at a time; picking :pressed replaces :hover.
        side.update(cx, |side, cx| side.choose_interaction("hover", cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        assert!(cx.update(|_, cx| side.read(cx).state_on("pressed")));
        assert!(!cx.update(|_, cx| side.read(cx).state_on("hover")));
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.state.pressed && !pinned.state.hovered && pinned.checked);

        // Back to Test drops the picks: the key is the checkbox as it is, drawn live.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Test, cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), None);
        assert!(!cx.update(|_, cx| side.read(cx).state_on("pressed")), "picks dropped");
        assert!(cx.update(|_, cx| side.read(cx).state_on("checked")), "follows the checkbox again");
    }

    #[gpui::test]
    fn matrix_labels_take_the_stage_modes_text_colors() {
        let mut app = TestAppContext::single();
        let (_, pane, cx) = open(&mut app);
        let side = cx.update(|_, cx| pane.read(cx).side());
        // Text and muted text sit well apart from the stage background: darker on the light stage,
        // lighter on the dark one.
        for (mode, light) in [("light", true), ("dark", false)] {
            side.update(cx, |side, cx| side.choose_mode(mode, cx));
            cx.run_until_parked();
            let (_, background) = cx.update(|_, cx| side.read(cx).mode_and_stage());
            let background = background.expect("stage background");
            let (strong, muted) = cx.update(|_, cx| side.read(cx).stage_text_colors()).expect("stage text");
            for (name, color) in [("text", strong), ("muted", muted)] {
                let gap = if light {
                    background.l - color.l
                } else {
                    color.l - background.l
                };
                assert!(gap > 0.3, "{mode} {name} reads on the stage: {color:?} on {background:?}");
            }
            assert!(
                (strong.l - background.l).abs() > (muted.l - background.l).abs(),
                "{mode}: text is stronger than muted"
            );
        }
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-checkbox-matrix").is_some(), "the matrix draws in dark");
    }
}
