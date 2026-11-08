//! Reproducible SDK workloads shared by headless measurements and an opt-in native app.
use std::{rc::Rc, sync::Arc};
use gpui::{prelude::*, AnyView, App, Context, Entity, Window, div, px};
use xgpui_luma::{controls::*, theme::LumaThemeSyncExt};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_radix as radix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneKind {
    Forms,
    Navigation,
    Collections,
    Layout,
    Color,
    Plot,
}
impl SceneKind {
    pub const ALL: [Self; 6] =
        [Self::Forms, Self::Navigation, Self::Collections, Self::Layout, Self::Color, Self::Plot];
    pub fn name(self) -> &'static str {
        match self {
            Self::Forms => "forms",
            Self::Navigation => "navigation",
            Self::Collections => "collections",
            Self::Layout => "layout",
            Self::Color => "color",
            Self::Plot => "plot",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookKind {
    Sdk,
    Shadcn,
    Radix,
}
impl LookKind {
    pub const ALL: [Self; 3] = [Self::Sdk, Self::Shadcn, Self::Radix];
    pub fn name(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::Shadcn => "shadcn",
            Self::Radix => "radix",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|look| look.name() == value)
    }
}

pub type ReleaseProbe = Rc<dyn Fn() -> bool>;
type Update = Box<dyn Fn(usize, &mut App)>;
type Invalidate = Box<dyn Fn(&mut App)>;

pub struct Scene {
    views: Vec<AnyView>,
    invalidations: Vec<Invalidate>,
    updates: Vec<Update>,
    probes: Vec<ReleaseProbe>,
    render_count: usize,
}

impl Scene {
    pub fn new(kind: SceneKind, look: LookKind, size: usize, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        anyhow::ensure!(size > 0 && size <= 100_000, "size must be between 1 and 100000");
        anyhow::ensure!(
            kind == SceneKind::Forms || look == LookKind::Sdk,
            "adapter comparison currently covers forms only"
        );
        let mut scene = Self {
            views: Vec::new(),
            invalidations: Vec::new(),
            updates: Vec::new(),
            probes: Vec::new(),
            render_count: 0,
        };
        match kind {
            SceneKind::Forms => scene.forms(look, size, cx),
            SceneKind::Navigation => scene.navigation(size, cx),
            SceneKind::Collections => scene.collections(size, cx),
            SceneKind::Layout => scene.layout(size, cx),
            SceneKind::Color => scene.color(size, cx),
            SceneKind::Plot => scene.plot(size, cx)?,
        }
        Ok(scene)
    }

    fn add<T: Render + 'static>(&mut self, entity: Entity<T>) {
        let weak = entity.downgrade();
        self.probes.push(Rc::new(move || weak.upgrade().is_none()));
        let redraw = entity.clone();
        self.invalidations.push(Box::new(move |cx| redraw.update(cx, |_, cx| cx.notify())));
        self.views.push(entity.into());
    }

    // Bound full-size controls so composed workloads exercise visible content.
    fn add_sized<T: Render + 'static>(&mut self, entity: Entity<T>, width: f32, height: f32, cx: &mut Context<Self>) {
        self.add(entity.clone());
        self.views.pop();
        let id = format!("sdk-perf-panel-{}", self.views.len());
        let panel = frame::FrameBuilder::new(id.clone())
            .w(px(width))
            .h(px(height))
            .flex_none()
            .child_render(move || {
                let selector = id.clone();
                div().id(id.clone()).debug_selector(move || selector).size_full().child(entity.clone())
            })
            .spawn(cx);
        let redraw = panel.clone();
        self.invalidations.push(Box::new(move |app| redraw.update(app, |_, cx| cx.notify())));
        self.views.push(panel.into());
    }

    pub fn invalidate(&self, cx: &mut App) {
        for operation in &self.invalidations {
            operation(cx);
        }
    }
    pub fn update_data(&self, step: usize, cx: &mut App) {
        for operation in &self.updates {
            operation(step, cx);
        }
    }
    pub fn has_updates(&self) -> bool {
        !self.updates.is_empty()
    }
    pub fn release_probes(&self) -> Vec<ReleaseProbe> {
        self.probes.clone()
    }
    pub fn render_count(&self) -> usize {
        self.render_count
    }
    pub fn control_count(&self) -> usize {
        self.views.len()
    }
    pub fn invalidate_theme(cx: &mut App) {
        cx.bump_luma_theme_revision();
    }

    fn forms(&mut self, look: LookKind, count: usize, cx: &mut Context<Self>) {
        let shadcn = shadcn::ShadcnLook::built_in();
        let radix = radix::Look::built_in();
        for index in 0..count {
            let id = format!("button-{index}");
            let button = match look {
                LookKind::Sdk => button::ButtonBuilder::new(id).spawn(cx),
                LookKind::Shadcn => shadcn::Button::new(id).look(&shadcn).spawn(cx),
                LookKind::Radix => radix::Button::new(id).look(&radix).spawn(cx),
            };
            self.add(button);
            let id = format!("checkbox-{index}");
            let checkbox = match look {
                LookKind::Sdk => checkbox::new(id).motion_enabled(false).spawn(cx),
                LookKind::Shadcn => shadcn::Checkbox::new(id).look(&shadcn).motion_enabled(false).spawn(cx),
                LookKind::Radix => radix::Checkbox::new(id).look(&radix).motion_enabled(false).spawn(cx),
            };
            let checked = checkbox.clone();
            self.updates.push(Box::new(move |step, app| {
                checked.update(app, |control, cx| control.set_data(step % 2 == 0, cx))
            }));
            self.add(checkbox);
            let id = format!("switch-{index}");
            self.add(match look {
                LookKind::Sdk => switch::new(id).motion_enabled(false).spawn(cx),
                LookKind::Shadcn => shadcn::Switch::new(id).look(&shadcn).motion_enabled(false).spawn(cx),
                LookKind::Radix => radix::Switch::new(id).look(&radix).motion_enabled(false).spawn(cx),
            });
            let id = format!("field-{index}");
            let field = match look {
                LookKind::Sdk => textfield::TextFieldBuilder::new(id).value("Unicode é中🙂").spawn(cx),
                LookKind::Shadcn => shadcn::TextField::new(id).look(&shadcn).value("Unicode é中🙂").spawn(cx),
                LookKind::Radix => radix::TextField::new(id).look(&radix).value("Unicode é中🙂").spawn(cx),
            };
            let edit = field.clone();
            self.updates.push(Box::new(move |step, app| {
                edit.update(app, |control, cx| control.set_value(format!("Unicode é中🙂 {step}"), cx))
            }));
            self.add(field);
            let id = format!("area-{index}");
            let area = match look {
                LookKind::Sdk => textarea::TextAreaBuilder::new(id).value("office é中🙂 العربية").rows(2).spawn(cx),
                LookKind::Shadcn => {
                    shadcn::TextArea::new(id).look(&shadcn).value("office é中🙂 العربية").rows(2).spawn(cx)
                }
                LookKind::Radix => {
                    radix::TextArea::new(id).look(&radix).value("office é中🙂 العربية").rows(2).spawn(cx)
                }
            };
            let edit = area.clone();
            self.updates.push(Box::new(move |step, app| {
                edit.update(app, |control, cx| control.set_value(format!("office é中🙂 العربية {step}"), cx))
            }));
            self.add(area);
            let id = format!("slider-{index}");
            let slider = match look {
                LookKind::Sdk => slider::SliderBuilder::new(id).value(0.5).motion_enabled(false).spawn(cx),
                LookKind::Shadcn => shadcn::Slider::new(id).look(&shadcn).value(0.5).motion_enabled(false).spawn(cx),
                LookKind::Radix => radix::Slider::new(id).look(&radix).value(0.5).motion_enabled(false).spawn(cx),
            };
            let slide = slider.clone();
            self.updates.push(Box::new(move |step, app| {
                slide.update(app, |control, cx| control.set_value(if step % 2 == 0 { 0.25 } else { 0.75 }, cx))
            }));
            self.add(slider);
            let id = format!("progress-{index}");
            self.add(match look {
                LookKind::Sdk => progress::ProgressBuilder::new(id).value(0.5).motion_enabled(false).spawn(cx),
                LookKind::Shadcn => shadcn::Progress::new(id).look(&shadcn).value(0.5).motion_enabled(false).spawn(cx),
                LookKind::Radix => radix::Progress::new(id).look(&radix).value(0.5).motion_enabled(false).spawn(cx),
            });
        }
    }

    fn navigation(&mut self, count: usize, cx: &mut Context<Self>) {
        let mut menu = sidebar::SidebarMenuBuilder::new("destinations");
        for index in 0..count {
            menu = menu.item(sidebar::SidebarMenuItemBuilder::new(format!("nav-{index}"), "Destination"));
        }
        self.add(
            sidebar::SidebarControlBuilder::new("sidebar")
                .motion_enabled(false)
                .sidebar(sidebar::SidebarBuilder::new("panel").content(
                    sidebar::SidebarContentBuilder::new().group(sidebar::SidebarGroupBuilder::new().menu(menu)),
                ))
                .spawn(cx),
        );
        self.add(
            tabs::TabsBuilder::new("tabs")
                .motion_enabled(false)
                .items((0..count.min(20)).map(|index| tabs::TabsItem::new(format!("tab-{index}"))))
                .spawn(cx),
        );
        self.add(
            accordion::AccordionBuilder::new("accordion")
                .motion_enabled(false)
                .items((0..count.min(100)).map(|index| {
                    accordion::AccordionItem::new(
                        format!("section-{index}"),
                        accordion::AccordionTrigger::new("Section"),
                        accordion::AccordionContent::custom(|_, _| div().child("Details").into_any_element()),
                    )
                }))
                .spawn(cx),
        );
        self.add(stepper::StepperBuilder::new("stepper", 5).motion_enabled(false).spawn(cx));
        self.add(pager::PagerBuilder::new("pager").spawn(cx));
        self.add(
            popup_menu::PopupMenuBuilder::new("menu")
                .label("Actions")
                .items((0..count).map(|index| xgpui_luma::infra::menu_item::MenuItem::new(format!("action-{index}"))))
                .spawn(cx),
        );
        let selector = selector::SelectorBuilder::new("selector")
            .items((0..count).map(|index| selector::SelectorItem::new(format!("option-{index}"))))
            .spawn(cx);
        let select = selector.clone();
        self.updates.push(Box::new(move |step, app| {
            select.update(app, |control, cx| {
                control.set_selected_id(format!("option-{}", step % count), cx);
            });
        }));
        self.add(selector);
    }

    fn collections(&mut self, count: usize, cx: &mut Context<Self>) {
        let tree = tree_view::TreeViewBuilder::new("tree")
            .motion_enabled(false)
            .items((0..count).map(|index| tree_view::TreeNode::new(format!("row-{index}"), "Record", ())))
            .spawn(cx);
        let select = tree.clone();
        self.updates.push(Box::new(move |step, app| {
            select.update(app, |control, cx| {
                control.select_node_by_id(format!("row-{}", step % count), cx);
            })
        }));
        self.add_sized(tree, 580.0, 700.0, cx);
        self.add_sized(
            table::TableBuilder::new("table")
                .items((0..count).map(|index| table::TableLabel::new(format!("Record {index}"))))
                .spawn(cx),
            580.0,
            700.0,
            cx,
        );
    }

    fn layout(&mut self, count: usize, cx: &mut Context<Self>) {
        let action = button::ButtonBuilder::new("pane-action").spawn(cx);
        self.add_sized(
            split_view::SplitViewBuilder::new("split")
                .motion_enabled(false)
                .sidebar(move || action.clone())
                .content(|| div().child("Content pane"))
                .spawn(cx),
            450.0,
            180.0,
            cx,
        );
        for index in 0..count {
            self.add(
                frame::FrameBuilder::new(format!("frame-{index}"))
                    .w(px(180.0))
                    .h(px(60.0))
                    .p(px(4.0))
                    .child_render(|| div().flex().gap(px(4.0)).child(div().child("Label")).child(div().child("Value")))
                    .spawn(cx),
            );
        }
    }

    fn color(&mut self, count: usize, cx: &mut Context<Self>) {
        for index in 0..count {
            use xgpui_luma_color::{color_field::ColorFieldState, color_slider::color_spec::Hsv};
            let field = cx.new(|_| {
                ColorFieldState::saturation_value_rect(
                    format!("color-field-{index}"),
                    Hsv { h: 120.0, s: 0.5, v: 0.5, a: 1.0 },
                    16.0,
                )
                .vector()
                .w(px(120.0))
                .h(px(120.0))
            });
            let change = field.clone();
            self.updates.push(Box::new(move |step, app| {
                change.update(app, |control, cx| control.set_hsv_components((step % 360) as f32, 0.5, 0.5, cx))
            }));
            self.add(field);
            let ring = xgpui_luma_color::color_ring::ColorRingBuilder::new(
                format!("ring-{index}"),
                120.0,
                Arc::new(xgpui_luma_color::color_ring::HueRingDelegate { saturation: 0.8, lightness: 0.5 }),
            )
            .size(px(120.0))
            .spawn(cx);
            ring.update(cx, |control, cx| control.set_motion_enabled(false, cx));
            let update = ring.clone();
            self.updates.push(Box::new(move |step, app| {
                update.update(app, |control, cx| control.set_value((step % 360) as f64, cx))
            }));
            self.add(ring);
        }
    }

    fn plot(&mut self, count: usize, cx: &mut Context<Self>) -> anyhow::Result<()> {
        use xgpui_luma_plot::{Dataset, Series, Sample};
        let dataset = Dataset::new().with_series(Series::new(
            "series",
            "Signal",
            (0..count).map(|index| Sample::new(index as f64, (index as f64 * 0.01).sin())).collect::<Vec<_>>(),
        ))?;
        self.add(cx.new(|_| PlotScene { dataset }));
        Ok(())
    }
}

impl Render for Scene {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.render_count += 1;
        // Noninteractive layout only; every interactive item is an SDK control.
        div()
            .size_full()
            .flex()
            .flex_wrap()
            .content_start()
            .gap(px(8.0))
            .children(self.views.iter().cloned())
    }
}

struct PlotScene {
    dataset: xgpui_luma_plot::Dataset,
}
impl Render for PlotScene {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use xgpui_luma_plot::{CartesianChartBuilder, LineLayer};
        div()
            .w(px(900.0))
            .h(px(500.0))
            .child(CartesianChartBuilder::new(self.dataset.clone()).line(LineLayer::new("series")).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, size};

    #[test]
    fn composed_collection_panels_are_both_visible_and_do_not_overlap() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| Scene::new(SceneKind::Collections, LookKind::Sdk, 100, cx).unwrap());
        cx.simulate_resize(size(px(1200.0), px(800.0)));
        cx.run_until_parked();
        let first = cx.debug_bounds("sdk-perf-panel-0").unwrap();
        let second = cx.debug_bounds("sdk-perf-panel-1").unwrap();
        for bounds in [first, second] {
            assert_eq!(bounds.size, size(px(580.0), px(700.0)));
            assert!(bounds.left() >= px(0.0) && bounds.top() >= px(0.0));
            assert!(bounds.right() <= px(1200.0) && bounds.bottom() <= px(800.0));
        }
        assert!(first.right() < second.left(), "collections must have distinct visible viewports");
    }
}
