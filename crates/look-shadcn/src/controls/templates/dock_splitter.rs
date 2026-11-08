use std::sync::Arc;

use xgpui_luma::controls::dock_splitter::DockSplitterTheme;

use crate::look::ShadcnLook;

pub fn dock_splitter_theme(theme: ShadcnLook) -> Arc<dyn DockSplitterTheme> {
    Arc::new(ShadcnDockSplitterTheme { theme: theme.clone() })
}

struct ShadcnDockSplitterTheme {
    theme: ShadcnLook,
}

impl DockSplitterTheme for ShadcnDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> xgpui_luma::controls::dock_splitter::DockSplitterLook {
        let tokens = self.theme.mode_tokens();
        let border = tokens.palette.border_default;
        let disabled = tokens.palette.disabled_foreground;

        let geometry = tokens.stylesheet().common.dock_splitter.resolve_geometry("md", Default::default());
        xgpui_luma::controls::dock_splitter::DockSplitterLook {
            line_color: if enabled { border } else { disabled },
            hover_color: if enabled { border } else { disabled },
            thumb_color: if enabled {
                tokens.palette.primary.background
            } else {
                disabled
            },
            hit_target_px: geometry.hit_target,
            visible_line_px: geometry.visible_line,
            geometry,
        }
    }
}
