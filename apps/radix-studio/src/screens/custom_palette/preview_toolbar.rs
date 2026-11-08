//! SDK toolbar and Actions menu above the signup preview.

use gpui::{Context, Entity};
use xgpui_luma::controls::button::ControlIcon;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::popup_menu::PopupMenu;
use xgpui_luma::controls::toolbar::{Toolbar, ToolbarItem};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma_look_radix::{self as radix, ButtonSize, Look};

use xgpui_luma::prelude::TooltipEntityExt;

pub fn spawn<T: 'static>(look: &Look, cx: &mut Context<T>) -> (Toolbar, Entity<PopupMenu>) {
    let mut toolbar = radix::Toolbar::new("signup-preview-toolbar").look(look);
    for (index, (icon, label)) in [
        ("plus", "Add"),
        ("grid", "Grid"),
        ("square", "Box"),
        ("component-1", "Component"),
        ("dots-horizontal", "More"),
        ("text", "Text"),
        ("font-italic", "Typography"),
        ("lightning-bolt", "Interactions"),
        ("scissors", "Slice"),
        ("cube", "3D"),
    ]
    .into_iter()
    .enumerate()
    {
        if matches!(index, 1 | 5 | 8) {
            toolbar = toolbar.separator(format!("preview-tools-separator-{index}"));
        }
        let id = format!("preview-tool-{icon}");
        let button = radix::Button::new(id.clone())
            .look(look)
            .ghost_quiet()
            .gray()
            .size(ButtonSize::One)
            .role(ButtonFamilyRole::Icon)
            .tab_stop(false)
            .icon(ControlIcon::SvgPath(format!("assets/react-icons/{icon}.svg").into()))
            .icon_size(16.0)
            .spawn(cx)
            .help(label, cx);
        toolbar = toolbar.item(ToolbarItem::command_button(id, button, cx).label(label));
    }
    let toolbar = toolbar.spawn(cx);
    let actions = radix::PopupMenu::new("signup-preview-actions")
        .look(look)
        .outline()
        .label("Actions")
        .items([
            MenuItem::new("copy").label("Copy"),
            MenuItem::new("paste").label("Paste"),
            MenuItem::new("paste-replace").label("Paste to replace"),
            MenuItem::new("layers").label("Layers").submenu([
                MenuItem::new("bring-front").label("Bring to front"),
                MenuItem::new("send-back").label("Send to back"),
            ]),
            MenuItem::new("boolean-groups").label("Boolean groups").submenu([
                MenuItem::new("union").label("Union"),
                MenuItem::new("subtract").label("Subtract"),
                MenuItem::new("intersect").label("Intersect"),
                MenuItem::new("exclude").label("Exclude"),
            ]),
        ])
        .spawn(cx);
    (toolbar, actions)
}
