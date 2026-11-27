crate::motion_profile!(PopupMenuMotion => popup_menu {
    submenu: crate::motion::Transition,
    highlight: crate::motion::Transition,
    disclosure: crate::motion::Transition,
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{AppContext, Bounds, Entity, Pixels, SharedString, div, prelude::*, px, svg};
use lucide_svg_static::Icon as LucideIcon;

use super::{ControlFocusState, MenuPath, PopupMenu, PopupMenuTemplate, default_popup_menu_template};
use crate::theme::InteractionState;
use super::template::modified_popup_menu_template;
use crate::controls::button::ControlIcon;
use crate::infra::menu_item::MenuItem;
use crate::controls::floating_menu::FloatingMenuHighlight;
use crate::infra::icon::DisclosureIcons;
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::presenter::{ControlPresenter, HasPresenter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuPlacement {
    #[default]
    Smart,
    BelowStart,
    AboveStart,
    /// Opens to the right of the trigger, bottom edges aligned (sidebar account menus).
    RightEnd,
    CenteredOnTrigger,
}

/// Trigger chrome aligned with command button variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuTriggerStyle {
    Primary,
    Secondary,
    #[default]
    Outline,
    Ghost,
}

/// Owned snapshot passed to trigger face presenters.
#[derive(Clone, Debug)]
pub struct PopupMenuTriggerModel {
    pub id: SharedString,
    pub label: SharedString,
    pub open: bool,
    pub enabled: bool,
    pub state: InteractionState,
}

fn default_trigger_content() -> ControlPresenter<PopupMenuTriggerModel> {
    Arc::new(|model, _| div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element())
}

/// Presenter used by the icon-only face preset ([`PopupMenuBuilder::icon`] / [`PopupMenu::set_icon`](super::PopupMenu::set_icon)).
pub fn icon_content(icon: impl Into<ControlIcon>) -> ControlPresenter<PopupMenuTriggerModel> {
    match icon.into() {
        ControlIcon::Lucide(icon) => Arc::new(move |_, _| {
            svg()
                .path(icon.asset_path())
                .size(px(crate::theme::stylesheet::SharedGeometry::sdk().popup_icon_size))
                .into_any_element()
        }),
        ControlIcon::SvgPath(path) => Arc::new(move |_, _| {
            svg()
                .size(px(crate::theme::stylesheet::SharedGeometry::sdk().popup_icon_size))
                .path(path.clone())
                .into_any_element()
        }),
    }
}

#[derive(Clone)]
pub struct PopupMenuModel {
    pub(crate) motion: crate::motion::MotionConfig<PopupMenuMotion>,
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) content: ControlPresenter<PopupMenuTriggerModel>,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    /// Drawn as `:invalid` (e.g. a required choice left unmade).
    pub(crate) invalid: bool,
    pub(crate) tab_stop: bool,
    pub(crate) placement: PopupMenuPlacement,
    pub(crate) dismiss: crate::infra::popup_dismiss::PopupDismiss,
    pub(crate) trigger_style: PopupMenuTriggerStyle,
    /// Square icon-button chrome (no trailing end icon). Set by the icon face preset.
    pub(crate) icon_only: bool,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) end_icon: Option<LucideIcon>,
    pub(crate) disclosure_icons_explicit: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) full_width: bool,
    pub(crate) without_elevation: bool,
    pub(crate) split: bool,
    pub(crate) primary_enabled: bool,
    pub(crate) template: Arc<dyn PopupMenuTemplate>,
}

pub struct PopupMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub content: ControlPresenter<PopupMenuTriggerModel>,
    pub items: &'a [MenuItem],
    pub open: bool,
    pub disclosure_progress: f32,
    pub presence: OverlayPresence,
    pub submenu_transition: Option<(usize, f32)>,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: PopupMenuPlacement,
    pub trigger_style: PopupMenuTriggerStyle,
    pub icon_only: bool,
    pub icon: Option<LucideIcon>,
    pub end_icon: Option<LucideIcon>,
    pub disclosure_icons: &'a DisclosureIcons,
    pub full_width: bool,
    pub without_elevation: bool,
    pub split: bool,
    /// Independent face feedback for split triggers.
    pub split_state: crate::controls::split_button::SplitButtonState,
    pub trigger_radius_override: Option<f32>,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub highlight: Option<FloatingMenuHighlight>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: InteractionState,
}

pub struct PopupMenuBuilder {
    pub(crate) callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::popup_menu::PopupMenuEvent>,

    pub(crate) model: PopupMenuModel,
}

impl PopupMenuBuilder {
    crate::motion_methods!(PopupMenuMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::popup_menu::PopupMenuEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::popup_menu::PopupMenuEvent::ActionClick = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::Select { item_id, label } = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::OpenChanged { open } = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::Dismiss = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::FocusChanged { focused } = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::HoverChanged { hovered } = event {
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
            if let crate::controls::popup_menu::PopupMenuEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            callbacks: Default::default(),

            model: PopupMenuModel {
                motion: Default::default(),
                label: id.clone(),
                id,
                content: default_trigger_content(),
                items: Vec::new(),
                enabled: true,
                invalid: false,
                tab_stop: true,
                placement: PopupMenuPlacement::Smart,
                dismiss: crate::infra::popup_dismiss::PopupDismiss::LIGHT,
                trigger_style: PopupMenuTriggerStyle::default(),
                icon_only: false,
                icon: None,
                end_icon: None,
                disclosure_icons_explicit: false,
                disclosure_icons: DisclosureIcons::menu_from_stylesheet(
                    crate::theme::stylesheet::CommonStylesheet::sdk_default(),
                ),
                full_width: false,
                without_elevation: false,
                split: false,
                primary_enabled: true,
                template: default_popup_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self.model.content = default_trigger_content();
        self.model.icon_only = false;
        self.model.icon = None;
        self
    }

    /// Icon-only face preset: [`icon_content`] + square chrome (`icon_only`).
    ///
    /// Not a parallel field beside the presenter — use [`HasPresenter::content`] / [`PopupMenu::set_presenter`](super::PopupMenu::set_presenter)
    /// for a custom face. Mirrors [`PopupMenu::set_icon`](super::PopupMenu::set_icon).
    /// Lucide glyphs also fill `icon`; SVG paths render through the presenter.
    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        let icon = icon.into();
        self.model.content = icon_content(icon.clone());
        self.model.icon_only = true;
        self.model.icon = match icon {
            ControlIcon::Lucide(icon) => Some(icon),
            ControlIcon::SvgPath(_) => None,
        };
        self
    }

    /// Trailing adornment on a labeled trigger (distinct from the icon face preset).
    pub fn end_icon(mut self, icon: LucideIcon) -> Self {
        self.model.end_icon = Some(icon);
        self
    }

    /// Sets the open and closed trigger icons, including SVG-path sources.
    /// Capture shared icon defaults without replacing explicit control icons.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.disclosure_icons_explicit {
            self.model.disclosure_icons = DisclosureIcons::menu_from_stylesheet(stylesheet);
        }
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons_explicit = true;
        self.model.disclosure_icons = icons;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Draw the trigger as `:invalid`, like [`crate::controls::selector::SelectorBuilder::invalid`].
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    /// What closes the popup: an outside click, focus loss or Escape by default.
    pub fn dismiss(mut self, dismiss: crate::infra::popup_dismiss::PopupDismiss) -> Self {
        self.model.dismiss = dismiss;
        self
    }

    pub fn placement(mut self, placement: PopupMenuPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn trigger_style(mut self, style: PopupMenuTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Ghost)
    }

    /// Uses the trigger as the secondary face of a single-focus split button.
    pub fn split(mut self) -> Self {
        self.model.split = true;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn template(mut self, template: Arc<dyn PopupMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &PopupMenuRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_popup_menu_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<PopupMenu> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| PopupMenu::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

impl HasPresenter<PopupMenuTriggerModel> for PopupMenuBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<PopupMenuTriggerModel>) {
        self.model.content = content;
        self.model.icon_only = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_popup_menu_template();
        let builder = PopupMenuBuilder::new("popup-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn svg_path_icon_keeps_icon_only_chrome() {
        let builder = PopupMenuBuilder::new("icon").icon(ControlIcon::SvgPath("M1 1h4v4H1z".into()));
        assert!(builder.model.icon_only);
        assert!(builder.model.icon.is_none());
    }
}
