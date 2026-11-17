use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use fluent_bundle::{concurrent::FluentBundle, FluentArgs, FluentResource};

use crate::Locale;

/// Invalid locale or resource input. Invalid resources never partially enter a bundle.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid locale: {0}")]
    InvalidLocale(String),
    #[error("invalid {origin} locale: {value}")]
    InvalidLocaleSource { origin: crate::LocaleSource, value: String },
    #[error("invalid Fluent resource for {locale}: {details}")]
    InvalidResource { locale: String, details: String },
}

/// Active locale and application-owned Fluent bundles.
///
/// Later resources override earlier definitions within a locale. Message lookup
/// falls back independently; malformed translations also fall back rather than
/// displaying Fluent's partial error output. Fluent argument isolation stays on.
pub struct Localizer {
    locale: Locale,
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    reported: Mutex<HashSet<String>>,
}

impl Localizer {
    pub fn new(locale: Locale) -> Self {
        Self { locale, bundles: HashMap::new(), reported: Mutex::new(HashSet::new()) }
    }

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Switch the engine locale. GPUI apps should use the crate's `set_locale`
    /// helper to also refresh windows.
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// Load a resource (typically `include_str!`) into its locale's bundle.
    pub fn add_resource(&mut self, locale: &Locale, source: &str) -> Result<(), Error> {
        let resource = FluentResource::try_new(source.to_owned()).map_err(|(_, errors)| Error::InvalidResource {
            locale: locale.to_string(),
            details: format!("{errors:?}"),
        })?;
        let language = locale.0.id.to_string().parse().map_err(|_| Error::InvalidLocale(locale.to_string()))?;
        let bundle = self
            .bundles
            .entry(locale.to_string())
            .or_insert_with(|| FluentBundle::new_concurrent(vec![language]));
        bundle.add_resource_overriding(resource);
        Ok(())
    }

    pub fn tr(&self, id: &str) -> String {
        self.translate(id, None)
    }

    pub fn tr_args(&self, id: &str, args: &FluentArgs<'_>) -> String {
        self.translate(id, Some(args))
    }

    fn translate(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        for tag in self.locale.fallback_chain() {
            let Some(bundle) = self.bundles.get(&tag) else { continue };
            let Some(message) = bundle.get_message(id) else {
                continue;
            };
            let Some(pattern) = message.value() else { continue };
            let mut errors = Vec::new();
            let value = bundle.format_pattern(pattern, args, &mut errors);
            if errors.is_empty() {
                return value.into_owned();
            }
            self.report_once(format!("{tag}:{id}"), &format!("{errors:?}"));
        }
        self.report_once(format!("{}:{id}", self.locale), "message unavailable in fallback chain");
        if cfg!(debug_assertions) {
            format!("⟦{id}⟧")
        } else {
            id.to_owned()
        }
    }

    fn report_once(&self, key: String, reason: &str) {
        if let Ok(mut reported) = self.reported.lock() {
            if reported.insert(key.clone()) {
                log::warn!("localization {key}: {reason}");
            }
        }
    }
}
