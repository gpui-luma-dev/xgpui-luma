use serde::Deserialize;

/// The shadcn look's stylesheet: the common settings (`[common]`). Typography, sizes and colors are the
/// look's `.lss`, so a sheet that still sets `typography`, `button`, `toggle` or `sidebar` is an error (#355).
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StylesheetConfig {
    #[serde(default)]
    pub common: xgpui_luma::theme::stylesheet::CommonStylesheet,
}

impl StylesheetConfig {
    /// Construct bundled control definitions directly in Rust.
    pub fn native_default() -> Self {
        Self { common: super::native_shadcn_common_stylesheet() }
    }

    /// Parses stylesheet TOML without imposing an input-size limit.
    /// Applications own file access and any limits required for untrusted input.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        let mut stylesheet: Self = toml::from_str(source).map_err(|err| anyhow::anyhow!("parse stylesheet: {err}"))?;
        stylesheet.common = xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[source])?;
        Ok(stylesheet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Typography, button, toggle and sidebar settings are the look's `.lss` now (#355): a sheet that sets
    /// them is an error naming the section, not silently ignored.
    #[test]
    fn removed_sections_are_rejected() {
        for section in ["typography", "button", "toggle", "sidebar", "navigation_sidebar"] {
            let error = StylesheetConfig::parse(&format!("[{section}.metrics.md]\nfont_size = 14")).expect_err(section);
            assert!(error.to_string().contains(section), "{section}: {error}");
        }
    }

    #[test]
    fn common_settings_still_parse() {
        let config = StylesheetConfig::parse("[common.row_limits.pager]\nmax = 9").expect("parses");
        assert_eq!(config.common.row_limits.resolve("pager").1, 9);
    }
}
