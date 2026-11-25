use std::sync::Arc;

use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, IntoElement, MouseButton, Pixels, Render, SharedString, Window,
    div, prelude::*,
};

use super::model::{
    PopoverButtonBuilder, PopoverButtonEvent, PopoverDismissPolicy, PopoverPlacement, PopoverRenderModel,
    PopoverTrigger,
};
use super::panel::{PopoverPanel, PopoverPanelDismissPolicy, PopoverPanelEvent, PopoverPanelPlacement};

pub struct PopoverButton<D = ()> {
    id: SharedString,
    data: Arc<D>,
    trigger: PopoverTrigger<D>,
    panel: Entity<PopoverPanel>,
    trigger_on_pointer_down: bool,
    measure_trigger: bool,
    /// The builder set these explicitly, so [`Self::set_geometry`] leaves them alone.
    explicit_offset_y: bool,
    explicit_window_margin: bool,
}

impl<D: 'static> EventEmitter<PopoverButtonEvent> for PopoverButton<D> {}

impl<D: Default + 'static> PopoverButton<D> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopoverButtonBuilder<D> {
        PopoverButtonBuilder::new(id)
    }
    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.set_motion_enabled(enabled, cx));
        cx.notify();
    }
    pub(super) fn from_builder(builder: PopoverButtonBuilder<D>, cx: &mut Context<Self>) -> Self {
        let motion = builder.motion_profile();
        let id = builder.id.clone();
        let data = Arc::new(builder.data);
        let content = builder.content;
        let content_id = id.clone();
        let content_data_for_panel = data.clone();
        let panel = PopoverPanel::new(format!("{id}-panel"))
            .content(move |_, window, cx| {
                let model = PopoverRenderModel { id: &content_id, data: &*content_data_for_panel, open: true };
                (content)(&model, window, cx)
            })
            .placement(match builder.placement {
                PopoverPlacement::Smart => PopoverPanelPlacement::SmartStart,
                PopoverPlacement::BelowStart => PopoverPanelPlacement::BelowStart,
                PopoverPlacement::BelowCenter => PopoverPanelPlacement::BelowCenter,
                PopoverPlacement::AboveStart => PopoverPanelPlacement::AboveStart,
            })
            .dismiss_policy(match builder.dismiss_policy {
                PopoverDismissPolicy::CloseOnClickAway => PopoverPanelDismissPolicy::CloseOnClickAway,
                PopoverDismissPolicy::CloseOnFocusLoss => PopoverPanelDismissPolicy::CloseOnFocusLoss,
                PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss => {
                    PopoverPanelDismissPolicy::CloseOnClickAwayOrFocusLoss
                }
                PopoverDismissPolicy::KeepOpen => PopoverPanelDismissPolicy::KeepOpen,
            })
            .offset_y(builder.offset_y)
            .window_margin(builder.window_margin)
            .focus_on_open(builder.focus_on_open)
            .motion_enabled(builder.motion_enabled)
            .motion_defaults(motion)
            .look(builder.panel_look);
        let panel = if let Some(size) = builder.initial_content_size {
            panel.initial_content_size(size).spawn(cx)
        } else {
            panel.spawn(cx)
        };
        let measure_trigger = builder.measure_trigger;
        cx.subscribe(&panel, move |_popover, _, event: &PopoverPanelEvent, cx| {
            if let PopoverPanelEvent::OpenChanged { open } = event {
                cx.emit(PopoverButtonEvent::OpenChanged { open: *open });
            }
        })
        .detach();
        Self {
            id,
            data,
            trigger: builder.trigger,
            panel,
            trigger_on_pointer_down: builder.trigger_on_pointer_down,
            measure_trigger,
            explicit_offset_y: builder.explicit_offset_y,
            explicit_window_margin: builder.explicit_window_margin,
        }
    }
    /// Rebind where the popover opens (a look's `popover.lss`), e.g. after a theme change. Values set
    /// explicitly on the builder are kept.
    pub fn set_geometry(&mut self, geometry: crate::theme::stylesheet::PopoverGeometry, cx: &mut Context<Self>) {
        let (offset_y, window_margin) = (!self.explicit_offset_y, !self.explicit_window_margin);
        self.panel.update(cx, |panel, cx| {
            if offset_y {
                panel.set_offset_y(gpui::px(geometry.button_offset_y), cx);
            }
            if window_margin {
                panel.set_window_margin(gpui::px(geometry.window_margin), cx);
            }
        });
    }
    /// Rebind the panel drawn around the content (a look's `popover.lss` `panel` part).
    pub fn set_panel_look(&mut self, look: super::panel::PopoverPanelLook, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.set_look(look, cx));
    }
    pub fn is_open(&self, cx: &App) -> bool {
        self.panel.read(cx).is_open()
    }
    /// Open without a pointer-dismissal guard, for programmatic and hover entry.
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.open(cx));
    }
    pub fn toggle_guarded(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.toggle_guarded(cx));
    }
    pub fn toggle_guarded_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.toggle_guarded_from(opener, cx));
    }
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.dismiss(cx));
    }
    pub fn set_anchor_bounds(&mut self, bounds: gpui::Bounds<Pixels>, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.set_anchor_bounds(bounds, cx));
    }
}

impl<D: 'static> Render for PopoverButton<D> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = PopoverRenderModel { id: &self.id, data: &*self.data, open: self.panel.read(cx).is_open() };
        let trigger = (self.trigger)(&model, cx);
        let panel_for_bounds = self.panel.clone();
        let panel_for_trigger = self.panel.clone();
        let trigger = if self.trigger_on_pointer_down {
            div()
                .id((self.id.clone(), 0usize))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    panel_for_trigger.update(cx, |panel, cx| panel.toggle_guarded(cx));
                })
                .child(trigger)
        } else {
            div().id((self.id.clone(), 0usize)).child(trigger)
        };
        let mut root = div().relative().child(trigger);
        if self.measure_trigger {
            root = root.on_children_prepainted(move |bounds, _, cx| {
                if let Some(bounds) = bounds.first() {
                    panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(*bounds, cx));
                }
            });
        }
        root = root.child(self.panel.clone());
        root
    }
}

#[cfg(all(test, feature = "test-support"))]
mod panel_look_tests {
    use gpui::{Context, Entity, Render, TestAppContext, Window, div, px};

    use super::*;
    use crate::controls::popover_button::PopoverPanelLook;

    struct Host(Entity<PopoverButton<()>>);

    impl Host {
        fn new(look: Option<PopoverPanelLook>, cx: &mut Context<Self>) -> Self {
            let mut builder = PopoverButton::<()>::new("popover")
                .trigger(|_, _| div().w(px(80.)).h(px(20.)).into_any_element())
                .content(|_, _, _| div().w(px(100.)).h(px(40.)).into_any_element())
                .motion_enabled(false);
            if let Some(look) = look {
                builder = builder.panel_look(look);
            }
            Self(builder.spawn(cx))
        }
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(self.0.clone())
        }
    }

    fn open(
        app: &mut TestAppContext,
        look: Option<PopoverPanelLook>,
    ) -> (Entity<PopoverButton<()>>, &mut gpui::VisualTestContext) {
        let (host, cx) = app.add_window_view(|_, cx| Host::new(look, cx));
        let button = cx.update(|_, cx| host.read(cx).0.clone());
        button.update(cx, |button, cx| button.open(cx));
        cx.run_until_parked();
        (button, cx)
    }

    /// The SDK popover draws no panel; turned on, it draws the part's fill around border and padding (#397).
    #[test]
    fn the_panel_is_off_in_the_sdk_and_drawn_when_shown() {
        let mut app = TestAppContext::single();
        let (_, cx) = open(&mut app, None);
        assert!(cx.debug_bounds("popover-panel").is_none(), "no panel in the SDK");

        let look = PopoverPanelLook::sdk().shown(true);
        let mut app = TestAppContext::single();
        let (button, cx) = open(&mut app, Some(look.clone()));
        let panel = cx.debug_bounds("popover-panel").expect("a shown panel is drawn");
        let width = 100. + 2. * look.padding + 2. * look.border_width;
        assert_eq!(panel.size.width, px(width), "content, padding and border");
        let filled = cx.update(|window, _| {
            window.painted_quads().iter().any(|quad| quad.background == gpui::Background::from(look.background))
        });
        assert!(filled, "the panel's fill is painted");

        // Rebinding to the SDK's look turns the panel off again.
        button.update(cx, |button, cx| button.set_panel_look(PopoverPanelLook::sdk(), cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("popover-panel").is_none(), "rebinding turns it off");
    }
}
