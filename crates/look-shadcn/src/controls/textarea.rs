//! Text area — surface/filled/soft variants share the text field token resolver.

#[cfg(feature = "inspect")]
use xgpui_luma::theme::StandardBoxScale;

#[cfg(feature = "inspect")]
use crate::mode::ShadcnModeTokens;
#[cfg(feature = "inspect")]
use crate::size::ShadcnSize;

#[cfg(feature = "inspect")]
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
