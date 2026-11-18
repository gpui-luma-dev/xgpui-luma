//! Shared label typography for radix control themes.

use gpui::{FontWeight, SharedString};
use xgpui_luma::theme::{LumaTextStyle};

use crate::look::Look;

pub(crate) fn label_typography(size: crate::RadixSize) -> LumaTextStyle {
    match size {
        crate::RadixSize::One => LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
        crate::RadixSize::Two => LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        crate::RadixSize::Three => LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::NORMAL },
    }
}

pub(crate) fn font_family(look: &Look) -> SharedString {
    look.stylesheet_snapshot().tokens.typography.font.sans.family.clone().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_font_follows_the_look_stylesheet() {
        let look = Look::built_in();
        let mut stylesheet = look.common_stylesheet();
        stylesheet.tokens.typography.font.sans.family = "Embedded Sans".into();
        look.set_common_stylesheet(stylesheet);
        assert_eq!(font_family(&look).as_ref(), "Embedded Sans");
    }
}
