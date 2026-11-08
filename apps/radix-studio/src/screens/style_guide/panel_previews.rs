//! Static, bounded SDK panel surfaces; no live example controls or window overlays.
use std::sync::Arc;
use gpui::{AnyElement, App, Context, Entity, FocusHandle, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::{resizable_panels::*, slide_panel::*, tabs::Tabs};
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_radix::{self as radix, Accent, Gray, Look, ScaleFamily, SemanticRole};
use super::{matrix_grid::*, preview_handlers, states};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Resizable,
    Slide,
}
impl Kind {
    fn key(self) -> &'static str {
        match self {
            Self::Resizable => "resizable-panels",
            Self::Slide => "slide-panel",
        }
    }
    fn rows(self) -> &'static [&'static str] {
        match self {
            Self::Resizable => &["Horizontal", "Vertical"],
            Self::Slide => &["Left", "Right", "Top", "Bottom"],
        }
    }
}
#[derive(Clone)]
pub(super) struct Previews {
    tabs: [Entity<Tabs>; 2],
    focus: FocusHandle,
}
impl Previews {
    pub fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        Self {
            tabs: [Kind::Resizable, Kind::Slide].map(|kind| super::state::preview_navigation(kind.key(), look, cx)),
            focus: cx.focus_handle(),
        }
    }
    pub fn render(&self, kind: Kind, look: &Arc<Look>, _window: &mut Window, cx: &mut App) -> AnyElement {
        let tabs = self.tabs[usize::from(kind == Kind::Slide)].clone();
        let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
        let fg = look.resolve_role(SemanticRole::Foreground).hsla();
        let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
        let border = look.resolve_role(SemanticRole::Border).hsla();
        let states = states::samples();
        let columns = match active.as_ref() {
            "colors" => kind.rows().len(),
            "all-sizes" => 3,
            _ if kind == Kind::Slide => 3,
            _ => states.len(),
        };
        let mut grid = fixed_grid(equal_data_columns(104.0, 160.0, columns), 12.0, 24.0).child(empty_corner(), 0, 0);
        match active.as_ref() {
            "colors" => {
                for (col, label) in kind.rows().iter().enumerate() {
                    grid = grid.child(column_header(*label, muted), 0, col + 1);
                }
                for (row, accent) in Accent::ALL.into_iter().enumerate() {
                    let fork = look.fork();
                    fork.set_palettes(accent, Gray::Auto);
                    grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), fg), row + 1, 0);
                    for variant in 0..kind.rows().len() {
                        grid = grid.child(
                            self.sample(
                                &fork,
                                Cell {
                                    kind,
                                    variant,
                                    size: ControlSize::Md,
                                    state: InteractionState { hovered: true, ..Default::default() },
                                    id: format!("color-{row}-{variant}"),
                                },
                            ),
                            row + 1,
                            variant + 1,
                        );
                    }
                }
            }
            _ => {
                for col in 0..columns {
                    let header = if active.as_ref() == "all-sizes" {
                        column_header(["Small", "Medium", "Large"][col], muted)
                    } else if kind == Kind::Slide {
                        column_header(["Open", "Hover", "Resizing"][col], muted)
                    } else {
                        states::header_cell(&states[col], muted)
                    };
                    grid = grid.child(header, 0, col + 1);
                }
                for (row, label) in kind.rows().iter().enumerate() {
                    grid = grid.child(row_label(*label, fg), row + 1, 0);
                    for col in 0..columns {
                        let sizing = active.as_ref() == "all-sizes";
                        let state = if sizing {
                            InteractionState { hovered: true, ..Default::default() }
                        } else if kind == Kind::Slide {
                            InteractionState { hovered: col == 1, pressed: col == 2, ..Default::default() }
                        } else {
                            states[col].state
                        };
                        grid = grid.child(
                            self.sample(
                                look,
                                Cell {
                                    kind,
                                    variant: row,
                                    size: if sizing {
                                        [ControlSize::Sm, ControlSize::Md, ControlSize::Lg][col]
                                    } else {
                                        ControlSize::Md
                                    },
                                    state,
                                    id: format!("{}-{row}-{col}", if sizing { "size" } else { "state" }),
                                },
                            ),
                            row + 1,
                            col + 1,
                        );
                    }
                }
            }
        }
        preview_tabbed(tabs, border, grid.into_any_element())
    }
    fn sample(&self, look: &Look, cell: Cell) -> AnyElement {
        let selector = format!("guide-{}-{}", cell.kind.key(), cell.id);
        let id = SharedString::from(selector.clone());
        let fg = look.resolve_role(SemanticRole::Foreground).hsla();
        let canvas = look.resolve_step(ScaleFamily::Gray, 2).hsla();
        let content = if cell.kind == Kind::Resizable {
            panels(look, &id, &cell)
        } else {
            let edge = [SlidePanelEdge::Left, SlidePanelEdge::Right, SlidePanelEdge::Top, SlidePanelEdge::Bottom]
                [cell.variant];
            let extent = match cell.size {
                ControlSize::Sm => 0.4,
                ControlSize::Md => 0.5,
                ControlSize::Lg => 0.65,
            };
            let size = if edge.is_horizontal_main_axis() {
                160.0 * extent
            } else {
                96.0 * extent
            };
            let mut state = radix::slide_panel_state(
                look,
                SlidePanelTopAnchor::WindowEdge,
                SlidePanelSizeConfig::new(size, 16.0, 160.0),
            )
            .motion_enabled(false);
            state.open(edge, self.focus.clone());
            state.set_resize_handle_hovered(cell.state.hovered);
            if cell.state.pressed {
                state.begin_resize(gpui::point(px(0.0), px(0.0)), edge);
            }
            let content_selector = format!("{id}-content");
            radix::render_slide_panel_surface(
                look,
                &state,
                gpui::size(px(160.0), px(96.0)),
                div()
                    .size_full()
                    .debug_selector(move || content_selector.clone())
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(fg)
                    .text_xs()
                    .child("Panel")
                    .into_any_element(),
                preview_handlers::slide_panel_handlers(),
                preview_handlers::slide_panel_resize_handlers(),
            )
        };
        div()
            .id(id)
            .debug_selector(move || selector.clone())
            .relative()
            .w(px(160.0))
            .h(px(96.0))
            .flex_none()
            .bg(canvas)
            .child(content)
            .into_any_element()
    }
}
struct Cell {
    kind: Kind,
    variant: usize,
    size: ControlSize,
    state: InteractionState,
    id: String,
}
fn panels(look: &Look, id: &SharedString, cell: &Cell) -> AnyElement {
    let theme = radix::resizable_panels_theme(look);
    let resolved = theme.resolve(cell.state);
    let size = match cell.size {
        ControlSize::Sm => ResizeHandleSize::Sm,
        ControlSize::Md => ResizeHandleSize::Md,
        ControlSize::Lg => ResizeHandleSize::Lg,
    };
    let orientation = if cell.variant == 0 {
        ResizablePanelsOrientation::Horizontal
    } else {
        ResizablePanelsOrientation::Vertical
    };
    let fg = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let panes = ["A", "B"].map(|label| {
        ResizablePanelSpec::new_render(move || {
            div().size_full().flex().items_center().justify_center().text_xs().text_color(fg).child(label)
        })
    });
    let extent = if cell.variant == 0 { 160.0 } else { 96.0 };
    let seam = (extent - 2.0 * resolved.border_width) / 2.0;
    let sizes = [seam, seam];
    let model = ResizablePanelsRenderModel {
        id,
        orientation,
        frame_width: Some(px(160.0)),
        frame_height: Some(px(96.0)),
        show_border: true,
        enabled: !cell.state.disabled,
        handle_visibility: ResizeHandleVisibility::Hover,
        double_click_collapse: None,
        hovered_handle: cell.state.hovered.then_some(0),
        dragging_handle: cell.state.pressed.then_some(0),
        resize_handle: size,
        handle_metrics: theme.resize_handle_metrics(size),
        handle_grip: true,
        panel_sizes_px: &sizes,
        panel_hidden: vec![false; 2],
        panels: &panes,
        measured_size: None,
    };
    let handle_selector = format!("{id}-handle");
    let handle = render_resizable_panels_handle(0, &model, &resolved, seam, cell.state.focused)
        .debug_selector(move || handle_selector.clone())
        .into_any_element();
    render_resizable_panels_surface(&model, &resolved, [handle]).into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext};
    use xgpui_luma::theme::ThemeMode;
    struct Harness {
        look: Arc<Look>,
        previews: Previews,
        kind: Kind,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.previews.render(self.kind, &self.look, window, cx)
        }
    }
    #[test]
    fn panel_tabs_render_in_both_modes_inside_their_frames() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(Look::built_in());
            Harness { previews: Previews::new(&look, cx), look, kind: Kind::Resizable }
        });
        cx.simulate_resize(gpui::size(px(1040.0), px(900.0)));
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for (index, kind) in [Kind::Resizable, Kind::Slide].into_iter().enumerate() {
                for tab in ["colors", "all-sizes", "template-preview"] {
                    cx.update(|_, app| {
                        view.update(app, |view, cx| {
                            view.look.set_mode(mode);
                            view.kind = kind;
                            view.previews.tabs[index].update(cx, |tabs, cx| tabs.set_active(tab, cx));
                            cx.notify();
                        })
                    });
                    cx.run_until_parked();
                }
                let (first_id, last_id) = if kind == Kind::Resizable {
                    ("guide-resizable-panels-state-0-0", "guide-resizable-panels-state-1-4")
                } else {
                    ("guide-slide-panel-state-0-0", "guide-slide-panel-state-3-2")
                };
                let first = cx.debug_bounds(first_id).unwrap();
                let last = cx.debug_bounds(last_id).unwrap();
                assert_eq!(first.size, gpui::size(px(160.0), px(96.0)));
                assert!(first.left() >= px(0.0));
                assert!(last.right() <= px(1040.0));
                assert!(last.bottom() <= px(900.0));
                if kind == Kind::Slide {
                    let content = cx.debug_bounds("guide-slide-panel-state-0-0-content").unwrap();
                    assert_eq!(content.size.width, px(80.0));
                    assert_eq!(content.left(), first.left());
                } else {
                    let handle = cx.debug_bounds("guide-resizable-panels-state-0-1-handle").unwrap();
                    assert!(handle.size.width < px(20.0));
                    assert!(handle.size.height <= px(96.0));
                }
            }
        }
    }
}
