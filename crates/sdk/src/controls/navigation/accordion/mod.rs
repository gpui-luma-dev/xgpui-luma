//! An undecorated stack of expandable sections. Compose it inside a
//! [`FrameBuilder`](crate::controls::frame::FrameBuilder) for an optional enclosure;
//! use `h_auto()` so the frame follows expansion instead of filling its parent.
//! Item dividers, trigger states, content padding, and animated clipping belong
//! to Accordion, independently of the frame's outer decoration and inset.

mod styled;
pub use styled::{StyledAccordion, StyledAccordionTheme};
mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{AccordionControl, AccordionEvent};
pub use model::{
    AccordionBuilder, AccordionContent, AccordionItem, AccordionItemRenderModel, AccordionModel, AccordionRenderModel,
    AccordionSelectionMode, AccordionTrigger,
};
pub use template::{
    AccordionTemplate, AccordionTemplateHandlers, AccordionTemplateModifier, ThemedAccordionTemplate,
    default_accordion_template,
};
pub use theme::{
    AccordionContentPalette, AccordionPalette, AccordionScale, AccordionTheme, DefaultAccordionTheme,
    default_accordion_theme,
};

pub use crate::infra::state::{CompositeItemState as AccordionItemState, ControlFocusState};
use gpui::{Entity, SharedString};

/// Entity handle for an accordion control. Use this type directly — do not wrap in [`Entity`] again.
pub type Accordion = Entity<AccordionControl>;

/// Creates a new builder for an Accordion.
pub fn new(id: impl Into<SharedString>) -> AccordionBuilder {
    AccordionBuilder::new(id)
}

mod callback;
pub use callback::AccordionEventExt;

mod drag;
mod geometry;
pub use drag::{AccordionDragBindings, AccordionDragEvent, AccordionDropProposal};
#[cfg(all(test, feature = "test-support"))]
mod drag_tests;

pub use model::AccordionMotion;
