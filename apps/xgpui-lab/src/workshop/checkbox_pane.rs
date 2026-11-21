//! Checkbox work pane (issue #291): the SDK checkbox and its state matrix, the style key built from what
//! the checkbox's `.lss` declares, and the control's events, beside the `.lss` inspector in a resizable
//! split.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FontWeight, Render, WeakEntity, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ButtonRenderModel};
use xgpui_luma::controls::checkbox::{self, CheckboxData, CheckboxEvent};
use xgpui_luma::controls::resizable_panels::{ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation};
use xgpui_luma::controls::switch::Switch;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::style::lss::{ControlId, StateId, StyleKey, Theme};
use xgpui_luma::theme::InteractionState;

use super::event_stream::ControlEventStream;
use super::lss_inspector::{self, LssInspector};
use super::sample_layout::ruled_background;
use crate::lab_controls::Segmented;
use crate::theme::{ColorVizLookExt, Look, MONO_FAMILY, TextSize, TypographyExt};

const MIN_SIDE_WIDTH: f32 = 360.;
/// Initial height of the lower panel (states and events) under the stage.
const LOWER_HEIGHT: f32 = 300.;
const MIN_STAGE_HEIGHT: f32 = 160.;
const MIN_LOWER_HEIGHT: f32 = 120.;
/// The workshop canvas's inner corner radius (`CANVAS_RADIUS` less its border); the pane's outer
/// corners match it so nothing square shows past the rounded canvas.
const CANVAS_INNER_RADIUS: f32 = 9.;
const STRIP_LABEL_WIDTH: f32 = 78.;
/// State matrix rows: (label, checked, disabled).
const MATRIX_ROWS: [(&str, bool, bool); 4] = [
    ("base", false, false),
    (":checked", true, false),
    (":disabled", false, true),
    (":checked:disabled", true, true),
];
/// State matrix columns: (label, hovered, pressed).
const MATRIX_COLUMNS: [(&str, bool, bool); 4] = [
    ("—", false, false),
    (":hover", true, false),
    (":pressed", false, true),
    (":hover:pressed", true, true),
];

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Control,
    Matrix,
}

/// The work pane: the control side and the inspector, half each, with a draggable divider between them.
pub(super) struct CheckboxPane {
    look: Arc<Look>,
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
        let side = cx.new(|cx| ControlSide::new(look.clone(), theme, control, inspector.clone(), cx));
        let (side_view, inspector_view) = (side.clone(), inspector.clone());
        let split = xgpui_luma_look_radix::resizable_panels_builder(&look, "workshop-checkbox-split")
            .show_border(false)
            .panel(ResizablePanelSpec::new_render(move || side_view.clone()).weight(1.).min(px(MIN_SIDE_WIDTH)))
            .panel(ResizablePanelSpec::new_render(move || inspector_view.clone()).weight(1.).min(px(MIN_SIDE_WIDTH)))
            .spawn(cx);
        Self { look, side, inspector, split, left_rounded: false }
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
            .typography_style(self.look.typography_scale(TextSize::Xs))
            .child(self.split.clone())
    }
}

/// The control and its state matrix above, the style key and the control's events below, with a
/// draggable divider between them.
pub(super) struct ControlSide {
    look: Arc<Look>,
    theme: Theme,
    control: ControlId,
    checkbox: checkbox::Checkbox,
    stage: Stage,
    stage_tabs: Segmented,
    /// One switch per state the checkbox's rules use.
    states: Vec<(StateId, String, Switch)>,
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
                _ => {}
            }
        })
        .detach();

        let stage_tabs = Segmented::new(
            "workshop-checkbox-stage",
            "",
            look.as_ref(),
            &["Control", "State matrix"],
            0,
            cx,
            |this: &mut Self, i, cx| {
                this.stage = if i == 0 { Stage::Control } else { Stage::Matrix };
                cx.notify();
            },
        );
        let states = theme
            .stylesheet()
            .axes(control)
            .states
            .into_iter()
            .map(|state| {
                let name = config.state_name(state).to_string();
                let switch = xgpui_luma_look_radix::Switch::new(format!("workshop-checkbox-state-{name}"))
                    .look(look.as_ref())
                    .label(format!(":{name}"))
                    .checked(false)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        // Ignore the switch catching up with a followed change; a person's toggle pins the key.
                        if this.key.active_states().contains(state) == on {
                            return;
                        }
                        this.set_follow(false, cx);
                        this.apply(this.key.set_state(state, on), cx);
                    })
                    .spawn(cx);
                (state, name, switch)
            })
            .collect();
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
        Self {
            look,
            theme,
            control,
            checkbox,
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

    fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }

    /// Set one state by name in the key; a state the checkbox does not declare is ignored.
    fn set_state(&mut self, name: &str, on: bool, cx: &mut Context<Self>) {
        if let Some(state) = self.theme.stylesheet().config().state(name)
            && self.states.iter().any(|(s, _, _)| *s == state)
        {
            self.apply(self.key.set_state(state, on), cx);
        }
    }

    /// Use `key` for the matrix highlight, the state switches and the inspector.
    fn apply(&mut self, key: StyleKey, cx: &mut Context<Self>) {
        self.key = key;
        for (state, _, switch) in &self.states {
            let on = key.active_states().contains(*state);
            if *switch.read(cx).data() != on {
                switch.update(cx, |switch, cx| switch.set_data(on, cx));
            }
        }
        self.inspector.update(cx, |inspector, cx| inspector.set_key(key, cx));
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
        cx.notify();
    }

    /// The key for the checkbox as it is: only `checked`, since hover and press are transient.
    fn followed_key(&self, cx: &Context<Self>) -> StyleKey {
        let base = StyleKey::new(self.control);
        match self.theme.stylesheet().config().state("checked") {
            Some(checked) if self.follow => base.set_state(checked, *self.checkbox.read(cx).data()),
            _ => base,
        }
    }

    fn is_on(&self, name: &str) -> bool {
        self.theme.stylesheet().config().state(name).is_some_and(|s| self.key.active_states().contains(s))
    }

    fn header(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let inspection = self.theme.inspect(&self.key, None);
        let source = lss_inspector::control_source(&inspection)
            .map_or_else(String::new, |(file, layer)| format!("{file} · layer {layer}"));
        div()
            .flex_none()
            .flex()
            .items_end()
            .gap(px(16.))
            .px(px(18.))
            .py(px(10.))
            .border_b_1()
            .border_color(chrome.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(10.))
                            .child(
                                div()
                                    .typography_style(self.look.typography_scale(TextSize::Base))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(chrome.title_text)
                                    .child("Checkbox"),
                            )
                            .child(div().font_family(MONO_FAMILY).child(self.control.to_string()))
                            .child(div().font_family(MONO_FAMILY).text_color(chrome.muted_text).child(source)),
                    )
                    .children(inspection.doc.map(|d| div().text_color(chrome.muted_text).child(d))),
            )
            .child(div().flex_1())
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

    /// The SDK checkbox template rendered for each state combination, drawn bare (no cell chrome). The
    /// current key's row and column labels are highlighted.
    fn matrix(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();
        let accent = self.look.resolve_step(xgpui_luma_look_radix::ScaleFamily::Color, 11).hsla();
        let template = checkbox::default_checkbox_template();
        let label = |text: &str, current: bool| {
            div()
                .font_family(MONO_FAMILY)
                .text_color(if current { accent } else { chrome.muted_text })
                .when(current, |d| d.font_weight(FontWeight::SEMIBOLD))
                .child(text.to_string())
        };
        let current_row =
            |checked: bool, disabled: bool| self.is_on("checked") == checked && self.is_on("disabled") == disabled;
        let current_column =
            |hovered: bool, pressed: bool| self.is_on("hover") == hovered && self.is_on("pressed") == pressed;
        let mut grid =
            div().debug_selector(|| "workshop-checkbox-matrix".into()).p(px(20.)).flex().flex_col().gap(px(10.));
        grid =
            grid.child(div().flex().gap(px(10.)).child(div().w(px(120.))).children(
                MATRIX_COLUMNS.iter().map(|&(text, hovered, pressed)| {
                    div().flex_1().child(label(text, current_column(hovered, pressed)))
                }),
            ));
        for (r, (row_label, checked, disabled)) in MATRIX_ROWS.into_iter().enumerate() {
            let mut row = div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(div().w(px(120.)).child(label(row_label, current_row(checked, disabled))));
            for (c, (_, hovered, pressed)) in MATRIX_COLUMNS.into_iter().enumerate() {
                let model = ButtonRenderModel {
                    id: format!("workshop-checkbox-matrix-{r}-{c}").into(),
                    data: CheckboxData::new(checked),
                    state: InteractionState { hovered, pressed, disabled, ..Default::default() },
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
        let modes = if axes.modes.is_empty() {
            "none".to_string()
        } else {
            axes.modes.join(" · ")
        };
        let heading = |text: &'static str| {
            div()
                .w(px(STRIP_LABEL_WIDTH))
                .flex_shrink_0()
                .font_weight(FontWeight::SEMIBOLD)
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
                    .children(self.states.iter().map(|(_, _, switch)| switch.clone()))
                    .child(div().flex_1())
                    .child(self.follow_switch.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(14.))
                    .child(heading("Attributes"))
                    .child(div().text_color(chrome.muted_text).child(attributes))
                    .child(heading("Modes"))
                    .child(div().text_color(chrome.muted_text).child(modes))
                    .child(div().flex_1())
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(chrome.border)
                            .font_family(MONO_FAMILY)
                            .child(lss_inspector::key_line(&self.theme, self.control, &self.key)),
                    )
                    .child(self.reset.clone()),
            )
            .into_any_element()
    }

    /// The SDK checkbox, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn checkbox(&self) -> checkbox::Checkbox {
        self.checkbox.clone()
    }

    /// Whether a state is on in the key, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn state_on(&self, name: &str) -> bool {
        self.is_on(name)
    }

    /// Toggle a state switch as a person would, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn toggle_state(&mut self, name: &str, cx: &mut Context<Self>) {
        if let Some(state) = self.states.iter().find(|(_, n, _)| n == name).map(|(s, _, _)| *s) {
            let on = !self.key.active_states().contains(state);
            self.set_follow(false, cx);
            self.apply(self.key.set_state(state, on), cx);
        }
    }

    /// Show the state matrix, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn show_matrix(&self, cx: &mut gpui::App) {
        self.stage_tabs.select(1, cx);
    }
}

impl ControlSide {
    fn stage_slot(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let stage = match self.stage {
            Stage::Control => self.control_stage(),
            Stage::Matrix => self.matrix(window, cx),
        };
        div()
            .id("workshop-checkbox-stage")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .child(ruled_background())
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
            .typography_style(self.look.typography_scale(TextSize::Xs))
            .child(self.header())
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
}
