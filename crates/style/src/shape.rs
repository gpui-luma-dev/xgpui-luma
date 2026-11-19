//! Field descriptors: the reflection data `#[derive(Style)]` generates.

/// The value type of a field, for editors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Px,
    Scalar,
    Opacity,
    Color,
    Shadow,
    FontFamily,
    FontWeight,
}

/// A field's value type and whether it may be `None`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldKind {
    pub value: ValueKind,
    pub optional: bool,
}

impl FieldKind {
    pub const fn value(value: ValueKind) -> Self {
        Self { value, optional: false }
    }
}

/// Describes one style: its fields and nested parts.
#[derive(Debug)]
pub struct StyleShape {
    /// Registered path for top-level styles (`"checkbox"`), empty for parts.
    pub name: &'static str,
    pub type_name: &'static str,
    pub doc: &'static str,
    pub fields: &'static [FieldDesc],
}

impl StyleShape {
    pub fn field(&self, name: &str) -> Option<&'static FieldDesc> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// Find a field by dotted path, descending into parts (`"indicator.background"`).
    pub fn find(&self, path: &str) -> Option<&'static FieldDesc> {
        let (head, rest) = match path.split_once('.') {
            Some((head, rest)) => (head, Some(rest)),
            None => (path, None),
        };
        let field = self.field(head)?;
        match (rest, &field.content) {
            (None, _) => Some(field),
            (Some(rest), FieldContent::Part(shape)) => shape.find(rest),
            (Some(_), FieldContent::Value { .. }) => None,
        }
    }

    /// Every value field as (dotted path, descriptor), parts flattened in order.
    pub fn leaves(&self) -> Vec<(String, &'static FieldDesc)> {
        let mut out = Vec::new();
        self.collect_leaves("", &mut out);
        out
    }

    fn collect_leaves(&self, prefix: &str, out: &mut Vec<(String, &'static FieldDesc)>) {
        for field in self.fields {
            let path = if prefix.is_empty() {
                field.name.to_owned()
            } else {
                format!("{prefix}.{}", field.name)
            };
            match &field.content {
                FieldContent::Value { .. } => out.push((path, field)),
                FieldContent::Part(shape) => shape.collect_leaves(&path, out),
            }
        }
    }
}

/// Describes one field.
#[derive(Debug)]
pub struct FieldDesc {
    pub name: &'static str,
    pub label: &'static str,
    /// The field's `///` doc comment.
    pub doc: &'static str,
    pub content: FieldContent,
}

#[derive(Debug)]
pub enum FieldContent {
    Value {
        kind: FieldKind,
        /// Editor range from `#[style(range = a..=b)]`.
        range: Option<(f32, f32)>,
        /// The field is a `States` table.
        stateful: bool,
        /// Derived in Rust; read-only in editors and absent from patches.
        computed: bool,
    },
    Part(&'static StyleShape),
}

/// A top-level style shape registered with `inventory`, for tools that list
/// every style (the lab inspector).
pub struct Registered(pub &'static StyleShape);

inventory::collect!(Registered);

/// Every registered top-level style.
pub fn registered() -> impl Iterator<Item = &'static StyleShape> {
    inventory::iter::<Registered>.into_iter().map(|entry| entry.0)
}
