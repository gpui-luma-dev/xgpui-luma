//! Radix group shells and toggle items over the SDK selection/focus engine.
use std::sync::Arc;
use gpui::{SharedString, Styled};
use xgpui_luma::controls::button::DefaultButtonTemplate;
use xgpui_luma::controls::control_group::*;
use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole, compose_button_family_look};
use xgpui_luma::theme::{ControlSize, InteractionState, MetricTokens, StandardBoxScale};
use crate::{ButtonVariant, Look, SemanticRole};

fn item_look(
    look: &Look,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
    scale: &StandardBoxScale,
) -> ButtonFamilyLook {
    let theme = crate::button_family_theme(look, ButtonVariant::Ghost);
    let role = ButtonFamilyRole::Toggle { selected };
    theme
        .resolve_look(role, size, state, scale, scale.radius)
        .unwrap_or_else(|| compose_button_family_look(&theme.resolve(role, size, state), role, scale, scale.radius))
}

struct GroupTheme {
    look: Look,
}
impl ControlGroupTheme for GroupTheme {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook {
        let mut shell =
            DefaultControlGroupTheme::new(crate::navigation::geometry_tokens(&self.look)).resolve_list(enabled);
        shell.background = self.look.resolve_role(SemanticRole::Surface).hsla();
        shell.border = self.look.resolve_role(SemanticRole::Border).hsla();
        shell
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let palette = item_look(&self.look, selected, state, size, scale);
        ControlGroupItemPalette {
            foreground: palette.foreground,
            background: palette.background,
            muted_foreground: palette.muted_foreground,
            typography: palette.typography,
            font_family: palette.font_family,
        }
    }
}
/// Live Radix shell and item palettes; mutations require notifying affected owners.
pub fn control_group_theme(look: &Look) -> Arc<dyn ControlGroupTheme> {
    Arc::new(GroupTheme { look: look.clone() })
}
/// SDK group shell. Pair with [`control_group_item_template`] for Radix item chrome.
pub fn control_group_template<T: ControlGroupItemLike + 'static>(look: &Look) -> ControlGroupTemplate<T> {
    control_group_template_with_theme(control_group_theme(look))
}
/// SDK icon-toolbar shell, including independent `icon_group` geometry overrides.
pub fn icon_group_template<T: ControlGroupItemLike + 'static>(look: &Look) -> ControlGroupTemplate<T> {
    xgpui_luma::controls::icon_group::icon_toolbar_template(control_group_theme(look))
}
fn item_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    size: ControlSize,
    icon: bool,
) -> ControlGroupItemElementTemplate<T> {
    let look = look.clone();
    let template = DefaultButtonTemplate::new(crate::button_family_theme(&look, ButtonVariant::Ghost));
    let template = button_item_element_template(template, size, icon, move |selected, state, scale| {
        let mut palette = item_look(&look, selected, state, size, scale);
        // Group rows retain their compact footprint; geometry remains look-owned.
        palette.padding_y = 0.0;
        palette.radius = if icon { look.metrics().radius.pill } else { scale.radius };
        if icon {
            palette.padding_x = 0.0;
        }
        palette
    });
    // Radix group recipes already resolve disabled colors, as before this adapter.
    Arc::new(move |item, custom, window, cx| template(item, custom, window, cx).opacity(1.0))
}
/// Radix toggle row; SDK item-content overrides are retained inside this chrome.
pub fn control_group_item_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    size: ControlSize,
) -> ControlGroupItemElementTemplate<T> {
    item_template(look, size, false)
}
/// Square icon hit target; supply glyphs through the SDK item-content template.
pub fn icon_group_item_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    size: ControlSize,
) -> ControlGroupItemElementTemplate<T> {
    item_template(look, size, true)
}
/// Themed SDK group builder, retaining selection, callbacks, and focus policies.
pub fn control_group_builder<T: ControlGroupItemLike + 'static>(
    look: &Look,
    id: impl Into<SharedString>,
    size: ControlSize,
) -> ControlGroupBuilder<T> {
    ControlGroupBuilder::new(id)
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().control_group)
        .template(control_group_template(look))
        .item_element_template(control_group_item_template(look, size))
}
/// Horizontal icon-toolbar preset. Configure selection and glyph content on the returned SDK builder.
pub fn icon_group_builder<T: ControlGroupItemLike + 'static>(
    look: &Look,
    id: impl Into<SharedString>,
    size: ControlSize,
) -> ControlGroupBuilder<T> {
    xgpui_luma::controls::icon_group::icon_toolbar(id, control_group_theme(look))
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().control_group)
        .item_element_template(icon_group_item_template(look, size))
}
