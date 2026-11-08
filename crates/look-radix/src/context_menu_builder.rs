//! Look-owned context-menu builder. Spawn synthesizes the SDK [`xgpui_luma::controls::context_menu::ContextMenu`].

use gpui::{App, Context, Div, Entity, IntoElement, SharedString, Stateful};
use xgpui_luma::controls::context_menu::{ContextMenuBuilder, ContextMenuRenderModel};
use xgpui_luma::infra::menu_item::MenuItem;

use crate::look::{Look, resolve_look};
use crate::{ContextMenuVariant, Tone, context_menu_template};

type ContextMenuModifier = Box<dyn Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Radix context menu backed by SDK input, focus, and submenu behavior.
///
/// ```no_run
/// use gpui::Context;
/// use xgpui_luma::controls::context_menu::MenuItem;
/// use xgpui_luma_look_radix::{ContextMenu, Look};
/// fn menu<M: 'static>(look: &Look, cx: &mut Context<M>) -> gpui::Entity<xgpui_luma::controls::context_menu::ContextMenu> {
///     ContextMenu::new("actions").look(look).soft().label("Right-click here")
///         .items([MenuItem::new("copy").label("Copy")]).spawn(cx)
/// }
/// ```
pub struct ContextMenu {
    motion_enabled: bool,
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::context_menu::ContextMenuMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::context_menu::ContextMenuEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: ContextMenuVariant,
    tone: Tone,
    label: Option<SharedString>,
    items: Vec<MenuItem>,
    enabled: bool,
    target: Option<xgpui_luma::controls::context_menu::ContextMenuTargetContent>,
    modifiers: Vec<ContextMenuModifier>,
}

impl ContextMenu {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::context_menu::ContextMenuEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ContextMenuEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `ContextMenuEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::context_menu::ContextMenuEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            motion: Default::default(),
            motion_enabled: true,
            variant: ContextMenuVariant::default(),
            tone: Tone::Accent,
            label: None,
            items: Vec::new(),
            enabled: true,
            target: None,
            modifiers: Vec::new(),
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::context_menu::ContextMenuMotion, edits motion);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = enabled;
        self
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: ContextMenuVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn solid(self) -> Self {
        self.variant(ContextMenuVariant::Solid)
    }
    pub fn soft(self) -> Self {
        self.variant(ContextMenuVariant::Soft)
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn accent(self) -> Self {
        self.tone(Tone::Accent)
    }
    pub fn gray(self) -> Self {
        self.tone(Tone::Gray)
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn target_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut gpui::App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.target = Some(std::sync::Arc::new(move |cx| content(cx).into_any_element()));
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::context_menu::ContextMenu> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> ContextMenuBuilder {
        let mut builder = xgpui_luma::controls::context_menu::ContextMenu::new(self.id)
            .motion_defaults(look.motion_defaults().context_menu)
            .with_motion_overrides(self.motion)
            .motion_enabled(self.motion_enabled)
            .template(context_menu_template(&look, self.variant, self.tone))
            .items(self.items)
            .enabled(self.enabled);
        if let Some(label) = self.label {
            builder = builder.label(label);
        }
        if let Some(target) = self.target {
            builder = builder.target_content(move |cx| target(cx));
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}
