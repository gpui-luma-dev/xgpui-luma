use crate::infra::template::{ModifiedTemplate, RenderModel};
use std::sync::Arc;

use gpui::{App, Div, Stateful, Window};

use super::ButtonRenderModel;

use crate::infra::template::Modifier;

pub type ButtonTemplateModifier<D> = Modifier<ButtonRenderModel<D>>;

pub trait ButtonTemplate<D = ()>: Send + Sync {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div>;

    /// The transition a control animates with when it moves to `data` (e.g. a checkbox becoming
    /// checked), from the template's style. `None` leaves the choice to the control.
    fn transition(&self, _data: &D) -> Option<crate::motion::Transition> {
        None
    }

    /// The `.lss` states this template sets in its style key from the control's interaction and
    /// data (e.g. `hover`, `focused`, `checked`), by name. Tools use it to offer states the template
    /// draws even when no rule mentions them, such as the focus ring. Empty for templates that do
    /// not render from `.lss`.
    fn style_states(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

impl<D> RenderModel for dyn ButtonTemplate<D> {
    type Model<'a> = ButtonRenderModel<D>;
}

/// The SDK button drawn from the SDK's `.lss` files.
pub fn default_button_template<D: Clone + 'static>() -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(super::StyledButtonTemplate::<D>::sdk())
}

pub(super) fn modified_button_template<D, F>(
    template: Arc<dyn ButtonTemplate<D>>,
    modifier: F,
) -> Arc<dyn ButtonTemplate<D>>
where
    D: 'static,
    F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTemplate::new(template, Box::new(modifier)))
}

impl<D: 'static> ButtonTemplate<D> for ModifiedTemplate<dyn ButtonTemplate<D>> {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let root = self.base.render(model, window, cx);
        self.apply(root, model)
    }

    fn transition(&self, data: &D) -> Option<crate::motion::Transition> {
        self.base.transition(data)
    }

    fn style_states(&self) -> Vec<&'static str> {
        self.base.style_states()
    }
}

#[cfg(test)]
mod tests {
    use gpui::Hsla;

    use crate::controls::button_family::{ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look};
    use crate::theme::{LumaTextStyle, StandardBoxScale};

    #[test]
    fn compose_button_family_look_uses_box_scale_geometry() {
        let palette = ButtonFamilyPalette {
            focus_border: gpui::transparent_black(),
            background: Hsla::default(),
            foreground: Hsla::default(),
            muted_foreground: Hsla::default(),
            border: Some(Hsla::default()),
            typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: gpui::FontWeight::MEDIUM },
            font_family: "test".into(),
        };
        let scale =
            StandardBoxScale { height: 36.0, padding_x: 14.0, padding_y: 8.0, gap: 8.0, radius: 6.0, icon_size: 16.0 };

        let look = compose_button_family_look(&palette, ButtonFamilyRole::Text, &scale, 999.0);
        assert_eq!(look.height, 36.0);
        assert_eq!(look.padding_x, 14.0);
        assert_eq!(look.padding_y, 8.0);
        assert_eq!(look.gap, 8.0);
        assert_eq!(look.radius, 6.0);
    }
}
