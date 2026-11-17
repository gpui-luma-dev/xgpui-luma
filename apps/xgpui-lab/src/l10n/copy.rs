//! Presentation-boundary bridge for the Lab's existing string-based screen API.
//! The extracted catalog generates typed message constructors and source maps.
//! Source text/route IDs stay stable; display text is resolved on every render.
use gpui::{AnyElement, App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};
use xgpui_luma_l10n::{Localizer, formatting::Decimal};
use crate::screen::localized::{self, TextKind};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/lab_copy.rs"));
}

pub(crate) fn resolve(source: &str, cx: &App) -> SharedString {
    let Some(localizer) = cx.try_global::<Localizer>() else {
        return source.to_owned().into();
    };
    if let Some(message) = generated::literal(source) {
        return localizer.format(message).into();
    }
    if let Some((owner, title)) = source.split_once("  /  ") {
        return format!("{owner}  /  {}", resolve(title, cx)).into();
    }
    for pattern in generated::PATTERNS {
        if let Some(values) = capture(source, pattern.parts) {
            let values: Vec<String> = values.into_iter().map(|value| number(value, cx).to_string()).collect();
            if let Some(message) = generated::formatted(pattern.id, &values) {
                return localizer.format(message).into();
            }
        }
    }
    source.to_owned().into()
}

pub(super) struct Pattern {
    pub id: &'static str,
    pub parts: &'static [&'static str],
}
fn capture<'a>(source: &'a str, parts: &[&str]) -> Option<Vec<&'a str>> {
    let (prefix, rest) = parts.split_first()?;
    let mut source = source.strip_prefix(prefix)?;
    let mut values = Vec::with_capacity(rest.len());
    for (index, part) in rest.iter().enumerate() {
        if index + 1 == rest.len() {
            values.push(source.strip_suffix(part)?);
            source = "";
        } else {
            // Adjacent fields cannot be recovered unambiguously at this legacy
            // string boundary; the extractor excludes those source patterns.
            if part.is_empty() {
                return None;
            }
            let end = source.find(part)?;
            values.push(&source[..end]);
            source = &source[end + part.len()..];
        }
    }
    source.is_empty().then_some(values)
}

pub(crate) fn number(source: &str, cx: &App) -> SharedString {
    let Some(localizer) = cx.try_global::<Localizer>() else {
        return source.to_owned().into();
    };
    if let Ok(number) = source.parse::<Decimal>() {
        if let Ok(formatted) = localizer.locale().fmt_number(&number) {
            return formatted.into();
        }
    }
    source.to_owned().into()
}

#[derive(gpui::IntoElement)]
pub(crate) struct UiText {
    source: SharedString,
    kind: TextKind,
}
pub(crate) fn ui(source: impl Into<SharedString>) -> UiText {
    UiText { source: source.into(), kind: TextKind::Ui }
}
pub(crate) fn data(source: impl Into<SharedString>) -> UiText {
    UiText { source: source.into(), kind: TextKind::Data }
}
impl RenderOnce for UiText {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let source = if self.kind == TextKind::Ui {
            resolve(&self.source, cx)
        } else {
            number(&self.source, cx)
        };
        let options = localized::options(cx, self.kind);
        let mut leaf = xgpui_parley::typeset::typeset(source.clone()).direction(options.direction);
        if let Some(language) = options.language {
            leaf = leaf.language(language);
        }
        let rtl = options.direction == gpui::TextDirection::RightToLeft;
        div().min_w_0().when(rtl, |text| text.text_right()).child(leaf)
    }
}
pub(crate) fn label<M>(source: impl Into<SharedString>) -> impl Fn(&M, &mut App) -> AnyElement + Send + Sync + 'static {
    let source = source.into();
    move |_, cx| {
        div()
            .id("lab-control-label")
            .role(gpui::accesskit::Role::Label)
            .aria_label(resolve(&source, cx))
            .child(ui(source.clone()))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formatted_copy_keeps_argument_boundaries_and_rejects_ambiguous_fields() {
        assert_eq!(capture("Frames: 12; GPU: 4.25 ms", &["Frames: ", "; GPU: ", " ms"]), Some(vec!["12", "4.25"]));
        assert!(capture("other 12", &["Frames: ", ""]).is_none());
        assert!(capture("a b", &["", "", ""]).is_none());
    }
}

#[derive(gpui::IntoElement)]
pub(crate) struct LogicalRow(gpui::Div);
pub(crate) fn row(row: gpui::Div) -> LogicalRow {
    LogicalRow(row)
}
impl RenderOnce for LogicalRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        use xgpui_luma::infra::locale::{LocaleContext, LogicalStyled};
        self.0.flex_row_logical(LocaleContext::current(cx).direction)
    }
}
