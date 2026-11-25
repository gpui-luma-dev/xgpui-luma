use gpui::{
    App, Context, Entity, EventEmitter, Focusable, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*,
};

use super::model::{ToolbarBuilder, ToolbarItem, ToolbarItemSource};
use super::theme::ToolbarVariant;
use crate::controls::button::ButtonEvent;
use crate::controls::control_group::{
    ControlGroupArrowPolicy, ControlGroupControl, ControlGroupFocusStrategy, ControlGroupFocusTarget,
    ControlGroupLayout, ControlSelectionMode,
};
use crate::controls::popup_menu::PopupMenuEvent;
use crate::controls::selector::SelectorEvent;
use crate::controls::textfield::TextFieldEvent;
use crate::controls::toggle::ToggleEvent;

#[derive(Clone, Debug)]
pub enum ToolbarValue {
    Bool(bool),
    String(SharedString),
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ToolbarEvent {
    /// Triggered when a button or toggle is clicked.
    Click { id: SharedString },
    /// Triggered when a toggle, menu, selector, or text field value changes.
    Change { id: SharedString, value: ToolbarValue },
}

pub struct ToolbarControl {
    group: Entity<ControlGroupControl<ToolbarItem>>,
    items: Vec<ToolbarItem>,
    variant: ToolbarVariant,
    focus_strategy: ControlGroupFocusStrategy,
    template: std::sync::Arc<dyn super::ToolbarTemplate>,
    enabled: bool,
    /// Hosted controls this toolbar disabled while it is disabled, re-enabled with it. Controls the host
    /// disabled itself are not in here, so they stay disabled.
    held_disabled: Vec<ToolbarItemSource>,
    _item_subscriptions: Vec<Subscription>,
}

impl EventEmitter<ToolbarEvent> for ToolbarControl {}

impl ToolbarControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ToolbarBuilder {
        ToolbarBuilder::new(id)
    }

    pub fn inner_group(&self) -> Entity<ControlGroupControl<ToolbarItem>> {
        self.group.clone()
    }

    pub(crate) fn from_builder(builder: ToolbarBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model.clone();
        let group = ControlGroupControl::new(model.id)
            .items(model.items.clone())
            .selection_mode(ControlSelectionMode::SingleAllowNone)
            .layout(ControlGroupLayout::Horizontal)
            .focus_strategy(model.focus_strategy)
            .with_focus_target_provider(|item, _window, _cx| {
                item.focus_handle
                    .clone()
                    .map(|focus_handle| ControlGroupFocusTarget { focus_handle, arrow_policy: item.arrow_policy })
            })
            .enabled(model.enabled)
            .template(builder.control_group_template())
            .spawn(cx);

        let mut toolbar = Self {
            group,
            items: model.items,
            variant: model.variant,
            focus_strategy: model.focus_strategy,
            template: model.template,
            enabled: model.enabled,
            held_disabled: Vec::new(),
            _item_subscriptions: Vec::new(),
        };
        toolbar.rewire_item_subscriptions(cx);
        toolbar.hold_sources(cx);
        toolbar
    }

    pub fn active_id(&self, cx: &App) -> Option<SharedString> {
        self.group.read(cx).active_id().cloned()
    }

    pub fn focus_strategy(&self) -> ControlGroupFocusStrategy {
        self.focus_strategy
    }

    pub fn set_focus_strategy(&mut self, strategy: ControlGroupFocusStrategy, cx: &mut Context<Self>) {
        if self.focus_strategy == strategy {
            return;
        }
        self.focus_strategy = strategy;
        self.group.update(cx, |group, cx| {
            group.set_focus_strategy(strategy, cx);
        });
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = ToolbarItem>, cx: &mut Context<Self>) {
        self.release_sources(cx);
        self.items = items.into_iter().collect();
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
        self.rewire_item_subscriptions(cx);
        self.hold_sources(cx);
    }

    /// Enable or disable the toolbar and the controls it hosts. A hosted control that was already
    /// disabled stays disabled when the toolbar is enabled again.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| {
            group.set_enabled(enabled, cx);
        });
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        if enabled {
            self.release_sources(cx);
        } else {
            self.hold_sources(cx);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_variant(&mut self, variant: ToolbarVariant, cx: &mut Context<Self>) {
        if self.variant == variant {
            return;
        }
        self.variant = variant;
        self.sync_group_template(cx);
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::ToolbarTemplate>, cx: &mut Context<Self>) {
        self.template = template;
        self.sync_group_template(cx);
    }

    /// Notify the toolbar shell and any hosted event-source controls.
    pub fn notify_items(&self, cx: &mut Context<Self>) {
        cx.notify();
        self.group.update(cx, |_, cx| cx.notify());
        for item in &self.items {
            let Some(source) = &item.event_source else {
                continue;
            };
            match source {
                ToolbarItemSource::CommandButton(entity) => entity.update(cx, |_, cx| cx.notify()),
                ToolbarItemSource::ToggleButton(entity) => entity.update(cx, |_, cx| cx.notify()),
                ToolbarItemSource::Menu(entity) => entity.update(cx, |_, cx| cx.notify()),
                ToolbarItemSource::Selector(entity) => entity.update(cx, |_, cx| cx.notify()),
                ToolbarItemSource::TextField(entity) => entity.update(cx, |_, cx| cx.notify()),
            }
        }
    }

    fn sync_group_template(&mut self, cx: &mut Context<Self>) {
        let template = super::template::toolbar_control_group_template(self.variant, self.template.clone());
        self.group.update(cx, |group, cx| {
            group.set_template(template, cx);
        });
    }

    fn rewire_item_subscriptions(&mut self, cx: &mut Context<Self>) {
        self._item_subscriptions = wire_item_subscriptions(&self.items, cx);
    }

    /// While disabled, disable every hosted control that is enabled, and remember it.
    fn hold_sources(&mut self, cx: &mut Context<Self>) {
        if self.enabled {
            return;
        }
        for source in self.items.iter().filter_map(|item| item.event_source.clone()) {
            if source.is_enabled(cx) {
                source.set_enabled(false, cx);
                self.held_disabled.push(source);
            }
        }
    }

    /// Re-enable the hosted controls [`Self::hold_sources`] disabled.
    fn release_sources(&mut self, cx: &mut Context<Self>) {
        for source in std::mem::take(&mut self.held_disabled) {
            source.set_enabled(true, cx);
        }
    }

    fn dispatch_click(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        if let Some(handler) = self.items.iter().find(|item| &item.id == id).and_then(|item| item.on_click.clone()) {
            handler(cx);
        }
        cx.emit(ToolbarEvent::Click { id: id.clone() });
    }

    fn dispatch_change(&mut self, id: &SharedString, value: ToolbarValue, cx: &mut Context<Self>) {
        if let Some(handler) = self.items.iter().find(|item| &item.id == id).and_then(|item| item.on_change.clone()) {
            handler(&value, cx);
        }
        cx.emit(ToolbarEvent::Change { id: id.clone(), value });
    }
}

fn wire_item_subscriptions(items: &[ToolbarItem], cx: &mut Context<ToolbarControl>) -> Vec<Subscription> {
    let mut subscriptions = Vec::new();
    for item in items {
        let Some(source) = item.event_source.clone() else {
            continue;
        };
        let id = item.id.clone();
        match source {
            ToolbarItemSource::CommandButton(entity) => {
                subscriptions.push(cx.subscribe(&entity, move |this, _, event: &ButtonEvent, cx| {
                    if matches!(event, ButtonEvent::Click) {
                        this.dispatch_click(&id, cx);
                    }
                }));
            }
            ToolbarItemSource::ToggleButton(entity) => {
                subscriptions.push(cx.subscribe(&entity, move |this, _, event: &ToggleEvent, cx| {
                    if let ToggleEvent::Change { selected } = event {
                        this.dispatch_click(&id, cx);
                        this.dispatch_change(&id, ToolbarValue::Bool(*selected), cx);
                    }
                }));
            }
            ToolbarItemSource::Menu(entity) => {
                subscriptions.push(cx.subscribe(&entity, move |this, _, event: &PopupMenuEvent, cx| {
                    if let PopupMenuEvent::Select { item_id, .. } = event {
                        this.dispatch_change(&id, ToolbarValue::String(item_id.clone()), cx);
                    }
                }));
            }
            ToolbarItemSource::Selector(entity) => {
                subscriptions.push(cx.subscribe(&entity, move |this, _, event: &SelectorEvent, cx| {
                    if let SelectorEvent::Change { item_id, .. } = event {
                        this.dispatch_change(&id, ToolbarValue::String(item_id.clone()), cx);
                    }
                }));
            }
            ToolbarItemSource::TextField(entity) => {
                subscriptions.push(cx.subscribe(&entity, move |this, _, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } = event {
                        this.dispatch_change(&id, ToolbarValue::String(SharedString::from(value.clone())), cx);
                    }
                }));
            }
        }
    }
    subscriptions
}

impl Focusable for ToolbarControl {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.group.read(cx).focus_handle(cx)
    }
}

impl Render for ToolbarControl {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.group.clone()).into_any_element()
    }
}

/// Convenience helper for hosted controls that need caret / horizontal arrow ownership.
pub fn horizontal_arrow_policy() -> ControlGroupArrowPolicy {
    ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused
}
