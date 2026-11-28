use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Window};

use super::engine::{SidebarPanelEngine, SidebarPanelEngineEvent, SidebarPanelTemplate};
use super::model::{
    SidebarBuilder, SidebarControlBuilder, SidebarControlModel, SidebarPanelModel, find_active_id, panel_to_nav_nodes,
};
use super::theme::SidebarPresentation;
use crate::controls::scrollbar::ScrollbarTemplate;
use crate::theme::observe_theme_revision;

/// Semantic events emitted by [`SidebarControl`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SidebarEvent {
    /// Shared drag lifecycle with Sidebar destinations.
    DragDrop(super::SidebarDragEvent),
    /// Navigation presentation changed; host pane visibility is independent.
    PresentationChanged { presentation: SidebarPresentation },
    /// Fired when an item or sub-item is selected.
    Select { id: SharedString },

    /// Fired when keyboard or pointer focus moves to an item.
    ItemFocused { id: SharedString },

    /// Fired when item hover state changes (used for collapsed popover tracking).
    HoverChanged { id: Option<SharedString> },

    /// Fired when the primary button goes down (`pressed`) or up on an item.
    ItemPressedChanged { id: SharedString, pressed: bool },

    /// Fired when keyboard focus enters (`focused`) or leaves the sidebar.
    FocusChanged { focused: bool },

    /// Fired when a sub-menu branch expands or collapses.
    SubMenuToggle { id: SharedString, open: bool },

    /// Fired when sidebar enabled state changes.
    EnabledChanged { enabled: bool },
}

/// Bare navigation. The host owns pane geometry; use a Frame for container styling.
pub struct SidebarControl {
    pub(super) model: SidebarControlModel,
    pub(super) panel: Entity<SidebarPanelEngine>,
    pub(super) active_drag: Option<super::drag::SidebarDrag>,
    pub(super) drop_handler: Option<super::drag::DropHandler>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SidebarEvent> for SidebarControl {}

impl SidebarControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SidebarControlBuilder {
        SidebarControlBuilder::new(id)
    }

    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn motion_enabled(&self) -> bool {
        self.model.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.motion_enabled = enabled;
        self.panel.update(cx, |engine, cx| engine.set_motion_enabled(enabled, cx));
        cx.notify();
    }
    crate::motion_snapshot!(super::SidebarMotion, model.motion);

    pub(crate) fn from_builder(builder: SidebarControlBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let mut model = builder.model;

        if model.selected_id.is_none()
            && let Some(panel) = &model.sidebar
        {
            model.selected_id = find_active_id(panel);
        }

        let panel = spawn_panel_engine(&model, cx);
        let owner = cx.weak_entity();
        panel.update(cx, |panel, _| panel.drag_owner = Some(owner));

        let mut subscriptions = vec![observe_theme_revision(cx, |_, cx| cx.notify())];
        subscriptions.push(cx.subscribe(&panel, |this, _, event: &SidebarPanelEngineEvent, cx| {
            this.handle_panel_engine_event(event, cx);
        }));

        Self { model, panel, active_drag: None, drop_handler: None, _subscriptions: subscriptions }
    }

    /// Bind the default theme for per-item tooltip attachments.
    pub fn set_tooltip_theme(
        &mut self,
        theme: Arc<dyn crate::controls::tooltip::TooltipTheme>,
        cx: &mut Context<Self>,
    ) {
        self.panel.update(cx, |panel, cx| panel.set_tooltip_theme(theme, cx));
    }

    pub fn presentation(&self) -> SidebarPresentation {
        self.model.presentation
    }

    pub fn set_presentation(&mut self, presentation: SidebarPresentation, cx: &mut Context<Self>) {
        if self.model.presentation == presentation {
            return;
        }
        self.panel.read(cx).drop_layout.cancel();
        self.cancel_drag(cx);
        self.model.presentation = presentation;
        self.panel
            .update(cx, |panel, cx| panel.set_collapsed(presentation == SidebarPresentation::Icons, cx));
        cx.emit(SidebarEvent::PresentationChanged { presentation });
        cx.notify();
    }

    /// Draw each row in the state `map` gives it, from its id and the state input gives it (`selected` is the current
    /// page), or as input gives it (`None`). Only the look changes; selection, input and events follow the real state
    /// (#397).
    pub fn set_item_state_override(
        &mut self,
        map: Option<crate::infra::state::ItemStateOverride<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.panel.update(cx, |panel, cx| panel.set_item_state_override(map, cx));
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        if !enabled {
            self.panel.read(cx).drop_layout.cancel();
            self.cancel_drag(cx);
        }
        self.model.enabled = enabled;
        self.panel.update(cx, |panel, cx| panel.set_enabled(enabled, cx));
        cx.emit(SidebarEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_selected_id(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_id = selected_id.into();
        self.model.selected_id = Some(selected_id.clone());
        self.panel.update(cx, |panel, cx| panel.set_selected_id(selected_id, cx));
        cx.notify();
    }

    pub fn set_panel_template(&mut self, template: Arc<dyn SidebarPanelTemplate>, cx: &mut Context<Self>) {
        self.model.panel_template = template.clone();
        self.panel.update(cx, |panel, cx| panel.set_template(template, cx));
    }

    pub fn set_scrollbar_template(&mut self, template: Arc<dyn ScrollbarTemplate>, cx: &mut Context<Self>) {
        self.model.scrollbar_template = template.clone();
        self.panel.update(cx, |panel, cx| panel.set_scrollbar_template(template, cx));
    }

    /// Replace the sidebar panel tree (header / groups / footer) at runtime.
    pub fn set_sidebar(&mut self, sidebar: SidebarBuilder, cx: &mut Context<Self>) {
        self.panel.read(cx).drop_layout.cancel();
        self.cancel_drag(cx);
        let panel = sidebar.build();
        self.apply_panel_model(panel, cx);
    }

    fn apply_panel_model(&mut self, panel: SidebarPanelModel, cx: &mut Context<Self>) {
        let (title, subtitle) = panel
            .header
            .as_ref()
            .map(|header| (header.title.clone(), header.subtitle.clone()))
            .unwrap_or((None, None));
        let (nodes, footer_nodes) = panel_to_nav_nodes(&panel);
        if self.model.selected_id.is_none() {
            self.model.selected_id = find_active_id(&panel);
        }
        let selected_id = self.model.selected_id.clone();
        self.model.sidebar = Some(panel);

        self.panel.update(cx, |engine, cx| {
            match title {
                Some(title) => engine.set_title(title, cx),
                None => engine.clear_title(cx),
            }
            match subtitle {
                Some(subtitle) => engine.set_subtitle(subtitle, cx),
                None => engine.clear_subtitle(cx),
            }
            engine.set_items(nodes, cx);
            engine.set_footer_nodes(footer_nodes, cx);
            if let Some(selected_id) = selected_id {
                engine.set_selected_id(selected_id, cx);
            }
        });
        cx.notify();
    }

    fn handle_panel_engine_event(&mut self, event: &SidebarPanelEngineEvent, cx: &mut Context<Self>) {
        match event {
            SidebarPanelEngineEvent::Activate { node_id, .. } => {
                self.model.selected_id = Some(node_id.clone());
                cx.emit(SidebarEvent::Select { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded } => {
                if let Some(panel) = self.model.sidebar.as_mut() {
                    super::drag::set_expanded(panel, node_id, *expanded);
                }
                cx.emit(SidebarEvent::SubMenuToggle { id: node_id.clone(), open: *expanded });
            }
            SidebarPanelEngineEvent::CollapsedChanged { .. } => {}
            SidebarPanelEngineEvent::ItemFocused { node_id, .. } => {
                cx.emit(SidebarEvent::ItemFocused { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::ItemHoverChanged { node_id, hovered } => {
                let id = if *hovered { Some(node_id.clone()) } else { None };
                cx.emit(SidebarEvent::HoverChanged { id });
            }
            SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id } => {
                cx.emit(SidebarEvent::HoverChanged { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::EnabledChanged { enabled } => {
                if self.model.enabled != *enabled {
                    self.model.enabled = *enabled;
                    cx.emit(SidebarEvent::EnabledChanged { enabled: *enabled });
                }
            }
            SidebarPanelEngineEvent::FocusChanged { focused } => {
                cx.emit(SidebarEvent::FocusChanged { focused: *focused });
            }
            SidebarPanelEngineEvent::ItemPressedChanged { node_id, pressed } => {
                cx.emit(SidebarEvent::ItemPressedChanged { id: node_id.clone(), pressed: *pressed });
            }
        }
    }
}

impl Render for SidebarControl {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.panel.clone()
    }
}

fn spawn_panel_engine(model: &SidebarControlModel, cx: &mut Context<SidebarControl>) -> Entity<SidebarPanelEngine> {
    let panel_model = model.sidebar.clone();
    let panel_id = panel_model
        .as_ref()
        .map(|panel| panel.id.clone())
        .unwrap_or_else(|| SharedString::from(format!("{}-panel", model.id)));

    let (title, subtitle) = panel_model
        .as_ref()
        .and_then(|panel| panel.header.as_ref())
        .map(|header| (header.title.clone(), header.subtitle.clone()))
        .unwrap_or((None, None));

    let (nodes, footer_nodes) = panel_model.as_ref().map(panel_to_nav_nodes).unwrap_or_default();

    let mut builder = SidebarPanelEngine::new(panel_id)
        .enabled(model.enabled)
        .motion_enabled(model.motion_enabled)
        .rail_dismiss(model.rail_dismiss)
        .motion_defaults(model.motion.profile().clone())
        .disclosure_icons(model.resolved_disclosure_icons())
        .collapsed(model.presentation == SidebarPresentation::Icons)
        .template(model.panel_template.clone())
        .scrollbar_template(model.scrollbar_template.clone())
        .scrollbar_placement(model.scrollbar_placement)
        .scrollbar_visibility(model.scrollbar_visibility)
        .scrollbar_auto_hide_activate(model.scrollbar_auto_hide_activate)
        .items(nodes)
        .footer_nodes(footer_nodes);

    if let Some(title) = title {
        builder = builder.title(title);
    }
    if let Some(subtitle) = subtitle {
        builder = builder.subtitle(subtitle);
    }
    if let Some(selected_id) = &model.selected_id {
        builder = builder.selected_id(selected_id.clone());
    }

    builder.spawn(cx)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, point, prelude::*, px};
    use crate::controls::sidebar::{sidebar, sidebar_content, sidebar_group, sidebar_menu, sidebar_menu_item};
    use lucide_svg_static::Icon;

    #[test]
    fn bare_navigation_preserves_selection_across_presentations() {
        let mut app = TestAppContext::single();
        let (sidebar, cx) = app.add_window_view(|_, cx| {
            SidebarControl::from_builder(
                SidebarControl::new("nav")
                    .motion_enabled(false)
                    .with_panel_template_modifier(|mut root| {
                        assert!(root.style().background.is_none());
                        assert!(root.style().padding.left.is_none());
                        assert!(root.style().corner_radii.top_left.is_none());
                        root.debug_selector(|| "bare-nav".into())
                    })
                    .sidebar(
                        sidebar("content").content(
                            sidebar_content().group(
                                sidebar_group().menu(
                                    sidebar_menu("main")
                                        .item(sidebar_menu_item("first", "First").icon(Icon::House))
                                        .item(sidebar_menu_item("second", "Second").icon(Icon::Folder).active(true)),
                                ),
                            ),
                        ),
                    ),
                cx,
            )
        });
        cx.run_until_parked();
        let bounds = cx.debug_bounds("bare-nav").unwrap();
        cx.simulate_click(point(bounds.left() + px(40.0), bounds.top() + px(15.0)), Default::default());
        cx.run_until_parked();
        for presentation in [SidebarPresentation::Icons, SidebarPresentation::Expanded] {
            sidebar.update(cx, |sidebar, cx| sidebar.set_presentation(presentation, cx));
            cx.run_until_parked();
            cx.update(|_, app| {
                let sidebar = sidebar.read(app);
                assert_eq!(sidebar.presentation(), presentation);
                assert_eq!(sidebar.model.selected_id.as_deref(), Some("first"));
                assert_eq!(sidebar.panel.read(app).collapsed(), presentation == SidebarPresentation::Icons);
            });
        }
    }
    #[test]
    fn item_help_preserves_navigation_and_once_history_between_presentations() {
        use crate::controls::tooltip::{Tooltip, TooltipEvent};
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };
        let events = Arc::new(Mutex::new(Vec::new()));
        let log = events.clone();
        let mut app = TestAppContext::single();
        let (sidebar, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            SidebarControl::from_builder(
                SidebarControl::new("nav")
                    .motion_enabled(false)
                    .with_panel_template_modifier(|root| root.debug_selector(|| "help-nav".into()))
                    .sidebar(
                        sidebar("content").content(
                            sidebar_content().group(
                                sidebar_group().menu(
                                    sidebar_menu("main").item(
                                        sidebar_menu_item("first", "First").icon(Icon::House).tooltip(
                                            Tooltip::new("First help")
                                                .delay(Duration::ZERO)
                                                .show_once()
                                                .on_event(move |event| log.lock().unwrap().push(event)),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                cx,
            )
        });
        cx.run_until_parked();
        let bounds = cx.debug_bounds("help-nav").unwrap();
        let position = point(bounds.left() + px(20.0), bounds.top() + px(15.0));
        cx.simulate_event(gpui::MouseMoveEvent { position, ..Default::default() });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        cx.simulate_click(position, Default::default());
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown, TooltipEvent::Hidden]);
        sidebar.update(cx, |sidebar, cx| sidebar.set_presentation(SidebarPresentation::Icons, cx));
        cx.run_until_parked();
        cx.simulate_event(gpui::MouseMoveEvent { position, ..Default::default() });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().len(), 2);
        cx.update(|_, app| assert_eq!(sidebar.read(app).model.selected_id.as_deref(), Some("first")));
    }
}
