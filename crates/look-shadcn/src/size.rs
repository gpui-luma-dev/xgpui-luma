//! Look-owned sizing for Shadcn control builders.
//!
//! Callers author [`ShadcnSize`] on look-owned builders. Spawn maps to SDK
//! [`xgpui_luma::theme::ControlSize`] inside `.into_sdk_builder(...)`.

use xgpui_luma::theme::ControlSize;

/// Shadcn size axis (`sm` / `default` / `lg`). Default is [`Self::Md`].
///
/// Icon-only chrome is a button role ([`crate::Button::icon_button`]), not a size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ShadcnSize {
    pub const ALL: [Self; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// Shadcn `default` size (same as [`Self::Md`]).
    pub const DEFAULT: Self = Self::Md;

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "default",
            Self::Lg => "lg",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sm => "Small",
            Self::Md => "Default",
            Self::Lg => "Large",
        }
    }

    /// SDK [`ControlSize`] for spawn / look resolvers. Not part of the caller-facing builder API.
    pub(crate) fn control_size(self) -> ControlSize {
        match self {
            Self::Sm => ControlSize::Sm,
            Self::Md => ControlSize::Md,
            Self::Lg => ControlSize::Lg,
        }
    }

    /// Map an SDK size onto [`ShadcnSize`] for spawn callbacks that still pass [`ControlSize`].
    pub(crate) fn from_control_size(size: ControlSize) -> Self {
        match size {
            ControlSize::Sm => Self::Sm,
            ControlSize::Md => Self::Md,
            ControlSize::Lg => Self::Lg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_onto_sdk_control_size() {
        assert_eq!(ShadcnSize::Sm.control_size(), ControlSize::Sm);
        assert_eq!(ShadcnSize::Md.control_size(), ControlSize::Md);
        assert_eq!(ShadcnSize::DEFAULT.control_size(), ControlSize::Md);
        assert_eq!(ShadcnSize::Lg.control_size(), ControlSize::Lg);
        assert_eq!(ShadcnSize::from_control_size(ControlSize::Sm), ShadcnSize::Sm);
        assert_eq!(ShadcnSize::from_control_size(ControlSize::Md), ShadcnSize::Md);
        assert_eq!(ShadcnSize::from_control_size(ControlSize::Lg), ShadcnSize::Lg);
    }

    #[test]
    fn default_is_md() {
        assert_eq!(ShadcnSize::default(), ShadcnSize::Md);
    }
}
