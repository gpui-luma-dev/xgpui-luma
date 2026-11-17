#[cfg(all(test, feature = "test-support"))]
#[path = "locale_tests.rs"]
mod locale_tests;

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    SharedString, Subscription, Window, div,
};

use crate::controls::control_group::{ControlGroupControl, ControlGroupEvent, ControlGroupItemLike};

use super::model::{RadioGroupBuilder, RadioGroupEvent};

pub struct RadioGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    group: Entity<ControlGroupControl<T>>,
    _subscriptions: Vec<Subscription>,
}

impl<T> EventEmitter<RadioGroupEvent> for RadioGroupControl<T> where T: ControlGroupItemLike + 'static {}

impl<T> RadioGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(super) fn from_builder(builder: RadioGroupBuilder<T>, cx: &mut Context<Self>) -> Self {
        let group = builder.inner.spawn(cx);
        let subscription = cx.subscribe(&group, Self::handle_group_event);
        Self { group, _subscriptions: vec![subscription] }
    }

    pub fn selected_id(&self, cx: &App) -> Option<SharedString> {
        self.group.read(cx).selected_id().cloned()
    }

    pub fn selected_ids(&self, cx: &App) -> Vec<SharedString> {
        self.group.read(cx).selected_ids().to_vec()
    }

    pub fn active_id(&self, cx: &App) -> Option<SharedString> {
        self.group.read(cx).active_id().cloned()
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.set_motion_enabled(enabled, cx));
        cx.notify();
    }
    pub fn inner_group(&self) -> Entity<ControlGroupControl<T>> {
        self.group.clone()
    }

    pub fn set_selected(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.set_selected_ids([selected_id.into()], cx));
        cx.notify();
    }

    pub fn set_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.group.update(cx, |group, cx| group.set_selected_ids(selected_ids, cx));
        cx.notify();
    }

    pub fn set_managed_selected(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.set_managed_selected_ids([selected_id.into()], cx));
        cx.notify();
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.group.update(cx, |group, cx| group.set_managed_selected_ids(selected_ids, cx));
        cx.notify();
    }

    pub fn clear_managed_selected_ids(&mut self, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.clear_managed_selected_ids(cx));
        cx.notify();
    }

    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.set_active(active_id, cx));
        cx.notify();
    }

    pub fn clear_active(&mut self, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.clear_active(cx));
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| group.set_enabled(enabled, cx));
        cx.notify();
    }

    fn handle_group_event(
        &mut self,
        _: Entity<ControlGroupControl<T>>,
        event: &ControlGroupEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ControlGroupEvent::Change { selected_ids, .. } => {
                cx.emit(RadioGroupEvent::Change { value: selected_ids.first().cloned() });
            }
            ControlGroupEvent::Activate { activated_id } => {
                cx.emit(RadioGroupEvent::Activate { value: activated_id.clone() });
            }
            ControlGroupEvent::FocusChanged { focused } => {
                cx.emit(RadioGroupEvent::FocusChanged { focused: *focused });
            }
            ControlGroupEvent::ItemFocused { item_id } => {
                cx.emit(RadioGroupEvent::ItemFocused { item_id: item_id.clone() });
            }
            ControlGroupEvent::ItemBoundsChanged { .. } => {}
        }
    }
}

impl<T> Focusable for RadioGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.group.read(cx).focus_handle(cx)
    }
}

impl<T> Render for RadioGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.group.clone()).into_any_element()
    }
}

impl<T> IntoElement for RadioGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
