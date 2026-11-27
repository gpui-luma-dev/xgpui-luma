//! A button that opens something: any content, a trailing chevron, and an open state (#430).
//!
//! [`TriggerButton`] is a [`Button`] with [`TriggerData`], drawn by the button template from the
//! `trigger_button` control (`trigger_button.lss`, which extends `button`). The selector and the popup menu
//! draw their triggers with the same face, so a look that restyles `trigger_button` reaches them all.
//!
//! ```ignore
//! let trigger = trigger_button::new("sort")
//!     .content(|_, _| div().child("Sort by").into_any_element())
//!     .with_data(TriggerData::default().full_width(true))
//!     .spawn(cx);
//! trigger.update(cx, |trigger, cx| trigger.set_data(TriggerData::default().open(true), cx));
//! ```

use std::sync::{Arc, OnceLock};

use gpui::{Entity, SharedString};
use lucide_svg_static::Icon as LucideIcon;

use super::{Button, ButtonBuilder, ButtonTemplate, StyledButtonTemplate};
use crate::infra::icon::IconSource;

/// A spawned trigger button.
pub type TriggerButton = Entity<Button<TriggerData>>;

/// A trigger's data: whether its popup is open, whether it stretches to its parent's width, and its chevron.
/// Build it from [`TriggerData::default`] and its setters; it may gain fields.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TriggerData {
    /// The popup is showing: the trigger draws hovered and sets the `open` state.
    pub open: bool,
    /// Stretch to the parent's width, the content filling the face and the chevron at the end.
    pub full_width: bool,
    /// The trailing icon, in the `icon` color; `None` draws none.
    pub chevron: Option<IconSource>,
}

impl Default for TriggerData {
    fn default() -> Self {
        Self { open: false, full_width: false, chevron: Some(LucideIcon::ChevronDown.into()) }
    }
}

impl TriggerData {
    pub fn open(self, open: bool) -> Self {
        Self { open, ..self }
    }

    pub fn full_width(self, full_width: bool) -> Self {
        Self { full_width, ..self }
    }

    pub fn chevron(self, chevron: Option<IconSource>) -> Self {
        Self { chevron, ..self }
    }
}

/// The SDK's trigger template: the button template reading `trigger_button`.
pub fn default_template() -> Arc<dyn ButtonTemplate<TriggerData>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<TriggerData>>> = OnceLock::new();
    TEMPLATE
        .get_or_init(|| Arc::new(StyledButtonTemplate::<TriggerData>::sdk().control("trigger_button")))
        .clone()
}

/// A trigger button labeled with its id; set its content with `.content(…)` and its data with `.with_data(…)`.
pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<TriggerData> {
    Button::new(id).typed(TriggerData::default()).template(default_template())
}
