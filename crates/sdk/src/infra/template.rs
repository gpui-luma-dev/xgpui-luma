use std::sync::Arc;

use gpui::{Div, Stateful};

/// A generic modifier for any model M
pub type Modifier<M> = Box<dyn Fn(Stateful<Div>, &M) -> Stateful<Div> + Send + Sync + 'static>;

/// Common trait for templates that support modifiers
pub trait TemplateWithModifiers<M>: Send + Sync {
    fn modifiers(&self) -> &[Modifier<M>];

    /// Helper to apply the pipeline to a base element
    fn apply_modifiers(&self, mut element: Stateful<Div>, model: &M) -> Stateful<Div> {
        for modifier in self.modifiers() {
            element = (modifier)(element, model);
        }
        element
    }
}

pub struct ControlTemplate<T: ?Sized, M> {
    pub theme: Arc<T>,
    pub modifiers: Vec<Modifier<M>>,
}

impl<T: ?Sized, M> ControlTemplate<T, M> {
    pub fn new(theme: Arc<T>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &M) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl<T: ?Sized, M> TemplateWithModifiers<M> for ControlTemplate<T, M>
where
    T: Send + Sync,
{
    fn modifiers(&self) -> &[Modifier<M>] {
        &self.modifiers
    }
}

/// Macro to define a control template alias and a default instance helper.
///
/// Usage:
/// ```rust
/// # use xgpui_luma::define_control_template;
/// # use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
/// # use xgpui_luma::controls::checkbox::{CheckboxData, CheckboxTheme, default_checkbox_theme};
/// define_control_template!(
///     ThemedCheckboxTemplate,
///     dyn CheckboxTheme,
///     ButtonRenderModel<CheckboxData>,
///     ButtonTemplate<CheckboxData>,
///     default_checkbox_theme()
/// );
/// ```
#[macro_export]
macro_rules! define_control_template {
    ($name:ident, $theme:ty, $model:ty, $trait:path, $default_theme:expr) => {
        pub type $name = $crate::infra::template::ControlTemplate<$theme, $model>;

        pub fn default_template() -> std::sync::Arc<dyn $trait> {
            static TEMPLATE: std::sync::OnceLock<std::sync::Arc<dyn $trait>> = std::sync::OnceLock::new();
            TEMPLATE.get_or_init(|| std::sync::Arc::new($name::new($default_theme))).clone()
        }
    };
}
