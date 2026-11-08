//! Local markup syntax under review, expanding into SDK contracts and the Shadcn builder.
//!
//! `scroll_view!` and the outer stack are declarations consumed by `listbox!`,
//! not new global macros. Template expressions are passed intact to Rust. No
//! template token inspection, generated event handlers, or parallel layout engine.
//!
//! The grammar arms intentionally require declarations in the shown order.
//! Inline templates use inferred parameter types. Named templates follow the SDK
//! convention of returning `Div`/`AnyElement`; an opaque Rust 2024 return can use
//! `impl IntoElement + use<>` to avoid capturing the render model's lifetimes.

macro_rules! listbox {
    (
        $window:expr, $cx:expr;
        id = $id:expr;
        control = $control:expr;
        look = $look:expr;
        $(aria_label = $label:expr;)?
        $(width = $width:expr;)?
        padding_x = $padding_x:expr;
        padding_y = $padding_y:expr;
        scroll_view! { vertical;
            viewport_height = $viewport:expr;
            vstack! {
                gap = $gap:expr;
                item_height = content;
                item_template = $template:expr;
            }
        }
    ) => {{
        let surface = ($look).render_listbox(
            $control, $id,
            xgpui_luma::controls::listbox::ListBoxFlow::VerticalContent { viewport_height: $viewport, gap: $gap },
            ($padding_x, $padding_y), $template, $window, $cx,
        );
        $(let surface = gpui::StatefulInteractiveElement::aria_label(surface, $label);)?
        $(let surface = gpui::Styled::flex_shrink_0(gpui::Styled::w(surface, gpui::px($width)));)?
        surface
    }};
    (
        $window:expr, $cx:expr;
        id = $id:expr;
        control = $control:expr;
        look = $look:expr;
        $(aria_label = $label:expr;)?
        $(width = $width:expr;)?
        padding_x = $padding_x:expr;
        padding_y = $padding_y:expr;
        scroll_view! { vertical;
            visible_items = $count:expr;
            vstack! {
                gap = $gap:expr;
                item_height = $height:expr;
                item_template = $template:expr;
            }
        }
    ) => {{
        let surface = ($look).render_listbox(
            $control, $id,
            xgpui_luma::controls::listbox::ListBoxFlow::Vertical {
                visible_items: $count, item_height: $height, gap: $gap,
            },
            ($padding_x, $padding_y), $template, $window, $cx,
        );
        $(let surface = gpui::StatefulInteractiveElement::aria_label(surface, $label);)?
        $(let surface = gpui::Styled::flex_shrink_0(gpui::Styled::w(surface, gpui::px($width)));)?
        surface
    }};
    (
        $window:expr, $cx:expr;
        id = $id:expr;
        control = $control:expr;
        look = $look:expr;
        $(aria_label = $label:expr;)?
        $(width = $width:expr;)?
        padding_x = $padding_x:expr;
        padding_y = $padding_y:expr;
        scroll_view! { horizontal;
            hstack! {
                gap = $gap:expr;
                item_width = $width_per_item:expr;
                item_height = $height:expr;
                item_template = $template:expr;
            }
        }
    ) => {{
        let surface = ($look).render_listbox(
            $control, $id,
            xgpui_luma::controls::listbox::ListBoxFlow::Horizontal {
                item_width: $width_per_item, item_height: $height, gap: $gap,
            },
            ($padding_x, $padding_y), $template, $window, $cx,
        );
        $(let surface = gpui::StatefulInteractiveElement::aria_label(surface, $label);)?
        $(let surface = gpui::Styled::flex_shrink_0(gpui::Styled::w(surface, gpui::px($width)));)?
        surface
    }};
}

pub(super) use listbox;

#[cfg(all(test, feature = "test-support"))]
#[path = "markup_tests.rs"]
mod tests;
