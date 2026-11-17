use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyWindowHandle, AnyTooltip, AppContext, AvailableSpace, Bounds, Context, Div, Entity, HitboxBehavior, IntoElement,
    App, WeakEntity, MouseDownEvent, MouseMoveEvent, PathBuilder, Pixels, Point, Render, ScrollEvent, Size, Stateful,
    Subscription, Task, Window, canvas, point, prelude::*, px,
};
use crate::theme::InteractionState;
use crate::infra::attachments::AttachmentTarget;
use super::{TooltipSettings, Tooltip, TooltipTheme, TooltipRenderModel, TooltipPlacement, TooltipDismissal, TooltipEvent};

#[derive(Clone)]
pub(crate) struct Attachment {
    controller: Entity<Controller>,
}

impl Attachment {
    pub(crate) fn new<T: AttachmentTarget>(
        config: Tooltip,
        theme: Arc<dyn TooltipTheme>,
        owner: &Entity<T>,
        cx: &mut App,
    ) -> Self {
        Self {
            controller: cx.new(|cx| Controller {
                _settings: cx.observe_global::<TooltipSettings>(|tip: &mut Controller, cx| {
                    if !tip.config.always_enabled && !cx.global::<TooltipSettings>().enabled {
                        tip.reset_trigger();
                    }
                    cx.notify();
                }),
                config,
                theme,
                window: None,
                _presence_release: None,
                _window_activation: None,
                _window_visibility: None,
                state: Activation::default(),
                target: Bounds::default(),
                bubble: Bounds::default(),
                owner_bounds: Bounds::default(),
                pending: None,
                presented: false,
                expiry: None,
                _owner_release: cx.observe_release(owner, |tip: &mut Controller, _, cx| {
                    tip.retire();
                    cx.notify();
                }),
                _keystrokes: cx.observe_keystrokes(|tip: &mut Controller, event, window, _| {
                    if tip.window != Some(window.window_handle()) {
                        return;
                    }
                    if matches!(event.keystroke.key.as_str(), "escape" | "enter" | "space") {
                        if event.keystroke.key == "escape"
                            && tip.config.dismissal == TooltipDismissal::Permanently
                            && (tip.state.visible || tip.state.waiting.is_some())
                        {
                            tip.config.enabled = false;
                        }
                        tip.dismiss(window);
                    }
                }),
            }),
        }
    }

    pub(crate) fn description(&self, cx: &App) -> Option<gpui::SharedString> {
        let config = &self.controller.read(cx).config;
        let text = config.text_provider.as_ref().map_or_else(|| config.text.clone(), |provider| provider(cx));
        if !config.enabled || text.is_empty() {
            return None;
        }
        Some(match config.shortcut.as_ref().filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{} ({})", text, hint).into(),
            None => text,
        })
    }

    pub(crate) fn dismiss(&self, cx: &mut App) {
        self.controller.update(cx, |tip, cx| {
            tip.state.dismiss();
            tip.hidden();
            tip.pending = None;
            cx.notify();
        });
    }

    pub(crate) fn retire(&self, cx: &mut App) {
        self.controller.update(cx, |tip, cx| {
            tip.retire();
            cx.notify();
        });
    }

    /// Same modifier for SDK buttons and sliders. Adds no layout wrapper or focus stop.
    pub(crate) fn attach<T: AttachmentTarget>(
        &self,
        root: Stateful<Div>,
        interaction: InteractionState,
        owner: WeakEntity<T>,
    ) -> Stateful<Div> {
        let controller = self.controller.clone();
        let painted_controller = controller.clone();
        root.relative().child(
            canvas(
                move |bounds, window, cx| {
                    // GPUI releases keyed state when its element leaves the rendered tree,
                    // even if the owning control entity remains cached.
                    let presence =
                        window.use_keyed_state(format!("tooltip-presence-{:?}", controller.entity_id()), cx, |_, _| ());
                    controller.update(cx, |tip, cx| {
                        tip.bind_window(window, cx);
                        if tip._presence_release.is_none() {
                            tip._presence_release = Some(cx.observe_release(&presence, |tip, _, cx| {
                                tip.reset_trigger();
                                tip._presence_release = None;
                                cx.notify();
                            }));
                        }
                        let moved = tip.owner_bounds != bounds && tip.owner_bounds.size != Size::default();
                        tip.owner_bounds = bounds;
                        tip.target =
                            owner.upgrade().map(|entity| entity.read(cx).attachment_anchor(bounds)).unwrap_or(bounds);
                        if moved && (tip.state.visible || tip.state.waiting.is_some()) {
                            tip.dismiss(window);
                        }
                        let focused = interaction.focused && (window.last_input_was_keyboard() || tip.state.focused);
                        let hovered =
                            interaction.hovered || (tip.state.visible && tip.bubble.contains(&window.mouse_position()));
                        tip.sync(hovered, focused, !interaction.disabled, window, cx);
                        if tip.state.visible {
                            let weak = cx.weak_entity();
                            window.set_tooltip(AnyTooltip {
                                view: cx.entity().into(),
                                mouse_position: window.mouse_position(),
                                check_visible_and_update: Rc::new(move |_, _, cx| {
                                    weak.upgrade().is_some_and(|entity| entity.read(cx).state.visible)
                                }),
                            });
                        }
                    });
                    window.insert_hitbox(bounds, HitboxBehavior::Normal)
                },
                move |_, hitbox, window, _| {
                    let weak = painted_controller.downgrade();
                    window.on_mouse_event(move |_: &MouseMoveEvent, phase, window, cx| {
                        if !phase.bubble() {
                            return;
                        }
                        if let Some(entity) = weak.upgrade() {
                            entity.update(cx, |tip, cx| {
                                let pointer = window.mouse_position();
                                let hovered =
                                    hitbox.is_hovered(window) || (tip.state.visible && tip.bubble.contains(&pointer));
                                tip.sync(hovered, tip.state.focused, tip.state.enabled, window, cx);
                            });
                        }
                    });
                    let weak = painted_controller.downgrade();
                    window.on_mouse_event(move |_: &MouseDownEvent, phase, window, cx| {
                        if phase.capture()
                            && let Some(entity) = weak.upgrade()
                        {
                            entity.update(cx, |tip, _| tip.dismiss(window));
                        }
                    });
                    let weak = painted_controller.downgrade();
                    window.on_mouse_event(move |_: &ScrollEvent, phase, window, cx| {
                        if phase.capture()
                            && let Some(entity) = weak.upgrade()
                        {
                            entity.update(cx, |tip, _| tip.dismiss(window));
                        }
                    });
                },
            )
            .absolute()
            .inset_0()
            .size_full(),
        )
    }
}

#[derive(Default)]
struct Activation {
    hovered: bool,
    focused: bool,
    enabled: bool,
    visible: bool,
    suppressed: bool,
    shown: bool,
    waiting: Option<bool>, // true = open timer, false = hide timer
}

#[derive(Debug, PartialEq)]
enum Transition {
    None,
    Cancel,
    Open,
    Wait(bool),
}

impl Activation {
    fn can_open(&self, once: bool) -> bool {
        !once || !self.shown || self.visible
    }

    fn sync(&mut self, hovered: bool, focused: bool, enabled: bool) -> Transition {
        self.hovered = hovered;
        self.focused = focused;
        self.enabled = enabled;
        if !hovered && !focused {
            self.suppressed = false;
        }
        if !enabled || self.suppressed {
            self.visible = false;
            self.waiting = None;
            return Transition::Cancel;
        }
        if focused {
            self.visible = true;
            self.waiting = None;
            return Transition::Open;
        }
        if hovered {
            if self.visible {
                self.waiting = None;
                Transition::Cancel
            } else if self.waiting == Some(true) {
                Transition::None
            } else {
                self.waiting = Some(true);
                Transition::Wait(true)
            }
        } else if self.visible {
            if self.waiting == Some(false) {
                Transition::None
            } else {
                self.waiting = Some(false);
                Transition::Wait(false)
            }
        } else {
            self.waiting = None;
            Transition::Cancel
        }
    }

    fn dismiss(&mut self) {
        self.visible = false;
        self.suppressed = self.hovered || self.focused;
        self.waiting = None;
    }

    fn elapsed(&mut self, open: bool) {
        if self.waiting != Some(open) {
            return;
        }
        self.waiting = None;
        self.visible = open && self.enabled && !self.suppressed && (self.hovered || self.focused);
    }
}

struct Controller {
    config: Tooltip,
    theme: Arc<dyn TooltipTheme>,
    window: Option<AnyWindowHandle>,
    _presence_release: Option<Subscription>,
    _window_activation: Option<Subscription>,
    _window_visibility: Option<Subscription>,
    owner_bounds: Bounds<Pixels>,
    state: Activation,
    target: Bounds<Pixels>,
    bubble: Bounds<Pixels>,
    pending: Option<Task<()>>,
    presented: bool,
    expiry: Option<Task<()>>,
    _settings: Subscription,
    _keystrokes: Subscription,
    _owner_release: Subscription,
}

impl Controller {
    fn reset_trigger(&mut self) {
        self.state.dismiss();
        self.state.hovered = false;
        self.state.focused = false;
        self.state.suppressed = false;
        self.hidden();
        self.pending = None;
        self.owner_bounds = Bounds::default();
    }

    fn bind_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.window == Some(window.window_handle()) {
            return;
        }
        self.reset_trigger();
        self.window = Some(window.window_handle());
        self._presence_release = None;
        self._window_activation = Some(cx.observe_window_activation(window, |tip, window, _| {
            if !window.is_window_active() {
                tip.reset_trigger();
                window.refresh();
            }
        }));
        self._window_visibility = Some(cx.observe_window_visibility(window, |tip, visibility, window, _| {
            if !visibility.is_visible() {
                tip.reset_trigger();
                window.refresh();
            }
        }));
    }

    fn hidden(&mut self) {
        self.expiry = None;
        if self.presented {
            self.presented = false;
            if let Some(callback) = &self.config.on_event {
                callback(TooltipEvent::Hidden);
            }
        }
    }

    fn retire(&mut self) {
        self.config.enabled = false;
        self.state.dismiss();
        self.hidden();
        self.pending = None;
    }

    fn dismiss(&mut self, window: &mut Window) {
        let was_active = self.state.visible || self.state.waiting.is_some();
        self.state.dismiss();
        self.hidden();
        self.pending = None;
        if was_active {
            window.refresh();
        }
    }

    fn sync(&mut self, hovered: bool, focused: bool, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        // Keyboard input can clear GPUI's hover flag while the pointer remains
        // over the owner. Escape suppression must survive that modality change.
        let hovered = hovered || (self.state.suppressed && self.owner_bounds.contains(&window.mouse_position()));
        let enabled = enabled
            && (self.config.always_enabled
                || cx.try_global::<TooltipSettings>().is_none_or(|settings| settings.enabled))
            && window.is_window_active()
            && window.is_visible();
        let previous = self.state.visible;
        let can_open = self.state.can_open(self.config.once);
        let transition = self.state.sync(
            hovered,
            focused,
            enabled
                && self.config.enabled
                && can_open
                && !self
                    .config
                    .text_provider
                    .as_ref()
                    .map_or_else(|| self.config.text.clone(), |provider| provider(cx))
                    .is_empty(),
        );
        match transition {
            Transition::None => {}
            Transition::Cancel | Transition::Open => self.pending = None,
            Transition::Wait(open) => {
                let delay = if open {
                    self.config.delay
                } else {
                    self.config.hide_delay
                };
                self.pending = Some(cx.spawn_in(window, async move |weak, cx| {
                    cx.background_executor().timer(delay).await;
                    let _ = weak.update_in(cx, |tip, window, _| {
                        tip.state.elapsed(open);
                        if !tip.state.visible {
                            tip.hidden();
                        }
                        window.refresh();
                    });
                }));
            }
        }
        if !self.state.visible {
            self.hidden();
        }
        if previous != self.state.visible {
            window.refresh();
        }
    }

    fn render_model(&self, cx: &App) -> TooltipRenderModel {
        let look = self.config.theme.as_ref().unwrap_or(&self.theme).resolve();
        TooltipRenderModel {
            direction: crate::infra::locale::LocaleContext::current(cx).direction,
            placement: look.placement,
            shortcut_gap: look.shortcut_gap,
            shortcut_font_size: look.shortcut_font_size,
            shortcut_opacity: look.shortcut_opacity,
            text: self.config.text_provider.as_ref().map_or_else(|| self.config.text.clone(), |provider| provider(cx)),
            shortcut: self.config.shortcut.clone(),
            background: look.background,
            foreground: look.foreground,
            padding: look.padding,
            padding_y: look.padding_y,
            radius: look.radius,
            max_width: look.max_width,
            text_size: look.text_size,
            line_height: look.line_height,
            shadow: look.shadow,
        }
    }
}

#[derive(Debug)]
struct Placement {
    origin: Point<Pixels>,
    above: bool,
    pointer_x: Pixels,
}

fn placement(
    target: Bounds<Pixels>,
    size: Size<Pixels>,
    viewport: Size<Pixels>,
    preferred: TooltipPlacement,
    geometry: super::TooltipPlacementGeometry,
) -> Placement {
    let center = target.origin.x + target.size.width / 2.0;
    let x = (center - size.width / 2.0)
        .max(px(geometry.margin))
        .min((viewport.width - size.width - px(geometry.margin)).max(px(geometry.margin)));
    let above_y = target.origin.y - size.height - px(geometry.gap);
    let below_fits = target.bottom() + px(geometry.gap) + size.height <= viewport.height - px(geometry.margin);
    let above = match preferred {
        TooltipPlacement::Above | TooltipPlacement::AboveStart | TooltipPlacement::AboveEnd => {
            above_y >= px(geometry.margin) || !below_fits
        }
        TooltipPlacement::Below | TooltipPlacement::BelowStart | TooltipPlacement::BelowEnd => {
            !below_fits && above_y >= px(geometry.margin)
        }
    };
    let y = (if above {
        above_y
    } else {
        target.bottom() + px(geometry.gap)
    })
    .max(px(geometry.margin))
    .min((viewport.height - size.height - px(geometry.margin)).max(px(geometry.margin)));
    Placement {
        origin: point(x, y),
        above,
        pointer_x: center.max(x + px(geometry.pointer_inset)).min(x + size.width - px(geometry.pointer_inset)),
    }
}

fn logical_placement(
    target: Bounds<Pixels>,
    size: Size<Pixels>,
    viewport: Size<Pixels>,
    preferred: TooltipPlacement,
    geometry: super::TooltipPlacementGeometry,
    direction: crate::infra::locale::LayoutDirection,
) -> Placement {
    let mut position = placement(target, size, viewport, preferred, geometry);
    let start = matches!(preferred, TooltipPlacement::AboveStart | TooltipPlacement::BelowStart);
    let end = matches!(preferred, TooltipPlacement::AboveEnd | TooltipPlacement::BelowEnd);
    if start || end {
        let align_left = start != (direction == crate::infra::locale::LayoutDirection::Rtl);
        position.origin.x = (if align_left {
            target.left()
        } else {
            target.right() - size.width
        })
        .max(px(geometry.margin))
        .min((viewport.width - size.width - px(geometry.margin)).max(px(geometry.margin)));
        position.pointer_x = target
            .center()
            .x
            .max(position.origin.x + px(geometry.pointer_inset))
            .min(position.origin.x + size.width - px(geometry.pointer_inset));
    }
    position
}

impl Render for Controller {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut model = self.render_model(cx);
        model.max_width = model
            .max_width
            .min((f32::from(window.viewport_size().width) - 2.0 * model.placement.margin).max(1.0));
        let template = self.config.template.clone();
        let target = self.target;
        let preferred = self.config.placement;
        let weak = cx.weak_entity();
        canvas(
            move |_, window, cx| {
                let mut bubble = template(&model);
                let size = bubble.layout_as_root(AvailableSpace::min_size(), window, cx);
                let position = logical_placement(
                    target,
                    size,
                    window.viewport_size(),
                    preferred,
                    model.placement,
                    model.direction,
                );
                if let Some(entity) = weak.upgrade() {
                    entity.update(cx, |tip, cx| {
                        tip.bubble = Bounds::new(position.origin, size);
                        tip.state.shown = true;
                        if !tip.presented {
                            tip.presented = true;
                            if let Some(duration) = tip.config.duration {
                                tip.expiry = Some(cx.spawn_in(window, async move |weak, cx| {
                                    cx.background_executor().timer(duration).await;
                                    let _ = weak.update_in(cx, |tip, window, _| tip.dismiss(window));
                                }));
                            }
                            if let Some(callback) = &tip.config.on_event {
                                callback(TooltipEvent::Shown);
                            }
                        }
                    });
                }
                window.with_absolute_element_offset(position.origin, |window| bubble.prepaint(window, cx));
                (bubble, position, size, model.background, model.placement.pointer_size)
            },
            |_, (mut bubble, position, size, background, pointer_size), window, cx| {
                bubble.paint(window, cx);
                let base = if position.above {
                    position.origin.y + size.height
                } else {
                    position.origin.y
                };
                let tip = base + px(if position.above { pointer_size } else { -pointer_size });
                let mut path = PathBuilder::fill();
                path.move_to(point(position.pointer_x - px(pointer_size), base));
                path.line_to(point(position.pointer_x, tip));
                path.line_to(point(position.pointer_x + px(pointer_size), base));
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, background);
                }
            },
        )
        .size_0()
    }
}

#[cfg(test)]
mod tests {
    const GAP: f32 = 10.0;
    const MARGIN: f32 = 8.0;
    use super::*;

    #[test]
    fn hover_delay_cancel_and_reentry() {
        let mut state = Activation::default();
        assert_eq!(state.sync(true, false, true), Transition::Wait(true));
        assert!(!state.visible);
        assert_eq!(state.sync(false, false, true), Transition::Cancel);
        state.elapsed(true);
        assert!(!state.visible);
        state.sync(true, false, true);
        state.elapsed(true);
        assert!(state.visible);
        assert_eq!(state.sync(false, false, true), Transition::Wait(false));
        state.sync(true, false, true);
        state.elapsed(false);
        assert!(state.visible);
    }

    #[test]
    fn focus_dismissal_requires_fresh_condition() {
        let mut state = Activation::default();
        assert_eq!(state.sync(false, true, true), Transition::Open);
        state.dismiss();
        state.sync(true, true, true);
        assert!(!state.visible);
        state.sync(false, false, true);
        state.sync(false, true, true);
        assert!(state.visible);
        state.sync(false, true, false);
        assert!(!state.visible);
    }

    #[test]
    fn dismissal_cancels_pending_open() {
        let mut state = Activation::default();
        state.sync(true, false, true);
        state.dismiss();
        state.elapsed(true);
        assert!(!state.visible);
    }

    #[test]
    fn placement_clears_control_and_flips_at_top() {
        let size = Size { width: px(100.0), height: px(36.0) };
        let viewport = Size { width: px(500.0), height: px(400.0) };
        let target = Bounds::new(point(px(200.0), px(100.0)), Size { width: px(32.0), height: px(32.0) });
        let position = placement(target, size, viewport, TooltipPlacement::Above, Default::default());
        assert!(position.above);
        assert_eq!(position.origin.x, px(166.0));
        assert_eq!(position.origin.y + size.height, target.origin.y - px(GAP));
        let top = Bounds::new(point(px(0.0), px(0.0)), target.size);
        let position = placement(top, size, viewport, TooltipPlacement::Above, Default::default());
        assert!(!position.above);
        assert_eq!(position.origin.y, top.bottom() + px(GAP));
        assert_eq!(position.origin.x, px(MARGIN));
    }

    #[test]
    fn disabled_and_blur_do_not_leave_a_tooltip_open() {
        let mut state = Activation::default();
        state.sync(true, false, false);
        state.elapsed(true);
        assert!(!state.visible);
        state.sync(false, true, true);
        assert!(state.visible);
        state.sync(false, false, true);
        state.elapsed(false);
        assert!(!state.visible);
    }

    #[test]
    fn long_bubble_clamps_horizontally_and_pointer_tracks_target() {
        let size = Size { width: px(240.0), height: px(60.0) };
        let viewport = Size { width: px(500.0), height: px(400.0) };
        let target = Bounds::new(point(px(460.0), px(340.0)), Size { width: px(32.0), height: px(32.0) });
        let position = placement(target, size, viewport, TooltipPlacement::Above, Default::default());
        assert_eq!(position.origin.x + size.width, viewport.width - px(MARGIN));
        assert!(position.above);
        assert_eq!(position.pointer_x, target.origin.x + target.size.width / 2.0);
    }

    #[test]
    fn below_preference_flips_and_tall_bubbles_stay_in_viewport() {
        let viewport = Size { width: px(500.0), height: px(400.0) };
        let size = Size { width: px(100.0), height: px(36.0) };
        let target = Bounds::new(point(px(200.0), px(100.0)), Size { width: px(32.0), height: px(32.0) });
        let below = placement(target, size, viewport, TooltipPlacement::Below, Default::default());
        assert!(!below.above);
        assert_eq!(below.origin.y, target.bottom() + px(GAP));
        let bottom = Bounds::new(point(px(200.0), px(360.0)), target.size);
        assert!(placement(bottom, size, viewport, TooltipPlacement::Below, Default::default()).above);
        let tall = Size { width: px(100.0), height: px(350.0) };
        let position = placement(target, tall, viewport, TooltipPlacement::Above, Default::default());
        assert!(position.origin.y >= px(MARGIN));
        assert!(position.origin.y + tall.height <= viewport.height - px(MARGIN));
    }

    #[test]
    fn once_policy_counts_rendering_not_cancelled_activation() {
        let mut state = Activation::default();
        state.sync(true, false, true);
        state.dismiss();
        assert!(state.can_open(true));
        state.sync(false, false, true);
        state.sync(true, false, true);
        state.elapsed(true);
        assert!(state.can_open(true));
        state.shown = true;
        assert!(state.can_open(true)); // current presentation may stay open
        state.dismiss();
        assert!(!state.can_open(true));
        assert!(state.can_open(false));
    }
}

#[cfg(all(test, feature = "test-support"))]
mod ownership_tests {
    use super::*;
    use gpui::TestAppContext;
    use crate::controls::{button::Button, tooltip::default_tooltip_theme};

    #[test]
    fn global_disable_dismisses_help_but_preserves_always_enabled_attachment() {
        let app = TestAppContext::single();
        let (_owner, regular, always) = app.update(|cx| {
            let owner = cx.new(|cx| Button::from_builder(Button::new("settings-owner"), cx));
            let regular = Attachment::new(Tooltip::new("Help"), default_tooltip_theme(), &owner, cx);
            let always = Attachment::new(Tooltip::new("Toggle").always_enabled(), default_tooltip_theme(), &owner, cx);
            for attachment in [&regular, &always] {
                attachment.controller.update(cx, |tip, _| {
                    tip.state.visible = true;
                    tip.state.enabled = true;
                    tip.state.hovered = true;
                });
            }
            (owner, regular, always)
        });
        app.update(|cx| cx.set_global(TooltipSettings { enabled: false }));
        app.update(|cx| {
            assert!(!regular.controller.read(cx).state.visible);
            assert!(always.controller.read(cx).state.visible);
            assert!(regular.controller.read(cx).config.enabled);
        });
        app.update(|cx| cx.set_global(TooltipSettings { enabled: true }));
    }

    #[test]
    fn live_text_provider_updates_bubble_and_description_without_resetting_dismissal() {
        struct Help(gpui::SharedString);
        impl gpui::Global for Help {}
        let app = TestAppContext::single();
        let (_owner, tip) = app.update(|cx| {
            cx.set_global(Help("Save".into()));
            let owner = cx.new(|cx| Button::from_builder(Button::new("localized-owner"), cx));
            let config = Tooltip::new("Save").shortcut("⌘S").text_provider(|cx| cx.global::<Help>().0.clone());
            let tip = Attachment::new(config, default_tooltip_theme(), &owner, cx);
            tip.controller.update(cx, |tip, _| {
                tip.state.suppressed = true;
            });
            (owner, tip)
        });
        for text in ["Speichern", "保存", "حفظ"] {
            app.update(|cx| {
                cx.set_global(Help(text.into()));
                assert_eq!(tip.description(cx).unwrap(), format!("{text} (⌘S)"));
                assert_eq!(tip.controller.read(cx).render_model(cx).text, text);
                assert!(tip.controller.read(cx).state.suppressed);
            });
        }
    }

    #[test]
    fn owner_release_retires_retained_tooltip() {
        let app = TestAppContext::single();
        let (owner, tip) = app.update(|cx| {
            let owner = cx.new(|cx| Button::from_builder(Button::new("owner"), cx));
            let tip = Attachment::new(Tooltip::new("Help"), default_tooltip_theme(), &owner, cx);
            (owner, tip)
        });
        drop(owner);
        app.update(|_| {}); // GPUI processes releases at the end of an update.
        app.update(|cx| {
            assert!(!tip.controller.read(cx).config.enabled);
            assert!(!tip.controller.read(cx).state.visible);
            assert!(tip.controller.read(cx).pending.is_none());
        });
    }
}

#[test]
fn tooltip_logical_edges_mirror_and_keep_viewport_fallback() {
    use crate::infra::locale::LayoutDirection;
    let target = Bounds::new(point(px(200.), px(100.)), gpui::size(px(80.), px(30.)));
    let viewport = gpui::size(px(800.), px(600.));
    for (direction, start_x, end_x) in [(LayoutDirection::Ltr, 200., 160.), (LayoutDirection::Rtl, 160., 200.)] {
        let start = logical_placement(
            target,
            gpui::size(px(120.), px(40.)),
            viewport,
            TooltipPlacement::BelowStart,
            Default::default(),
            direction,
        );
        let end = logical_placement(
            target,
            gpui::size(px(120.), px(40.)),
            viewport,
            TooltipPlacement::BelowEnd,
            Default::default(),
            direction,
        );
        assert_eq!(start.origin.x, px(start_x));
        assert_eq!(end.origin.x, px(end_x));
        assert!(!start.above);
        let near_top = Bounds::new(point(px(1.), px(1.)), target.size);
        let flipped = logical_placement(
            near_top,
            gpui::size(px(120.), px(40.)),
            viewport,
            TooltipPlacement::AboveStart,
            Default::default(),
            direction,
        );
        assert!(!flipped.above);
        assert!(flipped.origin.x >= px(super::TooltipPlacementGeometry::default().margin));
    }
}
