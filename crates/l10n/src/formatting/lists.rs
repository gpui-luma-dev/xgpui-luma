use icu_list::ListFormatter;
use super::{FormatError, icu_error};
use crate::Locale;

/// CLDR conjunction/disjunction/unit-list patterns, using their wide form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    And,
    Or,
    Unit,
}

impl Locale {
    /// Join already formatted items using CLDR list rules (including contextual
    /// conjunctions). Empty and one-item lists are supported. Item text remains
    /// unchanged; pass the resulting string to Fluent as an argument.
    pub fn fmt_list<S: AsRef<str>>(&self, items: &[S], kind: ListKind) -> Result<String, FormatError> {
        let prefs = (&self.0).into();
        let options = Default::default();
        let formatter = match kind {
            ListKind::And => ListFormatter::try_new_and(prefs, options),
            ListKind::Or => ListFormatter::try_new_or(prefs, options),
            ListKind::Unit => ListFormatter::try_new_unit(prefs, options),
        }
        .map_err(|error| icu_error("list formatter", error))?;
        Ok(formatter.format_to_string(items.iter().map(AsRef::as_ref)))
    }
}
