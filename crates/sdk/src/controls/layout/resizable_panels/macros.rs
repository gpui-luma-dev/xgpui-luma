//! Declarative layout DSL for [`ResizablePanels`](super::ResizablePanels).
//!
//! Terminate each panel arm with `;`. Use `|` between panels as a visual split marker
//! (pair with group-level `show_handle: true` or `handle_visibility: Hover`
//! for visible draggable handles).

#[macro_export]
macro_rules! resizable_panels {
    (
        $cx:expr,
        theme = $theme:expr,
        id: $id:expr,
        layout: $layout:ident,
        $( show_handle: $show_handle:expr, )?
        $( handle_visibility: $handle_visibility:ident, )?
        $( double_click_collapse: ($collapse_mode:ident, $collapse_direction:ident), )?
        $( handle_grip: $handle_grip:expr, )?
        $( show_border: $show_border:expr, )?
        $( width: $width_only:expr, )?
        $( height: $height_only:expr, )?
        $( size: ($frame_w:expr, $frame_h:expr), )?
        panels: [
            $($panel_items:tt)*
        ]
    ) => {{
        let panels = $crate::resizable_panels!(@collect_specs []; $($panel_items)*);
        let mut builder = $crate::controls::resizable_panels::ResizablePanels::new($id)
            .theme($theme);
        $crate::resizable_panels!(@finish_builder builder, $cx, $layout, panels
            $(, show_handle = $show_handle)*
            $(, handle_visibility = $handle_visibility)*
            $(, double_click_collapse = ($collapse_mode, $collapse_direction))*
            $(, handle_grip = $handle_grip)*
            $(, show_border = $show_border)*
            $(, width = $width_only)*
            $(, height = $height_only)*
            $(, frame_size = ($frame_w, $frame_h))*
        )
    }};

    (
        $cx:expr,
        id: $id:expr,
        layout: $layout:ident,
        $( show_handle: $show_handle:expr, )?
        $( handle_visibility: $handle_visibility:ident, )?
        $( double_click_collapse: ($collapse_mode:ident, $collapse_direction:ident), )?
        $( handle_grip: $handle_grip:expr, )?
        $( show_border: $show_border:expr, )?
        $( width: $width_only:expr, )?
        $( height: $height_only:expr, )?
        $( size: ($frame_w:expr, $frame_h:expr), )?
        panels: [
            $($panel_items:tt)*
        ]
    ) => {{
        let panels = $crate::resizable_panels!(@collect_specs []; $($panel_items)*);
        let mut builder = $crate::controls::resizable_panels::ResizablePanels::new($id);
        $crate::resizable_panels!(@finish_builder builder, $cx, $layout, panels
            $(, show_handle = $show_handle)*
            $(, handle_visibility = $handle_visibility)*
            $(, double_click_collapse = ($collapse_mode, $collapse_direction))*
            $(, handle_grip = $handle_grip)*
            $(, show_border = $show_border)*
            $(, width = $width_only)*
            $(, height = $height_only)*
            $(, frame_size = ($frame_w, $frame_h))*
        )
    }};

    (
        @finish_builder $builder:expr, $cx:expr, $layout:ident, $panels:expr
        $(, show_handle = $show_handle:expr)*
        $(, handle_visibility = $handle_visibility:ident)*
        $(, double_click_collapse = ($collapse_mode:ident, $collapse_direction:ident))*
        $(, handle_grip = $handle_grip:expr)*
        $(, show_border = $show_border:expr)*
        $(, width = $width_only:expr)*
        $(, height = $height_only:expr)*
        $(, frame_size = ($frame_w:expr, $frame_h:expr))*
    ) => {{
        let mut builder = $builder.orientation(
            $crate::controls::resizable_panels::ResizablePanelsOrientation::$layout,
        );
        $( builder = builder.show_handle($show_handle); )?
        $( builder = builder.handle_visibility(
            $crate::controls::resizable_panels::ResizeHandleVisibility::$handle_visibility,
        ); )?
        $( builder = builder.double_click_collapse(Some(
            $crate::controls::resizable_panels::ResizeCollapseBehavior::new(
                $crate::controls::resizable_panels::ResizeCollapseMode::$collapse_mode,
                $crate::controls::resizable_panels::ResizeCollapseDirection::$collapse_direction,
            ),
        )); )?
        $( builder = builder.handle_grip($handle_grip); )?
        $( builder = builder.show_border($show_border); )?
        $( builder = builder.width($width_only); )?
        $( builder = builder.height($height_only); )?
        $( builder = builder.size($frame_w, $frame_h); )?
        builder.panels($panels).spawn($cx)
    }};

    (@collect_specs [$($specs:expr),* $(,)?]) => {
        vec![$($specs),*]
    };

    (@collect_specs [$($specs:expr),* $(,)?];) => {
        vec![$($specs),*]
    };

    (@collect_specs [$($specs:expr),* $(,)?]; | $(;)? $($rest:tt)*) => {
        $crate::resizable_panels!(@collect_specs [$($specs),*]; $($rest)*)
    };

    (
        @collect_specs [$($specs:expr),* $(,)?];
        resizable_panels! {
            $($nested:tt)*
        }
        => weight($weight:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
        $($rest:tt)*
    ) => {{
        let nested = $crate::resizable_panels! { $($nested)* };
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render(move || {
            nested.clone()
        })
        .weight($weight);
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec]; $($rest)*)
    }};

    (
        @collect_specs [$($specs:expr),* $(,)?];
        resizable_panels! {
            $($nested:tt)*
        }
        => weight($weight:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
    ) => {{
        let nested = $crate::resizable_panels! { $($nested)* };
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render(move || {
            nested.clone()
        })
        .weight($weight);
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec])
    }};

    (
        @collect_specs [$($specs:expr),* $(,)?];
        $view:expr => px($size:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
        $($rest:tt)*
    ) => {{
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render($view)
            .size(::gpui::px($size));
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec]; $($rest)*)
    }};

    (
        @collect_specs [$($specs:expr),* $(,)?];
        $view:expr => px($size:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
    ) => {{
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render($view)
            .size(::gpui::px($size));
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec])
    }};

    (
        @collect_specs [$($specs:expr),* $(,)?];
        $view:expr => weight($weight:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
        $($rest:tt)*
    ) => {{
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render($view)
            .weight($weight);
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec]; $($rest)*)
    }};

    (
        @collect_specs [$($specs:expr),* $(,)?];
        $view:expr => weight($weight:expr)
        $(, min: $min:expr)?
        $(, max: $max:expr)?
        $(, bg: $bg:expr)?
        ;
    ) => {{
        let mut spec = $crate::controls::resizable_panels::ResizablePanelSpec::new_render($view)
            .weight($weight);
        $( spec = spec.min($min); )?
        $( spec = spec.max($max); )?
        $( spec = spec.bg($bg); )?
        $crate::resizable_panels!(@collect_specs [$($specs,)* spec])
    }};
}
