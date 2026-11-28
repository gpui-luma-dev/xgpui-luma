use crate::infra::template::RootModifiers;
use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, KeyDownEvent, MouseButton, MouseDownEvent, Stateful, Window, div, px, prelude::*};

use super::{OverlayWindowRenderModel, OverlayWindowTheme, default_overlay_window_theme, resolve_overlay_window};
use crate::style::PartRead;
use xgpui_luma_style::{PartView, Resolved};

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
    /// The shell and placement style in `mode` (#433): what the control places and [`Self::render`] draws from.
    fn resolve_style(&self, mode: super::OverlayWindowMode) -> Arc<Resolved> {
        resolve_overlay_window(&default_overlay_window_theme().style(), mode)
    }

    /// The shell for `model`, drawn from `look`, the root of [`Self::resolve_style`].
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: PartView<'_>,
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
    modifiers: RootModifiers<dyn OverlayWindowTemplate>,
}

impl ThemedOverlayWindowTemplate {
    pub fn new(theme: Arc<dyn OverlayWindowTheme>) -> Self {
        Self { theme, modifiers: RootModifiers::default() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &OverlayWindowRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
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
    fn resolve_style(&self, mode: super::OverlayWindowMode) -> Arc<Resolved> {
        self.base.resolve_style(mode)
    }
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: PartView<'_>,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, parts, window, cx);
        self.apply(root, model)
    }
}

impl OverlayWindowTemplate for ThemedOverlayWindowTemplate {
    fn resolve_style(&self, mode: super::OverlayWindowMode) -> Arc<Resolved> {
        resolve_overlay_window(&self.theme.style(), mode)
    }
    fn render(
        &self,
        model: &OverlayWindowRenderModel<'_>,
        look: PartView<'_>,
        parts: OverlayWindowTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let OverlayWindowTemplateParts { header_drag_handle, handlers } = parts;
        let OverlayWindowTemplateHandlers { key_down, header_mouse_down, shell_mouse_down, shell_mouse_down_out } =
            handlers;

        let shell_width = match model.width {
            Some(width) => width.max(look.number("min_width")),
            None => look.number("min_width").clamp(look.number("min_width"), look.number("max_width")),
        };

        let mut shell = div()
            .id(model.id.clone())
            .relative()
            .w(px(shell_width))
            .when(model.width.is_none(), |shell| shell.max_w(px(look.number("max_width"))))
            .p(px(look.number("padding")))
            .bg(look.color("background"))
            .border(px(look.number("border_width")))
            .border_color(look.color("border"))
            .rounded(px(look.number("radius")))
            .shadow(look.box_shadow())
            .text_color(look.color("foreground"))
            .font_family(look.font_family_name())
            .text_size(px(look.typography(14.0, 20.0).size))
            .line_height(px(look.typography(14.0, 20.0).line_height))
            .font_weight(look.typography(14.0, 20.0).weight)
            .on_key_down(key_down)
            .on_mouse_down(MouseButton::Left, shell_mouse_down)
            .on_mouse_down_out(shell_mouse_down_out);

        if let Some(header_drag_handle) = header_drag_handle {
            let _ = header_mouse_down;
            shell = shell.child(header_drag_handle);
        }

        let root = shell.child(div().w_full().min_w_0().child((model.content)(model, window, cx)));
        self.modifiers.apply(root, model)
    }
}
