//! Leaf value types: the values a style field can hold.

use gpui::{BoxShadow, FontWeight, Hsla, SharedString};

use crate::lerp::Lerp;
use crate::resolve::ResolveCx;
use crate::shape::{FieldKind, ValueKind};

/// A value without its Rust type: what tokens and field references produce.
#[derive(Clone, Debug, PartialEq)]
pub enum Dyn {
    Number(f32),
    Color(Hsla),
    Shadow(Vec<BoxShadow>),
    Text(SharedString),
    None,
}

/// A type a style field can hold, wrapped in [`V`](crate::V).
pub trait Leaf: Clone + std::fmt::Debug + Default + PartialEq + 'static {
    /// What templates read after resolving.
    type Resolved: Clone + std::fmt::Debug + PartialEq + Lerp;
    /// How descriptors and editors describe this type.
    const KIND: FieldKind;
    fn from_dyn(value: &Dyn) -> Option<Self>;
    fn to_dyn(&self) -> Dyn;
    /// Convert to the resolved form; `snap` asks lengths to snap to device pixels.
    fn finish(self, cx: &ResolveCx<'_>, snap: bool) -> Self::Resolved;
}

/// A leaf that supports arithmetic in expressions.
pub trait Numeric: Leaf {
    fn from_f32(value: f32) -> Self;
    fn to_f32(&self) -> f32;
}

macro_rules! numeric_leaf {
    ($(#[$meta:meta])* $name:ident, $kind:expr, |$value:ident, $cx:ident, $snap:ident| $finish:expr) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name(pub f32);

        impl Leaf for $name {
            type Resolved = f32;
            const KIND: FieldKind = FieldKind::value($kind);
            fn from_dyn(value: &Dyn) -> Option<Self> {
                match value {
                    Dyn::Number(number) => Some(Self(*number)),
                    _ => None,
                }
            }
            fn to_dyn(&self) -> Dyn {
                Dyn::Number(self.0)
            }
            fn finish(self, $cx: &ResolveCx<'_>, $snap: bool) -> f32 {
                let $value = self.0;
                $finish
            }
        }

        impl Numeric for $name {
            fn from_f32(value: f32) -> Self {
                Self(value)
            }
            fn to_f32(&self) -> f32 {
                self.0
            }
        }
    };
}

numeric_leaf!(
    /// A length in logical pixels. Snaps to device pixels when the style asks.
    Px,
    ValueKind::Px,
    |value, cx, snap| if snap { cx.snap(value) } else { value }
);
numeric_leaf!(
    /// A plain number: ratios and multipliers.
    Scalar,
    ValueKind::Scalar,
    |value, _cx, _snap| value
);
numeric_leaf!(
    /// An opacity from 0 to 1.
    Opacity,
    ValueKind::Opacity,
    |value, _cx, _snap| value.clamp(0.0, 1.0)
);

/// A paint color.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color(pub Hsla);

impl Leaf for Color {
    type Resolved = Hsla;
    const KIND: FieldKind = FieldKind::value(ValueKind::Color);
    fn from_dyn(value: &Dyn) -> Option<Self> {
        match value {
            Dyn::Color(color) => Some(Self(*color)),
            _ => None,
        }
    }
    fn to_dyn(&self) -> Dyn {
        Dyn::Color(self.0)
    }
    fn finish(self, _cx: &ResolveCx<'_>, _snap: bool) -> Hsla {
        self.0
    }
}

/// A list of box-shadow layers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shadow(pub Vec<BoxShadow>);

impl Leaf for Shadow {
    type Resolved = Vec<BoxShadow>;
    const KIND: FieldKind = FieldKind::value(ValueKind::Shadow);
    fn from_dyn(value: &Dyn) -> Option<Self> {
        match value {
            Dyn::Shadow(layers) => Some(Self(layers.clone())),
            _ => None,
        }
    }
    fn to_dyn(&self) -> Dyn {
        Dyn::Shadow(self.0.clone())
    }
    fn finish(self, _cx: &ResolveCx<'_>, _snap: bool) -> Vec<BoxShadow> {
        self.0
    }
}

/// A font family name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontFamily(pub SharedString);

impl Leaf for FontFamily {
    type Resolved = SharedString;
    const KIND: FieldKind = FieldKind::value(ValueKind::FontFamily);
    fn from_dyn(value: &Dyn) -> Option<Self> {
        match value {
            Dyn::Text(name) => Some(Self(name.clone())),
            _ => None,
        }
    }
    fn to_dyn(&self) -> Dyn {
        Dyn::Text(self.0.clone())
    }
    fn finish(self, _cx: &ResolveCx<'_>, _snap: bool) -> SharedString {
        self.0
    }
}

/// A font weight (100 to 900).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weight(pub FontWeight);

impl Default for Weight {
    fn default() -> Self {
        Self(FontWeight::NORMAL)
    }
}

impl Leaf for Weight {
    type Resolved = FontWeight;
    const KIND: FieldKind = FieldKind::value(ValueKind::FontWeight);
    fn from_dyn(value: &Dyn) -> Option<Self> {
        match value {
            Dyn::Number(weight) => Some(Self(FontWeight(*weight))),
            _ => None,
        }
    }
    fn to_dyn(&self) -> Dyn {
        Dyn::Number(self.0.0)
    }
    fn finish(self, _cx: &ResolveCx<'_>, _snap: bool) -> FontWeight {
        self.0
    }
}

/// `None` is "not painted". A token or reference resolving to [`Dyn::None`]
/// gives `None`.
impl<T: Leaf> Leaf for Option<T>
where
    Option<T::Resolved>: Lerp,
{
    type Resolved = Option<T::Resolved>;
    const KIND: FieldKind = FieldKind { optional: true, ..T::KIND };
    fn from_dyn(value: &Dyn) -> Option<Self> {
        match value {
            Dyn::None => Some(None),
            other => T::from_dyn(other).map(Some),
        }
    }
    fn to_dyn(&self) -> Dyn {
        self.as_ref().map_or(Dyn::None, Leaf::to_dyn)
    }
    fn finish(self, cx: &ResolveCx<'_>, snap: bool) -> Self::Resolved {
        self.map(|value| value.finish(cx, snap))
    }
}

/// A length literal.
pub fn px(value: f32) -> Px {
    Px(value)
}

/// A plain-number literal.
pub fn scalar(value: f32) -> Scalar {
    Scalar(value)
}

/// An opacity literal.
pub fn opacity(value: f32) -> Opacity {
    Opacity(value)
}
