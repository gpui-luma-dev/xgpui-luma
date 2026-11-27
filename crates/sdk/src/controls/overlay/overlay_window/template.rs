use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, KeyDownEvent, MouseButton, MouseDownEvent, Stateful, Window, div, px, prelude::*};

use super::{OverlayWindowLook, OverlayWindowRenderModel, OverlayWindowTheme, default_overlay_window_theme};

pub type OverlayWindowMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type OverlayWindowKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

pub struct OverlayWindowTemplateHandlers {
    pub key_down: OverlayWindowKeyDownHandler,
    pub header_mouse_down: OverlayWindowMouseDownHandler,
    pub shell_mouse_down: OverlayWindowMouseDownHandler,
    pub shell_mouse_down_out: OverlayWindowMouseDownHandler,
}

pub type OverlayWindowTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &OverlayWindowRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct OverlayWindowTemplateParts {
    pub header_drag_handle: Option<gpui::AnyElement>,
    pub handlers: OverlayWindowTemplateHandlers,
}

pub trait OverlayWindowTemplate: Send + Sync {
    /// Resolve shell and placement styling together for the active template.
    fn resolve_look(&self, mode: super::OverlayWindowMode) -> OverlayWindowLook {
        default_overlay_window_theme().resolve(mode)
    }
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: &OverlayWindowLook,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn OverlayWindowTemplate {
    type Model<'a> = OverlayWindowRenderModel<'a>;
}

pub struct ThemedOverlayWindowTemplate {
    theme: Arc<dyn OverlayWindowTheme>,
    modifiers: Vec<OverlayWindowTemplateModifier>,
}

impl ThemedOverlayWindowTemplate {
    pub fn new(theme: Arc<dyn OverlayWindowTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &OverlayWindowRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &OverlayWindowRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_overlay_window_template() -> Arc<dyn OverlayWindowTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn OverlayWindowTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedOverlayWindowTemplate::new(default_overlay_window_theme())))
        .clone()
}

pub(super) fn modified_overlay_window_template<F>(
    template: Arc<dyn OverlayWindowTemplate>,
    modifier: F,
) -> Arc<dyn OverlayWindowTemplate>
where
    F: Fn(Stateful<Div>, &OverlayWindowRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl OverlayWindowTemplate for ModifiedTemplate<dyn OverlayWindowTemplate> {
    fn resolve_look(&self, mode: super::OverlayWindowMode) -> OverlayWindowLook {
        self.base.resolve_look(mode)
    }
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: &OverlayWindowLook,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, parts, window, cx);
        self.apply(root, model)
    }
}

impl OverlayWindowTemplate for ThemedOverlayWindowTemplate {
    fn resolve_look(&self, mode: super::OverlayWindowMode) -> OverlayWindowLook {
        self.theme.resolve(mode)
    }
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: &OverlayWindowLook,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let OverlayWindowTemplateParts { header_drag_handle, handlers } = parts;
        let OverlayWindowTemplateHandlers { key_down, header_mouse_down, shell_mouse_down, shell_mouse_down_out } =
            handlers;

        let shell_width = match model.width {
            Some(width) => width.max(look.min_width),
            None => look.min_width.clamp(look.min_width, look.max_width),
        };

        let mut shell = div()
            .id(model.id.clone())
            .relative()
            .w(px(shell_width))
            .when(model.width.is_none(), |shell| shell.max_w(px(look.max_width)))
            .p(px(look.padding))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .text_color(look.foreground)
            .font_family(look.font_family.clone())
            .text_size(px(look.body.size))
            .line_height(px(look.body.line_height))
            .font_weight(look.body.weight)
            .on_key_down(key_down)
            .on_mouse_down(MouseButton::Left, shell_mouse_down)
            .on_mouse_down_out(shell_mouse_down_out);

        if let Some(header_drag_handle) = header_drag_handle {
            let _ = header_mouse_down;
            shell = shell.child(header_drag_handle);
        }

        let root = shell.child(div().w_full().min_w_0().child((model.content)(model, window, cx)));
        self.apply_modifiers(root, model)
    }
}
