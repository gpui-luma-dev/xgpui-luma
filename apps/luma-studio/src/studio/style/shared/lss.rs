//! Reading the Shadcn `.lss` styles from Studio previews (issue #285).

use std::sync::Arc;

use xgpui_luma::style::lss::{AttrValue, ControlId, Resolved, StyleKey};
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnSize};

/// The `.lss` variant for a button style; primary is the base.
fn variant(style: ShadcnButtonStyle) -> Option<&'static str> {
    match style {
        ShadcnButtonStyle::Primary => None,
        ShadcnButtonStyle::Secondary => Some("secondary"),
        ShadcnButtonStyle::Outline => Some("outline"),
        ShadcnButtonStyle::Ghost => Some("ghost"),
        ShadcnButtonStyle::ContentOnly => Some("content_only"),
    }
}

/// A Shadcn control's resolved `.lss` style for a style, the look's size, a state and an "on" state.
pub(crate) fn shadcn_style(
    look: &ShadcnLook,
    control: &'static str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    on: Option<(&str, bool)>,
) -> Arc<Resolved> {
    let theme = look.style_theme();
    let config = theme.stylesheet().config().clone();
    let mut key = StyleKey::new(ControlId::new(control));
    if let Some(attr) = config.attribute("variant") {
        key = key.attr_opt(attr, variant(style).map(AttrValue::new));
    }
    if let Some(attr) = config.attribute("size")
        && look.size() != ShadcnSize::Md
    {
        key = key.attr(attr, AttrValue::named(look.size().key()));
    }
    for (name, active) in [
        ("hover", state.hovered),
        ("pressed", state.pressed),
        ("focused", state.focused),
        ("disabled", state.disabled),
        ("invalid", state.invalid),
    ]
    .into_iter()
    .chain(on)
    {
        if let Some(state) = config.state(name) {
            key = key.state_if(state, active);
        }
    }
    theme.resolve(&key)
}

/// A Shadcn control's resolved `.lss` style at `size` with a corner radius preset (the `radius` attribute).
pub(crate) fn shadcn_radius_style(
    look: &ShadcnLook,
    control: &'static str,
    size: ShadcnSize,
    preset: xgpui_luma_look_shadcn::ButtonRadiusPreset,
) -> Arc<Resolved> {
    let theme = look.style_theme();
    let config = theme.stylesheet().config().clone();
    let mut key = StyleKey::new(ControlId::new(control));
    if let Some(attr) = config.attribute("radius") {
        key = key.attr(attr, AttrValue::named(preset.key()));
    }
    if let Some(attr) = config.attribute("size")
        && size != ShadcnSize::Md
    {
        key = key.attr(attr, AttrValue::named(size.key()));
    }
    theme.resolve(&key)
}

/// A length a resolved style sets on `part` (`None` for the root).
pub(crate) fn length(style: &Resolved, part: Option<&'static str>, property: &str) -> f32 {
    let part = match part {
        Some(name) => style.part(xgpui_luma::style::lss::PartId::new(name)),
        None => style.root(),
    };
    part.value(property).and_then(xgpui_luma::style::lss::Value::as_number).unwrap_or_default()
}
