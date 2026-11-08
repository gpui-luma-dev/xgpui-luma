//! Interactive hover cards composed with the SDK anchored popover engine.
use std::{sync::Arc, time::Duration};
use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Task, Window, div,
    prelude::*, px,
};
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::controls::popover_button::{PopoverButton, PopoverButtonEvent, PopoverDismissPolicy};
use xgpui_luma::controls::overlay_window::{OverlayWindowLook, OverlayWindowMode};
use xgpui_luma::theme::{ControlSize, ThemeMode};
use crate::{Look, ScaleFamily};
use crate::look::resolve_look;

/// Raised popup surface, distinct from the gray-step-2 page panels.
pub fn hover_card_look(look: &Look) -> OverlayWindowLook {
    let mut style = crate::overlay_window_theme(look).resolve(ControlSize::Md, OverlayWindowMode::Modeless);
    style.background = look.resolve_step(ScaleFamily::Gray, if look.mode() == ThemeMode::Dark { 3 } else { 1 }).hsla();
    style
}

/// Hover lifecycle events. Content can use these to update a shared detail view.
#[derive(Clone, Debug)]
pub enum HoverCardEvent {
    OpenChanged { open: bool },
}

/// SDK-backed hover card. Moving into its content cancels delayed dismissal.
pub struct HoverCard {
    popup: Entity<PopoverButton>,
    pending: Option<Task<()>>,
    over_trigger: bool,
    over_content: bool,
    _subscriptions: Vec<Subscription>,
}
impl EventEmitter<HoverCardEvent> for HoverCard {}

type HoverCardContent = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Look-owned builder for an interactive card attached to an SDK button.
pub struct HoverCardBuilder {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::popover_button::PopoverMotion>,
    motion_enabled: bool,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<HoverCardEvent>,
    id: SharedString,
    trigger: Entity<Button>,
    look: Option<Look>,
    content: HoverCardContent,
}
impl HoverCard {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>, trigger: Entity<Button>) -> HoverCardBuilder {
        HoverCardBuilder {
            callbacks: Default::default(),
            id: id.into(),
            trigger,
            look: None,
            motion: Default::default(),
            motion_enabled: true,
            content: Arc::new(|_, _| div().into_any_element()),
        }
    }
    pub fn is_open(&self, cx: &App) -> bool {
        self.popup.read(cx).is_open(cx)
    }
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        self.popup.update(cx, |popup, cx| popup.dismiss(cx));
    }
    fn enter(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        if self.is_open(cx) {
            return;
        }
        self.pending = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(200)).await;
            let _ = this.update(cx, |this, cx| this.open(cx));
        }));
    }
    fn open(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        if !self.is_open(cx) {
            self.popup.update(cx, |popup, cx| popup.open(cx));
        }
    }
    fn leave(&mut self, cx: &mut Context<Self>) {
        self.pending = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(200)).await;
            let _ = this.update(cx, |this, cx| {
                if !this.over_trigger && !this.over_content {
                    this.dismiss(cx);
                }
            });
        }));
    }
}
impl HoverCardBuilder {
    xgpui_luma::forward_motion!(xgpui_luma::controls::popover_button::PopoverMotion, edits motion);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = enabled;
        self
    }
    /// Append an owner-bound lifecycle callback. Owners are held weakly.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &HoverCardEvent, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }
    /// Handle the semantic open state.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            let HoverCardEvent::OpenChanged { open } = event;
            callback(owner, *open, cx);
        })
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn content(mut self, content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.content = Arc::new(content);
        self
    }
    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<HoverCard> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            cx.new(|cx| {
                let weak = cx.entity().downgrade();
                let trigger = self.trigger.clone();
                let popup = PopoverButton::new(self.id)
                    .stylesheet(&look.common_stylesheet())
                    .motion_defaults(look.motion_defaults().popover)
                    .with_motion_overrides(self.motion)
                    .motion_enabled(self.motion_enabled)
                    .typed(())
                    .trigger(move |_, _| trigger.clone().into_any_element())
                    .content(move |_, window, cx| {
                        let style = hover_card_look(&look);
                        let weak = weak.clone();
                        div()
                            .id("hover-card-content")
                            .bg(style.background)
                            .border_1()
                            .border_color(style.border)
                            .rounded(px(style.radius))
                            .shadow(style.shadow)
                            .p(px(style.padding))
                            .text_color(style.foreground)
                            .font_family(style.font_family)
                            .on_hover(move |hovered, _, cx| {
                                let _ = weak.update(cx, |card: &mut HoverCard, cx| {
                                    card.over_content = *hovered;
                                    if *hovered {
                                        card.pending = None;
                                    } else if !card.over_trigger {
                                        card.leave(cx);
                                    }
                                });
                            })
                            .child((self.content)(window, cx))
                            .into_any_element()
                    })
                    .focus_on_open(false)
                    .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
                    .offset_y(px(6.0))
                    .initial_content_size(gpui::size(px(440.0), px(180.0)))
                    .trigger_on_pointer_down(false)
                    .spawn(cx);
                let subscriptions = vec![
                    cx.subscribe(&self.trigger, |this: &mut HoverCard, _, event: &ButtonEvent, cx| match event {
                        ButtonEvent::HoverChanged { hovered } => {
                            this.over_trigger = *hovered;
                            if *hovered {
                                this.enter(cx);
                            } else if !this.over_content {
                                this.leave(cx);
                            }
                        }
                        ButtonEvent::Click => this.open(cx),
                        _ => {}
                    }),
                    cx.subscribe(&popup, |_, _, event: &PopoverButtonEvent, cx| {
                        let PopoverButtonEvent::OpenChanged { open } = event;
                        cx.emit(HoverCardEvent::OpenChanged { open: *open });
                    }),
                ];
                HoverCard {
                    popup,
                    pending: None,
                    over_trigger: false,
                    over_content: false,
                    _subscriptions: subscriptions,
                }
            })
        };
        callbacks.bind(&control, cx);
        control
    }
}
impl Render for HoverCard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.popup.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_surface_is_distinct_from_panel_in_both_modes() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let popup = hover_card_look(&look);
            assert_ne!(popup.background, look.resolve_role(crate::SemanticRole::Surface).hsla());
            assert!(popup.border.a > 0.0);
            assert!(!popup.shadow.is_empty());
        }
    }
}

/// Owner-bound lifecycle callbacks for existing HoverCards.
pub trait HoverCardEventExt: Sized {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &HoverCardEvent, &mut Context<M>) + 'static,
    ) -> Self;
    fn on_open_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            let HoverCardEvent::OpenChanged { open } = event;
            callback(owner, *open, cx);
        })
    }
}
impl HoverCardEventExt for Entity<HoverCard> {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &HoverCardEvent, &mut Context<M>) + 'static,
    ) -> Self {
        xgpui_luma::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod callback_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    #[derive(Default)]
    struct Owner {
        opened: Vec<bool>,
    }
    #[test]
    fn builder_and_factory_forward_open_lifecycle() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| cx.new(|_| Owner::default()));
        let card = app.update(|cx| {
            owner.update(cx, |_, cx| {
                let trigger = crate::Button::new("hover-trigger").spawn(cx);
                HoverCard::new("hover-card", trigger)
                    .on_open_changed(cx, |owner: &mut Owner, open, _| owner.opened.push(open))
                    .spawn(cx)
                    .on_open_changed(cx, |owner, open, _| owner.opened.push(open))
            })
        });
        app.update(|cx| card.update(cx, |_, cx| cx.emit(HoverCardEvent::OpenChanged { open: true })));
        app.read(|cx| assert_eq!(owner.read(cx).opened, [true, true]));
    }
}
