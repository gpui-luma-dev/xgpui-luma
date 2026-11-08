//! Bare sidebar navigation. Width and visibility belong to the host.
//!
//! ```no_run
//! use gpui::{Context, Entity};
//! use xgpui_luma::controls::sidebar::SidebarBuilder;
//! use xgpui_luma_look_shadcn::{Frame, ShadcnLook, Sidebar};
//!
//! fn standalone<M: 'static>(
//!     look: &ShadcnLook, content: SidebarBuilder, cx: &mut Context<M>,
//! ) -> Entity<xgpui_luma::controls::frame::FrameControl> {
//!     let navigation = Sidebar::new("nav").look(look).sidebar(content).spawn(cx);
//!     Frame::sidebar("nav-frame").look(look).child(navigation).spawn(cx)
//! }
//! ```

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use xgpui_luma::controls::sidebar::{
    SidebarBuilder, SidebarContentBuilder, SidebarControlBuilder, SidebarFooterBuilder, SidebarGroupBuilder,
    SidebarHeaderBuilder, SidebarMenuBuilder, SidebarMenuItemBuilder, SidebarMenuSubBuilder, SidebarPresentation,
};

use crate::look::{ShadcnLook, resolve_look_from};

/// Builder in the guise of a sidebar control: Shadcn templates plus SDK options, until `.spawn(cx)`.
pub struct Sidebar {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::sidebar::SidebarEvent>,

    look: Option<ShadcnLook>,
    builder: SidebarControlBuilder,
}

impl Sidebar {
    xgpui_luma::forward_motion!(xgpui_luma::controls::sidebar::SidebarMotion, builder builder);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::sidebar::SidebarDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }
    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::ItemsAdded { source, target, keys, location } =
                event
            {
                callback(owner, *source, *target, keys, location, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::ItemsReordered { target, keys, location } = event {
                callback(owner, *target, keys, location, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::sidebar::SidebarDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::Dropped {
                source,
                target,
                keys,
                location,
                changed,
            } = event
            {
                callback(owner, *source, *target, keys, location, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::sidebar::SidebarDropLocation,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::DropRejected {
                source,
                target,
                keys,
                location,
                error,
            } = event
            {
                callback(owner, *source, *target, keys, location, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            xgpui_luma::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarDragEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::sidebar::SidebarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SidebarEvent::PresentationChanged` through the existing event stream.
    pub fn on_presentation_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::sidebar::SidebarPresentation, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::PresentationChanged { presentation } = event {
                callback(owner, presentation, cx);
            }
        })
    }

    /// Handle `SidebarEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::Select { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::ItemFocused { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::HoverChanged { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::SubMenuToggle` through the existing event stream.
    pub fn on_sub_menu_toggle<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::SubMenuToggle { id, open } = event {
                callback(owner, id, *open, cx);
            }
        })
    }

    /// Handle `SidebarEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::sidebar::SidebarEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),
            look: None,
            builder: xgpui_luma::controls::sidebar::SidebarControl::new(id),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn presentation(mut self, presentation: SidebarPresentation) -> Self {
        self.builder = self.builder.presentation(presentation);
        self
    }

    /// Enable SDK-owned same-sidebar subtree moves.
    pub fn drag_drop(mut self, config: xgpui_luma::controls::sidebar::SidebarDragDrop) -> Self {
        self.builder = self.builder.drag_drop(config);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected_id(selected_id);
        self
    }

    pub fn sidebar(mut self, sidebar: SidebarBuilder) -> Self {
        self.builder = self.builder.sidebar(sidebar);
        self
    }

    pub fn overlay_scrollbar(mut self, overlay: bool) -> Self {
        self.builder = self.builder.overlay_scrollbar(overlay);
        self
    }

    pub fn scrollbar_placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.builder = self.builder.scrollbar_placement(placement);
        self
    }

    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.builder = self.builder.scrollbar_visibility(visibility);
        self
    }

    pub fn auto_hide_scrollbar(mut self, auto_hide: bool) -> Self {
        self.builder = self.builder.auto_hide_scrollbar(auto_hide);
        self
    }

    pub fn auto_hide_scrollbar_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.builder = self.builder.auto_hide_scrollbar_activate(activate);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::sidebar::SidebarControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            let theme = crate::tooltip_theme(&look);
            let entity = self
                .builder
                .stylesheet(&look.stylesheet().common)
                .motion_defaults(look.motion_defaults().sidebar)
                .panel_template(look.sidebar_panel_template())
                .scrollbar_template(look.scrollbar_template())
                .spawn(cx);
            entity.update(cx, |sidebar, cx| sidebar.set_tooltip_theme(theme, cx));
            entity
        };
        callbacks.bind(&control, cx);
        control
    }

    pub fn panel(id: impl Into<SharedString>) -> SidebarBuilder {
        xgpui_luma::controls::sidebar::sidebar(id)
    }

    pub fn header() -> SidebarHeaderBuilder {
        xgpui_luma::controls::sidebar::sidebar_header()
    }

    pub fn content() -> SidebarContentBuilder {
        xgpui_luma::controls::sidebar::sidebar_content()
    }

    pub fn group() -> SidebarGroupBuilder {
        xgpui_luma::controls::sidebar::sidebar_group()
    }

    pub fn menu(id: impl Into<SharedString>) -> SidebarMenuBuilder {
        xgpui_luma::controls::sidebar::sidebar_menu(id)
    }

    pub fn menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> SidebarMenuItemBuilder {
        xgpui_luma::controls::sidebar::sidebar_menu_item(id, label)
    }

    pub fn menu_sub() -> SidebarMenuSubBuilder {
        xgpui_luma::controls::sidebar::sidebar_menu_sub()
    }

    pub fn footer() -> SidebarFooterBuilder {
        xgpui_luma::controls::sidebar::sidebar_footer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_helpers_construct() {
        let _ = Sidebar::header().title("Properties");
        let _ = Sidebar::menu("pinned");
        let _ = Sidebar::menu_item("a", "A");
    }
}
