//! No-op template handlers for static style-guide previews.

use std::sync::Arc;

use gpui::{
    App, Bounds, ClickEvent, DragMoveEvent, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Window,
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

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_move(_: &MouseMoveEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_slider_drag(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}
fn noop_thumb_mouse_down(_: &ThumbId, _: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_textarea_drag(_: &DragMoveEvent<TextAreaDrag>, _: &mut Window, _: &mut App) {}
fn noop_textfield_drag(_: &DragMoveEvent<TextFieldDrag>, _: &mut Window, _: &mut App) {}

pub fn textfield_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(noop_hover) as TextFieldHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextFieldMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextFieldMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        click: Box::new(noop_click) as TextFieldClickHandler,
        key_down: Box::new(noop_key_down) as TextFieldKeyDownHandler,
        drag_move: Box::new(noop_textfield_drag) as TextFieldDragMoveHandler,
    }
}

pub fn textarea_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(noop_hover) as TextAreaHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextAreaMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextAreaMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        click: Box::new(noop_click) as TextAreaClickHandler,
        key_down: Box::new(noop_key_down) as TextAreaKeyDownHandler,
        drag_move: Box::new(noop_textarea_drag),
    }
}

pub fn slider_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as SliderBoundsHandler,
        hover: Box::new(noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as SliderMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as SliderMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Arc::new(noop_slider_drag),
        thumb_mouse_down: Arc::new(noop_thumb_mouse_down),
    }
}

pub fn selection_panel_handlers(count: usize) -> xgpui_luma::controls::selection_panel::SelectionPanelTemplateHandlers {
    xgpui_luma::controls::selection_panel::SelectionPanelTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as _).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as _).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as _).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as _).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as _).collect(),
    }
}

pub fn scrollbar_handlers() -> xgpui_luma::controls::scrollbar::ScrollbarTemplateHandlers {
    xgpui_luma::controls::scrollbar::ScrollbarTemplateHandlers {
        track_bounds: Box::new(noop_bounds),
        thumb_bounds: Box::new(noop_bounds),
        hover: Box::new(noop_hover),
        mouse_down: Box::new(noop_mouse_down),
        mouse_up: Box::new(noop_mouse_up),
        mouse_up_out: Box::new(noop_mouse_up),
        drag_move: Box::new(|_, _, _| {}),
        scroll_wheel: Box::new(|_, _, _| {}),
    }
}

pub fn control_group_handlers(count: usize) -> xgpui_luma::controls::control_group::ControlGroupTemplateHandlers {
    xgpui_luma::controls::control_group::ControlGroupTemplateHandlers {
        item_bounds: (0..count).map(|_| Box::new(noop_bounds) as _).collect(),
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as _).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as _).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as _).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as _).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as _).collect(),
    }
}
pub fn accordion_handlers(count: usize) -> xgpui_luma::controls::accordion::AccordionTemplateHandlers {
    xgpui_luma::controls::accordion::AccordionTemplateHandlers {
        drag: None,
        trigger_hovers: (0..count).map(|_| Box::new(noop_hover) as _).collect(),
        trigger_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as _).collect(),
        trigger_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as _).collect(),
        trigger_clicks: (0..count).map(|_| Box::new(noop_click) as _).collect(),
        content_height_reports: (0..count).map(|_| Box::new(|_: f32, _: &mut Window, _: &mut App| {}) as _).collect(),
    }
}
pub fn pager_handlers() -> xgpui_luma::controls::pager::PagerTemplateHandlers {
    xgpui_luma::controls::pager::PagerTemplateHandlers {
        outside_mouse_down: Arc::new(noop_mouse_down),
        toggle_page_size: Arc::new(noop_click),
        set_page: Arc::new(|_, _, _, _| {}),
        set_page_size: Arc::new(|_, _, _, _| {}),
    }
}

pub fn dock_splitter_handlers() -> xgpui_luma::controls::dock_splitter::DockSplitterTemplateHandlers {
    xgpui_luma::controls::dock_splitter::DockSplitterTemplateHandlers {
        hover: Box::new(noop_hover),
        mouse_down: Box::new(noop_mouse_down),
        mouse_up: Box::new(noop_mouse_up),
        mouse_up_out: Box::new(noop_mouse_up),
        key_down: Box::new(noop_key_down),
    }
}
pub fn split_view_handlers() -> xgpui_luma::controls::split_view::SplitViewTemplateHandlers {
    xgpui_luma::controls::split_view::SplitViewTemplateHandlers {
        separator_hover: Box::new(noop_hover),
        separator_mouse_down: Box::new(noop_mouse_down),
        separator_click: Box::new(noop_click),
        mouse_up: Box::new(noop_mouse_up),
        mouse_up_out: Box::new(noop_mouse_up),
        drag_move: Box::new(|_, _, _| {}),
    }
}

pub fn slide_panel_handlers() -> xgpui_luma::controls::slide_panel::SlidePanelOverlayHandlers {
    xgpui_luma::controls::slide_panel::SlidePanelOverlayHandlers {
        key_down: Box::new(noop_key_down),
        backdrop_mouse_down: Box::new(noop_mouse_down),
    }
}
pub fn slide_panel_resize_handlers() -> xgpui_luma::controls::slide_panel::SlidePanelResizeHandlers {
    xgpui_luma::controls::slide_panel::SlidePanelResizeHandlers {
        mouse_down: Box::new(noop_mouse_down),
        drag_move: Box::new(|_, _, _| {}),
        mouse_up: Box::new(noop_mouse_up),
        mouse_up_out: Box::new(noop_mouse_up),
        hover: Box::new(noop_hover),
    }
}
