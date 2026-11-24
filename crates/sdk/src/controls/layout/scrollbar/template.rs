use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Stateful,
    ScrollEvent, Window, canvas, div, px, prelude::*,
};

use super::{ScrollbarDrag, ScrollbarOrientation, ScrollbarRenderModel};
use crate::controls::scrollbar::{ScrollbarTheme, default_scrollbar_theme};

pub type ScrollbarBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type ScrollbarHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ScrollbarMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ScrollbarMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type ScrollbarDragMoveHandler = Box<dyn Fn(&DragMoveEvent<ScrollbarDrag>, &mut Window, &mut App) + 'static>;
pub type ScrollbarScrollWheelHandler = Box<dyn Fn(&ScrollEvent, &mut Window, &mut App) + 'static>;

pub struct ScrollbarTemplateHandlers {
    pub track_bounds: ScrollbarBoundsHandler,
    pub thumb_bounds: ScrollbarBoundsHandler,
    pub hover: ScrollbarHoverHandler,
    pub mouse_down: ScrollbarMouseDownHandler,
    pub mouse_up: ScrollbarMouseUpHandler,
    pub mouse_up_out: ScrollbarMouseUpHandler,
    pub drag_move: ScrollbarDragMoveHandler,
    pub scroll_wheel: ScrollbarScrollWheelHandler,
}

pub type ScrollbarTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ScrollbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ScrollbarTemplate: Send + Sync {
    fn render(
        &self,
        model: &ScrollbarRenderModel<'_>,
        handlers: ScrollbarTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    /// The shortest thumb this template draws, used by the control before the thumb has been measured.
    /// Defaults to the SDK's `scrollbar.lss` value; themed templates return their look's.
    fn min_thumb_length(&self, orientation: ScrollbarOrientation, style: super::ScrollbarStyle) -> f32 {
        let _ = (orientation, style);
        crate::style::sdk_number("scrollbar", "min_thumb_length")
    }
}

pub struct ThemedScrollbarTemplate {
    theme: Arc<dyn ScrollbarTheme>,
    modifiers: Vec<ScrollbarTemplateModifier>,
}

impl ThemedScrollbarTemplate {
    pub fn new(theme: Arc<dyn ScrollbarTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ScrollbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ScrollbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedScrollbarTemplate {
    base: Arc<dyn ScrollbarTemplate>,
    modifiers: Vec<ScrollbarTemplateModifier>,
}

impl ModifiedScrollbarTemplate {
    fn new(base: Arc<dyn ScrollbarTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ScrollbarTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ScrollbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_scrollbar_template() -> Arc<dyn ScrollbarTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ScrollbarTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedScrollbarTemplate::new(default_scrollbar_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn ScrollbarTemplate>, modifier: F) -> Arc<dyn ScrollbarTemplate>
where
    F: Fn(Stateful<Div>, &ScrollbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedScrollbarTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl ScrollbarTemplate for ModifiedScrollbarTemplate {
    fn render(
        &self,
        model: &ScrollbarRenderModel<'_>,
        handlers: ScrollbarTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }

    fn min_thumb_length(&self, orientation: ScrollbarOrientation, style: super::ScrollbarStyle) -> f32 {
        self.base.min_thumb_length(orientation, style)
    }
}

impl ScrollbarTemplate for ThemedScrollbarTemplate {
    fn min_thumb_length(&self, orientation: ScrollbarOrientation, style: super::ScrollbarStyle) -> f32 {
        self.theme.resolve(Default::default(), orientation, style).min_thumb_length
    }

    fn render(
        &self,
        model: &ScrollbarRenderModel<'_>,
        handlers: ScrollbarTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ScrollbarTemplateHandlers {
            track_bounds,
            thumb_bounds,
            hover,
            mouse_down,
            mouse_up,
            mouse_up_out,
            drag_move,
            scroll_wheel,
        } = handlers;
        let look = self.theme.resolve(model.state, model.orientation, model.style);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let thumb_fraction = model.thumb_fraction.clamp(0.05, 1.0);
        let thickness = if look.thickness.is_finite() && look.thickness >= 0.0 {
            look.thickness
        } else {
            crate::style::sdk_number("scrollbar", "thickness")
        };
        let track_thickness = clamped_thickness(look.track_thickness, 0.0, thickness);
        let thumb_thickness = clamped_thickness(look.thumb_thickness, 0.0, thickness);
        let track_cross_offset = (thickness - track_thickness) * 0.5;
        let thumb_cross_offset = (thickness - thumb_thickness) * 0.5;

        let (
            width,
            height,
            track_width,
            track_height,
            track_left,
            track_top,
            thumb_width,
            thumb_height,
            thumb_left,
            thumb_top,
        ) = match model.orientation {
            ScrollbarOrientation::Horizontal => {
                let length = model.length.unwrap_or(look.length);
                let thumb_length = (length * thumb_fraction).clamp(look.min_thumb_length.min(length), length);
                let thumb_left = ((length - thumb_length).max(0.0) * percentage).clamp(0.0, length);

                (
                    length,
                    thickness,
                    length,
                    track_thickness,
                    0.0,
                    track_cross_offset,
                    thumb_length,
                    thumb_thickness,
                    thumb_left,
                    thumb_cross_offset,
                )
            }
            ScrollbarOrientation::Vertical => {
                let length = model.length.unwrap_or(look.length);
                let thumb_length = (length * thumb_fraction).clamp(look.min_thumb_length.min(length), length);
                let thumb_top = ((length - thumb_length).max(0.0) * percentage).clamp(0.0, length);

                (
                    thickness,
                    length,
                    track_thickness,
                    length,
                    track_cross_offset,
                    0.0,
                    thumb_thickness,
                    thumb_length,
                    thumb_cross_offset,
                    thumb_top,
                )
            }
        };

        let track = div()
            .id("track")
            .absolute()
            .left(px(track_left))
            .top(px(track_top))
            .w(px(track_width))
            .h(px(track_height))
            .bg(look.track_background)
            .rounded(px(look.radius));

        let thumb = div()
            .id("thumb")
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .w(px(thumb_width))
            .h(px(thumb_height))
            .bg(look.thumb_background)
            .rounded(px(look.radius))
            .child(canvas(move |bounds, window, cx| thumb_bounds(&bounds, window, cx), |_, _, _, _| {}).size_full());

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w(px(width))
            .h(px(height))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_scroll(scroll_wheel)
            .on_drag(ScrollbarDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move)
            .child(track)
            .child(thumb)
            .child(
                canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            );

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(look.disabled_opacity);
        }

        self.apply_modifiers(root, model)
    }
}

fn clamped_thickness(value: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() { value.clamp(min, max) } else { min }
}
