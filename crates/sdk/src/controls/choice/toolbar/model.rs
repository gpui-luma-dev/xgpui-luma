use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, SharedString,
    Window, div,
};

use super::control::ToolbarControl;
use super::template::{ToolbarTemplate, default_toolbar_template, modified_toolbar_template};
use super::theme::ToolbarVariant;
use crate::controls::button::Button;
use crate::controls::control_group::{ControlGroupArrowPolicy, ControlGroupFocusStrategy, ControlGroupItemLike};
use crate::controls::popup_menu::PopupMenu;
use crate::infra::presenter::{HostedContent, Presenter};
use crate::controls::selector::Selector;
use crate::infra::state::CompositeItemState;
use crate::controls::textfield::TextField;
use crate::controls::toggle::Toggle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarItemKind {
    Hosted,
    Separator,
}

/// Hosted control that can fan events into [`super::ToolbarEvent`].
#[derive(Clone)]
pub enum ToolbarItemSource {
    CommandButton(Entity<Button<()>>),
    ToggleButton(Toggle),
    Menu(Entity<PopupMenu>),
    Selector(Entity<Selector>),
    TextField(TextField),
}

pub type ToolbarClickHandler = Arc<dyn Fn(&mut Context<ToolbarControl>) + 'static>;
pub type ToolbarChangeHandler = Arc<dyn Fn(&super::ToolbarValue, &mut Context<ToolbarControl>) + 'static>;

#[derive(Clone)]
pub struct ToolbarItem {
    pub(crate) id: SharedString,
    pub(crate) kind: ToolbarItemKind,
    pub(crate) enabled: bool,
    pub(crate) label: SharedString,
    pub(crate) focus_handle: Option<FocusHandle>,
    pub(crate) arrow_policy: ControlGroupArrowPolicy,
    pub(crate) content: Option<Presenter<ToolbarItemRenderModel>>,
    pub(crate) event_source: Option<ToolbarItemSource>,
    pub(crate) on_click: Option<ToolbarClickHandler>,
    pub(crate) on_change: Option<ToolbarChangeHandler>,
}

impl ToolbarItem {
    pub fn button<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn toggle<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn menu<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn hosted<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        let id = id.into();
        Self {
            label: id.clone(),
            id,
            kind: ToolbarItemKind::Hosted,
            enabled: true,
            focus_handle: None,
            arrow_policy: ControlGroupArrowPolicy::GroupOwns,
            content: Some(Presenter::new(move |model, window, cx| HostedContent {
                element: content(model, window, cx).into_any_element(),
                focus_handle: None,
            })),
            event_source: None,
            on_click: None,
            on_change: None,
        }
    }

    pub fn separator(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            label: id.clone(),
            id,
            kind: ToolbarItemKind::Separator,
            enabled: false,
            focus_handle: None,
            arrow_policy: ControlGroupArrowPolicy::GroupOwns,
            content: None,
            event_source: None,
            on_click: None,
            on_change: None,
        }
    }

    /// Host a command button and route its clicks into the toolbar event stream.
    pub fn command_button(id: impl Into<SharedString>, entity: Entity<Button<()>>, cx: &App) -> Self {
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        Self::button(id, move |_, _, _| render.clone())
            .focus_handle(focus)
            .event_source(ToolbarItemSource::CommandButton(entity))
    }

    /// Host a toggle button, auto-flip its value on click, and emit click/change events.
    pub fn toggle_button(id: impl Into<SharedString>, entity: Toggle, cx: &App) -> Self {
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        Self::toggle(id, move |_, _, _| render.clone())
            .focus_handle(focus)
            .event_source(ToolbarItemSource::ToggleButton(entity))
    }

    /// Host a popup menu and route selections into the toolbar event stream.
    pub fn menu_control(id: impl Into<SharedString>, entity: Entity<PopupMenu>, cx: &App) -> Self {
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        Self::menu(id, move |_, _, _| render.clone())
            .focus_handle(focus)
            .event_source(ToolbarItemSource::Menu(entity))
    }

    /// Host a selector and route changes into the toolbar event stream.
    pub fn selector_control(id: impl Into<SharedString>, entity: Entity<Selector>, cx: &App) -> Self {
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        Self::hosted(id, move |_, _, _| render.clone())
            .focus_handle(focus)
            .event_source(ToolbarItemSource::Selector(entity))
    }

    /// Host a text field and route value changes into the toolbar event stream.
    pub fn textfield_control(id: impl Into<SharedString>, entity: TextField, cx: &App) -> Self {
        let focus = entity.read(cx).focus_handle(cx);
        let render = entity.clone();
        Self::hosted(id, move |_, _, _| render.clone())
            .focus_handle(focus)
            .arrow_policy(ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused)
            .event_source(ToolbarItemSource::TextField(entity))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn arrow_policy(mut self, arrow_policy: ControlGroupArrowPolicy) -> Self {
        self.arrow_policy = arrow_policy;
        self
    }

    pub fn event_source(mut self, event_source: ToolbarItemSource) -> Self {
        self.event_source = Some(event_source);
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Context<ToolbarControl>) + 'static,
    {
        self.on_click = Some(Arc::new(handler));
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&super::ToolbarValue, &mut Context<ToolbarControl>) + 'static,
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn is_separator(&self) -> bool {
        self.kind == ToolbarItemKind::Separator
    }
}

impl ControlGroupItemLike for ToolbarItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled && !self.is_separator()
    }
}

#[derive(Clone)]
pub(crate) struct ToolbarModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ToolbarItem>,
    pub(crate) enabled: bool,
    pub(crate) variant: ToolbarVariant,
    pub(crate) focus_strategy: ControlGroupFocusStrategy,
    pub(crate) template: Arc<dyn ToolbarTemplate>,
}

#[derive(Clone)]
pub struct ToolbarItemRenderModel {
    pub id: SharedString,
    pub kind: ToolbarItemKind,
    pub index: usize,
    pub enabled: bool,
    pub state: CompositeItemState,
}

pub struct ToolbarRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ToolbarItemRenderModel>,
    pub enabled: bool,
    pub variant: ToolbarVariant,
}

pub struct ToolbarBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::toolbar::ToolbarEvent>,

    pub(crate) model: ToolbarModel,
}

impl ToolbarBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::toolbar::ToolbarEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::toolbar::ToolbarEvent::Click { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `ToolbarEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &crate::controls::toolbar::ToolbarValue, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::toolbar::ToolbarEvent::Change { id, value } = event {
                callback(owner, id, value, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: ToolbarModel {
                id: id.into(),
                items: Vec::new(),
                enabled: true,
                variant: ToolbarVariant::Outline,
                focus_strategy: ControlGroupFocusStrategy::RovingItemFocus,
                template: default_toolbar_template(),
            },
        }
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn variant(mut self, variant: ToolbarVariant) -> Self {
        self.model.variant = variant;
        self
    }

    pub fn outline(self) -> Self {
        self.variant(ToolbarVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ToolbarVariant::Ghost)
    }

    /// Configure the focus strategy for the underlying control group.
    pub fn focus_strategy(mut self, strategy: ControlGroupFocusStrategy) -> Self {
        self.model.focus_strategy = strategy;
        self
    }

    /// Arrow keys move between items; Tab exits the toolbar (web-style roving focus).
    pub fn roving_item_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::RovingItemFocus)
    }

    /// Group owns Tab; active item is the focused descendant (desktop sequential style).
    pub fn sequential_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::ActiveDescendant)
    }

    pub fn template(mut self, template: Arc<dyn ToolbarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ToolbarRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_toolbar_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<ToolbarControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| ToolbarControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }

    pub(crate) fn control_group_template(&self) -> crate::controls::control_group::ControlGroupTemplate<ToolbarItem> {
        super::template::toolbar_control_group_template(self.model.variant, Arc::clone(&self.model.template))
    }
}

pub(crate) fn fallback_item(model: &ToolbarItemRenderModel) -> AnyElement {
    div().child(model.id.clone()).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::control_group::ControlGroupArrowPolicy;

    #[test]
    fn separator_is_excluded_from_navigation() {
        let item = ToolbarItem::separator("break");
        assert!(item.is_separator());
        assert!(!item.is_enabled());
    }

    #[test]
    fn hosted_item_defaults_to_group_arrow_policy() {
        let item = ToolbarItem::hosted("field", |_, _, _| div());
        assert_eq!(item.arrow_policy, ControlGroupArrowPolicy::GroupOwns);
        assert!(item.is_enabled());
    }

    #[test]
    fn builder_defaults_to_outline_variant() {
        let builder = ToolbarBuilder::new("toolbar-test");
        assert_eq!(builder.model.variant, ToolbarVariant::Outline);

        let ghost = ToolbarBuilder::new("toolbar-test").ghost();
        assert_eq!(ghost.model.variant, ToolbarVariant::Ghost);
    }

    #[test]
    fn builder_defaults_to_roving_item_focus() {
        let builder = ToolbarBuilder::new("toolbar-test");
        assert_eq!(builder.model.focus_strategy, ControlGroupFocusStrategy::RovingItemFocus);
    }

    #[test]
    fn sequential_focus_sets_active_descendant() {
        let builder = ToolbarBuilder::new("toolbar-test").sequential_focus();
        assert_eq!(builder.model.focus_strategy, ControlGroupFocusStrategy::ActiveDescendant);
    }

    #[test]
    fn roving_item_focus_sets_strategy() {
        let builder = ToolbarBuilder::new("toolbar-test").sequential_focus().roving_item_focus();
        assert_eq!(builder.model.focus_strategy, ControlGroupFocusStrategy::RovingItemFocus);
    }
}
