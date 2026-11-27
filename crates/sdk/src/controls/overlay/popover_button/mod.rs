//! Trigger-owned popover. Placement and popup lifecycle live in the private panel engine.

mod control;
mod model;
pub(crate) mod panel;

pub use control::PopoverButton;
pub use model::{
    PopoverButtonBuilder, PopoverButtonEvent, PopoverContent, PopoverPlacement, PopoverRenderModel, PopoverTrigger,
};

mod callback;
pub use callback::PopoverButtonEventExt;

pub use panel::{PopoverMotion, PopoverPanelLook};
