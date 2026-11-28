//! The values a textarea draws, read from its resolved style (#433).

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, SharedString};
use xgpui_luma_style::Resolved;

use super::{TextAreaState, TextAreaTheme};
use crate::controls::text::{input_typography, resolve_text_area};
use crate::style::lss::Overrides;
use crate::style::PartRead;
use crate::theme::LumaTextStyle;

/// What a textarea's template and control read from its resolved root.
#[derive(Clone)]
pub(crate) struct AreaValues {
    pub helper_geometry: crate::theme::stylesheet::SharedGeometry,
    pub caret_width: f32,
    pub caret_height_extra: f32,
    pub opacity: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub border_width: f32,
}

impl AreaValues {
    pub(crate) fn read(resolved: &Resolved) -> Self {
        let root = resolved.root();
        Self {
            helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
            caret_width: root.number("caret_width"),
            caret_height_extra: root.number("caret_height_extra"),
            opacity: root.opacity(),
            background: root.color("background"),
            foreground: root.color("foreground"),
            border: root.color("border"),
            focus_border: root.color("focus_border"),
            placeholder: root.color("placeholder"),
            selection_background: root.color("selection_background"),
            selection_foreground: root.color("selection_foreground"),
            caret: root.color("caret"),
            shadow: root.box_shadow_opt(),
            typography: input_typography(root),
            font_family: root.font_family_name(),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            radius: root.radius().unwrap_or(0.0),
            border_width: root.number("border_width"),
        }
    }
}

/// The textarea's resolved style in `state`, with a look closure applied as overrides (#433).
pub(crate) fn resolve_area(
    theme: &Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    overrides: &Overrides,
) -> Arc<Resolved> {
    resolve_text_area(&theme.style().with_overrides(overrides), state, enabled)
}
