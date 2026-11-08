//! Look-owned selector builder. Spawn synthesizes the SDK [`xgpui_luma::controls::selector::Selector`].

use gpui::{App, Context, Div, Entity, IntoElement, SharedString, Stateful};
use xgpui_luma::controls::selector::{
    SelectorBuilder, SelectorIcons, SelectorItem, SelectorItemLike, SelectorItemRenderModel, SelectorPlacement,
    SelectorRenderModel, SelectorTriggerStyle, SelectorOpeningMode,
};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type SelectorModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync>;
type SelectorItemTemplate<T> =
    Box<dyn for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> gpui::AnyElement + Send + Sync>;

/// Builder in the guise of a selector: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Selector<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    motion_enabled: bool,
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::selector::SelectorMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::selector::SelectorEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    label: Option<SharedString>,
    items: Vec<T>,
    selected_id: Option<SharedString>,
    size: ShadcnSize,
    enabled: bool,
    scroll_interaction: xgpui_luma::interaction::ScrollInteraction,
    invalid: bool,
    tab_stop: bool,
    placement: SelectorPlacement,
    opening_mode: SelectorOpeningMode,
    trigger_style: SelectorTriggerStyle,
    icons: Option<SelectorIcons>,
    without_elevation: bool,
    item_template: Option<SelectorItemTemplate<T>>,
    modifiers: Vec<SelectorModifier<T>>,
}

impl Selector<SelectorItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id)
    }
}

impl<T> Selector<T>
where
    T: SelectorItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::selector::SelectorEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SelectorEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::Change { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SelectorEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SelectorEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SelectorEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::selector::SelectorEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            motion: Default::default(),
            motion_enabled: true,
            label: None,
            items: Vec::new(),
            selected_id: None,
            size: ShadcnSize::Md,
            enabled: true,
            scroll_interaction: xgpui_luma::interaction::ScrollInteraction::VIEWPORT,
            invalid: false,
            tab_stop: true,
            placement: SelectorPlacement::Smart,
            opening_mode: SelectorOpeningMode::default(),
            trigger_style: SelectorTriggerStyle::default(),
            icons: None,
            without_elevation: false,
            item_template: None,
            modifiers: Vec::new(),
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::selector::SelectorMotion, edits motion);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = enabled;
        self
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn outline(self) -> Self {
        self.trigger_style(SelectorTriggerStyle::Outline)
    }

    pub fn ghost(self) -> Self {
        self.trigger_style(SelectorTriggerStyle::Ghost)
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.trigger_style = style;
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Choose popup wheel eligibility without changing focus or boundary behavior.
    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.scroll_interaction.wheel = policy;
        self
    }

    /// Choose containment or whole-event chaining independently of wheel eligibility.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.scroll_interaction.boundary = policy;
        self
    }

    /// Choose which actual focus owners qualify for focus-required wheel input.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Choose dropdown or selected-item-centered opening.
    pub fn opening_mode(mut self, mode: SelectorOpeningMode) -> Self {
        self.opening_mode = mode;
        self
    }

    pub fn placement(mut self, placement: SelectorPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn icons(mut self, icons: SelectorIcons) -> Self {
        self.icons = Some(icons);
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.without_elevation = true;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.item_template = Some(Box::new(move |model, cx| template(model, cx).into_any_element()));
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::selector::Selector<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            use xgpui_luma::infra::attachments::TooltipEntityExt;
            let theme = crate::tooltip_theme(&look);
            self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SelectorBuilder<T> {
        let mut builder = xgpui_luma::controls::selector::Selector::new_typed(self.id)
            .stylesheet(&look.stylesheet().common)
            .motion_defaults(look.motion_defaults().selector)
            .with_motion_overrides(self.motion)
            .motion_enabled(self.motion_enabled)
            .template(look.selector_template())
            .items(self.items)
            .size(self.size.control_size())
            .enabled(self.enabled)
            .wheel_scroll_policy(self.scroll_interaction.wheel)
            .scroll_boundary_policy(self.scroll_interaction.boundary)
            .wheel_focus_scope(self.scroll_interaction.focus_scope)
            .invalid(self.invalid)
            .tab_stop(self.tab_stop)
            .placement(self.placement)
            .opening_mode(self.opening_mode)
            .trigger_style(self.trigger_style);
        if let Some(label) = self.label {
            builder = builder.label(label);
        }
        if let Some(selected_id) = self.selected_id {
            builder = builder.selected_id(selected_id);
        }
        if let Some(icons) = self.icons {
            builder = builder.icons(icons);
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(item_template) = self.item_template {
            builder = builder.with_item_template(move |model, cx| item_template(model, cx));
        }
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
    fn items_keep_shadcn_axes() {
        let selector = Selector::new("theme").ghost().size(ShadcnSize::Sm).label("Theme");
        assert_eq!(selector.trigger_style, SelectorTriggerStyle::Ghost);
        assert_eq!(selector.size, ShadcnSize::Sm);
        assert_eq!(selector.label.as_deref(), Some("Theme"));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Selector::new("ok")
            .look(&look)
            .items([SelectorItem::new("one").label("One")])
            .selected_id("one")
            .into_sdk_builder(look);
    }

    #[test]
    fn typed_into_sdk_builder_does_not_panic() {
        struct ThemeItem {
            id: SharedString,
            label: SharedString,
        }
        impl SelectorItemLike for ThemeItem {
            fn id(&self) -> &SharedString {
                &self.id
            }
            fn label(&self) -> &SharedString {
                &self.label
            }
        }

        let look = ShadcnLook::built_in();
        let _builder = Selector::new_typed("ok")
            .look(&look)
            .items([ThemeItem { id: "one".into(), label: "One".into() }])
            .selected_id("one")
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn forwards_popup_wheel_overrides_after_look_synthesis() {
    use xgpui_luma::interaction::{ScrollBoundaryPolicy, WheelFocusScope, WheelScrollPolicy};
    let mut app = gpui::TestAppContext::single();
    let control = Selector::new("policy")
        .wheel_scroll_policy(WheelScrollPolicy::PassThrough)
        .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
        .wheel_focus_scope(WheelFocusScope::Descendants)
        .into_sdk_builder(ShadcnLook::built_in())
        .spawn(&mut app);
    control.read_with(&app, |view, _| {
        let policy = view.scroll_interaction();
        assert_eq!(policy.wheel, WheelScrollPolicy::PassThrough);
        assert_eq!(policy.boundary, ScrollBoundaryPolicy::Chain);
        assert_eq!(policy.focus_scope, WheelFocusScope::Descendants);
    });
}
