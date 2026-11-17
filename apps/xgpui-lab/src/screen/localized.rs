//! Lab-first text prototype for #220; SDK and look templates are not migrated.
use gpui::{App, Div, ParentElement, SharedString, Styled, TextAlign, TextDirection, div};
use xgpui_luma_l10n::{LayoutDirection, Localizer};
use xgpui_luma_look_radix::Look;
use xgpui_parley::{TextLanguage, typeset::typeset};
use super::TextRole;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextKind {
    Ui,
    Data,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextOptions {
    pub direction: TextDirection,
    pub language: Option<TextLanguage>,
}

pub(crate) fn options(cx: &App, kind: TextKind) -> TextOptions {
    let locale = cx.try_global::<Localizer>().map(Localizer::locale);
    let direction = if kind == TextKind::Data {
        TextDirection::Auto
    } else {
        match locale.map(|locale| locale.direction()) {
            Some(LayoutDirection::Rtl) => TextDirection::RightToLeft,
            _ => TextDirection::LeftToRight,
        }
    };
    TextOptions { direction, language: locale.and_then(|locale| TextLanguage::parse(&locale.language_tag()).ok()) }
}

/// Inherits role typography, wrapping, truncation and line clamping from its
/// container. Parley maps left/right text alignment to logical start/end.
/// Without TypesetEngine, the same leaf falls back to GPUI StyledText; UI start
/// alignment follows the locale, but native bidi/language support is limited.
pub(crate) fn text(look: &Look, role: TextRole, content: impl Into<SharedString>, kind: TextKind, cx: &App) -> Div {
    let settings = options(cx, kind);
    let mut leaf = typeset(content).direction(settings.direction);
    if let Some(language) = settings.language {
        leaf = leaf.language(language);
    }
    let native_rtl =
        !crate::parley_lab::paragraph_api::typeset_active(cx) && settings.direction == TextDirection::RightToLeft;
    role.apply(look, div().min_w_0())
        .text_align(if native_rtl { TextAlign::Right } else { TextAlign::Left })
        .child(leaf)
}
