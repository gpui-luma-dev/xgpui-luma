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

/// A control's render model for any borrow (#433): implemented on a control's template trait object
/// (`dyn TableTemplate`), so one modifier type and one wrapper cover models that borrow (`TableRenderModel<'a>`).
pub trait RenderModel {
    type Model<'a>;
}

/// A modifier on a template's root element, for every borrow of the control's render model.
pub type RootModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &<T as RenderModel>::Model<'a>) -> Stateful<Div> + Send + Sync + 'static>;

/// A template with root modifiers applied after it renders (#433): the one wrapper behind every control's
/// `with_modifier`, instead of a private wrapper per control. The control's template trait is implemented
/// for `ModifiedTemplate<dyn ControlTemplate>`: render through [`Self::base`], then [`Self::apply`].
pub(crate) struct ModifiedTemplate<T: ?Sized + RenderModel> {
    pub(crate) base: Arc<T>,
    modifiers: Vec<RootModifier<T>>,
}

impl<T: ?Sized + RenderModel> ModifiedTemplate<T> {
    pub(crate) fn new(base: Arc<T>, modifier: RootModifier<T>) -> Self {
        Self { base, modifiers: vec![modifier] }
    }

    /// Run the modifiers over the base template's root, in the order they were added.
    pub(crate) fn apply(&self, mut root: Stateful<Div>, model: &T::Model<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}
