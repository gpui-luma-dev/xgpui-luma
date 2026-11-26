use super::super::SidebarMotion;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Bounds, Entity, FocusHandle, IntoElement, Pixels, SharedString, Size, div, prelude::*};
use lucide_svg_static::Icon as LucideIcon;

use super::{SidebarPanelEngine, SidebarPanelTemplate, default_sidebar_panel_template};
use super::template::modified_sidebar_panel_template;
use crate::infra::presenter::{Presenter, HostedContent, IntoPresenter};
use crate::infra::menu_item::MenuItem;
use crate::infra::state::MenuPath;
use crate::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::icon::DisclosureIcons;

pub type NavHostedContent = HostedContent;
pub type NavPresenter = Presenter<NavNodeState>;

#[derive(Clone, Debug)]
pub struct NavNodeState {
    pub id: SharedString,
    pub depth: usize,
    pub sidebar_collapsed: bool,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub expanded: bool,
    pub enabled: bool,
    /// Whether the row draws disabled: `!enabled`, unless a state override (`set_item_state_override`) changes it.
    /// Input follows `enabled` (#397).
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavNodeKind {
    Item,
    Section,
}

#[derive(Clone)]
pub struct NavNode {
    pub(crate) id: SharedString,
    pub(crate) kind: NavNodeKind,
    pub(crate) label: Option<SharedString>,
    pub(crate) tooltip: Option<crate::controls::tooltip::Tooltip>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) presenter: Option<NavPresenter>,
    pub(crate) children: Vec<NavNode>,
    pub(crate) branch: bool,
    pub(crate) expanded: bool,
    pub(crate) enabled: bool,
    pub(crate) visible: bool,
}

impl NavNode {
    pub(crate) fn has_children(&self) -> bool {
        self.branch || !self.children.is_empty()
    }
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind: NavNodeKind::Item,
            label: None,
            tooltip: None,
            icon: None,
            presenter: None,
            children: Vec::new(),
            branch: false,
            expanded: false,
            enabled: true,
            visible: true,
        }
    }

    pub fn section(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id).label(label).kind(NavNodeKind::Section).enabled(false)
    }

    pub fn kind(mut self, kind: NavNodeKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Configure help for the standard navigation row (custom presenters own their help).
    pub fn tooltip(mut self, tooltip: crate::controls::tooltip::Tooltip) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn presenter(mut self, presenter: impl IntoPresenter<NavNodeState>) -> Self {
        self.presenter = Some(presenter.into_presenter());
        self
    }

    pub fn child(mut self, child: NavNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = NavNode>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
}

#[derive(Clone)]
pub struct SidebarPanelEngineModel {
    pub(crate) motion: crate::motion::MotionConfig<SidebarMotion>,
    pub(crate) id: SharedString,
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
    pub(crate) header_nodes: Vec<NavNode>,
    pub(crate) nodes: Vec<NavNode>,
    pub(crate) footer_nodes: Vec<NavNode>,
    pub(crate) selected_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) collapsed: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn SidebarPanelTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) scrollbar_placement: ScrollbarPlacement,
    pub(crate) scrollbar_visibility: ScrollbarVisibility,
    pub(crate) scrollbar_auto_hide_activate: ScrollbarAutoHideActivate,
}

#[derive(Clone)]
pub struct RenderedRailSubmenu {
    pub id: SharedString,
    pub parent_node_id: SharedString,
    pub parent_bounds: Bounds<Pixels>,
    pub items: Vec<MenuItem>,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub presence: OverlayPresence,
    pub content_size: Option<Size<Pixels>>,
}

pub struct SidebarPanelEngineRenderModel {
    pub id: SharedString,
    pub title: Option<SharedString>,
    pub subtitle: Option<SharedString>,
    pub header_nodes: Vec<RenderedNavNode>,
    pub nodes: Vec<RenderedNavNode>,
    pub footer_nodes: Vec<RenderedNavNode>,
    pub rail_nodes: Vec<RenderedNavNode>,
    pub rail_footer_nodes: Vec<RenderedNavNode>,
    pub rail_submenu: Option<RenderedRailSubmenu>,
    pub selected_id: Option<SharedString>,
    pub collapsed: bool,
    pub disclosure_icons: DisclosureIcons,
}

pub struct RenderedNavNode {
    pub drop_layout: Option<crate::motion::DropLayoutMotion<SharedString>>,
    /// Native drag binding; custom templates attach this to the row only.
    pub drag_drop: Option<super::super::drag::SidebarItemDragBinding>,
    /// Retained per-item help. Custom templates apply this to the row's focus root.
    pub tooltip: Option<crate::infra::attachments::TooltipHandle>,
    pub id: SharedString,
    pub kind: NavNodeKind,
    pub label: Option<SharedString>,
    pub icon: Option<LucideIcon>,
    pub state: NavNodeState,
    pub custom_element: Option<AnyElement>,
    pub focus_handle: Option<FocusHandle>,
    pub has_children: bool,
    pub expansion_progress: f32,
    pub children_height_px: f32,
    pub children: Vec<RenderedNavNode>,
}

pub struct SidebarPanelEngineBuilder {
    pub(crate) model: SidebarPanelEngineModel,
}

impl SidebarPanelEngineBuilder {
    crate::motion_methods!(SidebarMotion, model.motion);
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SidebarPanelEngineModel {
                motion: Default::default(),
                id: id.into(),
                title: None,
                subtitle: None,
                header_nodes: Vec::new(),
                nodes: Vec::new(),
                footer_nodes: Vec::new(),
                selected_id: None,
                enabled: true,
                collapsed: false,
                motion_enabled: true,
                disclosure_icons: DisclosureIcons::default(),
                template: default_sidebar_panel_template(),
                scrollbar_template: default_scrollbar_template(),
                scrollbar_placement: ScrollbarPlacement::Inset,
                scrollbar_visibility: ScrollbarVisibility::AlwaysVisible,
                scrollbar_auto_hide_activate: ScrollbarAutoHideActivate::HoverOrMove,
            },
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = Some(title.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.model.subtitle = Some(subtitle.into());
        self
    }

    pub fn header_node(mut self, node: NavNode) -> Self {
        self.model.header_nodes.push(node);
        self
    }

    pub fn header_nodes(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.header_nodes = nodes.into_iter().collect();
        self
    }

    pub fn item(mut self, node: NavNode) -> Self {
        self.model.nodes.push(node);
        self
    }

    pub fn items(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.nodes = nodes.into_iter().collect();
        self
    }

    pub fn footer_node(mut self, node: NavNode) -> Self {
        self.model.footer_nodes.push(node);
        self
    }

    pub fn footer_nodes(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.footer_nodes = nodes.into_iter().collect();
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.model.collapsed = collapsed;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn SidebarPanelTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_sidebar_panel_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn scrollbar_placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.model.scrollbar_placement = placement;
        self
    }

    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    pub fn scrollbar_auto_hide_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.model.scrollbar_auto_hide_activate = activate;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SidebarPanelEngine> {
        cx.new(|cx| SidebarPanelEngine::from_builder(self, cx))
    }
}

pub fn entity_presenter<T>(entity: Entity<T>, focus_handle: FocusHandle) -> NavPresenter
where
    Entity<T>: IntoElement + Clone + 'static,
{
    Presenter::new(move |_, _, _| NavHostedContent {
        element: div().w_full().child(entity.clone()).into_any_element(),
        focus_handle: Some(focus_handle.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_sidebar_panel_template();
        let builder = SidebarPanelEngineBuilder::new("nav-test")
            .template(template.clone())
            .with_template_modifier(|element| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
