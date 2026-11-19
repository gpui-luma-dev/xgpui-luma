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
