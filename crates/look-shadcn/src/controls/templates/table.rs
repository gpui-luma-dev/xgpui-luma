use std::sync::Arc;

use xgpui_luma::controls::table::{StyledTable, TableTheme, table_drag_look, table_template_with_theme};
use xgpui_luma::theme::{InteractionState};

use crate::look::ShadcnLook;

/// The Shadcn table: frame, header and rows from `.lss` (`style/table.lss`); the drag preview is the look's
/// `.lss` drag preview.
struct ShadcnTableTheme {
    theme: ShadcnLook,
    styled: StyledTable,
}

impl TableTheme for ShadcnTableTheme {
    fn resolve_drag(&self) -> xgpui_luma::controls::table::TableDragLook {
        let geometry = self
            .theme
            .mode_tokens()
            .stylesheet()
            .common
            .table
            .with_sdk_defaults()
            .resolve_geometry_for(self.theme.size().key(), Default::default());
        table_drag_look(super::drag::drag_look(&self.theme), geometry)
    }

    fn resolve_look(&self, enabled: bool, focused: bool) -> xgpui_luma::controls::table::TableLook {
        self.styled.look(enabled, focused)
    }

    fn resolve_row(&self, selected: bool, state: InteractionState) -> xgpui_luma::controls::table::TableRowPalette {
        self.styled.row(selected, state)
    }

    fn resolve_row_look(
        &self,
        selected: bool,
        state: InteractionState,
        scale: &xgpui_luma::theme::ListRowScale,
    ) -> xgpui_luma::controls::table::TableRowLook {
        self.styled.row_look(selected, state, scale)
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics_for(self.theme.size())
    }
}

pub fn table_theme(theme: ShadcnLook) -> Arc<dyn TableTheme> {
    let size = theme.size();
    let style = theme.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "table",
        xgpui_luma::style::ThemeSource::Live(Arc::new(move || style.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(ShadcnTableTheme { theme, styled: StyledTable::new(binding) })
}

pub fn table_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::table::TableTemplate> {
    table_template_with_theme(table_theme(theme))
}

#[cfg(test)]
mod lss_tests {
    use super::*;

    #[test]
    fn lss_table_matches_the_native_table() {
        let look = crate::test_support::fallback_look();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for size in [crate::ShadcnSize::Sm, crate::ShadcnSize::Md, crate::ShadcnSize::Lg] {
                let theme = table_theme(look.with_size(size));
                let tokens = look.mode_tokens();
                for (enabled, focused) in [(true, false), (true, true), (false, false)] {
                    let got = theme.resolve_look(enabled, focused);
                    let want = crate::controls::table::table_look(tokens.as_ref(), enabled, focused, size);
                    crate::assert_close(got.background, want.background);
                    crate::assert_close(got.border, want.border);
                    crate::assert_close(got.header_background, want.header_background);
                    crate::assert_close(got.header_label_color, want.header_label_color);
                    assert_eq!((got.radius, got.padding_y), (want.radius, want.padding_y), "{mode:?} {size:?}");
                }
                for selected in [true, false] {
                    for bits in 0..16u8 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let got = theme.resolve_row(selected, state);
                        let want = crate::controls::table::table_row_palette(tokens.as_ref(), selected, state, size);
                        crate::assert_close(got.background, want.background);
                        crate::assert_close(got.label_color, want.label_color);
                        crate::assert_close(got.divider, want.divider);
                        assert_eq!(got.label_typography.size, want.label_typography.size);
                    }
                }
            }
        }
    }
}
