//! One editable color (optionally positioned): the building block of
//! [`crate::color_stop_list::ColorStopList`], and usable on its own (e.g. a
//! mesh background color).
//!
//! The control owns the color/position state, its child controls (position
//! `TextField`, remove `Button`, inline picker) and input isolation. Layout is
//! delegated to a [`ColorStopTemplate`]; see [`template`].
//!
//! Host setters (`set_*`) never emit; user edits emit [`ColorStopEvent`].

mod picker;
pub(crate) mod template;

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, EventEmitter, IntoElement, MouseButton, Pixels, Render,
    SharedString, Subscription, Window, div, prelude::*,
};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent};
use xgpui_luma::controls::tooltip::Tooltip;
use xgpui_luma::infra::attachments::TooltipEntityExt;
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
    /// Typed position, already clamped to `0.0..=1.0`.
    PositionChanged(f32),
    /// The remove action was pressed; the owner decides whether to remove.
    RemoveRequested,
    ExpandedChanged(bool),
}

pub(crate) struct ColorStopBuilder {
    id: SharedString,
    look: Option<Arc<Look>>,
    color: ColorValue,
    position: Option<f32>,
    removable: bool,
    expand_on_swatch_click: bool,
    template: Option<Arc<dyn ColorStopTemplate>>,
}

impl ColorStopBuilder {
    pub fn new(id: impl Into<SharedString>, color: ColorValue) -> Self {
        Self {
            id: id.into(),
            look: None,
            color,
            position: None,
            removable: false,
            expand_on_swatch_click: true,
            template: None,
        }
    }

    pub fn look(mut self, look: Arc<Look>) -> Self {
        self.look = Some(look);
        self
    }

    /// Give the stop an editable position (`0.0..=1.0`).
    pub fn position(mut self, position: f32) -> Self {
        self.position = Some(position.clamp(0.0, 1.0));
        self
    }

    /// Show a remove action that emits [`ColorStopEvent::RemoveRequested`].
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    /// Toggle the editor when the swatch is clicked. On by default; lists turn it
    /// off and drive expansion from their selection instead.
    pub fn expand_on_swatch_click(mut self, enabled: bool) -> Self {
        self.expand_on_swatch_click = enabled;
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
    expand_on_swatch_click: bool,
    position_field: Option<TextField>,
    remove_button: Option<Entity<Button>>,
    picker: Entity<InlineColorPicker>,
    height: Option<f32>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorStopEvent> for ColorStop {}

impl ColorStop {
    fn new(builder: ColorStopBuilder, cx: &mut Context<Self>) -> Self {
        let look = builder.look.unwrap_or_else(crate::theme::default_look);
        let id = builder.id;
        let mut subscriptions = Vec::new();

        let position_field = builder.position.map(|position| {
            let field = radix::TextField::new(format!("{id}-position"))
                .look(look.as_ref())
                .size(radix::TextFieldSize::One)
                .value(format_percent(position))
                .full_width(true)
                .select_all_on_tab_focus(true)
                .validator(Arc::new(|text: &str| parse_percent(text).is_some()))
                .spawn(cx);
            subscriptions.push(cx.subscribe(&field, |this, field, event: &TextFieldEvent, cx| match event {
                TextFieldEvent::Submit { value } => this.commit_position_text(value, cx),
                TextFieldEvent::FocusChanged { focused: false } => {
                    let value = field.read(cx).value();
                    this.commit_position_text(&value, cx);
                }
                _ => {}
            }));
            field
        });

        let remove_button = builder.removable.then(|| {
            let button = crate::theme::icon_button(format!("{id}-remove"), LucideIcon::X)
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
            expand_on_swatch_click: builder.expand_on_swatch_click,
            position_field,
            remove_button,
            picker,
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
        self.sync_position_field(cx);
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

    pub fn set_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        if self.expanded != expanded {
            self.expanded = expanded;
            cx.notify();
        }
    }

    pub fn set_can_remove(&mut self, can_remove: bool, cx: &mut Context<Self>) {
        if let Some(button) = &self.remove_button {
            button.update(cx, |button, cx| button.set_enabled(can_remove, cx));
        }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    // ---- User edits -------------------------------------------------------

    /// Commit typed text as a percentage; invalid input reverts to the current value.
    fn commit_position_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some(position) = parse_percent(text)
            && self.position.is_some()
            && self.position != Some(position)
        {
            self.position = Some(position);
            cx.emit(ColorStopEvent::PositionChanged(position));
            cx.notify();
        }
        self.sync_position_field(cx);
    }

    fn toggle_expanded_by_user(&mut self, cx: &mut Context<Self>) {
        self.expanded = !self.expanded;
        cx.emit(ColorStopEvent::ExpandedChanged(self.expanded));
        cx.notify();
    }

    fn sync_position_field(&self, cx: &mut Context<Self>) {
        if let (Some(field), Some(position)) = (&self.position_field, self.position) {
            let text = format_percent(position);
            if field.read(cx).value() != text {
                field.update(cx, |field, cx| field.set_value(text, cx));
            }
        }
    }

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

    fn handlers(&self, cx: &mut Context<Self>) -> ColorStopTemplateHandlers {
        let entity = cx.entity().downgrade();
        let swatch_entity = entity.clone();
        let expand_on_click = self.expand_on_swatch_click;
        ColorStopTemplateHandlers {
            swatch_click: Box::new(move |_, _, cx| {
                if expand_on_click {
                    let _ = swatch_entity.update(cx, |this, cx| this.toggle_expanded_by_user(cx));
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
        let model = ColorStopRenderModel {
            id: &self.id,
            look: &self.look,
            color: self.color,
            label: stop_label(self.color).into(),
            alpha: self.color.alpha(),
            position: self.position,
            index: self.index,
            selected: self.selected,
            expanded: self.expanded,
            position_field: self.position_field.clone().map(|field| self.isolated("position-slot", field)),
            remove_button: self.remove_button.clone().map(|button| self.isolated("remove-slot", button)),
            editor: self.expanded.then(|| self.isolated("editor", self.picker.clone())),
        };
        let handlers = self.handlers(cx);
        self.template.render(model, handlers, window, cx)
    }
}

/// Uppercase sRGB hex.
pub(crate) fn stop_label(color: ColorValue) -> String {
    gpui_bridge::to_rgba(color, GamutMapping::CssLocalMinde)
        .map(|rgba| {
            let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("#{:02X}{:02X}{:02X}", channel(rgba.r), channel(rgba.g), channel(rgba.b))
        })
        .unwrap_or_else(|_| "Invalid color".into())
}

fn format_percent(position: f32) -> String {
    format!("{}%", (position * 100.0).round() as i32)
}

/// Accepts `50`, `50%`, or `12.5 %`; returns a normalized `0.0..=1.0` position.
fn parse_percent(text: &str) -> Option<f32> {
    let value: f32 = text.trim().trim_end_matches('%').trim().parse().ok()?;
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value / 100.0)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
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
    fn position_text_commits_clamped_value_and_reverts_invalid_input() {
        let mut app = TestAppContext::single();
        let gray = ColorValue::srgb(0.5, 0.5, 0.5, 1.0);
        let (stop, events) = spawn_with_events(&mut app, ColorStopBuilder::new("t", gray).position(0.0));
        let text = |app: &mut TestAppContext| {
            stop.read_with(app, |stop, cx| stop.position_field.as_ref().unwrap().read(cx).value())
        };
        assert_eq!(text(&mut app), "0%");

        stop.update(&mut app, |stop, cx| stop.commit_position_text("75 %", cx));
        assert_eq!(*events.borrow(), vec![ColorStopEvent::PositionChanged(0.75)]);
        assert_eq!(text(&mut app), "75%");

        events.borrow_mut().clear();
        stop.update(&mut app, |stop, cx| stop.commit_position_text("abc", cx));
        assert!(events.borrow().is_empty());
        assert_eq!(text(&mut app), "75%", "invalid text reverts");

        stop.update(&mut app, |stop, cx| stop.set_position(0.5, cx));
        assert!(events.borrow().is_empty(), "host setters never emit");
        assert_eq!(text(&mut app), "50%");
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
    fn toggling_expansion_emits_expanded_changed() {
        let mut app = TestAppContext::single();
        let gray = ColorValue::srgb(0.5, 0.5, 0.5, 1.0);
        let (stop, events) = spawn_with_events(&mut app, ColorStopBuilder::new("t", gray));
        stop.update(&mut app, |stop, cx| stop.toggle_expanded_by_user(cx));
        assert_eq!(*events.borrow(), vec![ColorStopEvent::ExpandedChanged(true)]);

        assert!(stop.read_with(&app, |stop, _| stop.expanded()));
    }

    #[test]
    fn parse_percent_accepts_common_forms() {
        assert_eq!(parse_percent("50"), Some(0.5));
        assert_eq!(parse_percent(" 12.5 % "), Some(0.125));
        assert_eq!(parse_percent("100%"), Some(1.0));
        assert_eq!(parse_percent("101"), None);
        assert_eq!(parse_percent("-1"), None);
        assert_eq!(parse_percent("NaN"), None);
    }
}
