//! Resolved paint/layout overflow for inspector box-model occupation envelopes.

use gpui::BoxShadow;
use gpui::SharedString;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::theme::shadow::shadow_projection_insets;
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, paint};

use super::box_model::{InspectEdgeInsets, InspectOccupationSnapshot};

const SCALE_FACTOR: f32 = 1.0;

pub struct InspectMetricPropertyData {
    pub name: SharedString,
    pub value: SharedString,
    pub source: SharedString,
    pub provenance: Option<SharedString>,
}

pub fn radio_occupation(look: &ShadcnLook, focus_oversize: f32) -> InspectOccupationSnapshot {
    let state = InteractionState::default();
    let tokens = look.mode_tokens();
    let palette = paint::radio_button_look(tokens.as_ref(), ShadcnButtonStyle::Primary, false, state, ControlSize::Md);
    choice_indicator_occupation(palette.indicator_shadow.as_deref(), focus_oversize, state.disabled)
}

pub fn checkbox_occupation(look: &ShadcnLook, focus_oversize: f32) -> InspectOccupationSnapshot {
    let state = InteractionState::default();
    let tokens = look.mode_tokens();
    let palette = paint::checkbox_look(tokens.as_ref(), ShadcnButtonStyle::Primary, false, state, ControlSize::Md);
    choice_indicator_occupation(palette.indicator_shadow.as_deref(), focus_oversize, state.disabled)
}

pub fn switch_occupation(look: &ShadcnLook, focus_oversize: f32) -> InspectOccupationSnapshot {
    let state = InteractionState::default();
    let tokens = look.mode_tokens();
    let palette =
        paint::switch_look(tokens.as_ref(), look.mode(), ShadcnButtonStyle::Primary, true, state, ControlSize::Md);
    let shadows = if palette.thumb_shadow.is_empty() {
        None
    } else {
        Some(palette.thumb_shadow.as_slice())
    };
    choice_indicator_occupation(shadows, focus_oversize, state.disabled)
}

fn choice_indicator_occupation(
    shadows: Option<&[BoxShadow]>,
    focus_oversize: f32,
    disabled: bool,
) -> InspectOccupationSnapshot {
    if disabled {
        return InspectOccupationSnapshot::default();
    }

    let shadow_insets = shadow_insets_from(shadows);
    let shadow_extent = shadow_insets.max_edge();
    let layout_extent = focus_oversize.max(shadow_extent);
    let layout = InspectEdgeInsets::symmetric(layout_extent);
    let paint = shadow_insets.union(InspectEdgeInsets::symmetric(focus_oversize));
    InspectOccupationSnapshot { paint, layout }
}

pub fn button_family_occupation(
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
    focus_oversize: f32,
) -> InspectOccupationSnapshot {
    if state.disabled {
        return InspectOccupationSnapshot::default();
    }

    let tokens = look.mode_tokens();
    let button_look = paint::button_look(tokens.as_ref(), look.mode(), style, role, size, state);
    let shadows = button_look.shadow.as_deref().filter(|shadows| !shadows.is_empty());
    button_family_occupation_from_shadows(shadows, focus_oversize)
}

fn button_family_occupation_from_shadows(
    shadows: Option<&[BoxShadow]>,
    focus_oversize: f32,
) -> InspectOccupationSnapshot {
    let shadow_insets = shadow_insets_from(shadows);
    let layout = InspectEdgeInsets::symmetric(focus_oversize);
    let paint = shadow_insets.union(InspectEdgeInsets::symmetric(focus_oversize));
    InspectOccupationSnapshot { paint, layout }
}

fn shadow_insets_from(shadows: Option<&[BoxShadow]>) -> InspectEdgeInsets {
    shadows
        .map(|shadows| {
            let insets = shadow_projection_insets(shadows, SCALE_FACTOR);
            InspectEdgeInsets { top: insets.top, right: insets.right, bottom: insets.bottom, left: insets.left }
        })
        .unwrap_or_default()
}

pub fn occupation_metric_properties(
    chrome_height: f32,
    occupation: &InspectOccupationSnapshot,
) -> Vec<InspectMetricPropertyData> {
    let mut rows = Vec::new();
    if occupation.has_overflow() {
        push_inset_row(&mut rows, "paint extent top", occupation.paint.top);
        push_inset_row(&mut rows, "paint extent right", occupation.paint.right);
        push_inset_row(&mut rows, "paint extent bottom", occupation.paint.bottom);
        push_inset_row(&mut rows, "paint extent left", occupation.paint.left);
        push_inset_row(&mut rows, "layout extent top", occupation.layout.top);
        push_inset_row(&mut rows, "layout extent right", occupation.layout.right);
        push_inset_row(&mut rows, "layout extent bottom", occupation.layout.bottom);
        push_inset_row(&mut rows, "layout extent left", occupation.layout.left);
    }

    let layout_height = chrome_height + occupation.layout.top + occupation.layout.bottom;
    let paint_height = chrome_height + occupation.paint.top + occupation.paint.bottom;
    if layout_height != chrome_height {
        rows.push(metric_row("occupation height (layout)", layout_height, "chrome height + layout extents"));
    }
    if paint_height != chrome_height && paint_height != layout_height {
        rows.push(metric_row("occupation height (paint)", paint_height, "chrome height + paint extents"));
    }

    rows
}

fn push_inset_row(rows: &mut Vec<InspectMetricPropertyData>, name: &'static str, value: f32) {
    if value > 0.0 {
        rows.push(metric_row(name, value, "shadow projection · focus oversize"));
    }
}

fn metric_row(name: &'static str, value: f32, source: &'static str) -> InspectMetricPropertyData {
    let formatted = if (value - value.round()).abs() < f32::EPSILON {
        format!("{}px", value.round() as i32)
    } else {
        format!("{value:.1}px")
    };
    InspectMetricPropertyData {
        name: SharedString::from(name),
        value: formatted.into(),
        source: SharedString::from(source),
        provenance: None,
    }
}
