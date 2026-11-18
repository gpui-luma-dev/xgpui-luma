use std::sync::Arc;

use gpui::{SharedString, Styled, px};

pub use crate::controls::control_group::{
    ControlGroupBuilder, ControlGroupControl, ControlGroupEvent, ControlGroupItem, ControlGroupItemLike,
    ControlGroupTheme,
};
use crate::controls::control_group::{self as control_group, control_group_template_with_theme, ControlGroupTemplate};

pub type IconGroup<T> = gpui::Entity<ControlGroupControl<T>>;
pub type IconGroupEvent = ControlGroupEvent;
pub type IconGroupBuilder<T> = ControlGroupBuilder<T>;
pub type IconGroupItem = ControlGroupItem;
pub use ControlGroupItemLike as IconGroupItemLike;

pub fn new<T>(id: impl Into<SharedString>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::new(id).single_allow_none()
}

/// Horizontal icon toolbar with themed group chrome (border, background, padding).
pub fn horizontal<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar(id, theme)
}

/// Icon toolbar preset: horizontal layout, live-themed housing, rounded group container.
pub fn icon_toolbar<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar_with_mode(id, theme, false)
}

/// Multi-select icon toolbar (same chrome as [`icon_toolbar`]).
pub fn icon_toolbar_multiple<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar_with_mode(id, theme, true)
}

fn icon_toolbar_with_mode<T>(
    id: impl Into<SharedString>,
    theme: Arc<dyn ControlGroupTheme>,
    multiple: bool,
) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    let mut builder = new(id).horizontal().template(icon_toolbar_template(theme));

    if multiple {
        builder = builder.multiple();
    }

    builder
}

/// Capsule group chrome with independent overrides, resolved before consumer modifiers.
pub fn icon_toolbar_template<T>(theme: Arc<dyn ControlGroupTheme>) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    let base = control_group_template_with_theme(theme.clone());
    Arc::new(move |model, handlers, window, cx| {
        let mut look = theme.resolve_list(model.enabled);
        if let Some(stylesheet) = model.shell_stylesheet {
            look = look.with_stylesheet(&stylesheet.control_group);
        }
        let geometry = model.stylesheet.icon_group.resolve_geometry_for(
            "toolbar",
            crate::theme::stylesheet::IconGroupGeometry {
                padding_x: look.padding_x,
                padding_y: look.padding_y,
                gap: look.gap,
                radius: theme.metrics().radius.pill,
            },
        );
        base(model, handlers, window, cx)
            .rounded(px(geometry.radius))
            .gap(px(geometry.gap))
            .px(px(geometry.padding_x))
            .py(px(geometry.padding_y))
    })
}
