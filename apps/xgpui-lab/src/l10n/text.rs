//! Retained typed UI messages resolve against the current locale at render time.
use std::sync::{Arc, Mutex, OnceLock};
use gpui::{App, SharedString};
use xgpui_luma_l10n::{Localizer, Message};

#[derive(Clone, gpui::IntoElement)]
pub(crate) enum UiCopy {
    Source(SharedString),
    Raw(SharedString),
    Numeric(String),
    Percent(String, u8),
    Bytes(u64),
    List(Vec<UiCopy>),
    Deferred(Arc<dyn Fn(&App) -> Message<'static> + Send + Sync>),
    Message(Arc<Mutex<Message<'static>>>),
    Joined(Vec<UiCopy>),
    Separated(&'static str, Vec<UiCopy>),
    Formatted(fn(&[String]) -> Message<'static>, Vec<String>),
    Compose(fn(&[String]) -> Message<'static>, Vec<UiCopy>),
}
impl UiCopy {
    pub(crate) fn deferred(factory: impl Fn(&App) -> Message<'static> + Send + Sync + 'static) -> Self {
        Self::Deferred(Arc::new(factory))
    }
    pub(crate) fn percentage(value: String, digits: u8) -> Self {
        Self::Percent(value, digits)
    }
    pub(crate) fn bytes(value: u64) -> Self {
        Self::Bytes(value)
    }
    pub(crate) fn numeric(value: String) -> Self {
        Self::Numeric(value)
    }
    pub(crate) fn counted(unit: &'static str, value: String) -> Self {
        Self::deferred(move |cx| {
            let count = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .map(xgpui_luma_l10n::FluentValue::from)
                .unwrap_or_else(|| xgpui_luma_l10n::FluentValue::from("other"));
            super::messages::msg_lab_count(count, super::number(&value, cx).to_string(), unit.to_owned())
        })
    }
    pub(crate) fn raw(value: impl Into<SharedString>) -> Self {
        Self::Raw(value.into())
    }
    pub(crate) fn state(key: impl Into<String>) -> Self {
        let key = key.into();
        Self::deferred(move |_| super::messages::msg_lab_state(key.clone()))
    }
    /// The call site chooses a typed constructor; arguments remain separate.
    /// Numeric display strings are localized after the active locale is known.
    pub(crate) fn compose(constructor: fn(&[String]) -> Message<'static>, arguments: Vec<UiCopy>) -> Self {
        Self::Compose(constructor, arguments)
    }
    pub(crate) fn formatted(constructor: fn(&[String]) -> Message<'static>, arguments: Vec<String>) -> Self {
        Self::Formatted(constructor, arguments)
    }
    pub(crate) fn resolve(&self, cx: &App) -> SharedString {
        match self {
            Self::Source(source) => super::resolve(source, cx),
            Self::Raw(source) => source.clone(),
            Self::Numeric(value) => super::number(value, cx),
            Self::Percent(value, digits) => cx
                .try_global::<Localizer>()
                .and_then(|localizer| {
                    value
                        .parse::<f64>()
                        .ok()
                        .and_then(|value| localizer.locale().fmt_percent(value / 100., *digits).ok())
                })
                .unwrap_or_else(|| format!("{value}%"))
                .into(),
            Self::Bytes(value) => cx
                .try_global::<Localizer>()
                .and_then(|localizer| {
                    localizer.locale().fmt_bytes(*value, xgpui_luma_l10n::formatting::ByteUnits::Binary, 1).ok()
                })
                .unwrap_or_else(|| format!("{value} B"))
                .into(),
            Self::List(values) => {
                let values: Vec<_> = values.iter().map(|value| value.resolve(cx).to_string()).collect();
                cx.try_global::<Localizer>()
                    .and_then(|localizer| {
                        localizer.locale().fmt_list(&values, xgpui_luma_l10n::formatting::ListKind::And).ok()
                    })
                    .unwrap_or_else(|| values.join("; "))
                    .into()
            }
            Self::Deferred(factory) => Self::from(factory(cx)).resolve(cx),
            Self::Message(message) => {
                #[cfg(all(test, feature = "test-support"))]
                if let Ok(message) = message.lock() {
                    super::usage::message(&message, cx);
                }
                static ENGLISH: OnceLock<Option<Localizer>> = OnceLock::new();
                let fallback = || {
                    ENGLISH.get_or_init(|| "en".parse().ok().and_then(|locale| super::localizer(locale).ok())).as_ref()
                };
                match (message.lock(), cx.try_global::<Localizer>().or_else(fallback)) {
                    (Ok(message), Some(localizer)) => localizer.format_ref(&message).into(),
                    _ => "⟦unavailable-message⟧".into(),
                }
            }
            Self::Compose(constructor, arguments) => {
                let arguments: Vec<_> = arguments.iter().map(|value| value.resolve(cx).to_string()).collect();
                Self::from(constructor(&arguments)).resolve(cx)
            }
            Self::Formatted(constructor, arguments) => {
                let arguments: Vec<_> = arguments.iter().map(|value| super::number(value, cx).to_string()).collect();
                let message = constructor(&arguments);
                Self::from(message).resolve(cx)
            }
            Self::Separated(separator, parts) => {
                parts.iter().map(|part| part.resolve(cx).to_string()).collect::<Vec<_>>().join(separator).into()
            }
            Self::Joined(parts) => {
                parts.iter().map(|part| part.resolve(cx).to_string()).collect::<Vec<_>>().join(" ").into()
            }
        }
    }
    pub(crate) fn data(&self, cx: &App) -> SharedString {
        match self {
            Self::Source(source) => super::number(source, cx),
            Self::Message(_)
            | Self::Joined(_)
            | Self::Formatted(..)
            | Self::Separated(..)
            | Self::Deferred(_)
            | Self::Raw(_)
            | Self::Compose(..)
            | Self::Numeric(_)
            | Self::List(_)
            | Self::Percent(..)
            | Self::Bytes(_) => self.resolve(cx),
        }
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn test_display(&self, cx: &App) -> String {
        self.resolve(cx).replace(['\u{2068}', '\u{2069}'], "")
    }
    pub(crate) fn is_empty(&self) -> bool {
        matches!(self, Self::Source(source) | Self::Raw(source) if source.is_empty())
    }
}
impl From<Message<'static>> for UiCopy {
    fn from(message: Message<'static>) -> Self {
        Self::Message(Arc::new(Mutex::new(message)))
    }
}
impl From<SharedString> for UiCopy {
    fn from(source: SharedString) -> Self {
        Self::Source(source)
    }
}
impl From<&SharedString> for UiCopy {
    fn from(source: &SharedString) -> Self {
        Self::Source(source.clone())
    }
}
impl From<String> for UiCopy {
    fn from(source: String) -> Self {
        Self::Source(source.into())
    }
}
impl From<&str> for UiCopy {
    fn from(source: &str) -> Self {
        Self::Source(source.to_owned().into())
    }
}

impl gpui::RenderOnce for UiCopy {
    fn render(self, _: &mut gpui::Window, _: &mut App) -> impl gpui::IntoElement {
        super::ui(self)
    }
}
