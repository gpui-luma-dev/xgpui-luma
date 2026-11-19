//! Diagnostics: problems found while compiling, binding or resolving styles.

use std::fmt;
use std::sync::Arc;

/// A position in a style file: byte range plus 1-based line and column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub(crate) file: u32,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// One problem, with its location when it has one.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// The style file's name, e.g. `"radix/checkbox.lss"`.
    pub source: Option<Arc<str>>,
    pub line: u32,
    pub column: u32,
    pub message: String,
    /// The source line the problem is on, for display.
    pub snippet: Option<String>,
}

impl Diagnostic {
    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self { severity: Severity::Error, source: None, line: 0, column: 0, message: message.into(), snippet: None }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        match &self.source {
            Some(source) => write!(f, "{source}:{}:{}: {level}: {}", self.line, self.column, self.message)?,
            None => write!(f, "{level}: {}", self.message)?,
        }
        if let Some(snippet) = &self.snippet {
            let gutter = self.line.to_string().len();
            write!(
                f,
                "\n{:gutter$} |\n{} | {snippet}\n{:gutter$} | {:>col$}",
                "",
                self.line,
                "",
                "^",
                col = self.column as usize
            )?;
        }
        Ok(())
    }
}

/// A list of diagnostics; the error type of compile and bind.
#[derive(Clone, Debug, Default, PartialEq, thiserror::Error)]
pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter()
    }

    /// True when some message contains `text` (handy in tests).
    pub fn mentions(&self, text: &str) -> bool {
        self.0.iter().any(|d| d.message.contains(text))
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, d) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{d}")?;
        }
        Ok(())
    }
}
