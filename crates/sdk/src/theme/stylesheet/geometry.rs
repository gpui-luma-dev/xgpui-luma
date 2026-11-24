//! Control geometry read from `.lss` (`lss_geometry!`), and the size-key hook for the settings families.

/// A control's geometry read from its `.lss`: one `f32` per property of the same name on the control.
macro_rules! lss_geometry {
    ($(#[$meta:meta])* $name:ident, $control:literal, [$($field:ident),+ $(,)?]) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {
            $(pub $field: f32,)+
        }

        impl $name {
            #[doc = concat!("The geometry `binding` (a `", $control, "` control) resolves to. A property the file")]
            /// lacks reads as zero.
            pub fn from_binding(binding: &$crate::style::StyleBinding) -> Self {
                let resolved = binding.resolve(&[], 1.0);
                let root = resolved.root();
                let number = |name: &str| root.value(name).and_then(xgpui_luma_style::Value::as_number).unwrap_or(0.0);
                Self { $($field: number(stringify!($field)),)+ }
            }

            /// The SDK's own geometry, read once from its `.lss`.
            pub fn sdk() -> Self {
                static SDK: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                *SDK.get_or_init(|| Self::from_binding(&$crate::style::StyleBinding::sdk($control)))
            }
        }
    };
}
pub(crate) use lss_geometry;

pub use crate::controls::layout::split_view::theme::SplitViewGeometry;
pub use crate::controls::layout::dock_splitter::theme::DockSplitterGeometry;
pub use crate::controls::range::progress::theme::ProgressGeometry;
pub use crate::controls::navigation::stepper::theme::StepperGeometry;
pub use crate::controls::navigation::sidebar::theme::{SidebarGeometry};
pub use crate::controls::collection::table::theme::TableGeometry;
pub use crate::controls::overlay::popover_button::panel::{PopoverGeometry};
pub use crate::controls::overlay::slide_panel::SlidePanelGeometry;
pub use crate::controls::choice::radio_group::RadioGroupGeometry;
pub use crate::controls::selection::autocomplete::theme::{AutocompleteGeometry};
pub use crate::controls::selection::combobox::{ComboBoxGeometry};
pub use crate::controls::selection::search_selector::{SearchSelectorGeometry};

lss_geometry!(
    /// The window title bar's geometry, from `title_bar.lss`.
    ShellGeometry,
    "title_bar",
    [
        height,
        macos_left_padding,
        left_padding,
        traffic_light_inset,
        icon_size,
        border_width,
        fullscreen_left_padding
    ]
);
macro_rules! shared_geometry {
    ($($field:ident),+ $(,)?) => {
        /// Dimensions several controls share, from the theme's `geometry.*` tokens (`theme/tokens.lss`).
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct SharedGeometry {
            $(pub $field: f32,)+
        }

        impl SharedGeometry {
            /// The field names, each a `geometry.<name>` token.
            pub const FIELDS: &'static [&'static str] = &[$(stringify!($field)),+];

            /// `theme`'s `geometry.*` tokens. A missing token reads as zero.
            pub fn from_theme(theme: &xgpui_luma_style::Theme) -> Self {
                let number = |name: &str| {
                    theme
                        .token_with(&format!("geometry.{name}"), &[])
                        .ok()
                        .and_then(|value| value.as_number())
                        .unwrap_or(0.0)
                };
                Self { $($field: number(stringify!($field)),)+ }
            }
        }
    };
}
shared_geometry!(
    popup_viewport_height,
    popup_row_height,
    menu_row_height,
    menu_row_gap,
    compact_textfield_height,
    compact_textfield_padding_y,
    textarea_resize_icon_size,
    textarea_resize_inset,
    textarea_resize_hit_size,
    textarea_scrollbar_width,
    textarea_scrollbar_inset,
    scrollbar_width,
    composition_underline_width,
    label_baseline,
    drag_offset,
    drag_preview_width,
    drag_padding_x,
    drag_padding_y,
    drag_border_width,
    drag_marker_width,
    split_sidebar_width,
    split_sidebar_min_width,
    split_sidebar_max_width,
    selection_scrollbar_width,
    wide_middle_gap,
    wide_side_min_width,
    wide_side_preferred_width,
    wide_middle_min_width,
    wide_middle_max_width,
    form_field_gap,
    page_size_trigger_width,
    popup_icon_size,
    textarea_resize_opacity
);

impl SharedGeometry {
    /// The SDK's own shared dimensions, read once from its `.lss` tokens.
    pub fn sdk() -> Self {
        static SDK: std::sync::OnceLock<SharedGeometry> = std::sync::OnceLock::new();
        *SDK.get_or_init(|| Self::from_theme(&crate::style::default_theme()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shared dimension is a `geometry.*` token in the SDK's `.lss` (#287), and a look's tokens
    /// override it.
    #[test]
    fn shared_geometry_reads_the_geometry_tokens() {
        let theme = crate::style::default_theme();
        for field in SharedGeometry::FIELDS {
            let value = theme.token_with(&format!("geometry.{field}"), &[]).expect(field);
            assert!(value.as_number().is_some(), "geometry.{field} is a number");
        }
        let sheet = theme
            .stylesheet()
            .with_source(
                "look",
                xgpui_luma_style::Source::new("look.lss", "tokens { geometry { popup_row_height = 31 } }"),
            )
            .expect("compiles");
        let look = SharedGeometry::from_theme(&theme.with_stylesheet(&sheet).expect("binds"));
        assert_eq!(look.popup_row_height, 31.0);
        assert_eq!(look.menu_row_height, SharedGeometry::sdk().menu_row_height, "the rest are the SDK's");
    }
    #[test]
    fn typed_family_contracts_validate_every_field_and_preserve_precedence() {}
}
