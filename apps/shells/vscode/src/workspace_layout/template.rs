use gpui::{Div, IntoElement, Pixels, Stateful, div, prelude::*, px};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole};

use crate::layout_config::PanelAlignment;

pub const ACTIVITY_BAR_W: f32 = 48.0;

#[derive(Clone, Copy)]
pub enum ActivityRailEdge {
    Left,
    Right,
}

pub struct ActivityRailSpec<'a> {
    pub id: &'static str,
    pub visible: bool,
    pub buttons: &'a [IconButton],
}

#[derive(Clone, Copy)]
pub struct ExternalPanelGeometry {
    pub left_offset: Pixels,
    pub left_span_width: Pixels,
    pub right_offset: Pixels,
    pub right_span_width: Pixels,
    pub justify_width: Pixels,
}

pub fn activity_bar_rail(
    id: &'static str,
    look: &ShadcnLook,
    buttons: &[IconButton],
    edge: ActivityRailEdge,
) -> impl IntoElement {
    let chrome = look.chrome();
    let mut rail = div()
        .id(id)
        .w(px(ACTIVITY_BAR_W))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .justify_between()
        .py(px(8.0))
        .bg(chrome.panel_background)
        .border_color(chrome.border);

    rail = match edge {
        ActivityRailEdge::Left => rail.border_r_1(),
        ActivityRailEdge::Right => rail.border_l_1(),
    };

    let mut primary = div().flex().flex_col().items_center().gap(px(4.0));
    for button in buttons.iter().take(5) {
        primary = primary.child(button.clone());
    }

    rail.child(primary)
}

pub fn floating_output_panel(
    look: &ShadcnLook,
    alignment: PanelAlignment,
    geometry: ExternalPanelGeometry,
    panel_height_px: f32,
    splitter: impl IntoElement,
) -> impl IntoElement {
    let panel = panel_surface(look).flex_1().min_h_0();
    let shell = div()
        .absolute()
        .bottom(px(0.0))
        .h(px(panel_height_px))
        .flex()
        .flex_col()
        .child(splitter)
        .child(panel);

    match alignment {
        PanelAlignment::Left => shell.left(geometry.left_offset).w(geometry.left_span_width),
        PanelAlignment::Right => shell.left(geometry.right_offset).w(geometry.right_span_width).border_l_1(),
        PanelAlignment::Justify => shell.left(geometry.left_offset).w(geometry.justify_width),
        PanelAlignment::Center => shell.left(px(0.0)).w_full(),
    }
}

fn panel_surface(look: &ShadcnLook) -> Stateful<Div> {
    let chrome = look.chrome();
    let label_style = look.typography_role(ShadcnTextRole::P);

    div()
        .id("panel")
        .bg(chrome.panel_background)
        .border_t_1()
        .border_color(chrome.border)
        .p(px(12.0))
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Panel"))
}
