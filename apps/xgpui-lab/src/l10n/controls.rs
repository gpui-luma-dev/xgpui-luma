//! Resolve copy in render snapshots, preserving retained IDs, handlers, focus and popup state.
use std::sync::Arc;
use gpui::{App, Div, Entity, Stateful, Window};
use xgpui_luma::controls::{selector::*, popup_menu::*, context_menu::*};
use super::resolve;
fn menu_items(items: &[MenuItem], cx: &App) -> Vec<MenuItem> {
    items
        .iter()
        .map(|item| item.clone().label(resolve(item.label_text(), cx)).submenu(menu_items(item.submenu_items(), cx)))
        .collect()
}
type SelectorLabel = Arc<dyn Fn(&SelectorItem, &App) -> gpui::SharedString + Send + Sync>;
struct SelectorCopy(Arc<dyn SelectorTemplate<SelectorItem>>, SelectorLabel);
impl SelectorTemplate<SelectorItem> for SelectorCopy {
    fn resolve_look(
        &self,
        model: &SelectorRenderModel<'_, SelectorItem>,
        window: &Window,
        cx: &mut App,
    ) -> xgpui_luma::controls::selector::SelectorLook {
        self.0.resolve_look(model, window, cx)
    }

    fn render(
        &self,
        model: &SelectorRenderModel<'_, SelectorItem>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let items = model.items.iter().map(|item| item.clone().label((self.1)(item, cx))).collect::<Vec<_>>();
        let label = model
            .selected_index
            .filter(|index| model.items.get(*index).is_some_and(|item| item.label_text() == model.label))
            .and_then(|index| items.get(index))
            .map(|item| item.label_text().clone())
            .unwrap_or_else(|| resolve(model.label, cx));
        self.0.render(
            &SelectorRenderModel {
                id: model.id,
                label: &label,
                selected_index: model.selected_index,
                items: &items,
                open: model.open,
                presence: model.presence.clone(),
                popup_scroll: model.popup_scroll,
                trigger_bounds: model.trigger_bounds,
                placement: model.placement,
                opening_mode: model.opening_mode,
                popup_geometry: model.popup_geometry,
                initialize_popup_scroll: model.initialize_popup_scroll,
                active_path: model.active_path,
                enabled: model.enabled,
                size: model.size,
                trigger_style: model.trigger_style,
                icons: model.icons,
                without_elevation: model.without_elevation,
                item_template: model.item_template,
                panel_template: model.panel_template,
                focus: model.focus,
                state: model.state,
                visual_state: model.visual_state,
            },
            handlers,
            window,
            cx,
        )
    }
}
pub(crate) fn localize_selector(view: Entity<Selector>, cx: &mut App) -> Entity<Selector> {
    localize_selector_with(view, Arc::new(|item, cx| resolve(item.label_text(), cx)), cx)
}
pub(crate) fn localize_selector_with(view: Entity<Selector>, labels: SelectorLabel, cx: &mut App) -> Entity<Selector> {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(SelectorCopy(base, labels)), cx));
    view
}
struct PopupCopy(Arc<dyn PopupMenuTemplate>);
impl PopupMenuTemplate for PopupCopy {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let label = resolve(model.label, cx);
        let items = menu_items(model.items, cx);
        self.0.render(
            &PopupMenuRenderModel {
                id: model.id,
                label: &label,
                content: model.content.clone(),
                items: &items,
                open: model.open,
                disclosure_progress: model.disclosure_progress,
                presence: model.presence.clone(),
                submenu_transition: model.submenu_transition,
                trigger_bounds: model.trigger_bounds,
                placement: model.placement,
                trigger_style: model.trigger_style,
                trigger_size: model.trigger_size,
                menu_size: model.menu_size,
                icon_only: model.icon_only,
                icon: model.icon,
                end_icon: model.end_icon.clone(),
                disclosure_icons: model.disclosure_icons,
                full_width: model.full_width,
                without_elevation: model.without_elevation,
                split: model.split,
                split_state: model.split_state,
                trigger_radius_override: model.trigger_radius_override,
                open_submenu: model.open_submenu,
                active_path: model.active_path,
                highlight: model.highlight,
                enabled: model.enabled,
                focus: model.focus,
                state: model.state,
            },
            handlers,
            window,
            cx,
        )
    }
}
pub(crate) fn localize_popup(view: Entity<PopupMenu>, cx: &mut App) -> Entity<PopupMenu> {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(PopupCopy(base)), cx));
    view
}
struct ContextCopy(Arc<dyn ContextMenuTemplate>);
impl ContextMenuTemplate for ContextCopy {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let label = resolve(model.label, cx);
        let items = menu_items(model.items, cx);
        self.0.render(
            &ContextMenuRenderModel {
                id: model.id,
                label: &label,
                target_content: model.target_content,
                items: &items,
                menu_position: model.menu_position,
                presence: model.presence.clone(),
                open_submenu: model.open_submenu,
                active_path: model.active_path,
                submenu_presence: model.submenu_presence.clone(),
                submenu_transition: model.submenu_transition,
                enabled: model.enabled,
                focus: model.focus,
                state: model.state,
            },
            handlers,
            window,
            cx,
        )
    }
}
pub(crate) fn localize_context(view: Entity<ContextMenu>, cx: &mut App) -> Entity<ContextMenu> {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(ContextCopy(base)), cx));
    view
}

use xgpui_luma::controls::button::{Button, ButtonRenderModel, ButtonTemplate};
struct NamedButton(Arc<dyn ButtonTemplate<()>>, gpui::SharedString);
impl ButtonTemplate<()> for NamedButton {
    fn render(&self, model: &ButtonRenderModel<()>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        use gpui::prelude::*;
        self.0.render(model, window, cx).aria_label(resolve(&self.1, cx))
    }
}
pub(crate) fn name_button(view: Entity<Button>, source: &str, cx: &mut App) -> Entity<Button> {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(NamedButton(base, source.to_owned().into())), cx));
    view
}

use xgpui_luma::controls::textfield::{
    TextField, TextFieldLook, TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTemplateHandlers,
};
struct FieldCopy(Arc<dyn TextFieldTemplate>);
impl TextFieldTemplate for FieldCopy {
    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: xgpui_luma::theme::ControlSize,
    ) -> TextFieldLook {
        self.0.resolve_look(state, enabled, size)
    }
    fn resolve_look_with_scale(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: xgpui_luma::theme::ControlSize,
        scale: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        self.0.resolve_look_with_scale(state, enabled, size, scale, cx)
    }
    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let placeholder = resolve(model.placeholder, cx);
        self.0.render(
            &TextFieldRenderModel {
                id: model.id,
                placeholder: &placeholder,
                content_insets: model.content_insets,
                value: model.value,
                prefix_icon: model.prefix_icon,
                size: model.size,
                enabled: model.enabled,
                full_width: model.full_width,
                state: model.state,
                marked_range: model.marked_range.clone(),
                caret_visible: model.caret_visible,
                horizontal_scroll: model.horizontal_scroll,
                character_offsets: model.character_offsets.clone(),
                look: model.look.clone(),
            },
            handlers,
            window,
            cx,
        )
    }
}
pub(crate) fn localize_field(view: TextField, cx: &mut App) -> TextField {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(FieldCopy(base)), cx));
    view
}

use xgpui_luma::controls::tabs::{Tabs, TabsTemplate, TabsRenderModel, TabsRenderItem, TabsTemplateHandlers};
struct TabsCopy(Arc<dyn TabsTemplate>);
impl TabsTemplate for TabsCopy {
    fn render(
        &self,
        model: &TabsRenderModel<'_>,
        handlers: TabsTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let labels: Vec<_> = model.items.iter().map(|item| resolve(item.label, cx)).collect();
        let items = model
            .items
            .iter()
            .zip(&labels)
            .map(|(item, label)| TabsRenderItem {
                id: item.id,
                label,
                trigger_kind: item.trigger_kind,
                leading_accessory: item.leading_accessory,
                trailing_accessory: item.trailing_accessory,
                active: item.active,
                enabled: item.enabled,
                state: item.state,
                disclosure_progress: item.disclosure_progress,
            })
            .collect();
        self.0.render(
            &TabsRenderModel {
                drag_drop: model.drag_drop.clone(),
                id: model.id,
                size: model.size,
                width_mode: model.width_mode,
                items,
                active_id: model.active_id,
                enabled: model.enabled,
                focus: model.focus,
                indicator: model.indicator,
                indicator_motion: model.indicator_motion,
                disclosure_icons: model.disclosure_icons,
            },
            handlers,
            window,
            cx,
        )
    }
}
pub(crate) fn localize_tabs(view: Entity<Tabs>, cx: &mut App) -> Entity<Tabs> {
    let base = view.read(cx).template();
    view.update(cx, |view, cx| view.set_template(Arc::new(TabsCopy(base)), cx));
    view
}

use xgpui_luma::controls::sidebar::{
    SidebarControl, SidebarPanelTemplate, SidebarPanelRenderModel, SidebarPanelTemplateHandlers, SidebarRenderedNode,
};
struct SidebarCopy(Arc<dyn SidebarPanelTemplate>);
fn sidebar_nodes(nodes: &mut [SidebarRenderedNode], cx: &App) {
    for node in nodes {
        if let Some(label) = &mut node.label {
            *label = resolve(label, cx);
        }
        sidebar_nodes(&mut node.children, cx);
    }
}
impl SidebarPanelTemplate for SidebarCopy {
    fn render(
        &self,
        mut model: SidebarPanelRenderModel,
        scroll: &xgpui_luma::controls::scroll_container::ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        if let Some(title) = &mut model.title {
            *title = resolve(title, cx);
        }
        if let Some(subtitle) = &mut model.subtitle {
            *subtitle = resolve(subtitle, cx);
        }
        for nodes in [
            &mut model.header_nodes,
            &mut model.nodes,
            &mut model.footer_nodes,
            &mut model.rail_nodes,
            &mut model.rail_footer_nodes,
        ] {
            sidebar_nodes(nodes, cx);
        }
        if let Some(submenu) = &mut model.rail_submenu {
            submenu.items = menu_items(&submenu.items, cx);
        }
        self.0.render(model, scroll, handlers, window, cx)
    }
}
pub(crate) fn localize_sidebar(view: Entity<SidebarControl>, cx: &mut App) -> Entity<SidebarControl> {
    let base = view.read(cx).panel_template();
    view.update(cx, |view, cx| view.set_panel_template(Arc::new(SidebarCopy(base)), cx));
    view
}
