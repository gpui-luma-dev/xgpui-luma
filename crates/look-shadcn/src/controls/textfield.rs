//! Text field property mappings for shadcn-inspired inputs:
//!
//! **Outline** (shadcn Input): `border-border`, light transparent fill, dark `input/30`,
//! `selection:bg-primary`, `selection:text-primary-foreground`, `placeholder:text-muted-foreground`.
//!
//! **Input** (selector / combobox triggers): same fill as Outline (`transparent` light,
//! `input/30` dark; hover `input/50` in selector palette). Opaque controls use `border-border`.
//! Used by selector, combobox, search_selector, and autocomplete — not standalone text fields.
//!
//! **Primary**: bordered control with opaque `background` fill and `shadow-xs` elevation.
//!
//! **Surface**: filled `muted` background, no border, same text/selection tokens.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Outline,
    Input,
    Primary,
    Surface,
}
