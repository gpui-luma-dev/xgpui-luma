//! Radix overlay-window theme for swatch info panels.

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, SharedString, black, point, px};
use xgpui_luma::controls::overlay_window::{OverlayWindowLook, OverlayWindowMode, OverlayWindowTheme};
use xgpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::look::Look;
use crate::semantic::SemanticRole;

struct OverlayWindowThemeAdapter {
    look: Look,
}

impl OverlayWindowTheme for OverlayWindowThemeAdapter {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        let metrics = self.look.metrics();
        let fallback = xgpui_luma::controls::overlay_window::default_overlay_window_theme().resolve(size, mode);
        let mut resolved = OverlayWindowLook {
            viewport_padding: fallback.viewport_padding,
            window_margin: fallback.window_margin,
            drag_header_height: fallback.drag_header_height,

            border_width: fallback.border_width,
            background: self.look.resolve_role(SemanticRole::Surface).hsla(),
            border: self.look.resolve_role(SemanticRole::Border).hsla(),
            foreground: self.look.resolve_role(SemanticRole::Foreground).hsla(),
            backdrop_background: if mode == OverlayWindowMode::Modal {
                gpui::Hsla { a: 0.42, ..black() }
            } else {
                gpui::Hsla { a: 0.0, ..black() }
            },
            shadow: vec![BoxShadow {
                offset: point(px(0.0), px(18.0)),
                blur_radius: px(38.0),
                spread_radius: px(-16.0),
                color: gpui::Hsla { a: 0.22, ..black() },
                inset: false,
            }],
            radius: metrics.radius.xl,
            padding: metrics.padding_x(size),
            min_width: fallback.min_width,
            max_width: fallback.max_width,
            estimated_height: fallback.estimated_height,
            body: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
            font_family: SharedString::from("System UI"),
        };
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        let geometry = self.look.common_stylesheet().overlay_window.resolve_geometry(
            key,
            xgpui_luma::theme::stylesheet::OverlayWindowGeometry {
                padding: resolved.padding,
                min_width: resolved.min_width,
                max_width: resolved.max_width,
                estimated_height: resolved.estimated_height,
                font_size: resolved.body.size,
                line_height: resolved.body.line_height,
                backdrop_opacity: resolved.backdrop_background.a,
                border_width: resolved.border_width,
                viewport_padding: resolved.viewport_padding,
                window_margin: resolved.window_margin,
                drag_header_height: resolved.drag_header_height,

                radius: resolved.radius,
            },
        );
        resolved.viewport_padding = geometry.viewport_padding;
        resolved.window_margin = geometry.window_margin;
        resolved.drag_header_height = geometry.drag_header_height;
        resolved.border_width = geometry.border_width;
        resolved.radius = geometry.radius;
        resolved.padding = geometry.padding;
        resolved.min_width = geometry.min_width;
        resolved.max_width = geometry.max_width;
        resolved.estimated_height = geometry.estimated_height;
        if mode == OverlayWindowMode::Modal {
            resolved.backdrop_background.a = geometry.backdrop_opacity;
        }
        resolved.body.size = geometry.font_size;
        resolved.body.line_height = geometry.line_height;
        resolved
    }
}

pub fn overlay_window_theme(look: &Look) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(OverlayWindowThemeAdapter { look: look.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_chrome_uses_selected_common_stylesheet() {
        let look = Look::built_in();
        look.set_common_stylesheet(
            xgpui_luma::theme::stylesheet::CommonStylesheet::resolve_layers(&[
                "[common.overlay_window.geometry]\nborder_width=0\nradius=19",
            ])
            .unwrap(),
        );
        for mode in [OverlayWindowMode::Modal, OverlayWindowMode::Modeless] {
            let resolved = overlay_window_theme(&look).resolve(ControlSize::Md, mode);
            assert_eq!((resolved.border_width, resolved.radius), (0.0, 19.0));
        }
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_overlay_window_stylesheet() -> xgpui_luma::theme::stylesheet::OverlayWindowStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    OverlayWindowStylesheet {
        geometry: OverlayWindowGeometryStylesheet { backdrop_opacity: Some(0.42), ..Default::default() },
        sizes: [("sm", 280.0, 360.0, 180.0), ("md", 360.0, 440.0, 220.0), ("lg", 440.0, 520.0, 260.0)]
            .into_iter()
            .map(|(key, min, max, height)| {
                (
                    key.into(),
                    OverlayWindowGeometryStylesheet {
                        min_width: Some(min),
                        max_width: Some(max),
                        estimated_height: Some(height),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}
