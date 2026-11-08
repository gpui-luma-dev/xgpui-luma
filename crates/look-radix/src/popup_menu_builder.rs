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

use crate::button_layout::ButtonSize;
use crate::look::{Look, resolve_look};
use crate::popup_menu::{PopupMenuVariant, popup_menu_template};
use crate::tone::Tone;

enum TriggerFace {
    Default,
    Label(SharedString),
    Icon(ControlIcon),
    Content(ControlPresenter<PopupMenuTriggerModel>),
}

/// Builder in the guise of a popup menu: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct PopupMenu {
    motion_enabled: bool,
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::popup_menu::PopupMenuMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::popup_menu::PopupMenuEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: PopupMenuVariant,
    tone: Tone,
    size: ButtonSize,
    menu_size: ButtonSize,
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
            variant: PopupMenuVariant::default(),
            tone: Tone::default(),
            size: ButtonSize::default(),
            menu_size: ButtonSize::default(),
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
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: PopupMenuVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn solid(self) -> Self {
        self.variant(PopupMenuVariant::Solid)
    }

    pub fn soft(self) -> Self {
        self.variant(PopupMenuVariant::Soft)
    }

    pub fn accent(mut self) -> Self {
        self.tone = Tone::Accent;
        self
    }

    pub fn gray(mut self) -> Self {
        self.tone = Tone::Gray;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn menu_size(mut self, size: ButtonSize) -> Self {
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
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> PopupMenuBuilder {
        let template = popup_menu_template(&look, self.variant, self.tone);
        let mut builder = xgpui_luma::controls::popup_menu::PopupMenu::new(self.id)
            .stylesheet(&look.common_stylesheet())
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

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(PopupMenu::new("s").size(ButtonSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(PopupMenu::new("s").size(ButtonSize::Two).size.control_size(), ControlSize::Md);
    }

    #[test]
    fn solid_primary_keeps_radix_axes() {
        let menu = PopupMenu::new("copy").solid().primary().gray().label("Copy");
        assert!(matches!(menu.variant, PopupMenuVariant::Solid));
        assert_eq!(menu.tone, Tone::Gray);
        assert_eq!(menu.trigger_style, PopupMenuTriggerStyle::Primary);
        assert!(matches!(menu.face, TriggerFace::Label(_)));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = PopupMenu::new("ok")
            .look(&look)
            .soft()
            .label("Actions")
            .items([MenuItem::new("one").label("One")])
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }

    #[test]
    fn svg_path_icon_is_forwarded() {
        let look = Look::built_in();
        let menu = PopupMenu::new("icon").icon(ControlIcon::SvgPath("M1 1h4v4H1z".into()));
        assert!(matches!(menu.face, TriggerFace::Icon(ControlIcon::SvgPath(_))));
        let _builder = menu.look(&look).into_sdk_builder(look);
    }
}
