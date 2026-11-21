//! Handles: cheap, copyable names for controls, parts, properties, states and attributes.
//!
//! Controls, parts, attribute values and properties are interned by name in a process-wide table,
//! so a handle stays valid across rebinds and when a different stylesheet is compiled. States and
//! attributes are indices into the shared [`Config`](crate::Config).

use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, OnceLock};

use crate::color::Color;
use crate::value::{Kind, Motion, ShadowLayer, TextStyle, Value};

/// The most attributes a [`Config`](crate::Config) may declare.
pub const MAX_ATTRIBUTES: usize = 8;

/// Intern a name for the life of the process.
pub(crate) fn intern(name: &str) -> &'static str {
    static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut names = NAMES.get_or_init(|| Mutex::new(HashSet::new())).lock().expect("name table lock");
    if let Some(existing) = names.get(name) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    names.insert(leaked);
    leaked
}

macro_rules! name_handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(&'static str);

        impl $name {
            /// A handle for a name known at compile time (no lookup).
            pub const fn new(name: &'static str) -> Self {
                Self(name)
            }

            /// A handle for a name known only at runtime (interned).
            pub fn named(name: &str) -> Self {
                Self(intern(name))
            }

            pub fn name(&self) -> &'static str {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.0)
            }
        }
    };
}

name_handle!(
    /// A control, e.g. `ControlId::new("checkbox")`.
    ControlId
);
name_handle!(
    /// A part of a control, e.g. `PartId::new("indicator")`. Every control has [`PartId::ROOT`].
    PartId
);
name_handle!(
    /// An attribute value, e.g. `AttrValue::new("soft")` for `variant soft`.
    AttrValue
);

impl PartId {
    /// The control itself.
    pub const ROOT: PartId = PartId("root");
}

/// A state declared in the [`Config`](crate::Config).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(pub(crate) u8);

/// An attribute declared in the [`Config`](crate::Config).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttrId(pub(crate) u8);

/// A set of active states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StateSet(pub(crate) u32);

impl StateSet {
    pub const EMPTY: StateSet = StateSet(0);

    pub fn with(self, state: StateId) -> Self {
        StateSet(self.0 | (1 << state.0))
    }

    pub fn with_if(self, state: StateId, active: bool) -> Self {
        if active { self.with(state) } else { self }
    }

    pub fn without(self, state: StateId) -> Self {
        StateSet(self.0 & !(1 << state.0))
    }

    pub fn contains(self, state: StateId) -> bool {
        self.0 & (1 << state.0) != 0
    }

    pub fn union(self, other: StateSet) -> Self {
        StateSet(self.0 | other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl FromIterator<StateId> for StateSet {
    fn from_iter<I: IntoIterator<Item = StateId>>(iter: I) -> Self {
        iter.into_iter().fold(StateSet::EMPTY, StateSet::with)
    }
}

/// What one render asks for: a control, its attribute values, its active states and the display scale.
/// Small and `Copy`; building one allocates nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyleKey {
    pub(crate) control: ControlId,
    pub(crate) states: StateSet,
    pub(crate) attrs: [Option<AttrValue>; MAX_ATTRIBUTES],
    pub(crate) scale_bits: u32,
}

impl StyleKey {
    pub fn new(control: ControlId) -> Self {
        Self { control, states: StateSet::EMPTY, attrs: [None; MAX_ATTRIBUTES], scale_bits: 1.0f32.to_bits() }
    }

    pub fn state(mut self, state: StateId) -> Self {
        self.states = self.states.with(state);
        self
    }

    pub fn state_if(mut self, state: StateId, active: bool) -> Self {
        self.states = self.states.with_if(state, active);
        self
    }

    /// Set or clear one state (tools toggling states; templates usually build keys with `state_if`).
    pub fn set_state(mut self, state: StateId, active: bool) -> Self {
        self.states = if active {
            self.states.with(state)
        } else {
            self.states.without(state)
        };
        self
    }

    pub fn states(mut self, states: StateSet) -> Self {
        self.states = self.states.union(states);
        self
    }

    pub fn attr(mut self, attr: AttrId, value: AttrValue) -> Self {
        self.attrs[attr.0 as usize] = Some(value);
        self
    }

    pub fn attr_opt(mut self, attr: AttrId, value: Option<AttrValue>) -> Self {
        self.attrs[attr.0 as usize] = value;
        self
    }

    /// Device pixels per logical pixel, used for snapping lengths.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale_bits = scale.to_bits();
        self
    }

    pub fn control(&self) -> ControlId {
        self.control
    }

    pub fn active_states(&self) -> StateSet {
        self.states
    }

    pub fn attr_value(&self, attr: AttrId) -> Option<AttrValue> {
        self.attrs[attr.0 as usize]
    }

    pub fn scale_factor(&self) -> f32 {
        f32::from_bits(self.scale_bits)
    }
}

/// A typed handle to a property. Core properties are constants in [`props`](crate::props); declared
/// properties come from [`Stylesheet::prop`](crate::Stylesheet::prop), which checks the kind.
pub struct Prop<T> {
    name: &'static str,
    _type: PhantomData<fn() -> T>,
}

impl<T> Prop<T> {
    pub const fn new(name: &'static str) -> Self {
        Self { name, _type: PhantomData }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Clone for Prop<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Prop<T> {}
impl<T> fmt::Debug for Prop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prop({})", self.name)
    }
}

/// A Rust type a property value can be read as.
pub trait FromValue: Sized {
    /// Property kinds this type can read.
    const KINDS: &'static [Kind];
    fn from_value(value: &Value) -> Option<Self>;
}

impl FromValue for f32 {
    const KINDS: &'static [Kind] = &[Kind::Length, Kind::Number, Kind::Weight];
    fn from_value(value: &Value) -> Option<Self> {
        value.as_number()
    }
}

impl FromValue for Color {
    const KINDS: &'static [Kind] = &[Kind::Color];
    fn from_value(value: &Value) -> Option<Self> {
        value.as_color()
    }
}

impl FromValue for Motion {
    const KINDS: &'static [Kind] = &[Kind::Motion];
    fn from_value(value: &Value) -> Option<Self> {
        value.as_motion()
    }
}

impl FromValue for Arc<[ShadowLayer]> {
    const KINDS: &'static [Kind] = &[Kind::Shadow];
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Shadow(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl FromValue for Arc<str> {
    const KINDS: &'static [Kind] = &[Kind::Font, Kind::Icon];
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(s) | Value::Icon(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl FromValue for TextStyle {
    const KINDS: &'static [Kind] = &[Kind::Text];
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Text(t) => Some(*t),
            _ => None,
        }
    }
}

/// Handles for the core properties (language §9).
pub mod props {
    use super::Prop;
    use crate::color::Color;
    use crate::value::{Motion, ShadowLayer};
    use std::sync::Arc;

    pub const BACKGROUND: Prop<Color> = Prop::new("background");
    pub const FOREGROUND: Prop<Color> = Prop::new("foreground");
    pub const BORDER: Prop<Color> = Prop::new("border");
    pub const FOCUS_BORDER: Prop<Color> = Prop::new("focus_border");
    pub const BORDER_WIDTH: Prop<f32> = Prop::new("border_width");
    pub const RADIUS: Prop<f32> = Prop::new("radius");
    pub const PADDING: Prop<f32> = Prop::new("padding");
    pub const PADDING_X: Prop<f32> = Prop::new("padding_x");
    pub const PADDING_Y: Prop<f32> = Prop::new("padding_y");
    pub const MARGIN: Prop<f32> = Prop::new("margin");
    pub const GAP: Prop<f32> = Prop::new("gap");
    pub const WIDTH: Prop<f32> = Prop::new("width");
    pub const HEIGHT: Prop<f32> = Prop::new("height");
    pub const MIN_WIDTH: Prop<f32> = Prop::new("min_width");
    pub const MAX_WIDTH: Prop<f32> = Prop::new("max_width");
    pub const MIN_HEIGHT: Prop<f32> = Prop::new("min_height");
    pub const MAX_HEIGHT: Prop<f32> = Prop::new("max_height");
    pub const OPACITY: Prop<f32> = Prop::new("opacity");
    pub const SHADOW: Prop<Arc<[ShadowLayer]>> = Prop::new("shadow");
    pub const FONT_FAMILY: Prop<Arc<str>> = Prop::new("font_family");
    pub const FONT_SIZE: Prop<f32> = Prop::new("font_size");
    pub const LINE_HEIGHT: Prop<f32> = Prop::new("line_height");
    pub const FONT_WEIGHT: Prop<f32> = Prop::new("font_weight");
    pub const TRANSITION: Prop<Motion> = Prop::new("transition");
}
