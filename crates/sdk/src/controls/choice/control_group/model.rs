crate::motion_profile!(ControlGroupMotion => control_group {
    selection: crate::motion::Transition,
});

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Div, Entity, FocusHandle, Hsla, IntoElement, ParentElement, ScrollHandle,
    SharedString, Stateful, Window, div,
};

use super::control::ControlGroupControl;
use super::menu_choice_template::{menu_choice_group_template, menu_choice_row_item_element_template};
use super::template::{
    ControlGroupItemTemplate, ControlGroupItemElementTemplate, ControlGroupTemplate,
    control_group_item_layout_template, default_control_group_template, item_template_with_modifier,
    make_control_group_item_template, modified_control_group_template,
};
use super::theme::{ControlGroupItemVisualContext, ControlGroupTheme};
use crate::infra::state::{CompositeItemState, ControlFocusState};
use crate::theme::ControlSize;

pub trait ControlGroupItemLike {
    fn id(&self) -> &SharedString;

    fn label(&self) -> &SharedString {
        self.id()
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct ControlGroupItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl ControlGroupItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl ControlGroupItemLike for ControlGroupItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlSelectionMode {
    SingleRequired,
    #[default]
    SingleAllowNone,
    Multiple,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupLayout {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlGroupChromeModel {
    pub enabled: bool,
    pub layout: ControlGroupLayout,
    pub item_count: usize,
}

impl<'a, T> From<&ControlGroupRenderModel<'a, T>> for ControlGroupChromeModel
where
    T: ControlGroupItemLike + 'static,
{
    fn from(model: &ControlGroupRenderModel<'a, T>) -> Self {
        Self { enabled: model.enabled, layout: model.layout, item_count: model.items.len() }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupStateMode {
    #[default]
    Unmanaged,
    Managed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupFocusStrategy {
    /// The group root owns GPUI focus and the active item is rendered as the focused descendant.
    #[default]
    ActiveDescendant,
    /// The group owns item order while focus may move to an item's supplied GPUI focus target.
    RovingItemFocus,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupArrowAxis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupArrowPolicy {
    /// Arrow-key actions move the group's active item.
    #[default]
    GroupOwns,
    /// All arrow-key actions are ignored by the group while the active item's target is focused.
    ChildOwnsWhenFocused,
    /// Left/Right are ignored by the group while focused; Up/Down still move the active item.
    ChildOwnsHorizontalWhenFocused,
    /// Up/Down are ignored by the group while focused; Left/Right still move the active item.
    ChildOwnsVerticalWhenFocused,
}

impl ControlGroupArrowPolicy {
    pub(crate) fn child_owns_axis(self, axis: ControlGroupArrowAxis) -> bool {
        match self {
            Self::GroupOwns => false,
            Self::ChildOwnsWhenFocused => true,
            Self::ChildOwnsHorizontalWhenFocused => axis == ControlGroupArrowAxis::Horizontal,
            Self::ChildOwnsVerticalWhenFocused => axis == ControlGroupArrowAxis::Vertical,
        }
    }
}

#[derive(Clone)]
pub struct ControlGroupFocusTarget {
    /// GPUI focus target associated with the control-group item.
    pub focus_handle: FocusHandle,
    /// Defines whether group arrow navigation remains active while this target is focused.
    ///
    /// Providers may return a different policy dynamically (for example `GroupOwns` while a
    /// popup is closed and `ChildOwnsWhenFocused` while it is open).
    pub arrow_policy: ControlGroupArrowPolicy,
}

/// Resolves an optional focus target for a control-group item at interaction time.
pub type ControlGroupFocusTargetProvider<T> =
    Arc<dyn for<'a> Fn(&'a T, &mut Window, &mut App) -> Option<ControlGroupFocusTarget> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ControlGroupModel<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(crate) motion: crate::motion::MotionConfig<ControlGroupMotion>,
    pub(crate) stylesheet: Arc<crate::theme::stylesheet::CommonStylesheet>,
    pub(crate) stylesheet_overrides_shell: bool,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) default_selected_ids: Vec<SharedString>,
    pub(crate) managed_selected_ids: Option<Vec<SharedString>>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) selection_mode: ControlSelectionMode,
    pub(crate) state_mode: ControlGroupStateMode,
    pub(crate) selection_follows_active: bool,
    pub(crate) animated_selection: bool,
    pub(crate) independent_toggles: bool,
    pub(crate) focus_strategy: ControlGroupFocusStrategy,
    pub(crate) focus_target_provider: Option<ControlGroupFocusTargetProvider<T>>,
    pub(crate) enabled: bool,
    pub(crate) tab_stop: bool,
    pub(crate) layout: ControlGroupLayout,
    pub(crate) locale_navigation: bool,
    pub(crate) scrollable: bool,
    pub(crate) template: ControlGroupTemplate<T>,
    pub(crate) item_template: Option<ControlGroupItemTemplate<T>>,
    pub(crate) item_element_template: Option<ControlGroupItemElementTemplate<T>>,
}

impl<T> ControlGroupModel<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub fn effective_selected_ids(&self) -> &[SharedString] {
        self.managed_selected_ids.as_deref().unwrap_or(self.default_selected_ids.as_slice())
    }

    pub fn effective_selected_id(&self) -> Option<&SharedString> {
        self.effective_selected_ids().first()
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.state_mode = ControlGroupStateMode::Managed;
        self.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
    }

    pub fn clear_managed_selected_ids(&mut self) {
        self.managed_selected_ids = None;
    }
}

#[derive(Clone, Debug)]
pub struct ControlGroupItemRenderModel<'a, T>
where
    T: ControlGroupItemLike + 'static,
{
    pub group_id: &'a SharedString,
    /// Captured visual defaults shared by the group and its item templates.
    pub stylesheet: &'a crate::theme::stylesheet::CommonStylesheet,
    pub layout: ControlGroupLayout,
    pub item: &'a T,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub selection_progress: f32,
    pub active: bool,
    pub enabled: bool,
    pub state: CompositeItemState,
    pub selection_mode: ControlSelectionMode,
    /// Optional color supplied by a themed parent container for item-owned SVG content.
    pub foreground: Option<Hsla>,
}

pub struct ControlGroupRenderModel<'a, T>
where
    T: ControlGroupItemLike + 'static,
{
    pub id: &'a SharedString,
    pub stylesheet: &'a crate::theme::stylesheet::CommonStylesheet,
    /// Explicit builder styles override the supplied shell theme; defaults do not.
    pub shell_stylesheet: Option<&'a crate::theme::stylesheet::CommonStylesheet>,
    pub items: Vec<ControlGroupItemRenderModel<'a, T>>,
    pub selected_ids: &'a [SharedString],
    pub active_id: Option<&'a SharedString>,
    pub selection_mode: ControlSelectionMode,
    pub state_mode: ControlGroupStateMode,
    pub enabled: bool,
    pub layout: ControlGroupLayout,
    pub scroll_handle: Option<&'a ScrollHandle>,
    pub focus: ControlFocusState,
    pub focus_strategy: ControlGroupFocusStrategy,
    pub item_template: Option<&'a ControlGroupItemTemplate<T>>,
    pub item_element_template: Option<&'a ControlGroupItemElementTemplate<T>>,
}

pub struct ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::control_group::ControlGroupEvent>,

    pub(crate) model: ControlGroupModel<T>,
}

impl<T> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    crate::motion_methods!(ControlGroupMotion, model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::control_group::ControlGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ControlGroupEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::Activate { activated_id } = event {
                callback(owner, activated_id, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::Change { changed_id, selected, selected_ids } =
                event
            {
                callback(owner, changed_id, *selected, selected_ids, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemBoundsChanged` through the existing event stream.
    pub fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::ItemBoundsChanged { item_id, bounds } = event {
                callback(owner, item_id, bounds, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: ControlGroupModel {
                motion: Default::default(),
                stylesheet: default_group_stylesheet(),
                stylesheet_overrides_shell: false,
                id: id.into(),
                items: Vec::new(),
                default_selected_ids: Vec::new(),
                managed_selected_ids: None,
                active_id: None,
                selection_mode: ControlSelectionMode::SingleAllowNone,
                state_mode: ControlGroupStateMode::Unmanaged,
                selection_follows_active: false,
                animated_selection: false,
                independent_toggles: false,
                focus_strategy: ControlGroupFocusStrategy::default(),
                focus_target_provider: None,
                enabled: true,
                tab_stop: true,
                layout: ControlGroupLayout::default(),
                locale_navigation: false,
                scrollable: false,
                template: default_control_group_template(),
                item_template: None,
                item_element_template: None,
            },
        }
    }

    /// Capture shared group/preset styles; consumer template modifiers apply afterward.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        self.model.stylesheet = Arc::new(stylesheet.with_sdk_defaults());
        self.model.stylesheet_overrides_shell = true;
        self
    }

    pub fn layout(mut self, layout: ControlGroupLayout) -> Self {
        self.model.layout = layout;
        self
    }

    pub fn horizontal(self) -> Self {
        self.layout(ControlGroupLayout::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.layout(ControlGroupLayout::Vertical)
    }

    pub fn scrollable(mut self) -> Self {
        self.model.scrollable = true;
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn mode(mut self, selection_mode: ControlSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn selection_mode(self, selection_mode: ControlSelectionMode) -> Self {
        self.mode(selection_mode)
    }

    pub fn selection_follows_active(mut self, selection_follows_active: bool) -> Self {
        self.model.selection_follows_active = selection_follows_active;
        self
    }

    /// Animates selection progress for item templates that support it.
    pub fn motion_enabled(self, enabled: bool) -> Self {
        self.animated_selection(enabled)
    }

    pub fn animated_selection(mut self, animated_selection: bool) -> Self {
        self.model.animated_selection = animated_selection;
        self
    }

    pub fn focus_strategy(mut self, focus_strategy: ControlGroupFocusStrategy) -> Self {
        self.model.focus_strategy = focus_strategy;
        self
    }

    pub fn active_descendant(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::ActiveDescendant)
    }

    pub fn roving_item_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::RovingItemFocus)
    }

    pub fn focus_target_provider(mut self, provider: ControlGroupFocusTargetProvider<T>) -> Self {
        self.model.focus_target_provider = Some(provider);
        self
    }

    pub fn with_focus_target_provider<F>(mut self, provider: F) -> Self
    where
        F: for<'a> Fn(&'a T, &mut Window, &mut App) -> Option<ControlGroupFocusTarget> + Send + Sync + 'static,
    {
        self.model.focus_target_provider = Some(Arc::new(provider));
        self
    }

    pub fn clear_focus_target_provider(mut self) -> Self {
        self.model.focus_target_provider = None;
        self
    }

    pub fn single_required(self) -> Self {
        self.mode(ControlSelectionMode::SingleRequired)
    }

    pub fn single_allow_none(self) -> Self {
        self.mode(ControlSelectionMode::SingleAllowNone)
    }

    pub fn multiple(self) -> Self {
        self.mode(ControlSelectionMode::Multiple)
    }

    /// Toggle each item independently in multiple-selection groups, without
    /// requiring a modifier key. Intended for checkbox-style groups.
    pub fn independent_toggles(mut self, enabled: bool) -> Self {
        self.model.independent_toggles = enabled;
        self
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.default_selected_ids = vec![selected_id.into()];
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn managed(mut self) -> Self {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self
    }

    pub fn unmanaged(mut self) -> Self {
        self.model.state_mode = ControlGroupStateMode::Unmanaged;
        self.model.managed_selected_ids = None;
        self
    }

    pub fn managed_selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(vec![selected_id.into()]);
        self
    }

    pub fn managed_selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.model.active_id = Some(active_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    pub fn item_template(mut self, template: ControlGroupItemTemplate<T>) -> Self {
        self.model.item_template = Some(template);
        self
    }

    pub fn item_element_template(mut self, template: ControlGroupItemElementTemplate<T>) -> Self {
        self.model.item_element_template = Some(template);
        self
    }

    pub fn with_item_template<F, E>(self, template: F) -> Self
    where
        F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.item_template(make_control_group_item_template(template))
    }

    pub fn with_item_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> gpui::AnyElement
            + Send
            + Sync
            + 'static,
    {
        let base = self.model.item_template.take().unwrap_or_else(|| {
            make_control_group_item_template(|item: &ControlGroupItemRenderModel<'_, T>, _, _| {
                div().child(item.item.label().clone())
            })
        });
        self.model.item_template = Some(item_template_with_modifier(base, modifier));
        self
    }

    pub fn clear_item_template(mut self) -> Self {
        self.model.item_template = None;
        self
    }

    pub fn clear_item_element_template(mut self) -> Self {
        self.model.item_element_template = None;
        self
    }

    pub fn template(mut self, template: ControlGroupTemplate<T>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = modified_control_group_template(self.model.template.clone(), modifier);
        self
    }

    pub fn with_template<F>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(
                &ControlGroupRenderModel<'a, T>,
                super::template::ControlGroupTemplateHandlers,
                &mut gpui::Window,
                &mut App,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = Arc::new(template);
        self
    }

    pub fn with_item_layout<F, E>(mut self, layout: F) -> Self
    where
        F: for<'a> Fn(
                super::template::ControlGroupItemElements,
                &ControlGroupRenderModel<'a, T>,
                &mut gpui::Window,
                &mut App,
            ) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.model.template = control_group_item_layout_template(layout);
        self
    }

    pub fn with_menu_row_item_content<F>(self, theme: Arc<dyn ControlGroupTheme>, content: F) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        {
            let geometry = crate::theme::stylesheet::CommonStylesheet::sdk_default().shared_geometry();
            self.with_menu_row_item_content_sized(
                theme,
                ControlSize::Sm,
                geometry.menu_row_height,
                geometry.menu_row_gap,
                content,
            )
        }
    }

    pub fn with_menu_row_item_content_sized<F>(
        mut self,
        theme: Arc<dyn ControlGroupTheme>,
        size: ControlSize,
        row_height: f32,
        row_radius: f32,
        content: F,
    ) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        let content = Arc::new(content);
        self.model.template = menu_choice_group_template();
        self.model.item_element_template =
            Some(menu_choice_row_item_element_template(theme, size, row_height, row_radius, content));
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<ControlGroupControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| ControlGroupControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_control_group_template::<ControlGroupItem>();
        let builder = ControlGroupBuilder::new("control-group-test")
            .template(template.clone())
            .with_template_modifier(|el, _| el);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_item_template_modifier_creates_template_from_default_content() {
        let builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test")
            .with_item_template_modifier(|content, _, _, _| content);

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn with_item_template_modifier_wraps_existing_item_template() {
        let item_template =
            make_control_group_item_template(|_item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _, _| {
                gpui::div()
            });
        let builder = ControlGroupBuilder::new("control-group-test")
            .item_template(item_template.clone())
            .with_item_template_modifier(|content, _, _, _| content);

        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn template_and_item_modifiers_compose_on_builder() {
        let template = default_control_group_template::<ControlGroupItem>();
        let item_template =
            make_control_group_item_template(|_item: &ControlGroupItemRenderModel<'_, ControlGroupItem>, _, _| {
                gpui::div()
            });
        let builder = ControlGroupBuilder::new("control-group-test")
            .template(template.clone())
            .item_template(item_template.clone())
            .with_template_modifier(|el, _| el)
            .with_item_template_modifier(|content, _, _, _| content);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn active_descendant_is_default_focus_strategy() {
        let builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test");

        assert_eq!(builder.model.focus_strategy, ControlGroupFocusStrategy::ActiveDescendant);
        assert!(builder.model.focus_target_provider.is_none());
    }

    #[test]
    fn roving_item_focus_sets_focus_strategy() {
        let builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test").roving_item_focus();

        assert_eq!(builder.model.focus_strategy, ControlGroupFocusStrategy::RovingItemFocus);
    }

    #[test]
    fn tab_stop_defaults_true_and_can_be_disabled() {
        let default_builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test");
        assert!(default_builder.model.tab_stop);

        let builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test").tab_stop(false);
        assert!(!builder.model.tab_stop);
    }

    #[test]
    fn scrolling_is_opt_in_and_preserves_layout_axis() {
        let vertical = ControlGroupBuilder::<ControlGroupItem>::new("vertical").scrollable();
        assert!(vertical.model.scrollable);
        assert_eq!(vertical.model.layout, ControlGroupLayout::Vertical);

        let horizontal = ControlGroupBuilder::<ControlGroupItem>::new("horizontal").horizontal().scrollable();
        assert!(horizontal.model.scrollable);
        assert_eq!(horizontal.model.layout, ControlGroupLayout::Horizontal);
    }

    #[test]
    fn animated_selection_is_opt_in() {
        let default_builder = ControlGroupBuilder::<ControlGroupItem>::new("control-group-test");
        assert!(!default_builder.model.animated_selection);

        let animated_builder =
            ControlGroupBuilder::<ControlGroupItem>::new("control-group-test").animated_selection(true);
        assert!(animated_builder.model.animated_selection);
    }

    #[test]
    fn arrow_policy_respects_directional_ownership() {
        assert!(!ControlGroupArrowPolicy::GroupOwns.child_owns_axis(ControlGroupArrowAxis::Horizontal));
        assert!(ControlGroupArrowPolicy::ChildOwnsWhenFocused.child_owns_axis(ControlGroupArrowAxis::Vertical));
        assert!(
            ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused.child_owns_axis(ControlGroupArrowAxis::Horizontal)
        );
        assert!(
            !ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused.child_owns_axis(ControlGroupArrowAxis::Vertical)
        );
        assert!(ControlGroupArrowPolicy::ChildOwnsVerticalWhenFocused.child_owns_axis(ControlGroupArrowAxis::Vertical));
        assert!(
            !ControlGroupArrowPolicy::ChildOwnsVerticalWhenFocused.child_owns_axis(ControlGroupArrowAxis::Horizontal)
        );
    }
}

fn default_group_stylesheet() -> Arc<crate::theme::stylesheet::CommonStylesheet> {
    static DEFAULT: std::sync::OnceLock<Arc<crate::theme::stylesheet::CommonStylesheet>> = std::sync::OnceLock::new();
    DEFAULT
        .get_or_init(|| Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()))
        .clone()
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use crate::theme::stylesheet::CommonStylesheet;

    #[test]
    fn grouped_builders_capture_defaults_and_custom_styles() {
        let defaults = ControlGroupBuilder::<ControlGroupItem>::new("defaults");
        assert!(!defaults.model.stylesheet_overrides_shell);
        assert_eq!(defaults.model.stylesheet.control_group.geometry.vertical_gap, Some(8.0));
        let mut sheet = CommonStylesheet::parse(
            r#"
[common.control_group.geometry]
border_width=0
vertical_gap=0
disabled_opacity=0
[common.radio_group.geometry]
item_gap=3
[common.radio_group.sizes.horizontal]
gap=17
[common.icon_group.geometry]
padding_x=0
radius=5
"#,
        )
        .unwrap();
        let group = ControlGroupBuilder::<ControlGroupItem>::new("custom").stylesheet(&sheet);
        sheet.control_group.geometry.border_width = Some(9.0);
        assert!(group.model.stylesheet_overrides_shell);
        assert_eq!(group.model.stylesheet.control_group.geometry.border_width, Some(0.0));
        assert_eq!(group.model.stylesheet.radio_group.authored_geometry("vertical").gap, Some(8.0));
        assert_eq!(group.model.stylesheet.icon_group.geometry.padding_x, Some(0.0));
        assert_eq!(group.model.stylesheet.icon_group.geometry.radius, Some(5.0));
        for family in ["control_group", "radio_group"] {
            for value in ["-1", "1.1", "nan"] {
                assert!(
                    CommonStylesheet::parse(&format!("[common.{family}.geometry]\ndisabled_opacity={value}")).is_err()
                );
            }
        }
    }
}
