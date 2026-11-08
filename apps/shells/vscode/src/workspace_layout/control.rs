use xgpui_luma::controls::dock_splitter::DockSplitterEventExt;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, EventEmitter, Pixels, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterEvent, SplitterOrientation, ThemedDockSplitterTemplate,
};
use xgpui_luma::controls::resizable_panels::{PanelHideMode, PanelId, ResizablePanels, ResizablePanelsTheme};
use xgpui_luma_look_shadcn::{ShadcnLook};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::layout_config::{LayoutConfig, PanelAlignment, PrimarySideBarPosition};
use crate::workbench_layout::{
    LEFT_EDGE_PANEL_INDEX, PRIMARY_SIDE_BAR_REGION, PrimarySideBar, RIGHT_EDGE_PANEL_INDEX, SECONDARY_SIDE_BAR_REGION,
    SecondarySideBar, WORKBENCH_PANEL_H, WorkbenchLayout,
};
use crate::workspace_layout::template::{
    ACTIVITY_BAR_W, ActivityRailEdge, ActivityRailSpec, ExternalPanelGeometry, activity_bar_rail, floating_output_panel,
};

pub struct WorkspaceLayout {
    look: Arc<ShadcnLook>,
    config: LayoutConfig,
    frame_width: Pixels,
    frame_height: Pixels,
    panel_height_px: f32,
    drag_start_panel_height_px: f32,
    workbench: WorkbenchLayout,
    panel_splitter: Entity<DockSplitter>,
    activity_bar_buttons: [IconButton; 5],
    secondary_activity_bar_buttons: [IconButton; 3],
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum WorkspaceLayoutEvent {
    PanelHeightChanged { height_px: f32 },
}

impl EventEmitter<WorkspaceLayoutEvent> for WorkspaceLayout {}

impl WorkspaceLayout {
    pub fn new(
        id: impl Into<SharedString>,
        look: Arc<ShadcnLook>,
        primary_side_bar: PrimarySideBar,
        secondary_side_bar: SecondarySideBar,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        let panel_splitter = DockSplitter::new(format!("{id}-panel-splitter"), SplitterOrientation::Horizontal)
            .template(Arc::new(ThemedDockSplitterTemplate::new(false)))
            .theme(look.dock_splitter_theme())
            .spawn(cx);
        let workbench = WorkbenchLayout::new(id.clone(), look.clone(), primary_side_bar, secondary_side_bar, cx);
        let center_panel_splitter = panel_splitter.clone();
        workbench.set_panel_splitter(Rc::new(move || center_panel_splitter.clone().into_any_element()), cx);
        let activity_bar_buttons = spawn_activity_rail_buttons(
            "shell-vscode-activity-bar",
            [LucideIcon::Files, LucideIcon::Search, LucideIcon::GitBranch, LucideIcon::Bug, LucideIcon::Blocks],
            &look,
            cx,
        );
        let secondary_activity_bar_buttons = spawn_activity_rail_buttons(
            "shell-vscode-secondary-activity-bar",
            [LucideIcon::User, LucideIcon::Bell, LucideIcon::Settings],
            &look,
            cx,
        );

        let this = Self {
            look,
            config: LayoutConfig::default(),
            frame_width: px(1.0),
            frame_height: px(1.0),
            panel_height_px: WORKBENCH_PANEL_H,
            drag_start_panel_height_px: WORKBENCH_PANEL_H,
            workbench,
            panel_splitter,
            activity_bar_buttons,
            secondary_activity_bar_buttons,
        };

        this.panel_splitter.clone().on_event(cx, |this, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start_panel_height_px = this.panel_height_px,
            DockSplitterEvent::Resize { total_delta } => {
                this.panel_height_px = this.clamp_panel_height(this.drag_start_panel_height_px - *total_delta);
                this.workbench.set_panel_height(px(this.panel_height_px), cx);
                cx.emit(WorkspaceLayoutEvent::PanelHeightChanged { height_px: this.panel_height_px });
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
            _ => {}
        });

        this
    }

    pub fn panels(&self) -> Entity<ResizablePanels> {
        self.workbench.panels()
    }

    pub fn set_frame_size(&mut self, width: Pixels, height: Pixels, cx: &mut Context<Self>) {
        if self.frame_width == width && self.frame_height == height {
            return;
        }
        self.frame_width = width;
        self.frame_height = height;
        // The first layout pass starts with a provisional 1px frame, so a
        // complete panel collapse can fail before the real viewport exists.
        // Reconcile sidebar visibility once the measured frame is available.
        self.sync_panel_visibility(&PanelId::new(PRIMARY_SIDE_BAR_REGION), self.config.primary_side_bar_visible, cx);
        self.sync_panel_visibility(
            &PanelId::new(SECONDARY_SIDE_BAR_REGION),
            self.config.secondary_side_bar_visible,
            cx,
        );
        cx.notify();
    }

    pub fn set_activity_bar_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.activity_bar_visible = visible;
        cx.notify();
    }

    pub fn set_secondary_activity_bar_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.secondary_activity_bar_visible = visible;
        cx.notify();
    }

    pub fn set_primary_side_bar_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.primary_side_bar_visible = visible;
        self.sync_panel_visibility(&PanelId::new(PRIMARY_SIDE_BAR_REGION), visible, cx);
        cx.notify();
    }

    pub fn set_secondary_side_bar_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.secondary_side_bar_visible = visible;
        self.sync_panel_visibility(&PanelId::new(SECONDARY_SIDE_BAR_REGION), visible, cx);
        cx.notify();
    }

    pub fn set_panel_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.panel_visible = visible;
        self.workbench.set_panel_layout(visible, self.config.panel_alignment, cx);
        self.panel_splitter.update(cx, |splitter, cx| splitter.set_enabled(visible, cx));
        cx.notify();
    }

    pub fn set_status_bar_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.config.status_bar_visible = visible;
        cx.notify();
    }

    pub fn set_primary_side_bar_position(&mut self, position: PrimarySideBarPosition, cx: &mut Context<Self>) {
        self.config.primary_side_bar_position = position;
        self.workbench.set_primary_side_bar_position(position, cx);
        cx.notify();
    }

    pub fn set_panel_alignment(&mut self, alignment: PanelAlignment, cx: &mut Context<Self>) {
        self.config.panel_alignment = alignment;
        self.workbench.set_panel_layout(self.config.panel_visible, alignment, cx);
        cx.notify();
    }

    pub fn set_panel_height(&mut self, height_px: f32, cx: &mut Context<Self>) {
        self.panel_height_px = self.clamp_panel_height(height_px);
        self.config.panel_height_px = self.panel_height_px;
        self.workbench.set_panel_height(px(self.panel_height_px), cx);
        cx.notify();
    }

    pub fn sync_chrome(&self, cx: &mut Context<Self>) {
        self.workbench.sync_chrome(&self.look, cx);
    }

    pub fn sync_theme(&self, theme: &Arc<dyn ResizablePanelsTheme>, cx: &mut Context<Self>) {
        self.workbench.sync_theme(theme, cx);
        self.panel_splitter.update(cx, |splitter, cx| {
            splitter.set_theme(self.look.dock_splitter_theme(), cx);
        });
    }

    fn sync_panel_visibility(&self, region: &PanelId, visible: bool, cx: &mut Context<Self>) {
        self.workbench.panels().update(cx, |panels, cx| {
            let Some(hidden) = panels.is_region_hidden(region) else {
                return;
            };
            if hidden == visible {
                panels.toggle_region_hidden(region, PanelHideMode::Completely, cx);
            }
        });
    }

    fn clamp_panel_height(&self, height_px: f32) -> f32 {
        let max_height = (self.frame_height.as_f32() - 120.0).max(120.0);
        height_px.clamp(96.0, max_height)
    }

    fn panel_splitter_element(&self) -> AnyElement {
        self.panel_splitter.clone().into_any_element()
    }
}

impl Render for WorkspaceLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel_visible = self.config.panel_visible;
        let activity_bar_visible = self.config.activity_bar_visible;
        let secondary_activity_bar_visible = self.config.secondary_activity_bar_visible;
        let (left_activity_rail, right_activity_rail) = match self.config.primary_side_bar_position {
            PrimarySideBarPosition::Left => (
                ActivityRailSpec {
                    id: "activity-bar",
                    visible: activity_bar_visible,
                    buttons: &self.activity_bar_buttons,
                },
                ActivityRailSpec {
                    id: "secondary-activity-bar",
                    visible: secondary_activity_bar_visible,
                    buttons: &self.secondary_activity_bar_buttons,
                },
            ),
            PrimarySideBarPosition::Right => (
                ActivityRailSpec {
                    id: "secondary-activity-bar",
                    visible: secondary_activity_bar_visible,
                    buttons: &self.secondary_activity_bar_buttons,
                },
                ActivityRailSpec {
                    id: "activity-bar",
                    visible: activity_bar_visible,
                    buttons: &self.activity_bar_buttons,
                },
            ),
        };
        let rails_width = [activity_bar_visible, secondary_activity_bar_visible]
            .into_iter()
            .filter(|visible| *visible)
            .count() as f32
            * ACTIVITY_BAR_W;
        let workbench_width = px((self.frame_width.as_f32() - rails_width).max(1.0));
        let floating_panel_visible = panel_visible
            && matches!(
                self.config.panel_alignment,
                PanelAlignment::Left | PanelAlignment::Right | PanelAlignment::Justify
            );
        self.panel_height_px = self.clamp_panel_height(self.panel_height_px);
        self.workbench.set_panel_height(px(self.panel_height_px), cx);
        let workbench_height = px(self.frame_height.as_f32().max(1.0));
        self.workbench.set_panel_layout(panel_visible, self.config.panel_alignment, cx);
        self.workbench.set_frame_size(workbench_width, workbench_height, cx);
        let panel_sizes = self.workbench.panel_sizes_px(cx);
        let left_edge_width = panel_sizes.get(LEFT_EDGE_PANEL_INDEX).copied().unwrap_or(0.0);
        let right_edge_width = panel_sizes.get(RIGHT_EDGE_PANEL_INDEX).copied().unwrap_or(0.0);
        let left_rail_width = if left_activity_rail.visible {
            ACTIVITY_BAR_W
        } else {
            0.0
        };
        let workbench_frame_width = workbench_width.as_f32();
        let external_panel_geometry = ExternalPanelGeometry {
            left_offset: px(left_rail_width),
            left_span_width: px((workbench_frame_width - right_edge_width).max(1.0)),
            right_offset: px(left_rail_width + left_edge_width),
            right_span_width: px((workbench_frame_width - left_edge_width).max(1.0)),
            justify_width: px(workbench_frame_width.max(1.0)),
        };

        div().id("workspace-layout").size_full().min_h_0().flex().flex_col().child(
            div()
                .id("workbench-body")
                .relative()
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_row()
                .overflow_hidden()
                .when(left_activity_rail.visible, |body| {
                    body.child(activity_bar_rail(
                        left_activity_rail.id,
                        &self.look,
                        left_activity_rail.buttons,
                        ActivityRailEdge::Left,
                    ))
                })
                .child(div().flex_1().min_w_0().min_h_0().child(self.workbench.panels()))
                .when(right_activity_rail.visible, |body| {
                    body.child(activity_bar_rail(
                        right_activity_rail.id,
                        &self.look,
                        right_activity_rail.buttons,
                        ActivityRailEdge::Right,
                    ))
                })
                .when(floating_panel_visible, |body| {
                    body.child(floating_output_panel(
                        &self.look,
                        self.config.panel_alignment,
                        external_panel_geometry,
                        self.panel_height_px,
                        self.panel_splitter_element(),
                    ))
                }),
        )
    }
}

fn spawn_activity_rail_buttons<const N: usize>(
    prefix: &'static str,
    icons: [LucideIcon; N],
    look: &Arc<ShadcnLook>,
    cx: &mut Context<WorkspaceLayout>,
) -> [IconButton; N] {
    std::array::from_fn(|index| {
        shadcn::Button::icon_button(format!("{prefix}-{index}"), icons[index])
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Md)
            .spawn(cx)
    })
}
