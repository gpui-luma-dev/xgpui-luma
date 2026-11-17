use gpui::{AbsoluteLength, DefiniteLength, Length, Styled};
use super::LayoutDirection;

/// Horizontal start/end styling for localized UI. These helpers write physical
/// GPUI properties using the supplied direction, without reading global state.
/// Call them while building a fresh element; switching direction on an existing
/// style does not clear previously set physical values. Vertical writing is not
/// supported. Nonlocalized geometry should continue to use physical properties.
pub trait LogicalStyled: Styled {
    /// Inline-start padding.
    fn ps(mut self, value: impl Into<DefiniteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().padding.left = Some(value.into()),
            LayoutDirection::Rtl => self.style().padding.right = Some(value.into()),
        };
        self
    }
    /// Inline-end padding.
    fn pe(mut self, value: impl Into<DefiniteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().padding.right = Some(value.into()),
            LayoutDirection::Rtl => self.style().padding.left = Some(value.into()),
        };
        self
    }
    /// Inline-start margin (including `auto`).
    fn ms(mut self, value: impl Into<Length>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().margin.left = Some(value.into()),
            LayoutDirection::Rtl => self.style().margin.right = Some(value.into()),
        };
        self
    }
    /// Inline-end margin (including `auto`).
    fn me(mut self, value: impl Into<Length>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().margin.right = Some(value.into()),
            LayoutDirection::Rtl => self.style().margin.left = Some(value.into()),
        };
        self
    }
    /// Inline-start inset; requires positioned layout, just like `left`/`right`.
    fn start(mut self, value: impl Into<Length>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().inset.left = Some(value.into()),
            LayoutDirection::Rtl => self.style().inset.right = Some(value.into()),
        };
        self
    }
    /// Inline-end inset.
    fn end(mut self, value: impl Into<Length>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().inset.right = Some(value.into()),
            LayoutDirection::Rtl => self.style().inset.left = Some(value.into()),
        };
        self
    }
    /// Inline-start border width.
    fn border_s(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().border_widths.left = Some(value.into()),
            LayoutDirection::Rtl => self.style().border_widths.right = Some(value.into()),
        };
        self
    }
    /// Inline-end border width.
    fn border_e(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().border_widths.right = Some(value.into()),
            LayoutDirection::Rtl => self.style().border_widths.left = Some(value.into()),
        };
        self
    }
    /// Block-start/inline-start corner radius (top-start).
    fn rounded_ss(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().corner_radii.top_left = Some(value.into()),
            LayoutDirection::Rtl => self.style().corner_radii.top_right = Some(value.into()),
        };
        self
    }
    /// Block-start/inline-end corner radius (top-end).
    fn rounded_se(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().corner_radii.top_right = Some(value.into()),
            LayoutDirection::Rtl => self.style().corner_radii.top_left = Some(value.into()),
        };
        self
    }
    /// Block-end/inline-start corner radius (bottom-start).
    fn rounded_es(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().corner_radii.bottom_left = Some(value.into()),
            LayoutDirection::Rtl => self.style().corner_radii.bottom_right = Some(value.into()),
        };
        self
    }
    /// Block-end/inline-end corner radius (bottom-end).
    fn rounded_ee(mut self, value: impl Into<AbsoluteLength>, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.style().corner_radii.bottom_right = Some(value.into()),
            LayoutDirection::Rtl => self.style().corner_radii.bottom_left = Some(value.into()),
        };
        self
    }
    /// Lay out children from inline start to inline end. Like GPUI's `flex_row`,
    /// this sets direction only; use `.flex()` to enable flex layout.
    fn flex_row_logical(self, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => self.flex_row(),
            LayoutDirection::Rtl => self.flex_row_reverse(),
        }
    }
}
impl<T: Styled> LogicalStyled for T {}
