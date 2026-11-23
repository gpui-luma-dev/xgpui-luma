//! Look-owned sizing for Shadcn control builders.
//!
//! Callers author [`ShadcnSize`] on look-owned builders. The look resolves the
//! selected size to pixel metrics and geometry before creating SDK controls; the
//! SDK itself has no size axis.

use xgpui_luma::theme::ControlMetricTokens;

/// Shadcn size axis (`sm` / `default` / `lg`). Default is [`Self::Md`].
///
/// Icon-only chrome is a button role ([`crate::Button::icon_button`]), not a size.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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

    /// Key for size-specific stylesheet entries (`sizes.sm`, `metrics.md`, ...).
    pub fn key(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// Shadcn control metrics for each [`ShadcnSize`], resolved to pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadcnControlMetrics {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

impl ShadcnControlMetrics {
    pub fn get(&self, size: ShadcnSize) -> ControlMetricTokens {
        match size {
            ShadcnSize::Sm => self.sm,
            ShadcnSize::Md => self.md,
            ShadcnSize::Lg => self.lg,
        }
    }

    /// Alias for [`Self::get`], matching the former SDK metric API.
    pub fn for_size(&self, size: ShadcnSize) -> ControlMetricTokens {
        self.get(size)
    }

    pub fn control_height(&self, size: ShadcnSize) -> f32 {
        self.get(size).control_height
    }

    pub fn padding_x(&self, size: ShadcnSize) -> f32 {
        self.get(size).padding_x
    }

    pub fn padding_y(&self, size: ShadcnSize) -> f32 {
        self.get(size).padding_y
    }

    pub fn gap(&self, size: ShadcnSize) -> f32 {
        self.get(size).gap
    }

    pub fn radius(&self, size: ShadcnSize) -> f32 {
        self.get(size).radius
    }

    pub fn icon_size(&self, size: ShadcnSize) -> f32 {
        self.get(size).icon_size
    }

    pub fn get_mut(&mut self, size: ShadcnSize) -> &mut ControlMetricTokens {
        match size {
            ShadcnSize::Sm => &mut self.sm,
            ShadcnSize::Md => &mut self.md,
            ShadcnSize::Lg => &mut self.lg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_md() {
        assert_eq!(ShadcnSize::default(), ShadcnSize::Md);
        assert_eq!(ShadcnSize::DEFAULT.key(), "md");
    }
}
