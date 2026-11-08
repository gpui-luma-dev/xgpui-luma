//! Convenient imports for shadcn look styling and control construction.
//!
//! ```ignore
//! use xgpui_luma_look_shadcn as shadcn;
//! use xgpui_luma_look_shadcn::prelude::*;
//!
//! let submit = shadcn::Button::new("submit").look(&look).primary().label("Submit").spawn(cx);
//! with_look(&look, || {
//!     div()
//!         .bg_cn(ShadcnToken::Background)
//!         .text_cn(ShadcnToken::Foreground)
//! });
//! ```

pub use crate::context::{sync_color_control_theme, with_look};
pub use crate::controls::{ShadcnButtonStyle, ShadcnToolbarItemExt};
pub use crate::elements::{Badge, BadgeIconPlacement, BadgeVariant};
pub use crate::ext::{LumaTypographyExt, ShadcnElementExt};
pub use crate::look::ShadcnLook;
pub use crate::size::ShadcnSize;
pub use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnStyle, ShadcnTextRole, ShadcnTextSize, ShadcnToken};
