use std::{fmt, str::FromStr};

/// A canonical Unicode locale, including extensions such as `-u-nu-latn`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locale(pub(crate) icu_locale::Locale);

impl Locale {
    /// Canonical BCP 47 tag for paragraph shaping and formatting.
    pub fn language_tag(&self) -> String {
        self.to_string()
    }

    /// Resource lookup order: full locale, language/script/region parents, English.
    /// This is structural fallback, not CLDR parent-locale negotiation.
    pub fn fallback_chain(&self) -> Vec<String> {
        let mut tags = vec![self.to_string()];
        let mut id = self.0.id.clone();
        let mut push = |tag: String| {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        };
        push(id.to_string());
        id.variants.clear();
        push(id.to_string());
        id.region = None;
        push(id.to_string());
        id.script = None;
        push(id.to_string());
        push("en".into());
        tags
    }
}

impl FromStr for Locale {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self).map_err(|_| crate::Error::InvalidLocale(value.into()))
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
