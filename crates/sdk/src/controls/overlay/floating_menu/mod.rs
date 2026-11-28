//! Shared menu renderer and highlight state.
//!
//! **Not a spawnable LMTP control.** There is no `FloatingMenu` entity and no
//! `look.floating_menu(id)` factory. `popup_menu` / `context_menu` own the entities
//! and call these templates for list chrome, highlight motion, and submenu presence.
//!
//! Prefer those spawnable controls for app composition; import this module only when
//! building a look template or a derived menu surface.

mod model;
mod styled;
#[allow(deprecated)]
pub use styled::styled_floating_menu_look;
pub use styled::{StyledFloatingMenuTheme, floating_menu_source};
mod template;
pub(crate) mod theme;

pub use model::{FloatingMenuActivateResult, FloatingMenuState, FloatingMenuStepDirection};
pub use template::{
    FloatingMenuClickHandler, FloatingMenuHighlight, FloatingMenuHoverHandler, FloatingMenuRenderModel, MenuStyle,
    RowColors, FloatingMenuSeparatorTemplate, ThemedFloatingMenuSeparatorTemplate, FloatingMenuTemplate,
    FloatingMenuTemplateHandlers, FloatingMenuTemplateModifier, ThemedFloatingMenuTemplate,
    default_floating_menu_template, floating_menu_size, floating_menu_template_with_modifier,
    floating_menu_window_margin, render_floating_menu, render_floating_menu_with_submenu_hovers_and_icons,
    render_floating_menu_with_submenu_hovers, render_floating_menu_with_submenu_hovers_and_icons_and_transition,
    render_floating_menu_with_submenu_presence, render_floating_menu_with_submenu_presence_and_transition,
    render_floating_menu_with_template,
};
#[allow(deprecated)]
pub use theme::FloatingMenuLook;
pub use theme::{FloatingMenuTheme, default_floating_menu_theme};
