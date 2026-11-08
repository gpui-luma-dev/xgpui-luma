use xgpui_luma::theme::{LumaTextRole, LumaTextScale};

/// Semantic shadcn color tokens mapped from CSS custom properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnToken {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
}

/// A color token with an optional opacity modifier (Tailwind `/opacity` suffix).
#[derive(Clone, Copy, Debug)]
pub struct ShadcnStyle {
    pub token: ShadcnToken,
    pub opacity: Option<f32>,
}

impl ShadcnToken {
    pub const ALL: [Self; 19] = [
        Self::Background,
        Self::Foreground,
        Self::Card,
        Self::CardForeground,
        Self::Popover,
        Self::PopoverForeground,
        Self::Primary,
        Self::PrimaryForeground,
        Self::Secondary,
        Self::SecondaryForeground,
        Self::Muted,
        Self::MutedForeground,
        Self::Accent,
        Self::AccentForeground,
        Self::Destructive,
        Self::DestructiveForeground,
        Self::Border,
        Self::Input,
        Self::Ring,
    ];

    /// CSS custom-property stem (without the `--` prefix).
    pub fn css_name(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Foreground => "foreground",
            Self::Card => "card",
            Self::CardForeground => "card-foreground",
            Self::Popover => "popover",
            Self::PopoverForeground => "popover-foreground",
            Self::Primary => "primary",
            Self::PrimaryForeground => "primary-foreground",
            Self::Secondary => "secondary",
            Self::SecondaryForeground => "secondary-foreground",
            Self::Muted => "muted",
            Self::MutedForeground => "muted-foreground",
            Self::Accent => "accent",
            Self::AccentForeground => "accent-foreground",
            Self::Destructive => "destructive",
            Self::DestructiveForeground => "destructive-foreground",
            Self::Border => "border",
            Self::Input => "input",
            Self::Ring => "ring",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::Background => 0,
            Self::Foreground => 1,
            Self::Card => 2,
            Self::CardForeground => 3,
            Self::Popover => 4,
            Self::PopoverForeground => 5,
            Self::Primary => 6,
            Self::PrimaryForeground => 7,
            Self::Secondary => 8,
            Self::SecondaryForeground => 9,
            Self::Muted => 10,
            Self::MutedForeground => 11,
            Self::Accent => 12,
            Self::AccentForeground => 13,
            Self::Destructive => 14,
            Self::DestructiveForeground => 15,
            Self::Border => 16,
            Self::Input => 17,
            Self::Ring => 18,
        }
    }

    pub(crate) fn is_foreground(self) -> bool {
        matches!(
            self,
            Self::Foreground
                | Self::CardForeground
                | Self::PopoverForeground
                | Self::PrimaryForeground
                | Self::SecondaryForeground
                | Self::MutedForeground
                | Self::AccentForeground
                | Self::DestructiveForeground
        )
    }

    /// Chainable opacity modifier matching the `/opacity` Tailwind suffix.
    pub fn opacity(self, alpha: f32) -> ShadcnStyle {
        ShadcnStyle { token: self, opacity: Some(alpha) }
    }
}

impl From<ShadcnToken> for ShadcnStyle {
    fn from(token: ShadcnToken) -> Self {
        ShadcnStyle { token, opacity: None }
    }
}

/// Font families defined in the CSS layout (e.g. `--font-sans`, `--font-mono`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnFont {
    Sans,
    Serif,
    Mono,
}

/// Border radius slots computed relative to the base theme `--radius`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnRadius {
    None,
    Sm,
    Md,
    Lg,
    Xl,
}

/// Box shadow elevations defined in the CSS (e.g. `--shadow-sm`, `--shadow-md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnShadow {
    None,
    TwoXs,
    Xs,
    Sm,
    Default,
    Md,
    Lg,
    Xl,
    TwoXl,
}

/// Semantic typography roles for structural headings and body copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnTextRole {
    H1,
    H2,
    H3,
    H4,
    P,
}

impl ShadcnTextRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::P => "p",
        }
    }
}

impl From<ShadcnTextRole> for LumaTextRole {
    fn from(value: ShadcnTextRole) -> Self {
        match value {
            ShadcnTextRole::H1 => Self::H1,
            ShadcnTextRole::H2 => Self::H2,
            ShadcnTextRole::H3 => Self::H3,
            ShadcnTextRole::H4 => Self::H4,
            ShadcnTextRole::P => Self::P,
        }
    }
}

/// Standardized typographic scale matching Tailwind text sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnTextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    TwoXl,
}

impl ShadcnTextSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Base => "base",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::TwoXl => "2xl",
        }
    }
}

impl From<ShadcnTextSize> for LumaTextScale {
    fn from(value: ShadcnTextSize) -> Self {
        match value {
            ShadcnTextSize::Xs => Self::Xs,
            ShadcnTextSize::Sm => Self::Sm,
            ShadcnTextSize::Base => Self::Md,
            ShadcnTextSize::Lg => Self::Lg,
            ShadcnTextSize::Xl => Self::Xl,
            ShadcnTextSize::TwoXl => Self::TwoXl,
        }
    }
}
