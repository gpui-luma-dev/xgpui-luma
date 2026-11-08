use gpui::{AnyElement, Entity, IntoElement, div, prelude::*, px};
use xgpui_luma::controls::sidebar::SidebarControl;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) fn render_sidebar_template_section(
    sidebar: Entity<SidebarControl>,
    look: &ShadcnLook,
    cx: &gpui::App,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Sidebar",
        "Navigation sidebar template preview.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .w_full()
            .flex()
            .justify_start()
            .child(div().w(px(300.0)).h(px(520.0)).overflow_hidden().child(
                xgpui_luma_look_shadcn::Frame::sidebar("sidebar-preview-frame").look(look).child(sidebar).render(cx),
            ))
            .into_any_element(),
    )
}
