use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::{ButtonContentContext, ButtonRenderModel, button_content_context};
use crate::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyRole, ButtonFamilyTheme, button_family_effective_border, compose_button_family_look,
    default_button_family_theme,
};

use crate::infra::template::{Modifier, TemplateWithModifiers};
use crate::theme::StandardBoxScale;

pub type ButtonTemplateModifier<D> = Modifier<ButtonRenderModel<D>>;

fn resolve_theme_look<D>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
    scale: &StandardBoxScale,
) -> ButtonFamilyLook {
    if let Some(look) = theme.resolve_look(model.role, model.size, model.state, scale, theme.metrics().radius.pill) {
        return look;
    }

    let palette = theme.resolve(model.role, model.size, model.state);
    compose_button_family_look(&palette, model.role, scale, theme.metrics().radius.pill)
}

fn resolve_look<D>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
    scale: &StandardBoxScale,
) -> ButtonFamilyLook {
    if let Some(resolve) = &model.look {
        return resolve(model);
    }

    resolve_theme_look(theme, model, scale)
}

fn presenter_model<D: Clone>(model: &ButtonRenderModel<D>, look: &ButtonFamilyLook) -> ButtonContentContext<D> {
    button_content_context(model, look.clone())
}

pub trait ButtonTemplate<D = ()>: Send + Sync {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct DefaultButtonTemplate<D = ()> {
    pub theme: Arc<dyn ButtonFamilyTheme>,
    pub modifiers: Vec<ButtonTemplateModifier<D>>,
}

struct ModifiedButtonTemplate<D = ()> {
    base: Arc<dyn ButtonTemplate<D>>,
    modifiers: Vec<ButtonTemplateModifier<D>>,
}

impl<D> DefaultButtonTemplate<D> {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl<D: 'static> TemplateWithModifiers<ButtonRenderModel<D>> for DefaultButtonTemplate<D> {
    fn modifiers(&self) -> &[ButtonTemplateModifier<D>] {
        &self.modifiers
    }
}

impl<D> ModifiedButtonTemplate<D> {
    fn new(base: Arc<dyn ButtonTemplate<D>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ButtonTemplateModifier<D>) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ButtonRenderModel<D>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }

    fn into_arc(self) -> Arc<dyn ButtonTemplate<D>>
    where
        D: 'static,
    {
        Arc::new(self)
    }
}

pub fn default_button_template<D: Clone + 'static>() -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()))
}

pub(super) fn modified_button_template<D, F>(
    template: Arc<dyn ButtonTemplate<D>>,
    modifier: F,
) -> Arc<dyn ButtonTemplate<D>>
where
    D: 'static,
    F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedButtonTemplate::new(template).with_modifier(Box::new(modifier)).into_arc()
}

impl<D: 'static> ButtonTemplate<D> for ModifiedButtonTemplate<D> {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let root = self.base.render(model, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl<D: 'static + Clone> ButtonTemplate<D> for DefaultButtonTemplate<D> {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(model.size, &self.theme.metrics(), scale_factor);
        let mut look = resolve_look(&self.theme, model, &scale);
        let presenter_model = presenter_model(model, &look);
        let focused = model.state.focused && !model.state.disabled;
        let border = button_family_effective_border(look.border);
        let focus = crate::infra::focus_adorner::FocusAdorner::new(self.theme.metrics().focus);
        let focus_extent = if model.compact { 0.0 } else { focus.extent() };

        let mut control = div()
            .id((model.id.clone(), 0usize))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .bg(look.background)
            .text_color(look.foreground)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .h(px(look.height));

        control = control.border(px(self.theme.metrics().border_width.default)).border_color(border);

        let square_icon_toggle = matches!(model.role, ButtonFamilyRole::Toggle { .. })
            && !model.round
            && look.padding_x <= 0.0
            && look.padding_y <= 0.0;

        if matches!(model.role, ButtonFamilyRole::Icon)
            || (matches!(model.role, ButtonFamilyRole::Toggle { .. }) && model.round)
            || square_icon_toggle
        {
            let corner_radius = model.radius_override.get().unwrap_or(look.radius);
            control = control.w(px(look.height)).flex_shrink_0().p_0();
            if model.round || corner_radius >= look.height / 2.0 {
                control = control.rounded_full();
            } else {
                control = control.rounded(px(corner_radius));
            }
        } else if model.round {
            control = control.w(px(look.height)).p_0().rounded_full();
        } else {
            let corner_radius = model.radius_override.get().unwrap_or(look.radius);
            control = control.px(px(look.padding_x)).py(px(look.padding_y)).rounded(px(corner_radius));
        }

        let content = if let Some(icon) = &model.icon {
            match icon {
                super::model::ControlIcon::Lucide(icon) => {
                    crate::infra::icon::lucide_icon(*icon, look.foreground, look.icon_size)
                }
                super::model::ControlIcon::SvgPath(path) => gpui::svg()
                    .size(px(look.icon_size))
                    .text_color(look.foreground)
                    .path(path.clone())
                    .into_any_element(),
            }
        } else {
            (model.content)(&presenter_model, cx)
        };
        let content = if matches!(model.role, ButtonFamilyRole::Icon) || square_icon_toggle {
            div()
                .size(px(look.icon_size))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(look.icon_size))
                .line_height(px(look.icon_size))
                .child(content)
                .into_any_element()
        } else {
            content
        };

        control = control.child(div().text_color(look.foreground).child(content));

        control = self.apply_modifiers(control, model);

        if crate::infra::shadow_layout::should_paint_shadow(
            model.elevation,
            model.state.disabled,
            look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()),
        ) && let Some(shadows) = look.shadow.take()
        {
            control = control.shadow(shadows);
        }

        let oversize_extent = focus_extent;

        let radius = if let Some(radius) = model.radius_override.get() {
            radius
        } else if model.round {
            look.height / 2.0
        } else {
            look.radius
        };

        let adorned = div().id((model.id.clone(), 1usize)).relative().child(control);

        let mut root = div().id(model.id.clone()).relative();
        root = if oversize_extent > 0.0 {
            root.p(px(oversize_extent)).child(adorned)
        } else {
            root.child(adorned)
        };

        root = root.child(focus.overlay(look.focus_border, focused, radius, model.compact));

        if model.state.disabled {
            root = root.opacity(self.theme.metrics().disabled_opacity);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui::Hsla;

    use super::*;
    use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonSize};
    use crate::theme::{InteractionState, LumaTextStyle};

    fn lime_look() -> ButtonFamilyLook {
        ButtonFamilyLook {
            focus_border: gpui::transparent_black(),
            background: Hsla { h: 120.0, s: 1.0, l: 0.5, a: 1.0 },
            foreground: Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 },
            muted_foreground: Hsla { h: 0.0, s: 0.0, l: 0.7, a: 1.0 },
            border: Some(Hsla { h: 120.0, s: 1.0, l: 0.3, a: 1.0 }),
            typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: gpui::FontWeight::MEDIUM },
            font_family: "test".into(),
            radius: 8.0,
            padding_x: 12.0,
            padding_y: 6.0,
            gap: 6.0,
            height: 32.0,
            icon_size: 14.0,
            shadow: None,
        }
    }

    #[test]
    fn with_look_overrides_style_resolution() {
        let template: DefaultButtonTemplate<()> = DefaultButtonTemplate::new(default_button_family_theme());
        let model = ButtonRenderModel {
            id: "look-test".into(),
            data: (),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: InteractionState::default(),
            round: false,
            radius_override: std::cell::Cell::new(None),
            elevation: true,
            compact: false,
            look: Some(Arc::new(|_| lime_look())),
            ..Default::default()
        };

        let scale =
            StandardBoxScale { height: 32.0, padding_x: 12.0, padding_y: 6.0, gap: 6.0, radius: 8.0, icon_size: 14.0 };
        let look = resolve_look(&template.theme, &model, &scale);
        assert_eq!(look.background, lime_look().background);
    }

    #[test]
    fn presenter_model_exposes_resolved_look() {
        let model = ButtonRenderModel::<()>::default();
        let look = lime_look();
        let presented = presenter_model(&model, &look);

        assert_eq!(presented.look.foreground, look.foreground);
    }

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
