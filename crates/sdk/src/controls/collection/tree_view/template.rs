use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ClickEvent, Div, MouseDownEvent, MouseUpEvent, Stateful, Window, SharedString, div, prelude::*, px,
};

use super::{FlatTreeNode, TreeViewRenderModel, TreeViewTheme, default_tree_view_theme};
use crate::infra::icon::render_disclosure_icon;

/// Shared callback retained across redraws; use `Rc::new` when supplying a custom handler.
pub type TreeViewHoverHandler = Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; use `Rc::new` when supplying a custom handler.
pub type TreeViewMouseDownHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; use `Rc::new` when supplying a custom handler.
pub type TreeViewMouseUpHandler = Rc<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
/// Shared callback retained across redraws; use `Rc::new` when supplying a custom handler.
pub type TreeViewClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Shared row callbacks. Custom templates bind these with a forwarding closure, for example
/// `.on_hover(move |event, window, cx| (handlers.hover)(event, window, cx))`.
#[derive(Clone)]
pub struct TreeViewTemplateHandlers {
    pub hover: TreeViewHoverHandler,
    pub mouse_down: TreeViewMouseDownHandler,
    pub mouse_up: TreeViewMouseUpHandler,
    pub click: TreeViewClickHandler,
    /// Independent disclosure gesture; the row shell prevents row selection/drag.
    pub disclosure: TreeViewClickHandler,
}

pub type TreeViewTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TreeViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TreeViewTemplate<T>: Send + Sync
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;

    /// Content-only customization. Legacy full templates retain their renderer
    /// unless they opt into this seam; the built-in and Shadcn templates support it.
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        _content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_node(node, handlers, window, cx)
    }

    /// Stable, noninteractive preview; content callbacks are not rebuilt during drag.
    fn render_drag_preview(&self, label: SharedString, _window: &mut Window, _cx: &mut App) -> AnyElement {
        {
            let scale = self.resolve_scale(1.0);
            div()
                .px(px(scale.preview_padding_x))
                .py(px(scale.preview_padding_y))
                .child(label)
                .into_any_element()
        }
    }

    /// Resolve all row, preview and insertion chrome through the theme scale.
    fn resolve_scale(&self, scale_factor: f32) -> super::TreeViewScale {
        super::TreeViewScale::compute(&crate::theme::MetricTokens::default(), scale_factor)
    }

    /// Color used by the SDK's keyed drop markers and preview outline.
    fn drop_color(&self) -> gpui::Hsla {
        crate::theme::ThemeTokens::default().palette.app.muted_foreground
    }
}

pub struct ThemedTreeViewTemplate {
    theme: Arc<dyn TreeViewTheme>,
}

impl ThemedTreeViewTemplate {
    pub fn new(theme: Arc<dyn TreeViewTheme>) -> Self {
        Self { theme }
    }
}

struct ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    base: Arc<dyn TreeViewTemplate<T>>,
    modifiers: Vec<TreeViewTemplateModifier>,
}

impl<T> ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    fn new(base: Arc<dyn TreeViewTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TreeViewTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TreeViewRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_tree_view_template<T>() -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
{
    Arc::new(ThemedTreeViewTemplate::new(default_tree_view_theme()))
}

pub(super) fn modified_tree_view_template<T, F>(
    template: Arc<dyn TreeViewTemplate<T>>,
    modifier: F,
) -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
    F: Fn(Stateful<Div>, &TreeViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTreeViewTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl<T> TreeViewTemplate<T> for ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, body, window, cx);
        self.apply_modifiers(root, model)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.base.render_node(node, handlers, window, cx)
    }
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.base.render_node_with_content(node, handlers, content, window, cx)
    }
    fn render_drag_preview(&self, label: SharedString, window: &mut Window, cx: &mut App) -> AnyElement {
        self.base.render_drag_preview(label, window, cx)
    }
    fn resolve_scale(&self, scale_factor: f32) -> super::TreeViewScale {
        self.base.resolve_scale(scale_factor)
    }
    fn drop_color(&self) -> gpui::Hsla {
        self.base.drop_color()
    }
}

impl<T> TreeViewTemplate<T> for ThemedTreeViewTemplate
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        div().id(model.id.clone()).size_full().overflow_hidden().child(body)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row(node, handlers, None, window, cx)
    }
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row(node, handlers, Some(content), window, cx)
    }
    fn render_drag_preview(&self, label: SharedString, _window: &mut Window, _cx: &mut App) -> AnyElement {
        let palette =
            self.theme.resolve_row(crate::theme::InteractionState { hovered: true, ..Default::default() }, true);
        let scale = self.theme.resolve_scale(_window.scale_factor());
        div()
            .px(px(scale.preview_padding_x))
            .py(px(scale.preview_padding_y))
            .rounded(px(scale.radius))
            .border(px(self.theme.metrics().border_width.default))
            .border_color(palette.chevron_color)
            .when_some(palette.background, |preview, color| preview.bg(color))
            .text_color(palette.foreground)
            .font_family(palette.font_family)
            .text_size(px(palette.typography.size))
            .shadow(self.theme.drag_preview_shadow())
            .child(label)
            .into_any_element()
    }
    fn resolve_scale(&self, scale_factor: f32) -> super::TreeViewScale {
        self.theme.resolve_scale(scale_factor)
    }
    fn drop_color(&self) -> gpui::Hsla {
        self.theme.resolve_row(Default::default(), true).chevron_color
    }
}

impl ThemedTreeViewTemplate {
    fn render_row<T: Send + Sync + 'static>(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: Option<AnyElement>,
        window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let scale_factor = window.scale_factor();
        let scale = self.theme.resolve_scale(scale_factor);

        let palette = self.theme.resolve_row(node.state.interaction_state(), node.state.selected);
        let icon_size = scale.icon_size;
        let chevron_size = scale.chevron_size;
        let left_padding = scale.base_padding_x + (node.depth as f32 * scale.indentation_width);

        let height_factor = node.row_height_factor.clamp(0.0, 1.0);
        let row_height = scale.row_height * height_factor;

        let mut row = div()
            .id((node.id.clone(), 1usize))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(row_height))
            .overflow_hidden()
            .opacity(height_factor.max(0.0))
            .pl(px(left_padding))
            .pr(px(scale.base_padding_x))
            .rounded(px(scale.radius))
            .text_color(palette.foreground)
            .text_size(px(palette.typography.size))
            .line_height(px(palette.typography.line_height))
            .font_family(palette.font_family.clone())
            .font_weight(palette.typography.weight)
            .cursor_pointer();

        if let Some(background) = palette.background {
            row = row.bg(background);
        }

        if node.enabled {
            row = row
                .on_hover(move |event, window, cx| (handlers.hover)(event, window, cx))
                .on_mouse_down(gpui::MouseButton::Left, move |event, window, cx| {
                    (handlers.mouse_down)(event, window, cx)
                })
                .on_mouse_up(gpui::MouseButton::Left, move |event, window, cx| (handlers.mouse_up)(event, window, cx))
                .on_click(move |event, window, cx| (handlers.click)(event, window, cx));
        } else {
            row = row.opacity(scale.disabled_opacity * height_factor.max(0.0));
        }

        if node.has_children {
            let chevron = div()
                .id((node.id.clone(), 2usize))
                .size(px(chevron_size))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .when(node.enabled, |chevron| {
                    chevron.on_click(move |event, window, cx| (handlers.disclosure)(event, window, cx))
                })
                .child(render_disclosure_icon(
                    node.disclosure_icons,
                    node.expand_progress.clamp(0.0, 1.0),
                    palette.chevron_color,
                    chevron_size,
                ));

            #[cfg(any(test, feature = "test-support"))]
            let chevron = {
                let debug_node_id = node.id.clone();
                chevron.debug_selector(move || format!("tree-disclosure-{}", debug_node_id))
            };
            row = row.child(chevron);
        } else {
            row = row.child(div().size(px(chevron_size)));
        }

        row = row.child(div().w(px(scale.inner_gap)));

        let display_icon = node.icon.map(crate::infra::icon::IconSource::from).unwrap_or_else(|| {
            use crate::theme::stylesheet::IconRole;
            self.theme.icon(if node.has_children {
                if node.expanded {
                    IconRole::FolderOpen
                } else {
                    IconRole::Folder
                }
            } else {
                IconRole::File
            })
        });
        row = row.child(crate::infra::icon::render_icon_source(&display_icon, palette.icon_color, icon_size));
        row = row.child(div().w(px(scale.inner_gap)));
        row = row.child(match content {
            Some(content) => div().flex_1().min_w(px(0.0)).child(content),
            None => div().flex_1().truncate().child(node.label.clone()),
        });

        if let Some(color) = self.theme.row_outline(node.state.interaction_state()) {
            row = row.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(scale.radius))
                    .border(px(self.theme.metrics().border_width.default))
                    .border_color(color),
            );
        }

        row.into_any_element()
    }
}
