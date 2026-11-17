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
        #[cfg(all(test, feature = "test-support"))]
        super::usage::message(&message, cx);
        return localizer.format(message).into();
    }
    if let Some((owner, title)) = source.split_once("  /  ") {
        return format!("{owner}  /  {}", resolve(title, cx)).into();
    }
    #[cfg(all(test, feature = "test-support"))]
    super::usage::unknown(source, cx);
    source.to_owned().into()
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
    source: super::UiCopy,
    kind: TextKind,
}
pub(crate) fn ui(source: impl Into<super::UiCopy>) -> UiText {
    UiText { source: source.into(), kind: TextKind::Ui }
}
pub(crate) fn data(source: impl Into<super::UiCopy>) -> UiText {
    UiText { source: source.into(), kind: TextKind::Data }
}
impl UiText {
    fn element(self, cx: &App) -> gpui::Div {
        let source = if self.kind == TextKind::Ui {
            self.source.resolve(cx)
        } else {
            self.source.data(cx)
        };
        let options = localized::options(cx, self.kind);
        let mut leaf = xgpui_parley::typeset::typeset(source.clone()).direction(options.direction);
        if let Some(language) = options.language {
            leaf = leaf.language(language);
        }
        // Parley maps Left/Right to logical start/end. Only native fallback
        // needs a physical right alignment for RTL UI.
        let rtl = !crate::parley_lab::paragraph_api::typeset_active(cx)
            && options.direction == gpui::TextDirection::RightToLeft;
        div().min_w_0().when(rtl, |text| text.text_right()).child(leaf)
    }
}
impl RenderOnce for UiText {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.element(cx)
    }
}

pub(crate) fn label<M>(
    source: impl Into<super::UiCopy>,
) -> impl Fn(&M, &mut App) -> AnyElement + Send + Sync + 'static {
    let source = source.into();
    move |_, cx| {
        div()
            .id("lab-control-label")
            .role(gpui::accesskit::Role::Label)
            .aria_label(source.resolve(cx))
            .child(ui(source.clone()))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "test-support")]
    #[test]
    fn parley_rtl_keeps_logical_start_and_native_fallback_uses_physical_right() {
        use gpui::{TestAppContext, TestDispatcher, TextAlign};
        for parley in [false, true] {
            let app = if parley {
                let backend = crate::parley_lab::application_text_system().unwrap();
                let app = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend.clone());
                app.update(|cx| crate::parley_lab::paragraph_api::install_typeset(backend, cx));
                app
            } else {
                TestAppContext::single()
            };
            app.update(|cx| {
                xgpui_luma_l10n::install(super::super::localizer("ar-XB".parse().unwrap()).unwrap(), cx).unwrap();
                let mut element = ui("Reset canvas").element(cx);
                let align = element.style().text.text_align;
                // No physical override allows Parley's natural/start alignment.
                assert_eq!(align, if parley { None } else { Some(TextAlign::Right) });
            });
        }
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
