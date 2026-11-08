use std::sync::Arc;

use gpui::{
    App, Bounds, ClickEvent, DragMoveEvent, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    ScrollEvent, Window,
};
use xgpui_luma::controls::scrollbar::{
    ScrollbarBoundsHandler, ScrollbarDrag, ScrollbarDragMoveHandler, ScrollbarHoverHandler, ScrollbarMouseDownHandler,
    ScrollbarMouseUpHandler, ScrollbarScrollWheelHandler, ScrollbarTemplateHandlers,
};
use xgpui_luma::controls::slider::{
    SliderBoundsHandler, SliderDrag, SliderHoverHandler, SliderMouseDownHandler, SliderMouseMoveHandler,
    SliderMouseUpHandler, SliderTemplateHandlers, ThumbId,
};
use xgpui_luma::controls::textarea::{
    TextAreaClickHandler, TextAreaDrag, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaMouseDownHandler,
    TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaTemplateHandlers,
};
use xgpui_luma::controls::textfield::{
    TextFieldClickHandler, TextFieldDrag, TextFieldDragMoveHandler, TextFieldHoverHandler, TextFieldKeyDownHandler,
    TextFieldMouseDownHandler, TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplateHandlers,
};

pub(crate) fn input_noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_mouse_move(_: &MouseMoveEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_scrollbar_drag_move(_: &DragMoveEvent<ScrollbarDrag>, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_scroll_wheel(_: &ScrollEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_slider_drag_move(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_thumb_mouse_down(_: &ThumbId, _: &MouseDownEvent, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_textarea_drag_move(_: &DragMoveEvent<TextAreaDrag>, _: &mut Window, _: &mut App) {}

pub(crate) fn input_noop_textfield_drag_move(_: &DragMoveEvent<TextFieldDrag>, _: &mut Window, _: &mut App) {}

pub(crate) fn input_textfield_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(input_noop_hover) as TextFieldHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as TextFieldMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as TextFieldMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as TextFieldMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as TextFieldMouseUpHandler,
        click: Box::new(input_noop_click) as TextFieldClickHandler,
        key_down: Box::new(input_noop_key_down) as TextFieldKeyDownHandler,
        drag_move: Box::new(input_noop_textfield_drag_move) as TextFieldDragMoveHandler,
    }
}

pub(crate) fn input_textarea_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(input_noop_hover) as TextAreaHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as TextAreaMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as TextAreaMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as TextAreaMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as TextAreaMouseUpHandler,
        click: Box::new(input_noop_click) as TextAreaClickHandler,
        key_down: Box::new(input_noop_key_down) as TextAreaKeyDownHandler,
        drag_move: Box::new(input_noop_textarea_drag_move),
    }
}

pub(crate) fn input_scrollbar_handlers() -> ScrollbarTemplateHandlers {
    ScrollbarTemplateHandlers {
        track_bounds: Box::new(input_noop_bounds) as ScrollbarBoundsHandler,
        thumb_bounds: Box::new(input_noop_bounds) as ScrollbarBoundsHandler,
        hover: Box::new(input_noop_hover) as ScrollbarHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as ScrollbarMouseDownHandler,
        mouse_up: Box::new(input_noop_mouse_up) as ScrollbarMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as ScrollbarMouseUpHandler,
        drag_move: Box::new(input_noop_scrollbar_drag_move) as ScrollbarDragMoveHandler,
        scroll_wheel: Box::new(input_noop_scroll_wheel) as ScrollbarScrollWheelHandler,
    }
}

pub(crate) fn input_slider_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(input_noop_bounds) as SliderBoundsHandler,
        hover: Box::new(input_noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as SliderMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as SliderMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Arc::new(input_noop_slider_drag_move),
        thumb_mouse_down: Arc::new(input_noop_thumb_mouse_down),
    }
}
