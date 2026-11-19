//! Values and kinds.

use std::fmt;
use std::sync::Arc;

use crate::color::Color;

/// What a property or token holds. A property's kind gives a plain number its meaning (no units).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Logical pixels.
    Length,
    Number,
    Color,
    Shadow,
    Text,
    Font,
    Weight,
    Icon,
    Motion,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "length" => Self::Length,
            "number" => Self::Number,
            "color" => Self::Color,
            "shadow" => Self::Shadow,
            "text" => Self::Text,
            "font" => Self::Font,
            "weight" => Self::Weight,
            "icon" => Self::Icon,
            "motion" => Self::Motion,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Number => "number",
            Self::Color => "color",
            Self::Shadow => "shadow",
            Self::Text => "text",
            Self::Font => "font",
            Self::Weight => "weight",
            Self::Icon => "icon",
            Self::Motion => "motion",
        }
    }

    /// Whether a value of kind `from` may be stored in a property of kind `self`.
    pub(crate) fn accepts(self, from: Kind) -> bool {
        self == from
            || matches!(
                (self, from),
                (Kind::Length | Kind::Number | Kind::Weight, Kind::Length | Kind::Number | Kind::Weight)
                    | (Kind::Icon, Kind::Font)
            )
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One layer of a box shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

/// A text style: `text(size, line_height, weight)`. Fields are readable by path (`body.size`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub line_height: f32,
    pub weight: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
}

impl Easing {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "linear" => Self::Linear,
            "in_quad" => Self::InQuad,
            "out_quad" => Self::OutQuad,
            "in_out_quad" => Self::InOutQuad,
            "in_cubic" => Self::InCubic,
            "out_cubic" => Self::OutCubic,
            "in_out_cubic" => Self::InOutCubic,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::InQuad => "in_quad",
            Self::OutQuad => "out_quad",
            Self::InOutQuad => "in_out_quad",
            Self::InCubic => "in_cubic",
            Self::OutCubic => "out_cubic",
            Self::InOutCubic => "in_out_cubic",
        }
    }
}

/// How a part animates when its state changes. Read by the animation runtime, not applied as style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Instant,
    Timed { duration_ms: f32, easing: Easing },
    Spring { stiffness: f32, damping: f32 },
}

/// A resolved value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f32),
    Color(Color),
    /// Shadow layers; empty means no shadow.
    Shadow(Arc<[ShadowLayer]>),
    Text(TextStyle),
    /// A string: a font family, or an argument such as an icon name.
    Str(Arc<str>),
    Icon(Arc<str>),
    Motion(Motion),
    /// A bare name passed to a function, such as `hover` in `layer(css.accent, hover)`.
    Symbol(Arc<str>),
    List(Arc<[Value]>),
    /// `none`: not painted.
    None,
}

impl Value {
    pub fn kind(&self) -> Option<Kind> {
        Some(match self {
            Value::Number(_) => Kind::Number,
            Value::Color(_) => Kind::Color,
            Value::Shadow(_) => Kind::Shadow,
            Value::Text(_) => Kind::Text,
            Value::Str(_) => Kind::Font,
            Value::Icon(_) => Kind::Icon,
            Value::Motion(_) => Kind::Motion,
            Value::List(items) if items.iter().all(|v| matches!(v, Value::Shadow(_))) => Kind::Shadow,
            Value::Symbol(_) | Value::List(_) | Value::None => return None,
        })
    }

    pub(crate) fn describe(&self) -> &'static str {
        match self {
            Value::Number(_) => "a number",
            Value::Color(_) => "a color",
            Value::Shadow(_) => "a shadow",
            Value::Text(_) => "a text style",
            Value::Str(_) => "a string",
            Value::Icon(_) => "an icon",
            Value::Motion(_) => "a motion",
            Value::Symbol(_) => "a bare name",
            Value::List(_) => "a list",
            Value::None => "none",
        }
    }

    pub fn as_number(&self) -> Option<f32> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<Color> {
        match self {
            Value::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_shadow(&self) -> Option<&[ShadowLayer]> {
        match self {
            Value::Shadow(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) | Value::Icon(s) | Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_motion(&self) -> Option<Motion> {
        match self {
            Value::Motion(m) => Some(*m),
            _ => None,
        }
    }

    /// Convert to the representation a property of `kind` stores, if the value fits.
    pub(crate) fn coerce(self, kind: Kind) -> Result<Value, Value> {
        match (kind, self) {
            (_, Value::None) => Ok(Value::None),
            (Kind::Length | Kind::Number | Kind::Weight, v @ Value::Number(_)) => Ok(v),
            (Kind::Color, v @ Value::Color(_)) => Ok(v),
            (Kind::Shadow, v @ Value::Shadow(_)) => Ok(v),
            (Kind::Shadow, Value::List(items)) => {
                let mut layers = Vec::new();
                for item in items.iter() {
                    match item {
                        Value::Shadow(s) => layers.extend_from_slice(s),
                        _ => return Err(Value::List(items)),
                    }
                }
                Ok(Value::Shadow(layers.into()))
            }
            (Kind::Text, v @ Value::Text(_)) => Ok(v),
            (Kind::Font, v @ Value::Str(_)) => Ok(v),
            (Kind::Icon, v @ Value::Icon(_)) => Ok(v),
            (Kind::Icon, Value::Str(s)) => Ok(Value::Icon(s)),
            (Kind::Motion, v @ Value::Motion(_)) => Ok(v),
            (_, other) => Err(other),
        }
    }

    /// Interpolate for animations: numbers, colors (in OKLab) and same-length shadows; others switch at 0.5.
    pub fn lerp(&self, other: &Value, t: f32) -> Value {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(mix(*a, *b)),
            (Value::Color(a), Value::Color(b)) => Value::Color(a.mix(*b, t)),
            (Value::Shadow(a), Value::Shadow(b)) if a.len() == b.len() => Value::Shadow(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| ShadowLayer {
                        x: mix(a.x, b.x),
                        y: mix(a.y, b.y),
                        blur: mix(a.blur, b.blur),
                        spread: mix(a.spread, b.spread),
                        color: a.color.mix(b.color, t),
                        inset: if t < 0.5 { a.inset } else { b.inset },
                    })
                    .collect(),
            ),
            _ => {
                if t < 0.5 {
                    self.clone()
                } else {
                    other.clone()
                }
            }
        }
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Value::Number(n)
    }
}

impl From<Color> for Value {
    fn from(c: Color) -> Self {
        Value::Color(c)
    }
}

impl From<Motion> for Value {
    fn from(m: Motion) -> Self {
        Value::Motion(m)
    }
}

impl From<Arc<[ShadowLayer]>> for Value {
    fn from(s: Arc<[ShadowLayer]>) -> Self {
        Value::Shadow(s)
    }
}

impl From<Arc<str>> for Value {
    fn from(s: Arc<str>) -> Self {
        Value::Str(s)
    }
}

impl From<TextStyle> for Value {
    fn from(t: TextStyle) -> Self {
        Value::Text(t)
    }
}
