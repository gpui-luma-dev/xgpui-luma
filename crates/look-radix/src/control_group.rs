//! Radix group shells and toggle items over the SDK selection/focus engine.
use std::sync::Arc;
use gpui::{SharedString, div, prelude::*, px};
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
    Arc::new(move |item, custom, window, cx| {
        let metrics = look.metrics();
        let scale = StandardBoxScale::compute(size, &metrics, window.scale_factor());
        let state = InteractionState { focused: item.state.focus_visible, ..item.state.interaction_state() };
        let palette = item_look(&look, item.selected, state, size, &scale);
        let content = match custom {
            Some(custom) => {
                let content_model = ControlGroupItemRenderModel {
                    group_id: item.group_id,
                    stylesheet: item.stylesheet,
                    layout: item.layout,
                    item: item.item,
                    index: item.index,
                    sibling_count: item.sibling_count,
                    selected: item.selected,
                    selection_progress: item.selection_progress,
                    active: item.active,
                    enabled: item.enabled,
                    state: item.state,
                    selection_mode: item.selection_mode,
                    foreground: Some(palette.foreground),
                };
                custom(&content_model, window, cx)
            }
            None => div().child(item.item.label().clone()).into_any_element(),
        };
        div()
            .id((item.item.id().clone(), 0usize))
            .h(px(palette.height))
            .min_h(px(palette.height))
            .flex()
            .items_center()
            .justify_center()
            .px(px(if icon { 0.0 } else { palette.padding_x }))
            .when(icon, |row| row.w(px(palette.height)).min_w(px(palette.height)).max_w(px(palette.height)))
            .rounded(px(if icon { metrics.radius.pill } else { scale.radius }))
            .bg(palette.background)
            .text_color(palette.foreground)
            .font_family(palette.font_family)
            .text_size(px(palette.typography.size))
            .line_height(px(palette.typography.line_height))
            .font_weight(palette.typography.weight)
            .border_1()
            .border_color(if item.state.focus_visible && !item.state.disabled {
                look.resolve_role(SemanticRole::Focus).hsla()
            } else {
                palette.border.unwrap_or(gpui::transparent_black())
            })
            .when(!item.state.disabled, |row| row.cursor_pointer())
            .child(content)
    })
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
