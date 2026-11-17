use gpui::{App, SharedString};
use crate::{FluentArgs, LayoutDirection, Locale, Localizer, Message};

impl gpui::Global for Localizer {}

/// Update the installed localizer and refresh every window. Returns false if no
/// localizer is installed, avoiding a panic during application initialization.
pub fn set_locale(locale: Locale, cx: &mut App) -> bool {
    if cx.try_global::<Localizer>().is_none() {
        return false;
    }
    cx.global_mut::<Localizer>().set_locale(locale);
    cx.refresh_windows();
    true
}

/// Translation access on App (and Context through its App dereference).
/// Without an installed global, IDs are returned and direction defaults to LTR.
pub trait L10nExt {
    fn tr(&self, id: &str) -> SharedString;
    fn tr_args(&self, id: &str, args: &FluentArgs<'_>) -> SharedString;
    /// Translate a catalog-generated message with checked argument names.
    fn tr_message(&self, message: Message<'_>) -> SharedString;
    fn locale(&self) -> Option<&Locale>;
    fn layout_direction(&self) -> LayoutDirection;
}

impl L10nExt for App {
    fn tr(&self, id: &str) -> SharedString {
        self.try_global::<Localizer>().map_or_else(|| id.to_owned(), |localizer| localizer.tr(id)).into()
    }

    fn tr_args(&self, id: &str, args: &FluentArgs<'_>) -> SharedString {
        self.try_global::<Localizer>()
            .map_or_else(|| id.to_owned(), |localizer| localizer.tr_args(id, args))
            .into()
    }

    fn locale(&self) -> Option<&Locale> {
        self.try_global::<Localizer>().map(Localizer::locale)
    }

    fn tr_message(&self, message: Message<'_>) -> SharedString {
        match self.try_global::<Localizer>() {
            Some(localizer) => localizer.format(message).into(),
            None => message
                .attribute
                .map_or_else(|| message.id.to_owned(), |attr| format!("{}.{attr}", message.id))
                .into(),
        }
    }

    fn layout_direction(&self) -> LayoutDirection {
        self.locale().map_or(LayoutDirection::Ltr, Locale::direction)
    }
}
