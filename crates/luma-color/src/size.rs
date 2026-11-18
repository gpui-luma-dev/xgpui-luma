//! Size vocabulary for luma-color components.
//!
//! These sizes are owned by this crate and resolve to pixel dimensions here; the
//! SDK controls underneath have no size axis.

/// Component size for color sliders and swatches. Default is [`Self::Medium`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorControlSize {
    Small,
    #[default]
    Medium,
    Large,
}
