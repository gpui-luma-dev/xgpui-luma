//! Look-owned toolbar builder. Spawn synthesizes the SDK [`xgpui_luma::controls::toolbar::ToolbarControl`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::control_group::ControlGroupFocusStrategy;
use xgpui_luma::controls::toolbar::{ToolbarBuilder, ToolbarItem, ToolbarRenderModel, ToolbarVariant};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type ToolbarModifier = Box<dyn Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a toolbar: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Toolbar {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::toolbar::ToolbarEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    items: Vec<ToolbarItem>,
    enabled: bool,
    size: ShadcnSize,
    variant: ToolbarVariant,
    focus_strategy: ControlGroupFocusStrategy,
    modifiers: Vec<ToolbarModifier>,
}

impl Toolbar {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::toolbar::ToolbarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ToolbarEvent::Click` through the existing event stream.
    pub fn on_click<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::toolbar::ToolbarEvent::Click { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `ToolbarEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &xgpui_luma::controls::toolbar::ToolbarValue, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::toolbar::ToolbarEvent::Change { id, value } = event {
                callback(owner, id, value, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            items: Vec::new(),
            enabled: true,
            size: ShadcnSize::Md,
            variant: ToolbarVariant::Outline,
            focus_strategy: ControlGroupFocusStrategy::RovingItemFocus,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ToolbarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn outline(self) -> Self {
        self.variant(ToolbarVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ToolbarVariant::Ghost)
    }

    pub fn focus_strategy(mut self, strategy: ControlGroupFocusStrategy) -> Self {
        self.focus_strategy = strategy;
        self
    }

    pub fn roving_item_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::RovingItemFocus)
    }

    pub fn sequential_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::ActiveDescendant)
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::toolbar::ToolbarControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ToolbarBuilder {
        let mut builder = xgpui_luma::controls::toolbar::new(self.id)
            .template(look.toolbar_template())
            .items(self.items)
            .enabled(self.enabled)
            .size(self.size.control_size())
            .variant(self.variant)
            .focus_strategy(self.focus_strategy);
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_helpers_set_ghost() {
        let toolbar = Toolbar::new("editor").ghost().size(ShadcnSize::Sm);
        assert_eq!(toolbar.variant, ToolbarVariant::Ghost);
        assert_eq!(toolbar.size, ShadcnSize::Sm);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Toolbar::new("ok").look(&look).outline().into_sdk_builder(look);
    }
}
