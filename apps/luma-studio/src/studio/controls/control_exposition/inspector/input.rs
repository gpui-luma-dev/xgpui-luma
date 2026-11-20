use xgpui_luma_look_shadcn::ShadcnSize;
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::inspect::{FloatingMenuInspectPalette, ShadcnInspect, TextFieldInspectPalette};

use super::provenance::color_row;
use super::schema::InspectColorRow;

pub fn textfield_color_rows_prefixed(prefix: &'static str, palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    match prefix {
        "textfield" => textfield_input_color_rows(palette),
        "field" => textfield_field_color_rows(palette),
        _ => textfield_field_color_rows(palette),
    }
}

fn textfield_field_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    let rows = vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
        color_row("placeholder", &palette.placeholder),
        color_row("icon", &palette.icon),
        color_row("selection background", &palette.selection_background),
        color_row("selection foreground", &palette.selection_foreground),
        color_row("caret", &palette.caret),
    ];
    rows
}

fn textfield_input_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    let rows = vec![
        color_row("textfield background", &palette.background),
        color_row("textfield foreground", &palette.foreground),
        color_row("textfield border", &palette.border),
        color_row("textfield placeholder", &palette.placeholder),
        color_row("textfield icon", &palette.icon),
    ];
    rows
}

pub fn floating_menu_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(ShadcnSize::Md);
    floating_menu_palette_rows(&palette)
}

pub fn floating_menu_palette_rows(palette: &FloatingMenuInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("menu background", &palette.background),
        color_row("menu foreground", &palette.foreground),
        color_row("menu border", &palette.border),
        color_row("menu item hover background", &palette.item_hover_background),
        color_row("menu item hover foreground", &palette.item_hover_foreground),
        color_row("menu item disabled foreground", &palette.item_disabled_foreground),
    ]
}
