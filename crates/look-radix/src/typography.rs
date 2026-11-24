//! Shared label typography for radix control themes.

use gpui::SharedString;

use crate::look::Look;

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
