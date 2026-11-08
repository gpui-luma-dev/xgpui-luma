//! Look-bound fluent factories for [`ToolbarItem`](xgpui_luma::controls::toolbar::ToolbarItem).
//!
//! SDK toolbar items are lookless hosted wrappers. These helpers spawn Shadcn-styled child
//! controls and return ready-to-use [`ToolbarItem`] values for declarative toolbar composition.

use std::sync::Arc;

use gpui::Context;
use gpui::Focusable;
use gpui::Pixels;
use gpui::SharedString;
use gpui::div;
use gpui::px;
use gpui::prelude::*;
use xgpui_luma::controls::button::ControlIcon;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::selector::SelectorItem;
use xgpui_luma::controls::toolbar::ToolbarItem;
use xgpui_luma::controls::toolbar::ToolbarItemSource;
use xgpui_luma::controls::toolbar::horizontal_arrow_policy;
use xgpui_luma::focus::EscapeFocus;

use crate::look::ShadcnLook;
use crate::Button;
use crate::PopupMenu;
use crate::Selector;
use crate::TextField;
use crate::Toggle;

/// Default fixed width for toolbar-hosted search/text fields.
const DEFAULT_TOOLBAR_TEXTFIELD_WIDTH: f32 = 140.0;

/// Fluent builder for a toolbar-hosted text field.
pub struct ToolbarTextFieldItemBuilder {
    look: ShadcnLook,
    id: SharedString,
    placeholder: SharedString,
    width: Pixels,
}

impl ToolbarTextFieldItemBuilder {
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Pin the text field to a fixed width (toolbar items do not stretch).
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> ToolbarItem {
        let width = self.width;
        // Size the host and keep the field full-width so the control fill matches the
        // focus ring (same pattern as search_selector / combobox embedded fields).
        let entity = TextField::new(format!("toolbar-field-{}", self.id))
            .look(&self.look)
            .placeholder(self.placeholder)
            .tab_stop(false)
            .full_width(true)
            .spawn(cx);
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        ToolbarItem::hosted(self.id, move |_, _, _| {
            // Absorb Escape so it does not bubble to the pane focus scope.
            div().w(width).flex_none().child(render.clone()).on_action(|_: &EscapeFocus, _window, cx| {
                cx.stop_propagation();
            })
        })
        .focus_handle(focus)
        .arrow_policy(horizontal_arrow_policy())
        .event_source(ToolbarItemSource::TextField(entity))
    }
}

/// Look-bound toolbar item factories matching the declarative toolbar builder sketch.
pub trait ShadcnToolbarItemExt {
    /// Ghost icon command button item (`ToolbarItem::button(id, icon)` in the issue sketch).
    fn toolbar_button<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem;

    /// Ghost icon toggle button item.
    fn toolbar_toggle<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem;

    /// Ghost popup menu item with a trigger icon.
    fn toolbar_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem;

    /// Ghost text-label popup menu item (no trigger icon).
    fn toolbar_label_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem;

    /// Selector item with a default selected id.
    fn toolbar_selector<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = SelectorItem>,
        selected_id: impl Into<SharedString>,
        cx: &mut Context<M>,
    ) -> ToolbarItem;

    /// Start a fluent text-field item (`ToolbarItem::textfield(id).placeholder(...)`).
    fn toolbar_textfield(&self, id: impl Into<SharedString>) -> ToolbarTextFieldItemBuilder;
}

impl ShadcnToolbarItemExt for ShadcnLook {
    fn toolbar_button<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        let id = id.into();
        let entity =
            Button::icon_button(format!("toolbar-btn-{id}"), icon).look(self).ghost().tab_stop(false).spawn(cx);
        ToolbarItem::command_button(id, entity, cx)
    }

    fn toolbar_toggle<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        let id = id.into();
        let icon = icon.into();
        let entity =
            Toggle::new(format!("toolbar-toggle-{id}")).look(self).ghost().icon(icon).tab_stop(false).spawn(cx);
        ToolbarItem::toggle_button(id, entity, cx)
    }

    fn toolbar_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        let id = id.into();
        let mut builder = PopupMenu::new(format!("toolbar-menu-{id}"))
            .look(self)
            .label(label)
            .ghost()
            .tab_stop(false)
            .items(items);
        if let ControlIcon::Lucide(lucide) = icon.into() {
            builder = builder.icon(lucide);
        }
        let entity = builder.spawn(cx);
        ToolbarItem::menu_control(id, entity, cx)
    }

    fn toolbar_label_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        let id = id.into();
        let entity = PopupMenu::new(format!("toolbar-menu-{id}"))
            .look(self)
            .label(label)
            .ghost()
            .tab_stop(false)
            .items(items)
            .spawn(cx);
        ToolbarItem::menu_control(id, entity, cx)
    }

    fn toolbar_selector<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = SelectorItem>,
        selected_id: impl Into<SharedString>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        let id = id.into();
        let entity = Selector::new(format!("toolbar-selector-{id}"))
            .look(self)
            .label(label)
            .tab_stop(false)
            .items(items)
            .selected_id(selected_id)
            .spawn(cx);
        ToolbarItem::selector_control(id, entity, cx)
    }

    fn toolbar_textfield(&self, id: impl Into<SharedString>) -> ToolbarTextFieldItemBuilder {
        ToolbarTextFieldItemBuilder {
            look: self.clone(),
            id: id.into(),
            placeholder: SharedString::new(""),
            width: px(DEFAULT_TOOLBAR_TEXTFIELD_WIDTH),
        }
    }
}

impl ShadcnToolbarItemExt for Arc<ShadcnLook> {
    fn toolbar_button<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        self.as_ref().toolbar_button(id, icon, cx)
    }

    fn toolbar_toggle<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        self.as_ref().toolbar_toggle(id, icon, cx)
    }

    fn toolbar_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        self.as_ref().toolbar_menu(id, label, icon, items, cx)
    }

    fn toolbar_label_menu<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = MenuItem>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        self.as_ref().toolbar_label_menu(id, label, items, cx)
    }

    fn toolbar_selector<M: 'static>(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = SelectorItem>,
        selected_id: impl Into<SharedString>,
        cx: &mut Context<M>,
    ) -> ToolbarItem {
        self.as_ref().toolbar_selector(id, label, items, selected_id, cx)
    }

    fn toolbar_textfield(&self, id: impl Into<SharedString>) -> ToolbarTextFieldItemBuilder {
        self.as_ref().toolbar_textfield(id)
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_toolbar_stylesheet() -> xgpui_luma::theme::stylesheet::ToolbarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ToolbarStylesheet {
        geometry: ToolbarGeometryStylesheet { padding_x: Some(6.0), padding_y: Some(4.0), ..Default::default() },
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct ToolbarMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub gap: crate::ResolvedMetric,
    pub separator_height: crate::ResolvedMetric,
}

pub fn resolve_toolbar_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> ToolbarMetricTable {
    use crate::tables::metrics::helpers::{control_size_key, derived_metric, scaffold_control_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, Default::default());
    let metrics = ctx.metrics();
    let control = metrics.for_size(size);
    let size_key = control_size_key(size);

    let mut table = ToolbarMetricTable {
        radius: derived_metric("toolbar radius = radius.md", metrics.radius.md),
        padding_x: derived_metric("toolbar padding x = 6px", 6.0),
        padding_y: derived_metric("toolbar padding y = 4px", 4.0),
        gap: derived_metric(format!("{size_key} toolbar gap = control gap"), control.gap),
        separator_height: scaffold_control_metric(size_key, "height", control.height),
    };
    let geometry = mode.stylesheet().common.toolbar.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::ToolbarGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
            ..Default::default()
        },
    );
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table
}
