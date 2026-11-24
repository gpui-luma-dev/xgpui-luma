//! Layout and form helper macros shared across apps.

/// Spawns a vertical stack flexbox layout with custom gap and cross-axis alignment.
#[macro_export]
macro_rules! vstack {
    // 1. Gap, cross-axis alignment, and main-axis justification
    (
        gap=$gap:tt align=$align:ident justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@align_items panel, $align);
        panel = $crate::vstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Gap and cross-axis alignment
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and main-axis justification
    (
        gap=$gap:tt justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Gap only
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Cross-axis alignment only
    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        panel = $crate::vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Simple column wrap
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        $( panel = panel.child($child); )*
        panel
    }};

    // Internal resolution rules
    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };

    (@justify_content $panel:ident, start) => { $panel.justify_start() };
    (@justify_content $panel:ident, center) => { $panel.justify_center() };
    (@justify_content $panel:ident, end) => { $panel.justify_end() };
    (@justify_content $panel:ident, between) => { $panel.justify_between() };
    (@justify_content $panel:ident, around) => { $panel.justify_around() };
    (@justify_content $panel:ident, evenly) => { $panel.justify_evenly() };
}

/// Spawns a horizontal stack flexbox layout with custom gap, alignment, and justification.
#[macro_export]
macro_rules! hstack {
    // 1. Gap, cross-axis alignment, and main-axis justification
    (
        gap=$gap:tt align=$align:ident justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@align_items panel, $align);
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Gap and cross-axis alignment
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and main-axis justification
    (
        gap=$gap:tt justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Gap only
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Cross-axis alignment only
    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        panel = $crate::hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Main-axis justification only
    (
        justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 7. Justification and cross-axis alignment
    (
        justify=$justify:ident align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        panel = $crate::hstack!(@align_items panel, $align);
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        align=$align:ident justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {
        $crate::hstack! {
            justify=$justify align=$align;
            $( $child ),*
        }
    };

    // 8. Justification, cross-axis alignment, and gap
    (
        justify=$justify:ident align=$align:ident gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@align_items panel, $align);
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        align=$align:ident justify=$justify:ident gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {
        $crate::hstack! {
            justify=$justify align=$align gap=$gap;
            $( $child ),*
        }
    };

    // 9. Simple row wrap
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        $( panel = panel.child($child); )*
        panel
    }};

    // Internal resolution rules
    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };

    (@justify_content $panel:ident, start) => { $panel.justify_start() };
    (@justify_content $panel:ident, center) => { $panel.justify_center() };
    (@justify_content $panel:ident, end) => { $panel.justify_end() };
    (@justify_content $panel:ident, between) => { $panel.justify_between() };
    (@justify_content $panel:ident, around) => { $panel.justify_around() };
    (@justify_content $panel:ident, evenly) => { $panel.justify_evenly() };
}

/// Declarative builder sugar for [`DockPanel`](crate::DockPanel).
#[macro_export]
macro_rules! dock_panel {
    (@build $panel:expr; ) => {
        $panel
    };

    (@build $panel:expr; last_child_fill = $value:expr; $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.last_child_fill($value); $($rest)*)
    };
    (@build $panel:expr; last_child_fill = $value:expr $(;)?) => {
        $panel.last_child_fill($value)
    };

    (@build $panel:expr; top: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.top($child); $($rest)*)
    };
    (@build $panel:expr; bottom: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.bottom($child); $($rest)*)
    };
    (@build $panel:expr; left: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.left($child); $($rest)*)
    };
    (@build $panel:expr; right: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.right($child); $($rest)*)
    };
    (@build $panel:expr; child: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.child($child); $($rest)*)
    };
    (@build $panel:expr; fill: $child:expr, $($rest:tt)*) => {
        $crate::dock_panel!(@build $panel.fill($child); $($rest)*)
    };

    (@build $panel:expr; top: $child:expr $(,)?) => {
        $panel.top($child)
    };
    (@build $panel:expr; bottom: $child:expr $(,)?) => {
        $panel.bottom($child)
    };
    (@build $panel:expr; left: $child:expr $(,)?) => {
        $panel.left($child)
    };
    (@build $panel:expr; right: $child:expr $(,)?) => {
        $panel.right($child)
    };
    (@build $panel:expr; child: $child:expr $(,)?) => {
        $panel.child($child)
    };
    (@build $panel:expr; fill: $child:expr $(,)?) => {
        $panel.fill($child)
    };

    (@build $panel:expr; $field:ident : $child:expr $(, $($rest:tt)*)?) => {
        compile_error!(concat!("dock_panel! only supports top, bottom, left, right, child, and fill entries; found `", stringify!($field), "`."))
    };

    () => {
        $crate::DockPanel::new()
    };

    ( $($tokens:tt)+ ) => {
        $crate::dock_panel!(@build $crate::DockPanel::new(); $($tokens)+)
    };
}

/// Declarative builder sugar for [`GridLayout`](crate::GridLayout).
#[macro_export]
macro_rules! grid_layout {
    (@colspan) => {
        1usize
    };
    (@colspan $colspan:expr) => {
        $colspan
    };

    (
        rows: $rows:expr,
        columns: [ $($col:expr),* $(,)? ]
        $(, gap: $gap:expr )?
        $(, gap_x: $gap_x:expr )?
        $(, gap_y: $gap_y:expr )?
        ;
        $(
            [ $r:expr, $c:expr $(, colspan: $cs:expr)? ] => $child:expr
        ),* $(,)?
    ) => {{
        let mut grid = $crate::GridLayout::new()
            .rows($rows)
            .columns(vec![ $($col),* ]);

        $( grid = grid.gap($gap as f32); )?
        $( grid = grid.gap_x($gap_x as f32); )?
        $( grid = grid.gap_y($gap_y as f32); )?

        $(
            grid = grid.child_with_span($child, $r, $c, $crate::grid_layout!(@colspan $($cs)?));
        )*

        grid
    }};
}

/// Spawns a flow wrapping flex layout (similar to a WPF WrapPanel).
#[macro_export]
macro_rules! wrappanel {
    // 1. Full parameters: orientation, gap, and cross-axis alignment
    (
        orientation=$orient:ident gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@orient panel, $orient);
        panel = $crate::wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Orientation and gap
    (
        orientation=$orient:ident gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@orient panel, $orient);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and alignment (defaults to horizontal)
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Orientation only
    (
        orientation=$orient:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        panel = $crate::wrappanel!(@orient panel, $orient);
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Gap only (defaults to horizontal)
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Simple wrapper (defaults to horizontal wrap, no gap)
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        $( panel = panel.child($child); )*
        panel
    }};

    // Helpers
    (@orient $panel:ident, horizontal) => { $panel };
    (@orient $panel:ident, vertical) => { $panel.flex_col() };
    (@orient $panel:ident, row) => { $panel };
    (@orient $panel:ident, col) => { $panel.flex_col() };

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}

/// Legacy wrapper alias for horizontal wrapping (retained for backward compatibility).
#[macro_export]
macro_rules! flow {
    ( $($args:tt)* ) => {
        $crate::wrappanel!(orientation=horizontal $($args)*)
    };
}

/// Spawns interactive forms and handles state/subscription mapping.
#[macro_export]
macro_rules! declare_form {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
            controls: {
                $(
                    $ctrl_name:ident : $ctrl_ty:ty = $builder:expr
                    $(=> $event_ty:ty |$this:ident, $event:ident, $cx:ident| $callback:block)?
                ),* $(,)?
            },
            args: {
                $( $arg_name:ident : $arg_ty:ty ),* $(,)?
            },
            fields: {
                $( $field_name:ident : $field_ty:ty = $field_init:expr ),* $(,)?
            }
        }
    ) => {
        $(#[$meta])*
        $vis struct $struct_name {
            $( $ctrl_name: $ctrl_ty, )*
            $( #[allow(dead_code)] $arg_name: $arg_ty, )*
            $( $field_name: $field_ty, )*
            _subscriptions: ::std::vec::Vec<::gpui::Subscription>,
        }

        impl $struct_name {
            $vis fn new(
                cx: &mut ::gpui::Context<Self>,
                $( $arg_name: $arg_ty ),*
            ) -> Self {
                $(
                    let $ctrl_name = $builder.spawn(cx);
                )*

                $(
                    let $field_name = $field_init;
                )*

                let mut _subscriptions = ::std::vec::Vec::new();
                $(
                    $crate::declare_form!(@push_subscription _subscriptions, cx, $ctrl_name $(=> $event_ty |$this, $event, $cx| $callback)?);
                )*

                Self {
                    $( $ctrl_name, )*
                    $( $arg_name, )*
                    $( $field_name, )*
                    _subscriptions,
                }
            }
        }
    };

    (@push_subscription $subscriptions:ident, $context:ident, $control:ident) => {};

    (@push_subscription $subscriptions:ident, $context:ident, $control:ident => $event_ty:ty |$this:ident, $event:ident, $cx:ident| $callback:block) => {
        $subscriptions.push($context.subscribe(&$control, |$this, _, $event: &$event_ty, $cx| $callback));
    };
}

/// Wraps a control in a labeled column with the standard 4px gap.
#[macro_export]
macro_rules! form_field {
    ($label:expr, $chrome:expr; $control:expr) => {
        ::gpui::div()
            .flex()
            .flex_col()
            .gap(::gpui::px($crate::theme::stylesheet::SharedGeometry::sdk().form_field_gap))
            .child($crate::infra::field_label::field_label($label, $chrome.body_text))
            .child($control)
    };
}
