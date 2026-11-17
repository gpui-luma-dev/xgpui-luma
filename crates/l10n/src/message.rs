use crate::FluentArgs;

/// A message and its arguments, normally constructed by catalog-generated Rust
/// functions. Constructors require exactly the arguments declared by English.
pub struct Message<'a> {
    pub(crate) id: &'static str,
    pub(crate) attribute: Option<&'static str>,
    pub(crate) args: FluentArgs<'a>,
}

impl<'a> Message<'a> {
    /// For generated code. Applications should use their catalog's constructors.
    #[doc(hidden)]
    pub fn new(id: &'static str, attribute: Option<&'static str>, args: FluentArgs<'a>) -> Self {
        Self { id, attribute, args }
    }
}
