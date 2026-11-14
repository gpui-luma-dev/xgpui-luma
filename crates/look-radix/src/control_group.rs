//! Radix group shells and toggle items over the SDK selection/focus engine.
use std::sync::Arc;
use gpui::{AnyElement, SharedString, div, prelude::*, px};
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
        let radius = scale.radius;
        div()
            .id((item.item.id().clone(), 0usize))
            .relative()
            .h(px(palette.height))
            .min_h(px(palette.height))
            .flex()
            .items_center()
            .justify_center()
            .px(px(palette.padding_x))
            .rounded(px(radius))
            .bg(palette.background)
            .text_color(palette.foreground)
            .font_family(palette.font_family)
            .text_size(px(palette.typography.size))
            .line_height(px(palette.typography.line_height))
            .font_weight(palette.typography.weight)
            .border_1()
            .border_color(palette.border.unwrap_or_else(gpui::transparent_black))
            .when(!item.state.disabled, |row| row.cursor_pointer())
            .child(content)
            .child(xgpui_luma::infra::focus_adorner::FocusAdorner::new(metrics.focus).overlay(
                palette.focus_border,
                item.state.focus_visible && !item.state.disabled,
                radius,
                true,
            ))
    })
}
/// Radix toggle row; SDK item-content overrides are retained inside this chrome.
pub fn control_group_item_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    size: ControlSize,
) -> ControlGroupItemElementTemplate<T> {
    item_template(look, size)
}
/// Toggle items for an icon group, composed with the SDK toggle renderer.
/// Supply this through `ControlGroupBuilder::item_template`; content supplies only
/// the icon, while the SDK renders the toggle chrome and selection transition.
pub fn icon_group_item_template<T>(
    look: &Look,
    size: ControlSize,
    content: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    let size = match size {
        ControlSize::Sm => crate::ButtonSize::One,
        ControlSize::Md => crate::ButtonSize::Two,
        ControlSize::Lg => crate::ButtonSize::Three,
    };
    let family = crate::button::button_family_theme_for_size(look, ButtonVariant::Ghost, crate::Paint::accent(), size);
    animated_toggle_button_item_template(crate::toggle::toggle_template_with_family(family), true, content)
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
/// Horizontal SDK icon-toolbar shell. Supply toggle items with
/// [`icon_group_item_template`] through the returned builder's `item_template`.
/// Item size belongs to that template, independently of the group shell.
pub fn icon_group_builder<T: ControlGroupItemLike + 'static>(
    look: &Look,
    id: impl Into<SharedString>,
) -> ControlGroupBuilder<T> {
    xgpui_luma::controls::icon_group::icon_toolbar(id, control_group_theme(look))
        .stylesheet(&look.common_stylesheet())
        .motion_defaults(look.motion_defaults().control_group)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext, Window};

    struct Harness {
        group: ControlGroup<ControlGroupItem>,
        changes: usize,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div().size_full().child(self.group.clone())
        }
    }

    #[test]
    fn icon_groups_compose_sdk_toggles_with_custom_content_and_group_input() {
        for multiple in [false, true] {
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                let mut app = TestAppContext::single();
                let (view, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    xgpui_luma::focus::bind_default_focus_keys(cx);
                    xgpui_luma::key_handling::bind_default_control_keys(cx);
                    let look = Look::built_in();
                    let template = icon_group_item_template(&look, size, |item: &ControlGroupItem| {
                        let selector = format!("glyph-{}", item.id());
                        div()
                            .id(item.id().clone())
                            .debug_selector(move || selector.clone())
                            .size(px(12.0))
                            .child(item.label().clone())
                            .into_any_element()
                    });
                    let mut builder = icon_group_builder(&look, "icons")
                        .items([
                            ControlGroupItem::new("a").label("A"),
                            ControlGroupItem::new("b").label("B"),
                            ControlGroupItem::new("c").label("C").enabled(false),
                        ])
                        .item_template(template)
                        .on_value_changed(cx, |owner: &mut Harness, _, _, _, _| owner.changes += 1);
                    if multiple {
                        builder = builder.multiple().independent_toggles(true);
                    }
                    Harness { group: builder.spawn(cx), changes: 0 }
                });
                cx.run_until_parked();
                let click = |cx: &mut gpui::VisualTestContext, selector: &'static str| {
                    let bounds = cx.debug_bounds(selector).expect("custom glyph rendered inside SDK toggle");
                    cx.simulate_click(bounds.center(), Default::default());
                    cx.run_until_parked();
                };
                click(cx, "glyph-a");
                click(cx, "glyph-b");
                click(cx, "glyph-c");
                cx.update(|_, app| {
                    let harness = view.read(app);
                    let selected = harness.group.read(app).selected_ids();
                    assert_eq!(selected, if multiple { vec!["a", "b"] } else { vec!["b"] });
                    assert_eq!(harness.changes, 2);
                });
                cx.simulate_keystrokes("right space");
                cx.run_until_parked();
                let group = cx.update(|_, app| view.read(app).group.clone());
                cx.update(|_, app| {
                    assert_eq!(group.read(app).active_id(), Some(&"a".into()), "keyboard skips disabled C");
                    assert_eq!(group.read(app).selected_ids().iter().any(|id| id == "a"), !multiple);
                    assert_eq!(view.read(app).changes, 3);
                });
                // The existing toggle renderer supplies a square face at the requested Radix size.
                cx.update(|window, _| {
                    let height = crate::button_box_for(
                        match size {
                            ControlSize::Sm => crate::ButtonSize::One,
                            ControlSize::Md => crate::ButtonSize::Two,
                            ControlSize::Lg => crate::ButtonSize::Three,
                        },
                        crate::Radius::Full,
                    )
                    .height;
                    assert!(window.painted_quads().iter().any(|quad| {
                        quad.bounds.size.width == px(height).scale(window.scale_factor())
                            && quad.bounds.size.height == px(height).scale(window.scale_factor())
                    }));
                });
                group.update(cx, |group, cx| group.set_enabled(false, cx));
                cx.run_until_parked();
                click(cx, "glyph-b");
                cx.simulate_keystrokes("space");
                cx.run_until_parked();
                cx.update(|_, app| assert_eq!(view.read(app).changes, 3));
            }
        }
    }
}
