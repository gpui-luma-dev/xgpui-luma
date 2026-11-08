pub(crate) mod customization;

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::sections::switch::customization::SwitchCustomizationPreview;
use crate::studio::style::shared::choice_matrix::{
    ChoiceTemplateControl, render_choice_control_sizes_body, render_choice_control_template_preview_body,
};
use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) fn render_switch_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    customization_preview: Entity<SwitchCustomizationPreview>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Switch",
        "Style variants by row. Sizes tab: Sm/Md/Lg × radius (No radius → Full), primary style.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_switch_preview_tabbed_content(
            look,
            preview_tabs,
            customization_preview,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_switch_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    customization_preview: Entity<SwitchCustomizationPreview>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "sizes" => render_choice_control_sizes_body(look.clone(), ChoiceTemplateControl::Switch, window, cx),
        "customization" => customization::render_switch_customization_body(customization_preview),
        _ => render_choice_control_template_preview_body(look.clone(), ChoiceTemplateControl::Switch, window, cx),
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}
