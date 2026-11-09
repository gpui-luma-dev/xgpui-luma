//! One editable color (optionally positioned): the building block of
//! [`crate::color_stop_list::ColorStopList`], and usable on its own (e.g. a
//! mesh background color).
//!
//! The control owns the color/position state, its child controls (remove
//! `Button`, inline picker), focus and input isolation. Layout is delegated to
//! a [`ColorStopTemplate`]; see [`template`].
//!
//! The picker is shown while focus is inside the stop (clicking the swatch
//! focuses it; focus moving anywhere else, or a click outside, hides it), or
//! while a host holds it open with [`ColorStop::set_open`] (e.g. while the
//! stop's knob on a gradient bar owns focus).
//!
//! Host setters (`set_*`) never emit; user edits emit [`ColorStopEvent`].

mod picker;
pub(crate) mod template;

use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, AppContext, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    MouseButton, Pixels, Render, SharedString, Subscription, Window, div, prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::color::ColorValue;
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::controls::tooltip::Tooltip;
use xgpui_luma::infra::attachments::{TooltipEntityExt, TooltipHandle};
use xgpui_luma_look_radix as radix;

use crate::theme::Look;

use picker::{InlineColorPicker, InlineColorPickerEvent};
pub(crate) use template::{
    ColorStopRenderModel, ColorStopTemplate, ColorStopTemplateHandlers, DEFAULT_LINE_HEIGHT,
    default_color_stop_template, make_color_stop_template,
};

/// User edits. A parent may veto or adjust them and push the result back with
/// the host setters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ColorStopEvent {
    ColorChanged(ColorValue),
    /// The remove action was pressed; the owner decides whether to remove.
    RemoveRequested,
    /// Focus entering or leaving the stop showed (`true`) or hid (`false`) the
    /// picker. Not emitted for [`ColorStop::set_open`].
    ExpandedChanged(bool),
}

pub(crate) struct ColorStopBuilder {
    id: SharedString,
    look: Option<Arc<Look>>,
    color: ColorValue,
    position: Option<f32>,
    removable: bool,
    template: Option<Arc<dyn ColorStopTemplate>>,
}

impl ColorStopBuilder {
    pub fn new(id: impl Into<SharedString>, color: ColorValue) -> Self {
        Self { id: id.into(), look: None, color, position: None, removable: false, template: None }
    }

    pub fn look(mut self, look: Arc<Look>) -> Self {
        self.look = Some(look);
        self
    }

    /// Give the stop a position (`0.0..=1.0`), shown as a percentage.
    pub fn position(mut self, position: f32) -> Self {
        self.position = Some(position.clamp(0.0, 1.0));
        self
    }

    /// Show a remove action that emits [`ColorStopEvent::RemoveRequested`].
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    pub fn template(mut self, template: Arc<dyn ColorStopTemplate>) -> Self {
        self.template = Some(template);
        self
    }

    pub fn spawn(self, cx: &mut App) -> Entity<ColorStop> {
        cx.new(|cx| ColorStop::new(self, cx))
    }
}

pub(crate) struct ColorStop {
    id: SharedString,
    look: Arc<Look>,
    template: Arc<dyn ColorStopTemplate>,
    color: ColorValue,
    position: Option<f32>,
    index: Option<usize>,
    selected: bool,
    expanded: bool,
    /// Focus is on the stop or inside its picker.
    focus_within: bool,
    /// Held open by the host regardless of focus.
    open: bool,
    focus_handle: FocusHandle,
    swatch_hovered: bool,
    /// Swatch help for the closed and open states.
    swatch_tooltips: [TooltipHandle; 2],
    /// Focus in/out listeners; registered on first render (they need a window).
    focus_subscriptions: Option<[Subscription; 2]>,
    remove_button: Option<Entity<Button>>,
    picker: Entity<InlineColorPicker>,
    /// Host text shown when the stop has no position (e.g. point coordinates).
    detail: Option<SharedString>,
    /// Host content shown under the picker while the editor is open.
    editor_extra: Option<AnyView>,
    height: Option<f32>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorStopEvent> for ColorStop {}

impl ColorStop {
    fn new(builder: ColorStopBuilder, cx: &mut Context<Self>) -> Self {
        let look = builder.look.unwrap_or_else(crate::theme::default_look);
        let id = builder.id;
        let mut subscriptions = Vec::new();

        let remove_button = builder.removable.then(|| {
            let button = crate::theme::icon_button(format!("{id}-remove"), LucideIcon::Trash)
                .look(look.as_ref())
                .content_only()
                .radius(radix::Radius::Medium)
                .spawn(cx)
                .tooltip(Tooltip::new("Remove color"), cx);
            subscriptions.push(cx.subscribe(&button, |_, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    cx.emit(ColorStopEvent::RemoveRequested);
                }
            }));
            button
        });

        let swatch_tooltips = swatch_tooltips(&look, cx);
        let picker = InlineColorPicker::spawn(format!("{id}-picker"), look.as_ref(), builder.color, cx);
        subscriptions.push(cx.subscribe(&picker, |this, _, event: &InlineColorPickerEvent, cx| {
            let InlineColorPickerEvent::Change(color) = *event;
            if this.color != color {
                this.color = color;
                cx.emit(ColorStopEvent::ColorChanged(color));
                cx.notify();
            }
        }));

        Self {
            id,
            look,
            template: builder.template.unwrap_or_else(default_color_stop_template),
            color: builder.color,
            position: builder.position,
            index: None,
            selected: false,
            expanded: false,
            focus_within: false,
            open: false,
            focus_handle: cx.focus_handle(),
            swatch_hovered: false,
            swatch_tooltips,
            focus_subscriptions: None,
            remove_button,
            picker,
            detail: None,
            editor_extra: None,
            height: None,
            _subscriptions: subscriptions,
        }
    }

    // ---- Host API (never emits) ------------------------------------------

    #[allow(dead_code)]
    pub fn color(&self) -> ColorValue {
        self.color
    }

    #[allow(dead_code)]
    pub fn position(&self) -> Option<f32> {
        self.position
    }

    /// Order within a parent list, if any.
    #[allow(dead_code)]
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    #[allow(dead_code)]
    pub fn expanded(&self) -> bool {
        self.expanded
    }

    /// Laid-out height reported by the template, once rendered.
    pub fn height(&self) -> Option<f32> {
        self.height
    }

    pub fn set_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        if self.color != color {
            self.color = color;
            self.picker.update(cx, |picker, cx| picker.set_color(color, cx));
            cx.notify();
        }
    }

    pub fn set_position(&mut self, position: f32, cx: &mut Context<Self>) {
        let position = Some(position.clamp(0.0, 1.0));
        if self.position != position {
            self.position = position;
            cx.notify();
        }
    }

    pub fn set_index(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        if self.index != index {
            self.index = index;
            cx.notify();
        }
    }

    pub fn set_selected(&mut self, selected: bool, cx: &mut Context<Self>) {
        if self.selected != selected {
            self.selected = selected;
            cx.notify();
        }
    }

    /// Hold the picker open (or release it) independent of focus.
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            self.expanded = self.open || self.focus_within;
            cx.notify();
        }
    }

    /// Trailing text for a stop without a position.
    pub fn set_detail(&mut self, detail: Option<SharedString>, cx: &mut Context<Self>) {
        if self.detail != detail {
            self.detail = detail;
            cx.notify();
        }
    }

    /// Host content (e.g. per-point settings) placed under the picker.
    pub fn set_editor_extra(&mut self, extra: Option<AnyView>, cx: &mut Context<Self>) {
        let id = |view: &Option<AnyView>| view.as_ref().map(AnyView::entity_id);
        if id(&self.editor_extra) != id(&extra) {
            self.editor_extra = extra;
            cx.notify();
        }
    }

    pub fn set_can_remove(&mut self, can_remove: bool, cx: &mut Context<Self>) {
        if let Some(button) = &self.remove_button {
            button.update(cx, |button, cx| button.set_enabled(can_remove, cx));
        }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.swatch_tooltips = swatch_tooltips(&look, cx);
        self.look = look;
        cx.notify();
    }

    // ---- User edits -------------------------------------------------------

    /// Wrap an interactive child so pointer input stays with it and never
    /// reaches a surrounding row (selection, drag source).
    fn isolated(&self, name: &str, child: impl IntoElement) -> AnyElement {
        div()
            .id(format!("{}-{name}", self.id))
            .w_full()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
            .child(child)
            .into_any_element()
    }

    /// Swatch pressed while the picker is open: close it, giving up focus and
    /// any host hold.
    fn close_by_user(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_handle.contains_focused(window, cx) {
            window.blur(cx);
        }
        self.open = false;
        self.set_focus_within(false, cx);
    }

    fn set_focus_within(&mut self, focus_within: bool, cx: &mut Context<Self>) {
        self.focus_within = focus_within;
        let expanded = self.open || focus_within;
        if self.expanded != expanded {
            self.expanded = expanded;
            cx.emit(ColorStopEvent::ExpandedChanged(expanded));
            cx.notify();
        }
    }

    fn ensure_focus_listeners(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_subscriptions.is_some() {
            return;
        }
        let focus_in = cx.on_focus_in(&self.focus_handle, window, |this, _, cx| this.set_focus_within(true, cx));
        let focus_out = cx.on_focus_out(&self.focus_handle, window, |this, _, _, cx| this.set_focus_within(false, cx));
        self.focus_subscriptions = Some([focus_in, focus_out]);
    }

    fn handlers(&self, cx: &mut Context<Self>) -> ColorStopTemplateHandlers {
        let entity = cx.entity().downgrade();
        let focus = self.focus_handle.clone();
        ColorStopTemplateHandlers {
            swatch_click: Box::new({
                let entity = entity.clone();
                move |_, window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        if this.expanded {
                            this.close_by_user(window, cx);
                        } else {
                            window.focus(&focus, cx);
                        }
                    });
                }
            }),
            swatch_hover: Box::new({
                let entity = entity.clone();
                move |hovered, _, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        if this.swatch_hovered != hovered {
                            this.swatch_hovered = hovered;
                            cx.notify();
                        }
                    });
                }
            }),
            bounds: Box::new(move |bounds: Bounds<Pixels>, _, cx| {
                let height = bounds.size.height.as_f32();
                let _ = entity.update(cx, |this, cx| {
                    if this.height.is_none_or(|current| (current - height).abs() > 0.5) {
                        this.height = Some(height);
                        cx.notify();
                    }
                });
            }),
        }
    }
}

impl Render for ColorStop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_focus_listeners(window, cx);
        let model = ColorStopRenderModel {
            id: &self.id,
            look: &self.look,
            color: self.color,
            label: stop_label(self.color).into(),
            alpha: self.color.alpha(),
            position: self.position,
            detail_label: self.position.map(|position| format_percent(position).into()).or_else(|| self.detail.clone()),
            index: self.index,
            selected: self.selected,
            expanded: self.expanded,
            remove_button: self.remove_button.clone().map(|button| self.isolated("remove-slot", button)),
            editor: self.expanded.then(|| {
                let editor = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.picker.clone())
                    .children(self.editor_extra.clone());
                self.isolated("editor", editor)
            }),
            swatch_tooltip: Some(self.swatch_tooltips[usize::from(self.expanded)].clone()),
            swatch_hovered: self.swatch_hovered,
        };
        let handlers = self.handlers(cx);
        let content = self.template.render(model, handlers, window, cx);
        // The focus scope wraps the whole template so the picker's own controls
        // count as "inside"; a click elsewhere hides the picker.
        let focus = self.focus_handle.clone();
        div()
            .track_focus(&self.focus_handle)
            .w_full()
            .on_mouse_down_out(move |_, window, cx| {
                if focus.contains_focused(window, cx) {
                    window.blur(cx);
                }
            })
            .child(content)
    }
}

impl Focusable for ColorStop {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn swatch_tooltips(look: &Look, cx: &mut App) -> [TooltipHandle; 2] {
    let theme = radix::tooltip_theme(look);
    ["Edit color", "Close color editor"]
        .map(|text| TooltipHandle::new_with_theme(Tooltip::new(text), theme.clone(), cx))
}

/// CSS `hsl()` text (alpha is shown separately), e.g. `hsl(312 48% 44%)`.
///
/// Derived from unmapped sRGB: a source outside sRGB (e.g. Display P3) shows
/// out-of-range saturation/lightness rather than being silently clipped, so a
/// later out-of-gamut indicator can build on it.
pub(crate) fn stop_label(color: ColorValue) -> String {
    use palette::{Hsl, convert::FromColorUnclamped};
    color
        .to_srgba_unclamped()
        .map(|rgba| {
            let hsl = Hsl::from_color_unclamped(rgba.color);
            format!(
                "hsl({:.0} {:.0}% {:.0}%)",
                hsl.hue.into_positive_degrees(),
                hsl.saturation * 100.0,
                hsl.lightness * 100.0
            )
        })
        .unwrap_or_else(|_| "Invalid color".into())
}

fn format_percent(position: f32) -> String {
    format!("{}%", (position * 100.0).round() as i32)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::color::{GamutMapping, gpui_bridge};
    use std::cell::RefCell;
    use std::rc::Rc;
    use gpui::TestAppContext;

    fn spawn_with_events(
        app: &mut TestAppContext,
        builder: ColorStopBuilder,
    ) -> (Entity<ColorStop>, Rc<RefCell<Vec<ColorStopEvent>>>) {
        app.update(|cx| {
            xgpui_luma::init(cx).expect("initialize SDK");
            let stop = builder.spawn(cx);
            let events = Rc::new(RefCell::new(Vec::new()));
            let sink = events.clone();
            cx.subscribe(&stop, move |_, event: &ColorStopEvent, _| sink.borrow_mut().push(*event)).detach();
            (stop, events)
        })
    }

    #[test]
    fn picker_edits_emit_and_host_color_syncs_picker() {
        let mut app = TestAppContext::single();
        let red = ColorValue::srgb(1.0, 0.0, 0.0, 1.0);
        let (stop, events) = spawn_with_events(&mut app, ColorStopBuilder::new("t", red));
        let picker = stop.read_with(&app, |stop, _| stop.picker.clone());

        let blue = ColorValue::srgb(0.0, 0.0, 1.0, 1.0);
        stop.update(&mut app, |stop, cx| stop.set_color(blue, cx));
        assert!(events.borrow().is_empty());
        let synced =
            gpui_bridge::to_rgba(picker.read_with(&app, |picker, _| picker.color()), GamutMapping::CssLocalMinde)
                .unwrap();
        assert!(synced.b > 0.99 && synced.r < 0.01, "picker follows host color");

        let green = ColorValue::srgb(0.0, 1.0, 0.0, 1.0);
        picker.update(&mut app, |_, cx| cx.emit(InlineColorPickerEvent::Change(green)));
        assert_eq!(*events.borrow(), vec![ColorStopEvent::ColorChanged(green)]);
        assert_eq!(stop.read_with(&app, |stop, _| stop.color()), green);
    }

    #[test]
    fn focus_inside_the_stop_shows_the_picker() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).expect("initialize SDK"));
        let gray = ColorValue::srgb(0.5, 0.5, 0.5, 1.0);
        let look = crate::theme::default_look();
        let (stop, cx) = app.add_window_view(|_, cx| ColorStop::new(ColorStopBuilder::new("t", gray).look(look), cx));
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        cx.update(|_, cx| {
            cx.subscribe(&stop, move |_, event: &ColorStopEvent, _| sink.borrow_mut().push(*event)).detach();
        });
        // Focus listeners only fire in an active window.
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        assert!(!stop.read_with(cx, |stop, _| stop.expanded()));

        let swatch = cx.debug_bounds("t-swatch").expect("swatch");
        cx.simulate_click(swatch.center(), Default::default());
        cx.run_until_parked();
        assert!(stop.read_with(cx, |stop, _| stop.expanded()));

        cx.update(|window, cx| window.blur(cx));
        // Focus listeners only fire in an active window.
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        assert!(!stop.read_with(cx, |stop, _| stop.expanded()));
        assert_eq!(
            *events.borrow(),
            vec![ColorStopEvent::ExpandedChanged(true), ColorStopEvent::ExpandedChanged(false)]
        );

        // The swatch toggles: open, then closed again, including a host hold.
        events.borrow_mut().clear();
        cx.simulate_click(swatch.center(), Default::default());
        cx.run_until_parked();
        assert!(stop.read_with(cx, |stop, _| stop.expanded()));
        cx.simulate_click(swatch.center(), Default::default());
        cx.run_until_parked();
        assert!(!stop.read_with(cx, |stop, _| stop.expanded()));
        stop.update(cx, |stop, cx| stop.set_open(true, cx));
        cx.simulate_click(swatch.center(), Default::default());
        cx.run_until_parked();
        assert!(!stop.read_with(cx, |stop, _| stop.expanded()), "a swatch press releases a host hold");
        assert_eq!(
            *events.borrow(),
            vec![
                ColorStopEvent::ExpandedChanged(true),
                ColorStopEvent::ExpandedChanged(false),
                ColorStopEvent::ExpandedChanged(false),
            ]
        );
    }

    #[test]
    fn labels_are_hsl_and_keep_out_of_gamut_values_visible() {
        assert_eq!(stop_label(ColorValue::srgb(1.0, 0.0, 0.0, 1.0)), "hsl(0 100% 50%)");
        assert_eq!(stop_label(ColorValue::srgb(0.5, 0.5, 0.5, 0.25)), "hsl(0 0% 50%)");
        // Display P3 red lies outside sRGB: saturation reads above 100% instead of clipping.
        let label = stop_label(ColorValue::display_p3(1.0, 0.0, 0.0, 1.0));
        let saturation: f32 =
            label.split_whitespace().nth(1).and_then(|s| s.trim_end_matches('%').parse().ok()).unwrap();
        assert!(saturation > 100.0, "{label}");
    }
}
