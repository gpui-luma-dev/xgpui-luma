//! Look-owned popup-menu builder. Spawn synthesizes the SDK [`xgpui_luma::controls::popup_menu::PopupMenu`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use xgpui_luma::controls::button::ControlIcon;
use xgpui_luma::controls::popup_menu::{
    PopupMenuBuilder, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplateModifier, PopupMenuTriggerModel,
    PopupMenuTriggerStyle,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::infra::presenter::{ControlPresenter, HasPresenter};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

enum TriggerFace {
    Default,
    Label(SharedString),
    Icon(ControlIcon),
    Content(ControlPresenter<PopupMenuTriggerModel>),
}

/// Builder in the guise of a popup menu: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct PopupMenu {
    motion_enabled: bool,
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::popup_menu::PopupMenuMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::popup_menu::PopupMenuEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    size: ShadcnSize,
    menu_size: ShadcnSize,
    items: Vec<MenuItem>,
    enabled: bool,
    tab_stop: bool,
    placement: PopupMenuPlacement,
    trigger_style: PopupMenuTriggerStyle,
    full_width: bool,
    without_elevation: bool,
    split: bool,
    end_icon: Option<ControlIcon>,
    disclosure_icons: Option<DisclosureIcons>,
    face: TriggerFace,
    modifiers: Vec<PopupMenuTemplateModifier>,
}

impl PopupMenu {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::popup_menu::PopupMenuEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `PopupMenuEvent::ActionClick` through the existing event stream.
    pub fn on_action_click<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::ActionClick = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `PopupMenuEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::popup_menu::PopupMenuEvent::EnabledChanged { enabled } = event {
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
            size: ShadcnSize::Md,
            menu_size: ShadcnSize::Md,
            items: Vec::new(),
            enabled: true,
            tab_stop: true,
            placement: PopupMenuPlacement::default(),
            trigger_style: PopupMenuTriggerStyle::default(),
            full_width: false,
            without_elevation: false,
            split: false,
            end_icon: None,
            disclosure_icons: None,
            face: TriggerFace::Default,
            modifiers: Vec::new(),
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::popup_menu::PopupMenuMotion, edits motion);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.motion_enabled = enabled;
        self
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn menu_size(mut self, size: ShadcnSize) -> Self {
        self.menu_size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.face = TriggerFace::Label(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.face = TriggerFace::Icon(icon.into());
        self
    }

    pub fn end_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.end_icon = Some(icon.into());
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.disclosure_icons = Some(icons);
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

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn placement(mut self, placement: PopupMenuPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn trigger_style(mut self, style: PopupMenuTriggerStyle) -> Self {
        self.trigger_style = style;
        self
    }

    pub fn primary(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Primary)
    }

    pub fn secondary(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Secondary)
    }

    pub fn outline(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Outline)
    }

    pub fn ghost(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Ghost)
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.without_elevation = true;
        self
    }

    pub fn split(mut self) -> Self {
        self.split = true;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::popup_menu::PopupMenu> {
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

    pub(crate) fn into_sdk_builder(self, look: ShadcnLook) -> PopupMenuBuilder {
        let template = look.popup_menu_template();
        let mut builder = xgpui_luma::controls::popup_menu::PopupMenu::new(self.id)
            .motion_defaults(look.motion_defaults().popup_menu)
            .with_motion_overrides(self.motion)
            .motion_enabled(self.motion_enabled)
            .template(template)
            .size(self.size.control_size())
            .menu_size(self.menu_size.control_size())
            .items(self.items)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .placement(self.placement)
            .trigger_style(self.trigger_style)
            .full_width(self.full_width);
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if self.split {
            builder = builder.split();
        }
        if let Some(ControlIcon::Lucide(icon)) = self.end_icon {
            builder = builder.end_icon(icon);
        }
        builder = builder.disclosure_icons(DisclosureIcons::menu_from_stylesheet(&look.stylesheet().common));
        if let Some(icons) = self.disclosure_icons {
            builder = builder.disclosure_icons(icons);
        }
        match self.face {
            TriggerFace::Default => {}
            TriggerFace::Label(label) => builder = builder.label(label),
            TriggerFace::Icon(icon) => builder = builder.icon(icon),
            TriggerFace::Content(content) => HasPresenter::set_presenter(&mut builder, content),
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

impl HasPresenter<PopupMenuTriggerModel> for PopupMenu {
    fn set_presenter(&mut self, content: ControlPresenter<PopupMenuTriggerModel>) {
        self.face = TriggerFace::Content(content);
    }
}

/// Look-owned builder for the retained SDK SplitButton specialization.
pub struct SplitButton {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::split_button::SplitButtonEvent>,

    inner: PopupMenu,
    primary_enabled: bool,
    modifiers: Vec<PopupMenuTemplateModifier>,
}

impl SplitButton {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::split_button::SplitButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SplitButtonEvent::ActionClick` through the existing event stream.
    pub fn on_action_click<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::ActionClick = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Select` through the existing event stream.
    pub fn on_select<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Dismiss` through the existing event stream.
    pub fn on_dismiss<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::split_button::SplitButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),
            inner: PopupMenu::new(id),
            primary_enabled: true,
            modifiers: Vec::new(),
        }
    }

    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.inner = self.inner.look(look);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn menu_size(mut self, size: ShadcnSize) -> Self {
        self.inner = self.inner.menu_size(size);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(label);
        self
    }

    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }

    pub fn end_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.inner = self.inner.end_icon(icon);
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.inner = self.inner.disclosure_icons(icons);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.inner = self.inner.items(items);
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.inner = self.inner.item(item);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.inner = self.inner.enabled(enabled);
        self
    }

    /// Keep dropdown actions available when the primary action is unavailable.
    pub fn primary_enabled(mut self, enabled: bool) -> Self {
        self.primary_enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.inner = self.inner.tab_stop(tab_stop);
        self
    }

    pub fn placement(mut self, placement: PopupMenuPlacement) -> Self {
        self.inner = self.inner.placement(placement);
        self
    }

    pub fn trigger_style(mut self, style: PopupMenuTriggerStyle) -> Self {
        self.inner = self.inner.trigger_style(style);
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.inner = self.inner.full_width(full_width);
        self
    }

    pub fn primary(mut self) -> Self {
        self.inner = self.inner.primary();
        self
    }

    pub fn secondary(mut self) -> Self {
        self.inner = self.inner.secondary();
        self
    }

    pub fn outline(mut self) -> Self {
        self.inner = self.inner.outline();
        self
    }

    pub fn ghost(mut self) -> Self {
        self.inner = self.inner.ghost();
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.inner = self.inner.without_elevation();
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &PopupMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::split_button::SplitButton> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let look = self.inner.resolve_look(cx);
        let tooltip_theme = crate::tooltip_theme(&look);
        let template = look.split_button_template();
        let mut builder = xgpui_luma::controls::split_button::SplitButtonBuilder::from_popup_builder(
            self.inner.into_sdk_builder(look),
        )
        .primary_enabled(self.primary_enabled)
        .template(template);
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| modifier(root, model));
        }
        use xgpui_luma::infra::attachments::TooltipEntityExt;
        let control = builder.spawn(cx).with_tooltip_theme(tooltip_theme, cx);
        callbacks.bind(&control, cx);
        control
    }
}

impl HasPresenter<PopupMenuTriggerModel> for SplitButton {
    fn set_presenter(&mut self, content: ControlPresenter<PopupMenuTriggerModel>) {
        self.inner.set_presenter(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_primary_keeps_shadcn_axes() {
        let menu = PopupMenu::new("copy").primary().label("Copy");
        assert_eq!(menu.trigger_style, PopupMenuTriggerStyle::Primary);
        assert!(matches!(menu.face, TriggerFace::Label(_)));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = PopupMenu::new("ok")
            .look(&look)
            .ghost()
            .label("Actions")
            .items([MenuItem::new("one").label("One")])
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod split_integration_tests {
    use super::*;
    use gpui::{Focusable, Render, TestAppContext, Window, div, prelude::*};
    use xgpui_luma::controls::split_button::SplitButton as SdkSplitButton;
    use std::sync::{Arc, Mutex};

    struct Preview {
        control: Entity<SdkSplitButton>,
        selected: usize,
        activated: usize,
    }
    impl Render for Preview {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(self.control.clone())
        }
    }

    #[test]
    fn split_builder_preserves_options_modifiers_and_real_semantic_callbacks() {
        let mut app = TestAppContext::single();
        let snapshots = Arc::new(Mutex::new(Vec::new()));
        let capture = snapshots.clone();
        let (preview, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            let control = SplitButton::new("look-split")
                .look(&ShadcnLook::built_in())
                .ghost()
                .size(ShadcnSize::Sm)
                .menu_size(ShadcnSize::Lg)
                .label("Save")
                .primary_enabled(false)
                .full_width(true)
                .items([MenuItem::new("alternate")])
                .with_template_modifier(move |root, model| {
                    capture.lock().unwrap().push((
                        model.split,
                        model.split_state.primary.disabled,
                        model.split_state.disclosure.disabled,
                        model.trigger_style,
                        model.trigger_size,
                        model.menu_size,
                        model.full_width,
                    ));
                    root
                })
                .on_select(cx, |owner: &mut Preview, _, _, _| owner.selected += 1)
                .on_action_click(cx, |owner, _| owner.activated += 1)
                .spawn(cx);
            control.read(cx).focus_handle(cx).focus(window, cx);
            Preview { control, selected: 0, activated: 0 }
        });
        cx.run_until_parked();
        assert!(snapshots.lock().unwrap().iter().all(|snapshot| *snapshot
            == (
                true,
                true,
                false,
                PopupMenuTriggerStyle::Ghost,
                xgpui_luma::theme::ControlSize::Sm,
                xgpui_luma::theme::ControlSize::Lg,
                true
            )));
        assert!(!snapshots.lock().unwrap().is_empty());
        cx.simulate_keystrokes("enter space down enter");
        cx.update(|_, app| {
            assert_eq!(preview.read(app).selected, 1);
            assert_eq!(preview.read(app).activated, 0);
        });
    }
}
