mod input;
mod nodes;
mod render;

use std::{cell::RefCell, collections::HashMap};

use gpui::{Bounds, Context, EventEmitter, FocusHandle, Pixels, SharedString, Size, Subscription, Window};

use super::{NavNode, SidebarPanelEngineBuilder, SidebarPanelEngineModel, RenderedRailSubmenu};
use crate::controls::scroll_container::ScrollContainer;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::infra::state::MenuPath;
use crate::motion::popup_lifecycle::PopupLifecycle;
use crate::motion::VisualTransition;
use crate::theme::observe_theme_revision;

use self::nodes::{set_expanded_in_nodes, toggle_expanded_in_nodes};

pub(super) type NavigationFocusTarget = SharedString;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SidebarPanelEngineEvent {
    Activate { node_id: SharedString, label: SharedString },
    BranchExpandedChanged { node_id: SharedString, expanded: bool },
    CollapsedChanged { collapsed: bool },
    FocusChanged { focused: bool },
    ItemFocused { node_id: SharedString, label: SharedString },
    ItemHoverChanged { node_id: SharedString, hovered: bool },
    RailSubmenuOpenChanged { node_id: Option<SharedString> },
    EnabledChanged { enabled: bool },
}

pub struct SidebarPanelEngine {
    pub(crate) drop_layout: crate::motion::DropLayoutMotion<SharedString>,
    pub(crate) drag_owner: Option<gpui::WeakEntity<crate::controls::sidebar::SidebarControl>>,
    model: SidebarPanelEngineModel,
    row_handlers: RefCell<Option<input::CachedSidebarHandlers>>,
    tooltip_theme: std::sync::Arc<dyn crate::controls::tooltip::TooltipTheme>,
    tooltips: HashMap<SharedString, crate::infra::attachments::TooltipHandle>,
    main_scroll: ScrollContainer,
    drag_scroll_pending: std::rc::Rc<std::cell::Cell<bool>>,
    hovered_node: Option<SharedString>,
    pressed_node: Option<SharedString>,
    row_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_node_bounds: HashMap<SharedString, Bounds<Pixels>>,
    open_rail_submenu: Option<SharedString>,
    rail_submenu_open_submenu: Option<usize>,
    rail_submenu_active_path: Option<MenuPath>,
    rail_submenu_lifecycle: PopupLifecycle,
    rail_submenu_content: Option<RenderedRailSubmenu>,
    rail_submenu_content_size: Option<Size<Pixels>>,
    visible_focus_nodes: Vec<(NavigationFocusTarget, FocusHandle)>,
    focus_subscriptions: Vec<Subscription>,
    focus_subscription_keys: Vec<(NavigationFocusTarget, FocusHandle)>,
    emitted_focused: bool,
    _subscriptions: Vec<Subscription>,
    branch_transitions: HashMap<SharedString, VisualTransition>,
    branch_heights_px: HashMap<SharedString, f32>,
}

impl EventEmitter<SidebarPanelEngineEvent> for SidebarPanelEngine {}
impl SidebarPanelEngine {
    pub(crate) fn move_items(&mut self, nodes: Vec<NavNode>, cx: &mut Context<Self>) {
        // IDs survive a move: retain focus handles, tooltip history, scroll and motion.
        // Reparenting changes ancestor heights. Discard only those measurements
        // so the first committed layout uses actual new child extents.
        fn collect(nodes: &[NavNode], old: &mut HashMap<SharedString, std::collections::HashSet<SharedString>>) {
            for node in nodes {
                old.insert(node.id.clone(), node.children.iter().map(|child| child.id.clone()).collect());
                collect(&node.children, old);
            }
        }
        fn invalidate(
            nodes: &[NavNode],
            old: &HashMap<SharedString, std::collections::HashSet<SharedString>>,
            heights: &mut HashMap<SharedString, f32>,
        ) -> bool {
            let mut changed = false;
            for node in nodes {
                let descendants = invalidate(&node.children, old, heights);
                let children_changed = old.get(&node.id).is_none_or(|before| {
                    before.len() != node.children.len() || node.children.iter().any(|child| !before.contains(&child.id))
                });
                if descendants || children_changed {
                    heights.remove(&node.id);
                    changed = true;
                }
            }
            changed
        }
        let mut old = HashMap::new();
        collect(&self.model.nodes, &mut old);
        invalidate(&nodes, &old, &mut self.branch_heights_px);
        self.model.nodes = nodes;
        cx.notify();
    }
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SidebarPanelEngineBuilder {
        SidebarPanelEngineBuilder::new(id)
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.motion_enabled = enabled;
        let definition = self.model.motion.profile().expansion.enabled(enabled);
        for motion in self.branch_transitions.values_mut() {
            motion.set_transition(definition.clone());
        }
        self.rail_submenu_lifecycle.set_motion_enabled(enabled);
        self.main_scroll.set_motion_enabled(enabled);
        if !enabled {
            self.drop_layout.cancel();
        }
        cx.notify();
    }

    pub(crate) fn from_builder(builder: SidebarPanelEngineBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let motion_enabled = builder.model.motion_enabled;
        let main_scroll = ScrollContainer::new(
            format!("{}-main-scroll", builder.model.id),
            builder.model.scrollbar_template.clone(),
            cx,
        )
        .placement(builder.model.scrollbar_placement)
        .visibility(builder.model.scrollbar_visibility)
        .auto_hide_activate(builder.model.scrollbar_auto_hide_activate);
        let scrollbar = main_scroll.scrollbar();
        let subscriptions = vec![
            cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
                if let ScrollbarEvent::Change { value } = event {
                    this.main_scroll.set_vertical_offset(*value, cx);
                }
            }),
            observe_theme_revision(cx, |_, cx| cx.notify()),
        ];

        let mut engine = Self {
            drop_layout: Default::default(),
            drag_owner: None,
            model: builder.model,
            row_handlers: RefCell::new(None),
            tooltip_theme: crate::controls::tooltip::default_tooltip_theme(),
            tooltips: HashMap::new(),
            main_scroll,
            drag_scroll_pending: Default::default(),
            hovered_node: None,
            pressed_node: None,
            row_focus_handles: HashMap::new(),
            rail_focus_handles: HashMap::new(),
            rail_node_bounds: HashMap::new(),
            open_rail_submenu: None,
            rail_submenu_open_submenu: None,
            rail_submenu_active_path: None,
            rail_submenu_lifecycle: PopupLifecycle::with_motion(motion_enabled, motion.popup.clone()),
            rail_submenu_content: None,
            rail_submenu_content_size: None,
            visible_focus_nodes: Vec::new(),
            focus_subscriptions: Vec::new(),
            focus_subscription_keys: Vec::new(),
            emitted_focused: false,
            _subscriptions: subscriptions,
            branch_transitions: HashMap::new(),
            branch_heights_px: HashMap::new(),
        };
        engine.rebuild_tooltips(cx);
        engine.sync_branch_state();
        engine
    }

    /// Bind the default tooltip look for standard navigation items.
    pub fn set_tooltip_theme(
        &mut self,
        theme: std::sync::Arc<dyn crate::controls::tooltip::TooltipTheme>,
        cx: &mut Context<Self>,
    ) {
        self.tooltip_theme = theme;
        self.rebuild_tooltips(cx);
        cx.notify();
    }

    fn rebuild_tooltips(&mut self, cx: &mut Context<Self>) {
        fn collect(
            nodes: &[NavNode],
            theme: &std::sync::Arc<dyn crate::controls::tooltip::TooltipTheme>,
            tips: &mut HashMap<SharedString, crate::infra::attachments::TooltipHandle>,
            cx: &mut gpui::App,
        ) {
            for node in nodes {
                if node.kind == super::NavNodeKind::Item
                    && node.presenter.is_none()
                    && let Some(config) = &node.tooltip
                {
                    tips.insert(
                        node.id.clone(),
                        crate::infra::attachments::TooltipHandle::new_with_theme(config.clone(), theme.clone(), cx),
                    );
                }
                collect(&node.children, theme, tips, cx);
            }
        }
        self.tooltips.clear();
        for nodes in [&self.model.header_nodes, &self.model.nodes, &self.model.footer_nodes] {
            collect(nodes, &self.tooltip_theme, &mut self.tooltips, cx);
        }
    }

    pub fn set_header_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.header_nodes = nodes.into_iter().collect();
        self.rebuild_tooltips(cx);
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_items(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.nodes = nodes.into_iter().collect();
        self.rebuild_tooltips(cx);
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_footer_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.footer_nodes = nodes.into_iter().collect();
        self.rebuild_tooltips(cx);
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::SidebarPanelTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_scrollbar_template(
        &mut self,
        template: std::sync::Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.scrollbar_template = template.clone();
        self.main_scroll.scrollbar().update(cx, |scrollbar, cx| scrollbar.set_template(template, cx));
        cx.notify();
    }

    pub fn set_selected_id(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_id = selected_id.into();
        if self.model.selected_id.as_ref() != Some(&selected_id) {
            self.model.selected_id = Some(selected_id);
            cx.notify();
        }
    }

    pub fn clear_selected_id(&mut self, cx: &mut Context<Self>) {
        if self.model.selected_id.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.title = Some(title.into());
        cx.notify();
    }

    pub fn clear_title(&mut self, cx: &mut Context<Self>) {
        self.model.title = None;
        cx.notify();
    }

    pub fn set_subtitle(&mut self, subtitle: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.subtitle = Some(subtitle.into());
        cx.notify();
    }

    pub fn clear_subtitle(&mut self, cx: &mut Context<Self>) {
        self.model.subtitle = None;
        cx.notify();
    }

    pub fn collapsed(&self) -> bool {
        self.model.collapsed
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_subscriptions.clear();
        self.focus_subscription_keys.clear();
        if !enabled {
            self.clear_hovered_node(cx);
            self.pressed_node = None;
            self.close_rail_submenu(cx);
            self.emit_focus_changed(false, cx);
        }
        cx.emit(SidebarPanelEngineEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.model.collapsed != collapsed {
            self.model.collapsed = collapsed;
            self.clear_hovered_node(cx);
            self.pressed_node = None;
            self.close_rail_submenu(cx);
            cx.emit(SidebarPanelEngineEvent::CollapsedChanged { collapsed });
            cx.notify();
        }
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        self.set_collapsed(!self.model.collapsed, cx);
    }

    pub fn set_node_expanded(&mut self, node_id: impl Into<SharedString>, expanded: bool, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if set_expanded_in_nodes(&mut self.model.nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.header_nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.footer_nodes, &node_id, expanded)
        {
            self.set_branch_target(&node_id, expanded);
            cx.emit(SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded });
            cx.notify();
        }
    }

    pub fn toggle_node_expanded(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        let Some(expanded) = toggle_expanded_in_nodes(&mut self.model.nodes, &node_id)
            .or_else(|| toggle_expanded_in_nodes(&mut self.model.header_nodes, &node_id))
            .or_else(|| toggle_expanded_in_nodes(&mut self.model.footer_nodes, &node_id))
        else {
            return;
        };

        self.set_branch_target(&node_id, expanded);
        cx.emit(SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded });
        cx.notify();
    }
}

#[cfg(all(test, feature = "test-support"))]
mod locale_tests;
