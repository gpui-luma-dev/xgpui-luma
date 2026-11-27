//! Button family: LMTP button, icon preset, theme recipe, and split-button specialization.
//!
//! - [`Button`] / [`IconButton`] — spawnable controls
//! - [`family`] — shared look/metrics recipe (`button_family` re-export)
//! - [`split_button`] — single-focus retained action/menu specialization
//! - [`CommandCore`] — presentation-agnostic activation/interaction core used by `Button`

mod callback;
mod control;
mod core;
pub(crate) mod face;
mod model;
mod template;
mod styled;

pub mod family;
pub mod icon_button;
pub mod split_button;
pub mod trigger_button;

pub use control::{Button, ButtonEvent};
pub use callback::{ButtonCallbacks, ButtonEventExt};

pub use core::{CommandCore, CommandEvent};
pub use crate::infra::template::{ControlTemplate, Modifier, TemplateWithModifiers};
pub use model::{
    button_content_context, ButtonBuilder, ButtonContentContext, ButtonLookSource, ButtonModel, ButtonRenderModel,
    ControlIcon, ControlPresenter, HasPresenter,
};
pub use template::{ButtonTemplate, ButtonTemplateModifier, default_button_template};
pub use styled::{ButtonData, StyledButtonTemplate};
pub use icon_button::IconButton;
pub use trigger_button::{TriggerButton, TriggerData};

#[cfg(all(test, feature = "test-support"))]
mod face_tests;
