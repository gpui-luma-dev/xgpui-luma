crate::motion_profile!(SelectorMotion => selector {
    presence: crate::motion::PresenceMotion,
});

use std::sync::Arc;

use gpui::{App, AppContext, Bounds, Entity, IntoElement, Pixels, SharedString};

use super::{ControlFocusState, Selector, SelectorTemplate, SelectorVisualState, default_selector_template};
use crate::theme::InteractionState;
use super::template::modified_selector_template;
pub use crate::controls::selector_list::{
    SelectorItem, SelectorItemLike, SelectorItemRenderModel, SelectorItemTemplate, SelectorItemsTemplate, SelectorPath,
    default_selector_items_template, make_selector_item_template, normalize_selector_items,
};
use crate::controls::selector_list::items_template_with_modifier;
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::icon::IconSource;

/// Trigger chrome aligned with command button variants (read-only select).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectorTriggerStyle {
    #[default]
    Outline,
    Ghost,
}

/// How a selector popup opens. Placement options apply to `Dropdown` only.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectorOpeningMode {
    /// Preserve the configured popup placement.
    #[default]
    Dropdown,
    /// Align the selected row over the trigger; fall back to the first enabled row.
    Centered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPlacement {
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
    OverlayOnTrigger,
}

pub struct SelectorModel<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    pub(crate) motion: crate::motion::MotionConfig<SelectorMotion>,
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) enabled: bool,
    pub(crate) scroll_interaction: crate::interaction::ScrollInteraction,
    pub(crate) invalid: bool,
    pub(crate) tab_stop: bool,
    pub(crate) placement: SelectorPlacement,
    pub(crate) dismiss: crate::infra::popup_dismiss::PopupDismiss,
    pub(crate) opening_mode: SelectorOpeningMode,
    pub(crate) trigger_style: SelectorTriggerStyle,
    pub(crate) icons_explicit: bool,
    pub(crate) icons: SelectorIcons,
    pub(crate) without_elevation: bool,
    pub(crate) item_template: Option<SelectorItemTemplate<T>>,
    pub(crate) panel_template: Arc<dyn SelectorItemsTemplate<T>>,
    pub(crate) template: Arc<dyn SelectorTemplate<T>>,
}

#[derive(Clone, Debug)]
pub struct SelectorIcons {
    pub trigger: IconSource,
    pub selected: IconSource,
}

impl Default for SelectorIcons {
    fn default() -> Self {
        Self::from_stylesheet(crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }
}
impl SelectorIcons {
    /// Resolve default icon roles before applying explicit consumer icon overrides.
    pub fn from_stylesheet(sheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        use crate::theme::stylesheet::IconRole;
        Self {
            trigger: sheet.icons.resolve(IconRole::SelectorTrigger),
            selected: sheet.icons.resolve(IconRole::Selected),
        }
    }
}

pub struct SelectorRenderModel<'a, T>
where
    T: SelectorItemLike + 'static,
{
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected_index: Option<usize>,
    pub items: &'a [T],
    pub open: bool,
    pub presence: OverlayPresence,
    /// Retained popup geometry. Custom templates must pair this with the popup wheel handler.
    pub popup_scroll: Option<&'a gpui::ScrollHandle>,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: SelectorPlacement,
    pub opening_mode: SelectorOpeningMode,
    /// Measured panel geometry for selected-row alignment.
    pub popup_geometry: Option<crate::controls::selector_list::SelectorPopupGeometry>,
    /// Initialize the opening scroll offset after layout measurement.
    pub initialize_popup_scroll: bool,
    pub active_path: Option<SelectorPath>,
    pub enabled: bool,
    pub trigger_style: SelectorTriggerStyle,
    pub icons: &'a SelectorIcons,
    pub without_elevation: bool,
    pub item_template: Option<&'a SelectorItemTemplate<T>>,
    pub panel_template: Option<&'a dyn SelectorItemsTemplate<T>>,
    pub focus: ControlFocusState,
    pub state: InteractionState,
    pub visual_state: SelectorVisualState,
}

pub struct SelectorBuilder<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::selector::SelectorEvent>,

    pub(crate) model: SelectorModel<T>,
    pub(crate) initial_selected_id: Option<SharedString>,
}

impl<T> SelectorBuilder<T>
where
    T: SelectorItemLike + 'static,
{
    crate::motion_methods!(SelectorMotion, model.motion);
    crate::motion_policy!(model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::selector::SelectorEvent, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::selector::SelectorEvent::Change { item_id, label } = event {
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
            if let crate::controls::selector::SelectorEvent::FocusChanged { focused } = event {
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
            if let crate::controls::selector::SelectorEvent::OpenChanged { open } = event {
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
            if let crate::controls::selector::SelectorEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            callbacks: Default::default(),

            model: SelectorModel {
                motion: Default::default(),
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                scroll_interaction: crate::interaction::ScrollInteraction::VIEWPORT,
                invalid: false,
                tab_stop: true,
                placement: SelectorPlacement::Smart,
                dismiss: crate::infra::popup_dismiss::PopupDismiss::LIGHT,
                opening_mode: SelectorOpeningMode::default(),
                trigger_style: SelectorTriggerStyle::default(),
                icons_explicit: false,
                icons: SelectorIcons::default(),
                without_elevation: false,
                item_template: None,
                panel_template: default_selector_items_template(),
                template: default_selector_template::<T>(),
            },
            initial_selected_id: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.model.trigger_style = SelectorTriggerStyle::Ghost;
        self
    }

    /// Capture default glyphs; explicit `.icons(...)` wins in either method order.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.icons_explicit {
            self.model.icons = SelectorIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn icons(mut self, icons: SelectorIcons) -> Self {
        self.model.icons_explicit = true;
        self.model.icons = icons;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = normalize_selector_items(items);
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.initial_selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Choose popup wheel eligibility. Defaults to pointer scrolling without acquiring focus.
    pub fn wheel_scroll_policy(mut self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Choose containment or whole-event chaining independently of wheel eligibility.
    pub fn scroll_boundary_policy(mut self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Choose which real focus handles qualify for RequireFocus. No row gains focus implicitly.
    pub fn wheel_focus_scope(mut self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    /// Choose dropdown placement or selected-item-centered opening.
    pub fn opening_mode(mut self, mode: SelectorOpeningMode) -> Self {
        self.model.opening_mode = mode;
        self
    }

    /// What closes the popup: an outside click, focus loss or Escape by default.
    pub fn dismiss(mut self, dismiss: crate::infra::popup_dismiss::PopupDismiss) -> Self {
        self.model.dismiss = dismiss;
        self
    }

    pub fn placement(mut self, placement: SelectorPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_selector_item_template(template));
        self
    }

    pub fn template(mut self, template: Arc<dyn SelectorTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::Stateful<gpui::Div>, &SelectorRenderModel<'a, T>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_selector_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::Stateful<gpui::Div>,
                &crate::controls::selector_list::SelectorItemsRenderModel<'a, T>,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.panel_template = items_template_with_modifier(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<Selector<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| Selector::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_selector_template::<SelectorItem>();
        let builder = SelectorBuilder::new("selector-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_panel_template_modifier_wraps_panel_template() {
        let panel_template = default_selector_items_template::<SelectorItem>();
        let builder = SelectorBuilder::new("selector-test")
            .panel_template(panel_template.clone())
            .with_panel_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.panel_template, &panel_template));
    }
}
