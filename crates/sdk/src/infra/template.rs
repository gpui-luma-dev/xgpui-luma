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
    modifiers: RootModifiers<T>,
}

impl<T: ?Sized + RenderModel> ModifiedTemplate<T> {
    pub(crate) fn new(base: Arc<T>, modifier: RootModifier<T>) -> Self {
        let mut modifiers = RootModifiers::default();
        modifiers.push(modifier);
        Self { base, modifiers }
    }

    /// Run the modifiers over the base template's root, in the order they were added.
    pub(crate) fn apply(&self, root: Stateful<Div>, model: &T::Model<'_>) -> Stateful<Div> {
        self.modifiers.apply(root, model)
    }
}

/// A modifier on an element a template returns, with the app (templates whose modifiers read no model).
pub type ElementModifier<E> = Box<dyn Fn(E, &mut gpui::App) -> E + Send + Sync + 'static>;

/// [`ModifiedTemplate`] for templates whose modifiers take the element and the app ([`ElementModifier`]).
pub(crate) struct ModifiedElementTemplate<T: ?Sized, E> {
    pub(crate) base: Arc<T>,
    modifiers: ElementModifiers<E>,
}

impl<T: ?Sized, E> ModifiedElementTemplate<T, E> {
    pub(crate) fn new(base: Arc<T>, modifier: ElementModifier<E>) -> Self {
        let mut modifiers = ElementModifiers::default();
        modifiers.push(modifier);
        Self { base, modifiers }
    }

    /// Run the modifiers over the base template's element, in the order they were added.
    pub(crate) fn apply(&self, element: E, cx: &mut gpui::App) -> E {
        self.modifiers.apply(element, cx)
    }
}

/// A template's root modifiers, applied in the order they were added: the list behind a themed template's
/// public `with_modifier`, so no template keeps its own `apply_modifiers` loop (#433).
pub(crate) struct RootModifiers<T: ?Sized + RenderModel>(Vec<RootModifier<T>>);

impl<T: ?Sized + RenderModel> Default for RootModifiers<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: ?Sized + RenderModel> RootModifiers<T> {
    pub(crate) fn push(&mut self, modifier: RootModifier<T>) {
        self.0.push(modifier);
    }

    pub(crate) fn apply(&self, mut root: Stateful<Div>, model: &T::Model<'_>) -> Stateful<Div> {
        for modifier in &self.0 {
            root = modifier(root, model);
        }
        root
    }
}

/// [`RootModifiers`] for modifiers that take the element and the app ([`ElementModifier`]).
pub(crate) struct ElementModifiers<E>(Vec<ElementModifier<E>>);

impl<E> Default for ElementModifiers<E> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<E> ElementModifiers<E> {
    pub(crate) fn push(&mut self, modifier: ElementModifier<E>) {
        self.0.push(modifier);
    }

    pub(crate) fn apply(&self, mut element: E, cx: &mut gpui::App) -> E {
        for modifier in &self.0 {
            element = modifier(element, cx);
        }
        element
    }
}
