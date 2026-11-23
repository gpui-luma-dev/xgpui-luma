use gpui::FontWeight;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LumaTypography {
    pub font: FontTokens,
    pub text: TextTokens,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FontTokens {
    pub sans: FontFamilyToken,
    pub mono: FontFamilyToken,
    pub serif: FontFamilyToken,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FontFamilyToken {
    pub family: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LumaTextRole {
    H1,
    H2,
    H3,
    H4,
    P,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextRoleTokens {
    pub h1: LumaTextStyle,
    pub h2: LumaTextStyle,
    pub h3: LumaTextStyle,
    pub h4: LumaTextStyle,
    pub p: LumaTextStyle,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextTokens {
    pub body: LumaTextStyle,
    pub label: LumaTextStyle,
    pub caption: LumaTextStyle,
    pub title: LumaTextStyle,
    pub code: LumaTextStyle,
    pub role: TextRoleTokens,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LumaTextStyle {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub size: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub line_height: f32,
    #[serde(deserialize_with = "font_weight")]
    pub weight: FontWeight,
}

impl Default for LumaTypography {
    /// The SDK's font families and text styles, from `theme/tokens.lss`.
    ///
    /// The SDK has no text size scale; looks own their size vocabularies and resolve them to
    /// [`LumaTextStyle`] pixels.
    fn default() -> Self {
        super::from_lss::sdk(super::ThemeMode::Light).typography.clone()
    }
}

impl TextRoleTokens {
    pub fn style(&self, role: LumaTextRole) -> LumaTextStyle {
        match role {
            LumaTextRole::H1 => self.h1,
            LumaTextRole::H2 => self.h2,
            LumaTextRole::H3 => self.h3,
            LumaTextRole::H4 => self.h4,
            LumaTextRole::P => self.p,
        }
    }
}

impl TextTokens {
    pub fn role(&self, role: LumaTextRole) -> LumaTextStyle {
        self.role.style(role)
    }
}

fn font_weight<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<FontWeight, D::Error> {
    let weight = <f32 as Deserialize>::deserialize(deserializer)?;
    if !weight.is_finite() || !(1.0..=1000.0).contains(&weight) {
        return Err(serde::de::Error::custom("font weight must be finite and in 1..=1000"));
    }
    Ok(FontWeight(weight))
}
