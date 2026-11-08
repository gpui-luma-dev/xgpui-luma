//! High-contrast button strip: gray tone, normal vs `highContrast`, matching Radix docs.
//!
//! Ghost is omitted — Radix's gray high-contrast strip shows Solid…Outline only.
//! Classic is deferred with the rest of the Style Guide until elevation lands.

use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::hstack;
use xgpui_luma_look_radix::{Paint, ButtonVariant, Look, LookControlExt};

use super::buttons::{RADIX_VARIANTS, VariantDef};

pub fn strip(look: &Arc<Look>, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let variants: Vec<&VariantDef> =
        RADIX_VARIANTS.iter().filter(|def| !matches!(def.variant, ButtonVariant::Ghost)).collect();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(row_label("Normal", muted))
        .child(button_row(look, &variants, Paint::gray(), "normal", window, cx))
        .child(row_label("High contrast", muted))
        .child(button_row(look, &variants, Paint::gray().high_contrast(), "hc", window, cx))
        .into_any_element()
}

fn row_label(label: &'static str, muted: Hsla) -> AnyElement {
    div()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted)
        .child(label)
        .into_any_element()
}

fn button_row(
    look: &Arc<Look>,
    variants: &[&VariantDef],
    paint: Paint,
    row_id: &str,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let mut children = Vec::with_capacity(variants.len());
    for def in variants {
        let template = look.button_template(def.variant, paint);
        children.push(preview_button(&template, &format!("{row_id}-{}", def.id), window, cx));
    }

    hstack! {
        gap=12 align=center;
    }
    .children(children)
    .into_any_element()
}

fn preview_button(template: &Arc<dyn ButtonTemplate<()>>, id: &str, window: &mut Window, cx: &mut App) -> AnyElement {
    let model = ButtonRenderModel {
        id: SharedString::from(format!("style-guide-hc-{id}")),
        content: Arc::new(|_, _| div().child("Edit profile").into_any_element()),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        ..Default::default()
    };
    template.render(&model, window, cx).into_any_element()
}
