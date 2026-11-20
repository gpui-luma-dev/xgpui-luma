//! Popup menu property mappings:
//!
//! | Part    | Variant                          |
//! |---------|----------------------------------|
//! | Trigger | outline or ghost command button  |
//! | Menu    | floating menu surface            |

#[cfg(feature = "inspect")]
use crate::mode::ShadcnModeTokens;

#[cfg(feature = "inspect")]
pub(crate) fn popup_menu_geometry(
    mode: &ShadcnModeTokens,
    size: crate::ShadcnSize,
    scale: &xgpui_luma::theme::StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedPopupMenuGeometry {
    mode.stylesheet().common.popup_menu.inspect_geometry_for(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::PopupMenuGeometry {
            height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            gap: scale.gap,
            icon_size: scale.icon_size,
            font_size: typography.size,
            line_height: typography.line_height,
            radius: scale.radius,
            ..Default::default()
        },
    )
}
