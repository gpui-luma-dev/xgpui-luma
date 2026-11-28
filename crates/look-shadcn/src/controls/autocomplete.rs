//! Autocomplete / combobox chrome — textfield + floating menu tokens.

use crate::mode::ShadcnModeTokens;

/// The Shadcn autocomplete's style at `size` (#433): `style/autocomplete.lss` for `mode`'s catalog.
pub fn autocomplete_style(mode: &ShadcnModeTokens, size: crate::ShadcnSize) -> xgpui_luma::style::StyleBinding {
    super::selector_items_panel::mode_binding(mode, "autocomplete", size)
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::size::ShadcnSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::autocomplete_style;
    use xgpui_luma::style::PartRead;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
        ]))
    }

    #[test]
    fn autocomplete_chrome_uses_primary_and_muted_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let style = autocomplete_style(&mode, ShadcnSize::Md).resolve(&[], 1.0);
        let root = style.root();
        crate::assert_close(root.color("status"), catalog.color("primary").expect("primary"));
        crate::assert_close(root.color("muted_text"), catalog.color("muted-foreground").expect("muted-foreground"));
    }
}
