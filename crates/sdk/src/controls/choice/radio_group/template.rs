use crate::style::PartRead;
use std::cell::Cell;
use std::sync::Arc;

use gpui::{AnyElement, App, Window, div, px, prelude::*};

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::control_group::{
    self as control_group, ControlGroupItemElementTemplate, ControlGroupItemHandlerExt, ControlGroupItemLike,
    ControlGroupItemRenderModel, ControlGroupItemTemplate, ControlGroupLayout, ControlGroupRenderModel,
    ControlGroupTemplate, ControlGroupTemplateHandlers,
};
use crate::controls::radio_button::RadioButtonData;

use super::model::RadioGroupTemplate;
use crate::style::StyleBinding;

/// The radio group's layout, from `radio_group.lss`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadioGroupGeometry {
    /// Space between stacked rows.
    pub vertical_gap: f32,
    /// Space between side-by-side rows.
    pub horizontal_gap: f32,
    /// Space between a custom row's indicator and its content.
    pub item_gap: f32,
    /// A disabled custom row's content opacity: the `content` part's `:disabled { opacity }`. The radio draws
    /// its own disabled look.
    pub content_disabled_opacity: f32,
}

impl RadioGroupGeometry {
    /// The layout `binding` (a `radio_group` control) resolves to.
    pub fn from_binding(binding: &StyleBinding) -> Self {
        let resolved = binding.resolve(&[], 1.0);
        let root = resolved.root();
        let number = |name| root.value(name).and_then(xgpui_luma_style::Value::as_number).unwrap_or(0.0);
        Self {
            vertical_gap: number("vertical_gap"),
            horizontal_gap: number("horizontal_gap"),
            item_gap: number("item_gap"),
            content_disabled_opacity: binding
                .resolve(&[("disabled", true)], 1.0)
                .part(xgpui_luma_style::PartId::new("content"))
                .opacity(),
        }
    }

    /// The SDK's own layout.
    pub fn sdk() -> Self {
        Self::from_binding(&StyleBinding::sdk("radio_group"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupLayout {
    Vertical,
    Horizontal,
}

/// Radio rows laid out with the SDK's `radio_group.lss`.
pub fn radio_group_buttons_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_buttons_template_with(StyleBinding::sdk("radio_group"), button_template, layout)
}

/// Radio rows laid out with `binding` (a look's `radio_group` control), resolved on every render.
pub fn radio_group_buttons_template_with<T>(
    binding: StyleBinding,
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |model, handlers, window, cx| {
        let geometry = RadioGroupGeometry::from_binding(&binding);
        let rows = render_radio_button_rows(model, handlers, &button_template, &geometry, window, cx);
        let gap = match layout {
            RadioGroupLayout::Vertical => geometry.vertical_gap,
            RadioGroupLayout::Horizontal => geometry.horizontal_gap,
        };
        let root = div().id(model.id.clone()).flex().gap(px(gap));
        match layout {
            RadioGroupLayout::Vertical => root.flex_col().items_start().children(rows),
            RadioGroupLayout::Horizontal => root.items_center().children(rows),
        }
    })
}

pub fn radio_group_button_item_element_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |item, item_template, window, cx| {
        render_radio_button_option(item, item_template, &button_template, &RadioGroupGeometry::sdk(), window, cx)
    })
}

pub fn render_radio_button_option<T>(
    item: &ControlGroupItemRenderModel<'_, T>,
    item_template: Option<&ControlGroupItemTemplate<T>>,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    geometry: &RadioGroupGeometry,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div>
where
    T: ControlGroupItemLike + 'static,
{
    if let Some(item_template) = item_template {
        let indicator_render_model = ButtonRenderModel {
            id: "indicator".into(),
            data: RadioButtonData::new(item.selected),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::Icon,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            elevation: false,
            compact: true,
            ..Default::default()
        };
        let indicator = button_template.render(&indicator_render_model, window, cx);
        let content = item_template(item, window, cx);

        div()
            .id((gpui::ElementId::from(item.group_id.clone()), item.item.id().clone()))
            .flex()
            .items_center()
            .gap(px(geometry.item_gap))
            .child(indicator)
            .child(div().flex_1().min_w_0().child(content).map(|content| {
                if item.enabled {
                    content
                } else {
                    crate::style::fade(content, geometry.content_disabled_opacity)
                }
            }))
    } else {
        let render_model = ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: RadioButtonData::new(item.selected),
            content: Arc::new({
                let label = item.item.label().clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            role: ButtonFamilyRole::Text,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            elevation: true,
            compact: false,
            ..Default::default()
        };

        button_template.render(&render_model, window, cx)
    }
}

pub fn render_radio_button_rows<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    geometry: &RadioGroupGeometry,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    let mut rows = Vec::with_capacity(model.items.len());

    for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
        let mut row = render_radio_button_option(item, model.item_template, button_template, geometry, window, cx)
            .control_group_item_handlers(item_handlers);

        if model.layout == ControlGroupLayout::Horizontal {
            row = row.flex_1().h_full();
        }

        if item.enabled {
            row = row.cursor_pointer();
        }

        rows.push(row.into_any_element());
    }

    rows
}

pub fn radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::vertical_group_template()
}

pub fn horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::horizontal_group_template()
}

pub fn default_radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_container_template()
}

pub fn default_horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    horizontal_group_container_template()
}

#[cfg(test)]
mod tests {
    use super::RadioGroupGeometry;
    use crate::style::{StyleBinding, ThemeSource};

    /// The layout is the binding's `radio_group.lss`, and a look's rules override the SDK's (#287).
    #[test]
    fn geometry_reads_the_radio_group_lss() {
        let sdk = RadioGroupGeometry::sdk();
        let theme = crate::style::default_theme();
        let sheet = theme
            .stylesheet()
            .with_source(
                "sdk",
                xgpui_luma_style::Source::new(
                    "look.lss",
                    "radio_group { horizontal_gap = exact(5)  part content { :disabled { opacity = 0.5 } } }",
                ),
            )
            .expect("compiles");
        let look = RadioGroupGeometry::from_binding(&StyleBinding::new(
            "radio_group",
            ThemeSource::Fixed(theme.with_stylesheet(&sheet).expect("binds")),
        ));
        assert_eq!((look.horizontal_gap, look.content_disabled_opacity), (5.0, 0.5));
        assert_eq!((look.vertical_gap, look.item_gap), (sdk.vertical_gap, sdk.item_gap), "unset values are the SDK's");
        assert!(sdk.vertical_gap > 0.0 && sdk.horizontal_gap > 0.0);
        assert_eq!(sdk.content_disabled_opacity, 1.0, "the SDK's rows draw their own disabled colors (#397)");
    }
}
