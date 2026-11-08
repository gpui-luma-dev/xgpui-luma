use std::sync::Arc;

use xgpui_luma::controls::table::{TableTheme, table_template_with_theme};
use xgpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::table::{table_look, table_row_palette};
use crate::look::ShadcnLook;

struct ShadcnTableTheme {
    theme: ShadcnLook,
}

impl TableTheme for ShadcnTableTheme {
    fn resolve_drag(&self, size: ControlSize) -> xgpui_luma::controls::table::TableDragLook {
        let tokens = self.theme.mode_tokens();
        let palette = &tokens.palette;
        xgpui_luma::controls::table::TableDragLook {
            geometry: tokens
                .stylesheet()
                .common
                .table
                .with_sdk_defaults()
                .resolve_geometry(size.stylesheet_key(), Default::default()),
            background: self.theme.token_color("popover").unwrap_or(palette.panel_background),
            foreground: self.theme.token_color("popover-foreground").unwrap_or(palette.body_text),
            valid_marker: self.theme.token_color("ring").unwrap_or(palette.focus_ring),
            invalid_marker: self.theme.token_color("destructive").unwrap_or(palette.destructive_background),
            radius: tokens.metrics.radius(size),
        }
    }

    fn resolve_look(&self, enabled: bool, focused: bool, size: ControlSize) -> xgpui_luma::controls::table::TableLook {
        let tokens = self.theme.mode_tokens();
        table_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> xgpui_luma::controls::table::TableRowPalette {
        let tokens = self.theme.mode_tokens();
        table_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn resolve_row_look(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &xgpui_luma::theme::ListRowScale,
    ) -> xgpui_luma::controls::table::TableRowLook {
        let mut look = xgpui_luma::controls::table::TableRowLook::from_palette(
            &self.resolve_row(selected, state, size),
            scale,
            &self.theme.mode_tokens().stylesheet().common,
            size,
        );
        look.column_emphasis_weight = self.theme.mode_tokens().typography.text.title.weight;
        look
    }

    fn metrics(&self) -> xgpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn table_theme(theme: ShadcnLook) -> Arc<dyn TableTheme> {
    Arc::new(ShadcnTableTheme { theme: theme.clone() })
}

pub fn table_template(theme: ShadcnLook) -> Arc<dyn xgpui_luma::controls::table::TableTemplate> {
    table_template_with_theme(table_theme(theme))
}
