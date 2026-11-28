use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::{Arc, OnceLock};

use crate::style::PartRead;
use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use super::model::TableRenderModel;

pub type TableTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TableRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TableTemplate: Send + Sync {
    fn render(
        &self,
        model: &TableRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

impl RenderModel for dyn TableTemplate {
    type Model<'a> = TableRenderModel<'a>;
}

/// Paints list shell from resolved [`TableRenderModel`] only (no theme lookup).
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultTableShellTemplate;

impl DefaultTableShellTemplate {
    pub fn paint_shell(model: &TableRenderModel<'_>, header: Option<AnyElement>, body: AnyElement) -> Stateful<Div> {
        let (list, header_part) = (model.style.root(), model.style.part(xgpui_luma_style::PartId::new("header")));
        // Every layer shares the frame's radius; the shell's clip and border handle the inset.
        let radius = list.number("radius").max(0.0);
        let inner_radius = radius;
        let (border, border_width, background) =
            (list.color("border"), list.number("border_width"), list.color("background"));
        let (padding_x, padding_y) = (list.number("padding_x"), list.number("padding_y"));
        let header_typography = header_part.typography(14.0, 20.0);
        let has_header = header.is_some();

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(px(radius))
            .border(px(border_width))
            .border_color(border)
            .bg(if has_header {
                header_part.color("background")
            } else {
                background
            });

        if let Some(height) = model.shell_height {
            root = root.h(px(height));
        } else {
            root = root.h_full();
        }

        let mut column = div().w_full().flex_1().min_h(px(0.0)).flex().flex_col();

        if let Some(header) = header {
            let header_content = div()
                .w_full()
                .flex_none()
                .px(px(padding_x))
                .pt(px(padding_y))
                .pb(px(padding_y * list.number("header_padding_bottom_scale")))
                .border_b(px(list.number("header_divider_width")))
                .border_color(border)
                .text_color(header_part.color("foreground"))
                .text_size(px(header_typography.size))
                .line_height(px(header_typography.line_height))
                .font_weight(header_typography.weight)
                .child(header);
            column = column.child(div().w_full().flex_none().child(header_content));
        }

        let mut body_slot = if let Some(body_rows_height) = model.body_rows_height {
            div().w_full().flex_none().h(px(body_rows_height)).overflow_hidden().bg(background)
        } else {
            div().w_full().flex_1().min_h(px(0.0)).overflow_hidden().bg(background)
        };
        if has_header {
            body_slot = body_slot.rounded_bl(px(inner_radius)).rounded_br(px(inner_radius));
        } else {
            body_slot = body_slot.rounded(px(inner_radius));
        }

        column = column.child(body_slot.child(div().w_full().h_full().child(body)));

        root.child(column)
    }

    pub fn shell_height_for_model(model: &TableRenderModel<'_>) -> Option<f32> {
        model.shell_height
    }
}

impl TableTemplate for DefaultTableShellTemplate {
    fn render(
        &self,
        model: &TableRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        Self::paint_shell(model, header, body)
    }
}

impl TableTemplate for ModifiedTemplate<dyn TableTemplate> {
    fn render(
        &self,
        model: &TableRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, header, body, window, cx);
        self.apply(root, model)
    }
}

pub fn default_table_template() -> Arc<dyn TableTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TableTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(DefaultTableShellTemplate)).clone()
}

/// Shell template only; row/list colors come from [`TableTheme`](crate::controls::table::TableTheme) on the control builder.
pub fn table_template_with_theme(_theme: Arc<dyn crate::controls::table::TableTheme>) -> Arc<dyn TableTemplate> {
    default_table_template()
}

pub(super) fn modified_table_template<F>(template: Arc<dyn TableTemplate>, modifier: F) -> Arc<dyn TableTemplate>
where
    F: Fn(Stateful<Div>, &TableRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}
