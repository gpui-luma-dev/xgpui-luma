use std::fmt;

use crate::{Error, Locale};

/// Where the startup locale was selected. This is separate from message fallback:
/// an OS-selected locale without a catalog still uses English resource fallbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocaleSource {
    CommandLine,
    Setting,
    System,
    Default,
}

impl fmt::Display for LocaleSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CommandLine => "command-line",
            Self::Setting => "saved-setting",
            Self::System => "system",
            Self::Default => "default",
        })
    }
}

/// Resolved locale and its provenance for application settings and diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedLocale {
    pub locale: Locale,
    pub source: LocaleSource,
}

/// Optional startup preferences, in precedence order. `None` means no override;
/// an empty, malformed or undetermined (`und`) explicit value is an error rather
/// than an absent value.
///
/// This type does not parse command-line arguments or read app settings. Apps
/// supply those values, then resolve against the platform's preferred locales.
///
/// ```
/// use xgpui_luma_l10n::{LocaleRequest, LocaleSource, Localizer};
/// # fn main() -> Result<(), xgpui_luma_l10n::Error> {
/// let request = LocaleRequest { command_line: Some("fr-CA"), setting: Some("de") };
/// let selected = request.resolve()?;
/// assert_eq!(selected.source, LocaleSource::CommandLine);
/// let localizer = Localizer::new(selected.locale);
/// assert_eq!(localizer.locale().language_tag(), "fr-CA");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct LocaleRequest<'a> {
    pub command_line: Option<&'a str>,
    pub setting: Option<&'a str>,
}

impl LocaleRequest<'_> {
    /// CLI → saved setting → first usable OS preference → English.
    /// The OS is queried only when neither explicit override is present.
    pub fn resolve(self) -> Result<SelectedLocale, Error> {
        match self.explicit()? {
            Some(selected) => Ok(selected),
            None => Self::system_or_default(sys_locale::get_locales()),
        }
    }

    /// Deterministic resolution for tests or an application-provided locale list.
    /// Entries must be BCP 47 tags, as returned by `sys_locale::get_locales`.
    /// Invalid OS entries (including POSIX `C`/`POSIX`, which ICU rejects) and
    /// undetermined-language tags (`und`, `und-Latn`) are skipped in order.
    /// Lower-priority preferences are not validated once a locale is selected.
    pub fn resolve_with_system<S: AsRef<str>>(
        self,
        system_locales: impl IntoIterator<Item = S>,
    ) -> Result<SelectedLocale, Error> {
        match self.explicit()? {
            Some(selected) => Ok(selected),
            None => Self::system_or_default(system_locales),
        }
    }

    fn explicit(self) -> Result<Option<SelectedLocale>, Error> {
        let choice = self
            .command_line
            .map(|tag| (tag, LocaleSource::CommandLine))
            .or_else(|| self.setting.map(|tag| (tag, LocaleSource::Setting)));
        choice
            .map(|(tag, source)| {
                let locale =
                    ui_locale(tag).ok_or_else(|| Error::InvalidLocaleSource { origin: source, value: tag.into() })?;
                Ok(SelectedLocale { locale, source })
            })
            .transpose()
    }

    fn system_or_default<S: AsRef<str>>(system_locales: impl IntoIterator<Item = S>) -> Result<SelectedLocale, Error> {
        for tag in system_locales {
            if let Some(locale) = ui_locale(tag.as_ref()) {
                return Ok(SelectedLocale { locale, source: LocaleSource::System });
            }
        }
        Ok(SelectedLocale { locale: "en".parse()?, source: LocaleSource::Default })
    }
}

/// A tag usable as the UI language. `und` is a valid locale value (the CLDR
/// root), but names no language, so it never selects the UI locale.
fn ui_locale(tag: &str) -> Option<Locale> {
    tag.parse::<Locale>().ok().filter(|locale| !locale.0.id.language.is_unknown())
}
