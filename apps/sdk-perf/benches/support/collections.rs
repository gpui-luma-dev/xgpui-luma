//! Public-API collection fixtures. Test platform only; no native application.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use gpui::{
    prelude::*, App, AppContext, Context, Entity, Focusable, Render, SharedString, TestAppContext, VisualTestContext,
    Window, div, point, px, size,
};
use xgpui_luma::controls::{table::*, tree_view::*};

#[derive(Clone)]
pub struct Row {
    pub id: usize,
    pub key: SharedString,
    pub label: SharedString,
    pub value: usize,
}
impl Row {
    fn new(id: usize) -> Self {
        Self { id, key: format!("row-{id}").into(), label: format!("Record {id:06} é中").into(), value: id % 97 }
    }
}
#[derive(Default)]
pub struct Paints {
    pub calls: AtomicUsize,
    pub last: AtomicUsize,
}
impl Paints {
    fn observe(&self, id: usize) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.last.store(id, Ordering::Relaxed);
    }
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Table,
    TableNatural,
    Grid,
    RichGrid,
    FixedGrid,
    FixedRichGrid,
    TreeOpen,
    TreeClosed,
}
impl Case {
    pub const ALL: [Self; 8] = [
        Self::TableNatural,
        Self::Table,
        Self::Grid,
        Self::RichGrid,
        Self::FixedGrid,
        Self::FixedRichGrid,
        Self::TreeOpen,
        Self::TreeClosed,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Table => "table-fill",
            Self::TableNatural => "table-default",
            Self::Grid => "table-grid-6",
            Self::RichGrid => "table-rich-grid-6",
            Self::FixedGrid => "table-fixed-grid-6",
            Self::FixedRichGrid => "table-fixed-rich-grid-6",
            Self::TreeOpen => "tree-open",
            Self::TreeClosed => "tree-collapsed",
        }
    }
    pub fn is_tree(self) -> bool {
        matches!(self, Self::TreeOpen | Self::TreeClosed)
    }
    pub fn phases(self) -> Vec<&'static str> {
        let mut phases = vec![
            "construct_drop",
            "first_paint",
            "warm_redraw",
            "wheel",
            "keyboard",
            "selection",
            "replace",
            "single_value_replace",
            "sort",
            "filter",
            "jump",
        ];
        if self.is_tree() {
            phases.push("expand_collapse");
        }
        if matches!(self, Self::Grid | Self::RichGrid | Self::FixedGrid | Self::FixedRichGrid) {
            phases.push("column_resize");
        }
        phases
    }
}

// Depth two below each root: at most 10 folders with 9 leaves each.
// Every node has a unique ID; count includes folders and leaves.
pub fn nodes(count: usize, expanded: bool) -> Vec<TreeNode<Row>> {
    let mut next = 0;
    let mut roots = Vec::new();
    while next < count {
        let root = Row::new(next);
        next += 1;
        let mut folders = Vec::new();
        for _ in 0..10 {
            if next == count {
                break;
            }
            let folder = Row::new(next);
            next += 1;
            let mut leaves = Vec::new();
            for _ in 0..9 {
                if next == count {
                    break;
                }
                let row = Row::new(next);
                next += 1;
                leaves.push(TreeNode::new(row.key.clone(), row.label.clone(), row));
            }
            folders.push(
                TreeNode::new(folder.key.clone(), folder.label.clone(), folder).expanded(expanded).children(leaves),
            );
        }
        roots.push(TreeNode::new(root.key.clone(), root.label.clone(), root).expanded(expanded).children(folders));
    }
    roots
}

pub enum Control {
    Table(Entity<TableControl<Row>>),
    Tree(Entity<TreeViewControl<Row>>),
}
pub struct Host {
    pub control: Option<Control>,
    source: Vec<Row>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let view = self.control.as_ref().map(|control| match control {
            Control::Table(t) => t.clone().into_any_element(),
            Control::Tree(t) => t.clone().into_any_element(),
        });
        div().w(px(1100.0)).h(px(700.0)).overflow_hidden().children(view)
    }
}
pub fn control(case: Case, count: usize, paints: Arc<Paints>, cx: &mut impl AppContext) -> Control {
    if case.is_tree() {
        let branch = paints.clone();
        let tree = TreeViewBuilder::new("collection-tree")
            .items(nodes(count, case == Case::TreeOpen))
            .motion_enabled(false)
            .require_focus_for_scroll(false)
            .branch_content(move |node, _, _| {
                branch.observe(node.data.id);
                div().child(node.label.clone())
            })
            .leaf_content(move |node, _, _| {
                paints.observe(node.data.id);
                div().child(node.label.clone())
            })
            .spawn(cx);
        tree.update(cx, |tree, cx| tree.select_node_by_id("row-0", cx));
        Control::Tree(tree)
    } else {
        let row_paints = paints.clone();
        let mut builder = TableBuilder::<Row>::new_typed("collection-table").items((0..count).map(Row::new)).row_label(
            move |row: &Row| {
                row_paints.observe(row.id);
                row.label.clone()
            },
        );
        if case != Case::TableNatural {
            builder = builder.fill_height();
        }
        if matches!(case, Case::FixedGrid | Case::FixedRichGrid) {
            builder = builder.visible_row_height(36.0_f32);
        }
        if !matches!(case, Case::Table | Case::TableNatural) {
            builder = builder
                .grid_view((0..6).map(|column| {
                    let paints = paints.clone();
                    TableColumn::fixed(format!("Column {column}"), 150.0, move |row: &Row| {
                        if column == 0 {
                            paints.observe(row.id);
                        }
                        let text = if column == 0 {
                            row.label.to_string()
                        } else {
                            format!("{}", row.value + column)
                        };
                        if matches!(case, Case::RichGrid | Case::FixedRichGrid) {
                            div()
                                .flex()
                                .gap(px(6.0))
                                .child(div().child(text))
                                .child(div().text_xs().child("Details · active"))
                        } else {
                            div().child(text)
                        }
                    })
                }))
                .column_resizing(true);
        }
        let table = builder.spawn(cx);
        table.update(cx, |table, cx| {
            table.set_row_key(|row| row.key.clone(), cx).unwrap();
        });
        Control::Table(table)
    }
}

pub fn setup(
    app: &mut TestAppContext,
    case: Case,
    count: usize,
) -> (Entity<Host>, &mut VisualTestContext, Arc<Paints>) {
    let paints = Arc::new(Paints::default());
    let (host, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        xgpui_luma::key_handling::bind_default_control_keys(cx);
        Host {
            control: Some(control(case, count, paints.clone(), cx)),
            source: if case.is_tree() {
                Vec::new()
            } else {
                (0..count).map(Row::new).collect()
            },
        }
    });
    cx.simulate_resize(size(px(1100.0), px(700.0)));
    cx.run_until_parked();
    cx.update(|window, app| match host.read(app).control.as_ref().unwrap() {
        Control::Table(t) => {
            let t = t.clone();
            t.read(app).focus_handle(app).focus(window, app);
            t.update(app, |table, cx| table.set_active_index(Some(0), cx));
        }
        Control::Tree(t) => t.read(app).focus_handle(app).focus(window, app),
    });
    cx.run_until_parked();
    assert!(paints.calls() > 0, "fixture must paint visible rows");
    (host, cx, paints)
}

pub fn operate(host: &Entity<Host>, cx: &mut VisualTestContext, phase: &str, count: usize, step: usize) {
    if phase == "wheel" {
        cx.simulate_event(gpui::ScrollEvent {
            position: point(px(100.0), px(150.0)),
            delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(if step.is_multiple_of(2) { 250.0 } else { -250.0 }))),
            ..Default::default()
        });
    } else if phase == "keyboard" {
        cx.simulate_keystrokes(if step.is_multiple_of(2) { "up" } else { "down" });
    } else if phase == "column_resize" {
        // Fixed fixture columns and the SDK theme define the actual divider.
        // GPUI debug selectors are absent in release; integration tests verify
        // these coordinates against measured geometry and the committed resize.
        let look = DefaultTableTheme::default().resolve_look(true, true, xgpui_luma::theme::ControlSize::Md);
        let start = point(
            px(look.border_width + look.padding_x + 150.0 + if step.is_multiple_of(2) { 24.0 } else { 0.0 }),
            px(look.border_width + look.padding_y + look.header_typography.line_height * 0.5),
        );
        let end = start + point(px(if step.is_multiple_of(2) { -24.0 } else { 24.0 }), px(0.0));
        cx.simulate_mouse_down(start, gpui::MouseButton::Left, Default::default());
        cx.simulate_mouse_move(start + point(px(10.0), px(0.0)), gpui::MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, gpui::MouseButton::Left, Default::default());
        cx.simulate_mouse_up(end, gpui::MouseButton::Left, Default::default());
    } else {
        cx.update(|_, app| {
            let source = if matches!(phase, "replace" | "single_value_replace" | "sort" | "filter") {
                host.read(app).source.clone()
            } else {
                Vec::new()
            };
            match host.read(app).control.as_ref().unwrap() {
                Control::Table(table) => {
                    let table = table.clone();
                    table.update(app, |table, cx| match phase {
                        "warm_redraw" => cx.notify(),
                        "selection" => table.set_selected_keys([format!("row-{}", step % count)], cx).unwrap(),
                        "jump" => table.scroll_to_row(if step.is_multiple_of(2) { 0 } else { count - 1 }, cx),
                        "replace" | "single_value_replace" | "sort" | "filter" => {
                            let mut rows = source;
                            if phase == "single_value_replace" {
                                rows[step % count].value = 1000 + step;
                            }
                            if phase == "sort" {
                                rows.sort_by_key(|row| {
                                    if step.is_multiple_of(2) {
                                        row.value
                                    } else {
                                        96 - row.value
                                    }
                                });
                            }
                            if phase == "filter" {
                                rows.retain(|row| row.id % 2 == step % 2);
                            }
                            table.set_items(rows, cx).unwrap();
                        }
                        _ => unreachable!("unknown phase"),
                    });
                }
                Control::Tree(tree) => {
                    let tree = tree.clone();
                    tree.update(app, |tree, cx| match phase {
                        "warm_redraw" => cx.notify(),
                        "selection" => {
                            tree.select_node_by_id(format!("row-{}", (step % count.div_ceil(101)) * 101), cx);
                        }
                        "jump" => {
                            assert!(tree.reveal_node(
                                format!("row-{}", if step.is_multiple_of(2) { 0 } else { count - 1 }),
                                cx
                            ));
                        }
                        "expand_collapse" => tree.toggle_expand("row-0", cx),
                        "replace" | "single_value_replace" => {
                            let mut items = tree.items().to_vec();
                            if phase == "single_value_replace" {
                                items[0].data.value = step;
                                items[0].label = format!("Changed {step}").into();
                            }
                            tree.replace_items(items, cx).unwrap();
                        }
                        "sort" => tree.set_sort(
                            move |a, b| {
                                if step.is_multiple_of(2) {
                                    a.data.value.cmp(&b.data.value)
                                } else {
                                    b.data.value.cmp(&a.data.value)
                                }
                            },
                            cx,
                        ),
                        "filter" => tree.set_filter(move |node| node.data.id % 10 == step % 10, cx),
                        _ => unreachable!("unknown phase"),
                    });
                }
            }
        });
    }
    cx.run_until_parked();
    // Keyed positioning completes against the next frame's measured bounds.
    if phase == "jump" {
        for _ in 0..3 {
            cx.update(|window, app| window.simulate_next_frame(app));
            cx.run_until_parked();
        }
    }
}

pub fn released(host: &Entity<Host>, cx: &mut VisualTestContext) -> bool {
    let probe: Box<dyn Fn(&App) -> bool> = cx.update(|_, app| match host.read(app).control.as_ref().unwrap() {
        Control::Table(t) => {
            let weak = t.downgrade();
            Box::new(move |_: &App| weak.upgrade().is_none()) as Box<dyn Fn(&App) -> bool>
        }
        Control::Tree(t) => {
            let weak = t.downgrade();
            Box::new(move |_: &App| weak.upgrade().is_none())
        }
    });
    host.update(cx, |host, cx| {
        host.control = None;
        host.source = Vec::new();
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, app| window.simulate_next_frame(app));
    }
    cx.run_until_parked();
    cx.update(|_, app| probe(app))
}
