//! Always-visible context menu panels rendered from the shared SDK menu template.
use gpui::{AnyElement, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::controls::context_menu::MenuPath;
use xgpui_luma::controls::floating_menu::render_floating_menu;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::theme::InteractionState;
use xgpui_luma_look_radix::{ContextMenuVariant, Look, Tone, context_menu_theme};
use lucide_svg_static::Icon;

pub fn preview(look: &Look, fg: Hsla) -> AnyElement {
    let items = [
        MenuItem::new("open").label("Open").icon(Icon::FolderOpen),
        MenuItem::new("copy").label("Copy").icon(Icon::Copy),
        MenuItem::new("paste").label("Paste").icon(Icon::Clipboard).enabled(false),
        MenuItem::new("more")
            .label("More")
            .icon(Icon::Ellipsis)
            .submenu([MenuItem::new("download").label("Download").icon(Icon::Download)]),
    ];
    let mut panels = div().w_full().flex().flex_wrap().gap(px(24.0));
    for (name, variant, tone) in [
        ("Solid", ContextMenuVariant::Solid, Tone::Accent),
        ("Soft", ContextMenuVariant::Soft, Tone::Accent),
        ("Solid · Gray", ContextMenuVariant::Solid, Tone::Gray),
        ("Soft · Gray", ContextMenuVariant::Soft, Tone::Gray),
    ] {
        let id = SharedString::from(format!("guide-context-preview-{name}"));
        let palette = context_menu_theme(look, variant, tone).resolve(InteractionState::default()).floating_menu;
        let panel = render_floating_menu(
            &id,
            &items,
            None,
            Some(MenuPath::Root(1)),
            palette,
            items
                .iter()
                .map(|_| Box::new(|_: &bool, _: &mut gpui::Window, _: &mut gpui::App| {}) as _)
                .collect(),
            items
                .iter()
                .map(|_| Box::new(|_: &gpui::ClickEvent, _: &mut gpui::Window, _: &mut gpui::App| {}) as _)
                .collect(),
        );
        let panel = panel.block_mouse_except_scroll();
        panels = panels.child(
            div().flex().flex_col().gap(px(12.0)).child(div().text_sm().text_color(fg).child(name)).child(panel),
        );
    }
    panels.into_any_element()
}
