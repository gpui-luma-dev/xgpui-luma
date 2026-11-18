use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*};
use lucide_svg_static::Icon as LucideIcon;

use super::{AccordionRenderModel, AccordionTheme, default_accordion_theme};
use crate::infra::icon::render_disclosure_icon;

pub type AccordionHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type AccordionMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type AccordionMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type AccordionClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type AccordionContentHeightHandler = Box<dyn Fn(f32, &mut Window, &mut App) + 'static>;
pub type AccordionTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

#[derive(Default)]
pub struct AccordionTemplateHandlers {
    /// Native drag handles, canonical targets and keyed geometry (opt-in).
    pub drag: Option<super::AccordionDragBindings>,
    pub trigger_hovers: Vec<AccordionHoverHandler>,
    pub trigger_mouse_downs: Vec<AccordionMouseDownHandler>,
    pub trigger_mouse_ups: Vec<AccordionMouseUpHandler>,
    pub trigger_clicks: Vec<AccordionClickHandler>,
    pub content_height_reports: Vec<AccordionContentHeightHandler>,
}

pub trait AccordionTemplate: Send + Sync {
    /// Theme for the retained drag-handle tooltip.
    fn drag_tooltip_theme(&self) -> Arc<dyn crate::controls::tooltip::TooltipTheme> {
        crate::controls::tooltip::default_tooltip_theme()
    }

    /// Appearance used by SDK native handles and previews.
    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        default_accordion_theme().resolve_drag()
    }

    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedAccordionTemplate {
    theme: Arc<dyn AccordionTheme>,
    modifiers: Vec<AccordionTemplateModifier>,
}

impl ThemedAccordionTemplate {
    pub fn new(theme: Arc<dyn AccordionTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &AccordionRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedAccordionTemplate {
    base: Arc<dyn AccordionTemplate>,
    modifiers: Vec<AccordionTemplateModifier>,
}

impl ModifiedAccordionTemplate {
    fn new(base: Arc<dyn AccordionTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: AccordionTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &AccordionRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }

    fn into_arc(self) -> Arc<dyn AccordionTemplate> {
        Arc::new(self)
    }
}

impl AccordionTemplate for ModifiedAccordionTemplate {
    fn drag_tooltip_theme(&self) -> Arc<dyn crate::controls::tooltip::TooltipTheme> {
        self.base.drag_tooltip_theme()
    }

    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        self.base.resolve_drag()
    }

    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_accordion_template() -> Arc<dyn AccordionTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AccordionTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedAccordionTemplate::new(default_accordion_theme()))).clone()
}

pub(super) fn modified_accordion_template<F>(
    template: Arc<dyn AccordionTemplate>,
    modifier: F,
) -> Arc<dyn AccordionTemplate>
where
    F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedAccordionTemplate::new(template).with_modifier(Box::new(modifier)).into_arc()
}

impl AccordionTemplate for ThemedAccordionTemplate {
    fn drag_tooltip_theme(&self) -> Arc<dyn crate::controls::tooltip::TooltipTheme> {
        self.theme.drag_tooltip_theme()
    }

    fn resolve_drag(&self) -> crate::infra::drag_drop::DragDropLook {
        self.theme.resolve_drag()
    }

    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let scale = self.theme.resolve_scale(scale_factor);

        let AccordionTemplateHandlers {
            drag,
            trigger_hovers,
            trigger_mouse_downs,
            trigger_mouse_ups,
            trigger_clicks,
            content_height_reports,
        } = handlers;

        let mut trigger_hovers = trigger_hovers.into_iter();
        let mut trigger_mouse_downs = trigger_mouse_downs.into_iter();
        let mut trigger_mouse_ups = trigger_mouse_ups.into_iter();
        let mut trigger_clicks = trigger_clicks.into_iter();
        let mut content_height_reports = content_height_reports.into_iter();

        let (root, handles, targets, bounds, motion) = match drag {
            Some(drag) => {
                let scroll = drag.scroll.clone();
                let root = drag.motion.begin_surface(drag.root, move || scroll.offset());
                (root, drag.handles, drag.targets, Some(drag.bounds), Some(drag.motion))
            }
            None => (div().id(model.id.clone()), Vec::new(), Vec::new(), None, None),
        };
        let mut handles = handles.into_iter();
        let mut root =
            root.relative().min_h(px(0.)).flex().flex_col().w_full().gap(px(scale.item_gap)).cursor_default();

        for (index, item) in model.items.iter().enumerate() {
            let Some(hover_handler) = trigger_hovers.next() else {
                break;
            };
            let Some(down_handler) = trigger_mouse_downs.next() else {
                break;
            };
            let Some(up_handler) = trigger_mouse_ups.next() else {
                break;
            };
            let Some(click_handler) = trigger_clicks.next() else {
                break;
            };
            let height_report = content_height_reports.next();

            let content_visible = item.expanded || item.progress > 0.0;
            let trigger_palette = self.theme.resolve_trigger(item.state.interaction_state());
            let content_palette = self.theme.resolve_content(content_visible);
            let trigger_min_height = model.trigger_min_height.unwrap_or(scale.trigger_height);
            let trigger_padding_y = model.trigger_padding_y.unwrap_or(scale.padding_y);

            let handle = handles.next().flatten();
            let mut trigger = div()
                .id("trigger")
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .min_h(px(trigger_min_height))
                .px(px(scale.padding_x))
                .py(px(trigger_padding_y))
                .rounded(px(scale.radius))
                .text_color(trigger_palette.foreground)
                .text_size(px(trigger_palette.typography.size))
                .line_height(px(trigger_palette.typography.line_height))
                .font_family(trigger_palette.font_family.clone())
                .font_weight(trigger_palette.typography.weight)
                .cursor_pointer();

            if let Some(background) = trigger_palette.background {
                trigger = trigger.bg(background);
            }

            if item.enabled {
                trigger = trigger
                    .on_hover(hover_handler)
                    .on_mouse_down(gpui::MouseButton::Left, down_handler)
                    .on_mouse_up(gpui::MouseButton::Left, up_handler)
                    .on_click(click_handler);
            } else {
                trigger = trigger.opacity(scale.disabled_opacity);
            }

            let trigger_content = match &item.trigger.custom_element {
                Some(renderer) => renderer(window, cx),
                None => {
                    let mut row = div().flex().items_center().gap(px(scale.inner_gap));
                    if let Some(icon) = item.trigger.icon {
                        row = row.child(render_icon(icon, trigger_palette.icon_color, scale.icon_size));
                    }
                    row = row.child(item.trigger.label.clone().unwrap_or_default());
                    row.into_any_element()
                }
            };

            let chevron = render_disclosure_icon(
                model.disclosure_icons,
                item.progress,
                trigger_palette.chevron_color,
                scale.chevron_size,
            );

            let trigger_el = if let Some(handle) = handle {
                trigger
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(scale.inner_gap))
                            .child(
                                handle
                                    .flex_none()
                                    .p(px(scale.drag_handle_padding))
                                    .opacity(if item.state.hovered { 1. } else { 0. })
                                    .child(crate::infra::icon::render_icon_source(
                                        &self.theme.drag_handle_icon(),
                                        trigger_palette.chevron_color,
                                        scale.icon_size,
                                    )),
                            )
                            .child(trigger_content),
                    )
                    .child(chevron)
            } else {
                trigger.child(trigger_content).child(chevron)
            };
            #[cfg(any(test, feature = "test-support"))]
            let trigger_el = {
                let id = item.id.clone();
                trigger_el.debug_selector(move || format!("accordion-trigger-{id}"))
            };

            let content_padding_y = model.content_padding_y.unwrap_or(scale.content_padding_y);
            let content_padding_top = model.content_padding_top.unwrap_or(content_padding_y);
            let content_padding_bottom = model.content_padding_bottom.unwrap_or(content_padding_y);

            let content_el = if content_visible {
                // Clip host uses explicit `h` (not only `max_h`) so collapse shrinks layout
                // height each frame and siblings below reflow smoothly.
                let mut clip = div().w_full().overflow_hidden().opacity(item.progress).flex_shrink_0();

                if item.content_height_px > f32::EPSILON {
                    clip = clip.h(px(item.content_height_px * item.progress));
                } else if item.progress < 1.0 - f32::EPSILON {
                    clip = clip.h(px(0.0));
                }

                if let Some(height_report) = height_report {
                    // Inner already includes vertical padding; do not add pad_y again.
                    clip = clip.on_children_prepainted(move |bounds, window, cx| {
                        let height = bounds.iter().map(|child| child.size.height.as_f32()).fold(0.0_f32, f32::max);
                        if height > f32::EPSILON {
                            height_report(height, window, cx);
                        }
                    });
                }

                let mut inner = div()
                    .id("content")
                    .w_full()
                    .px(px(scale.padding_x))
                    .pt(px(content_padding_top))
                    .pb(px(content_padding_bottom))
                    .text_color(content_palette.foreground);

                if let Some(background) = content_palette.background {
                    inner = inner.bg(background);
                }

                if let Some(renderer) = &item.content.element {
                    inner = inner.child(renderer(window, cx));
                }

                Some(clip.child(inner))
            } else {
                None
            };

            let mut item_container =
                div().id((item.id.clone(), 0usize)).relative().flex().flex_col().w_full().child(trigger_el);

            if model.item_dividers && index + 1 < model.items.len() {
                item_container = item_container
                    .border_b(px(self.theme.metrics().border_width))
                    .border_color(trigger_palette.border_color);
            }

            if let Some(cel) = content_el {
                item_container = item_container.child(cel);
            }

            #[cfg(any(test, feature = "test-support"))]
            let item_container = {
                let id = item.id.clone();
                item_container.debug_selector(move || format!("accordion-item-{id}"))
            };
            root = match motion.as_ref().zip(bounds.as_ref()) {
                Some((motion, bounds)) => root.child(super::AccordionDragBindings::measure_item(
                    bounds,
                    item.id.clone(),
                    motion.item(item.id.clone(), item_container),
                )),
                None => root.child(item_container),
            };
        }

        root = root.children(targets);
        if let Some(motion) = motion {
            root = motion.end_surface(root);
        }
        self.apply_modifiers(root, model)
    }
}

fn render_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(crate::infra::icon::lucide_icon(icon, color, size))
        .into_any_element()
}
