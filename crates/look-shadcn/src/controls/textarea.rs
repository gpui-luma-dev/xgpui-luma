//! Text area — surface/filled/soft variants share the text field token resolver.

use xgpui_luma::theme::StandardBoxScale;

use crate::mode::ShadcnModeTokens;
use crate::size::ShadcnSize;

pub(crate) fn textarea_geometry(
    mode: &ShadcnModeTokens,
    size: ShadcnSize,
    scale: &StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedTextAreaGeometry {
    mode.stylesheet().common.textarea.inspect_geometry_for(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::TextAreaGeometry {
            min_height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            font_size: typography.size,
            line_height: typography.line_height,
            ..Default::default()
        },
    )
}
