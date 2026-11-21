//! The work pane's type in one place (issue #291): every font, text style and weight the pane's own
//! chrome uses — the title row, the `.lss` inspector, the key strip, the State matrix labels and the box
//! model diagram. Change them here to change how the whole pane reads; the control and the State matrix
//! cells draw with the SDK theme and are not affected.

use gpui::{FontWeight, Pixels, px};
use xgpui_luma::theme::LumaTextStyle;

use crate::screen::TextRole;
use crate::theme::{ColorVizLookExt, Look, TextSize};

/// The monospace family: property names, values, expressions, selectors, file text, the key line.
pub(super) const MONO: &str = crate::theme::MONO_FAMILY;

/// The pane's text style: the inspector, the key strip and the lower panel.
pub(super) fn body(look: &Look) -> LumaTextStyle {
    look.typography_scale(TextSize::Xs)
}

/// The weight of labels: column, section and key strip headings, and the State matrix's current labels.
pub(super) const LABEL_WEIGHT: FontWeight = FontWeight::SEMIBOLD;

/// The title row's text roles (the lab screens' roles; the look gives their size and weight).
pub(super) const CRUMB: TextRole = TextRole::Crumb;
pub(super) const TITLE: TextRole = TextRole::Title;
/// The `.lss` control name and its file · layer line.
pub(super) const CONTROL: TextRole = TextRole::Value;
pub(super) const LEDE: TextRole = TextRole::Lede;

/// The box model diagram's labels: layer names, edge values and the content size.
pub(super) const BOX_LAYER_LABEL: Pixels = px(10.0);
pub(super) const BOX_LAYER_WEIGHT: FontWeight = FontWeight::MEDIUM;
pub(super) const BOX_EDGE_VALUE: Pixels = px(10.0);
pub(super) const BOX_CONTENT_LABEL: Pixels = px(11.0);
