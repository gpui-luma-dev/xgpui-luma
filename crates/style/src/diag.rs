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

/// What kind of problem a diagnostic is, for tools that group, filter or offer fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Code {
    Syntax,
    Config,
    UnknownLayer,
    UnknownControl,
    UnknownPart,
    UnknownProperty,
    UnknownState,
    UnknownAttribute,
    UnknownMode,
    UnknownKind,
    UnknownName,
    UnknownFunction,
    WrongArgumentCount,
    KindMismatch,
    AmbiguousName,
    Duplicate,
    Declaration,
    Extends,
    Cycle,
    AbsentToken,
    Evaluation,
    Io,
}

/// One problem, with its location when it has one.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Code,
    /// The style file's name, e.g. `"radix/checkbox.lss"`.
    pub source: Option<Arc<str>>,
    pub line: u32,
    pub column: u32,
    /// Byte range in the source, for editors to underline.
    pub range: Option<(u32, u32)>,
    pub message: String,
    /// The source line the problem is on, for display.
    pub snippet: Option<String>,
}

impl Diagnostic {
    pub(crate) fn error(code: Code, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            source: None,
            line: 0,
            column: 0,
            range: None,
            message: message.into(),
            snippet: None,
        }
    }

    /// A diagnostic located at `span` in a file with the given name and text.
    pub(crate) fn at(
        severity: Severity,
        code: Code,
        span: Span,
        file: Option<(&Arc<str>, &str)>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code,
            source: file.map(|(name, _)| name.clone()),
            line: span.line,
            column: span.column,
            range: file.map(|_| (span.start, span.end)),
            message: message.into(),
            snippet: file
                .and_then(|(_, text)| text.lines().nth(span.line.saturating_sub(1) as usize))
                .map(str::to_owned),
        }
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

    pub fn len(&self) -> usize {
        self.0.len()
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

    pub fn has_code(&self, code: Code) -> bool {
        self.0.iter().any(|d| d.code == code)
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

/// `, did you mean "x"?` for the closest candidate within a small edit distance, or empty.
pub(crate) fn suggest<'a>(name: &str, candidates: impl IntoIterator<Item = &'a str>) -> String {
    let len = name.chars().count();
    let limit = (len / 3).clamp(2, 3).min(len.saturating_sub(1));
    candidates
        .into_iter()
        .filter(|c| *c != name)
        .map(|c| (distance(name, c), c))
        .filter(|(d, _)| *d <= limit)
        .min()
        .map(|(_, c)| format!(" — did you mean \"{c}\"?"))
        .unwrap_or_default()
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    #[test]
    fn suggests_close_names_only() {
        assert_eq!(super::suggest("indicatr", ["indicator", "glyph"]), " — did you mean \"indicator\"?");
        assert_eq!(super::suggest("xyz", ["indicator", "glyph"]), "");
    }
}
