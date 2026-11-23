//! Shared label typography for radix control themes.

use gpui::SharedString;

use crate::look::Look;

/// Radix label text for a size (tests compare control text against it).
#[cfg(test)]
pub(crate) fn label_typography(size: crate::RadixSize) -> xgpui_luma::theme::LumaTextStyle {
    use gpui::FontWeight;
    use xgpui_luma::theme::LumaTextStyle;
    match size {
        crate::RadixSize::One => LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
        crate::RadixSize::Two => LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        crate::RadixSize::Three => LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::NORMAL },
    }
}

pub(crate) fn font_family(look: &Look) -> SharedString {
    look.typography().font.sans.family.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_font_follows_the_look_fonts() {
        let look = Look::built_in();
        let mut fonts = look.typography().font;
        fonts.sans.family = "Embedded Sans".into();
        look.set_fonts(fonts);
        assert_eq!(font_family(&look).as_ref(), "Embedded Sans");
    }
}
