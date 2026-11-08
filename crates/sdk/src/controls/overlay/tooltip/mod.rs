//! Attached, noninteractive tooltips for SDK controls.
//!
//! Entity modifiers return the original entity. A second attachment replaces
//! the first; `clear_tooltip` cancels it. Disabled/empty targets do not show help.
//! Scrolling or owner movement dismisses help; a slider's moving active thumb
//! updates the anchor without moving the owner. GPUI selects one window tooltip
//! by paint order. Help text and its hint are associated with the target as an
//! accessible description, without replacing its label.
//! Templates must respect the resolved width constraint.
//!
//! Button, Slider and TextField own attachments. Checkbox, RadioButton, Switch
//! and Toggle forward attachments to their internal Button. Other custom controls
//! opt in through AttachmentTarget.
//! Radix and Shadcn button/slider factories bind their look automatically.
//! Ordinary application code needs only `.spawn(cx).help("Help", cx)`.
//! Custom owner controls implement `AttachmentTarget` and pass their existing
//! render root and interaction state to `AttachmentHost::render`. Give that root
//! an accessible role and label so its description is exposed to assistive technology.
//!
//! Defaults: hover opens after 500 ms, keyboard focus opens immediately, and
//! leaving both target and bubble hides after 180 ms. Escape, Enter, Space,
//! pointer down, scrolling, and owner movement cancel waiting/visible help.
//! Escape suppresses reopening until hover and focus leave; `Permanently`
//! disables that attachment until replacement. Show-once counts actual painting,
//! not a cancelled delay. Duration also starts when painted, and expiry suppresses
//! reopening until leave. Replacement and owner release cancel both timers.
//!
//! Advanced configuration remains separate from the control's own template:
//!
//! ```no_run
//! use std::time::Duration;
//! use xgpui_luma::prelude::*;
//! let help = Tooltip::new("Save changes")
//!     .shortcut("⌘S")
//!     .delay(Duration::from_millis(300))
//!     .hide_delay(Duration::from_millis(100))
//!     .duration(Some(Duration::from_secs(5)))
//!     .placement(TooltipPlacement::Below)
//!     .dismissal(TooltipDismissal::UntilLeave);
//! ```
//!
//! ```no_run
//! use gpui::{Context, Entity};
//! use xgpui_luma::prelude::*;
//!
//! fn save<M: 'static>(cx: &mut Context<M>) -> Entity<Button> {
//!     let button = Button::new("save").spawn(cx).help("Save changes", cx);
//!     button.clone().tooltip(Tooltip::new("Save changes").shortcut("⌘S").show_once(), cx);
//!     button
//! }
//! ```
mod model;
mod control;
mod template;
pub(crate) mod theme;

pub use model::{TooltipSettings, Tooltip, TooltipPlacement, TooltipDismissal, TooltipEvent};
pub use template::{TooltipTemplate, bubble};
pub use theme::{TooltipPlacementGeometry, TooltipLook, TooltipRenderModel, TooltipTheme, default_tooltip_theme};
pub(crate) use control::Attachment;
