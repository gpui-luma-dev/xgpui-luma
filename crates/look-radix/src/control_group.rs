//! Radix group shells and toggle items over the SDK selection/focus engine.
use std::sync::Arc;
use gpui::{AnyElement, App, BoxShadow, SharedString, Window, div, point, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
use xgpui_luma::controls::toggle::{ToggleData, apply_toggle_progress_chrome};
use xgpui_luma::controls::control_group::*;
use xgpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use xgpui_luma::theme::{InteractionState, MetricTokens, StandardBoxScale};
use crate::{ButtonVariant, Look, ScaleFamily, SemanticRole};

fn item_look(
    look: &Look,
    selected: bool,
    state: InteractionState,
    size: crate::RadixSize,
    scale: &StandardBoxScale,
) -> ButtonFamilyLook {
    let theme = crate::button_family_theme(&look.with_size(size), ButtonVariant::Ghost);
    let role = ButtonFamilyRole::Toggle { selected };
    theme
        .resolve_look(role, state, scale, scale.radius)
        .unwrap_or_else(|| compose_button_family_look(&theme.resolve(role, state), role, scale, scale.radius))
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
        self.look.metrics_for(self.look.size())
    }
    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let size = self.look.size();
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
/// Joined Radix toggle-group shell, composed through the SDK item-layout API.
pub fn icon_group_template<T: ControlGroupItemLike + 'static>(look: &Look) -> ControlGroupTemplate<T> {
    let look = look.clone();
    let template = control_group_item_layout_template(move |items, model, _, _| {
        div()
            .relative()
            .flex()
            .items_center()
            .when(model.layout == ControlGroupLayout::Vertical, |shell| shell.flex_col())
            .rounded(px(look.metrics().radius.md))
            .bg(look.resolve_role(SemanticRole::Surface).hsla())
            .shadow(vec![BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, 0.2),
                offset: point(px(0.0), px(2.0)),
                blur_radius: px(10.0),
                spread_radius: px(0.0),
                inset: false,
            }])
            .children(items.into_elements().map(|item| div().flex_none().child(item)))
    });
    Arc::new(move |model, handlers, window, cx| template(model, handlers, window, cx).flex().items_start())
}

fn item_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    size: crate::RadixSize,
) -> ControlGroupItemElementTemplate<T> {
    let look = look.clone();
    Arc::new(move |item, custom, window, cx| {
        let scale = StandardBoxScale::compute(&look.metrics_for(size), window.scale_factor());
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
            .child(xgpui_luma::infra::focus_adorner::FocusAdorner::new(look.metrics().focus).overlay(
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
    size: crate::RadixSize,
) -> ControlGroupItemElementTemplate<T> {
    item_template(look, size)
}
/// Toggle items for an icon group, composed with the SDK toggle renderer.
/// Supply this through `ControlGroupBuilder::item_template`; content supplies only
/// text, icons, or both; the SDK renders adjoining rectangular toggle faces.
pub fn icon_group_item_template<T>(
    look: &Look,
    size: crate::RadixSize,
    content: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    let size = match size {
        crate::RadixSize::One => crate::ButtonSize::One,
        crate::RadixSize::Two => crate::ButtonSize::Two,
        crate::RadixSize::Three => crate::ButtonSize::Three,
    };
    let look = look.clone();
    let content = Arc::new(content);
    make_control_group_item_template(move |item, window, cx| {
        let template = Arc::new(IconGroupToggleTemplate {
            look: look.clone(),
            size,
            first: item.index == 0,
            last: item.index + 1 == item.sibling_count,
            layout: item.layout,
        });
        let content = content.clone();
        animated_toggle_button_item_template(template, false, move |item| content(item))(item, window, cx)
    })
}

// Look-owned presentation adapter: the SDK still renders every toggle face,
// custom content, focus overlay, disabled state, and selection transition.
struct IconGroupToggleTemplate {
    look: Look,
    size: crate::ButtonSize,
    first: bool,
    last: bool,
    layout: ControlGroupLayout,
}

fn icon_group_palette(look: &Look, selected: bool, state: InteractionState) -> ButtonFamilyPalette {
    let mut palette =
        crate::button_family_theme(look, ButtonVariant::Ghost).resolve(ButtonFamilyRole::Toggle { selected }, state);
    let accent = |step| look.resolve_step(ScaleFamily::Color, step).hsla();
    let gray = |step| look.resolve_step(ScaleFamily::Gray, step).hsla();
    palette.background = if state.disabled {
        look.resolve_role(SemanticRole::Surface).hsla()
    } else if selected {
        accent(if state.pressed {
            5
        } else if state.hovered {
            4
        } else {
            3
        })
    } else if state.pressed {
        gray(4)
    } else if state.hovered {
        gray(3)
    } else {
        look.resolve_role(SemanticRole::Surface).hsla()
    };
    palette.foreground = if state.disabled {
        gray(8)
    } else if selected {
        accent(11)
    } else {
        gray(11)
    };
    palette.border = None;
    palette.focus_border = look.resolve_role(SemanticRole::Focus).hsla();
    palette
}

impl ButtonTemplate<ToggleData> for IconGroupToggleTemplate {
    fn render(
        &self,
        model: &ButtonRenderModel<ToggleData>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let off = icon_group_palette(&self.look, false, model.state);
        let on = icon_group_palette(&self.look, true, model.state);
        let palette = if model.data.selected { &on } else { &off };
        let mut look = crate::button_look_for(
            &self.look,
            ButtonVariant::Ghost,
            crate::Paint::accent(),
            self.size,
            crate::Radius::None,
        )(model);
        look.background = palette.background;
        look.foreground = palette.foreground;
        look.border = None;
        let focus_color = palette.focus_border;
        // The shared button's uniform ring is replaced by the position-aware adorner below.
        look.focus_border = gpui::transparent_black();
        look.padding_y = 0.0;
        look.padding_x = ((look.height - 16.0) / 2.0).max(0.0);
        let height = look.height;
        let model = ButtonRenderModel {
            id: model.id.clone(),
            data: model.data,
            content: model.content.clone(),
            role: model.role,
            state: model.state,
            compact: true,
            elevation: false,
            round: false,
            radius_override: std::cell::Cell::new(Some(self.look.metrics().radius.md)),
            look: Some(Arc::new(move |_| look.clone())),
            ..Default::default()
        };
        let first = self.first;
        let last = self.last;
        let radius = px(self.look.metrics().radius.md);
        let separator = !last;
        let vertical = self.layout == ControlGroupLayout::Vertical;
        let border = self.look.resolve_step(ScaleFamily::Gray, 6).hsla();
        let focus = xgpui_luma::infra::focus_adorner::FocusAdorner::new(self.look.metrics().focus);
        let outset = px(focus.width);
        let focus_radius = radius + outset;
        let outline = focus
            .overlay(focus_color, model.state.focused && !model.state.disabled, 0.0, true)
            .top(-outset)
            .right(-outset)
            .bottom(-outset)
            .left(-outset)
            .when(first && !vertical, |ring| ring.rounded_tl(focus_radius).rounded_bl(focus_radius))
            .when(last && !vertical, |ring| ring.rounded_tr(focus_radius).rounded_br(focus_radius))
            .when(first && vertical, |ring| ring.rounded_tl(focus_radius).rounded_tr(focus_radius))
            .when(last && vertical, |ring| ring.rounded_bl(focus_radius).rounded_br(focus_radius));
        DefaultButtonTemplate::<ToggleData>::new(crate::button_family_theme(&self.look, ButtonVariant::Ghost))
            .with_modifier(move |face, model| {
                apply_toggle_progress_chrome(face, &off, &on, model.data.progress)
                    .border_0()
                    .rounded_none()
                    .min_w(px(height))
                    .when(first && !vertical, |face| face.rounded_tl(radius).rounded_bl(radius))
                    .when(last && !vertical, |face| face.rounded_tr(radius).rounded_br(radius))
                    .when(first && vertical, |face| face.rounded_tl(radius).rounded_tr(radius))
                    .when(last && vertical, |face| face.rounded_bl(radius).rounded_br(radius))
                    .when(separator && !vertical, |face| face.border_r_1().border_color(border))
                    .when(separator && vertical, |face| face.border_b_1().border_color(border))
            })
            .render(&model, window, cx)
            // Draw above neighboring faces without changing item or group bounds.
            .child(gpui::deferred(outline))
            .debug_selector(|| model.id.to_string())
    }
}

/// Themed SDK group builder, retaining selection, callbacks, and focus policies.
pub fn control_group_builder<T: ControlGroupItemLike + 'static>(
    look: &Look,
    id: impl Into<SharedString>,
    size: crate::RadixSize,
) -> ControlGroupBuilder<T> {
    ControlGroupBuilder::new(id)
        .stylesheet(&look.sized_stylesheet(look.size()))
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
    xgpui_luma::controls::icon_group::new(id)
        .horizontal()
        .template(icon_group_template(look))
        .stylesheet(&look.sized_stylesheet(look.size()))
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
            for size in [crate::RadixSize::One, crate::RadixSize::Two, crate::RadixSize::Three] {
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
                            crate::RadixSize::One => crate::ButtonSize::One,
                            crate::RadixSize::Two => crate::ButtonSize::Two,
                            crate::RadixSize::Three => crate::ButtonSize::Three,
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

    #[test]
    fn joined_toggle_faces_support_text_and_icons_without_gaps() {
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            for vertical in [false, true] {
                for content_kind in 0..3 {
                    let look = Look::built_in();
                    look.set_mode(mode);
                    let mut app = TestAppContext::single();
                    let paint_look = look.clone();
                    let (_, cx) = app.add_window_view(|_, cx| {
                        let template = icon_group_item_template(
                            &paint_look,
                            crate::RadixSize::Two,
                            move |item: &ControlGroupItem| {
                                let selector = format!("content-{}", item.id());
                                div()
                                    .id(item.id().clone())
                                    .debug_selector(move || selector.clone())
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .when(content_kind != 1, |row| row.child(div().size(px(15.0))))
                                    .when(content_kind != 0, |row| row.child(item.label().clone()))
                                    .into_any_element()
                            },
                        );
                        let mut builder = icon_group_builder(&paint_look, "joined")
                            .items([
                                ControlGroupItem::new("a").label("Align left"),
                                ControlGroupItem::new("b").label("Center"),
                                ControlGroupItem::new("c").label("Align right"),
                            ])
                            .selected("b")
                            .item_template(template);
                        if vertical {
                            builder = builder.vertical();
                        }
                        Harness { group: builder.spawn(cx), changes: 0 }
                    });
                    cx.simulate_mouse_move(point(px(1000.0), px(1000.0)), None, Default::default());
                    cx.run_until_parked();
                    let a = cx.debug_bounds("joined-a").unwrap();
                    let b = cx.debug_bounds("joined-b").unwrap();
                    let c = cx.debug_bounds("joined-c").unwrap();
                    if vertical {
                        assert_eq!(a.bottom(), b.top());
                        assert_eq!(b.bottom(), c.top());
                    } else {
                        assert_eq!(a.right(), b.left());
                        assert_eq!(b.right(), c.left());
                    }
                    if content_kind != 0 {
                        assert!(a.size.width > a.size.height, "text items can grow beyond square icon targets");
                    }
                    let content = cx.debug_bounds("content-a").unwrap();
                    assert!(
                        content.left() >= a.left() && content.right() <= a.right(),
                        "custom content fits its toggle"
                    );
                    cx.update(|window, _| {
                        let quads = window.painted_quads();
                        let radius = px(look.metrics().radius.md).scale(window.scale_factor());
                        let face = |bounds: gpui::Bounds<gpui::Pixels>| {
                            quads
                                .iter()
                                .find(|quad| {
                                    quad.bounds == bounds.scale(window.scale_factor())
                                        && quad.background.as_solid().is_some_and(|color| color.a > 0.0)
                                })
                                .expect("painted toggle face")
                        };
                        let first = face(a).corner_radii;
                        let middle = face(b).corner_radii;
                        let last = face(c).corner_radii;
                        assert_eq!(first.top_left, radius, "first face owns its outer radius");
                        assert_eq!(last.bottom_right, radius, "last face owns its outer radius");
                        assert_eq!(middle.top_left, px(0.0).scale(window.scale_factor()));
                        if vertical {
                            assert_eq!(first.bottom_left, px(0.0).scale(window.scale_factor()));
                        } else {
                            assert_eq!(first.top_right, px(0.0).scale(window.scale_factor()));
                        }
                        let divider = look.resolve_step(ScaleFamily::Gray, 6).hsla();
                        let mut separator_faces = Vec::new();
                        for quad in quads.iter().filter(|quad| {
                            quad.border_color == divider
                                && if vertical {
                                    quad.border_widths.bottom > px(0.0).scale(window.scale_factor())
                                } else {
                                    quad.border_widths.right > px(0.0).scale(window.scale_factor())
                                }
                        }) {
                            // GPUI splits border paint into clipped quads for each face.
                            if !separator_faces.contains(&quad.bounds) {
                                separator_faces.push(quad.bounds);
                            }
                        }
                        assert_eq!(separator_faces.len(), 2, "only adjacent items have separators");
                        let tint = look.resolve_step(ScaleFamily::Color, 3).hsla();
                        assert!(
                            quads.iter().any(|quad| quad.background.as_solid().is_some_and(|color| (color.h - tint.h)
                                .abs()
                                < 0.00001
                                && (color.s - tint.s).abs() < 0.00001
                                && (color.l - tint.l).abs() < 0.00001
                                && color.a == tint.a)),
                            "selection uses the Radix accent tint"
                        );
                    });
                }
            }
        }
    }

    #[test]
    fn focus_outlines_follow_first_middle_last_and_single_item_corners() {
        use gpui::Focusable;
        for vertical in [false, true] {
            for count in 1..=3 {
                for index in 0..count {
                    let look = Look::built_in();
                    let metrics = look.metrics();
                    let mut app = TestAppContext::single();
                    let (_, cx) = app.add_window_view(|window, cx| {
                        window.activate_window();
                        let id = ["a", "b", "c"][index];
                        let mut builder = icon_group_builder(&look, "position")
                            .items(["a", "b", "c"].into_iter().take(count).map(ControlGroupItem::new))
                            .active(id)
                            .selected(id)
                            .item_template(icon_group_item_template(
                                &look,
                                crate::RadixSize::Two,
                                |_: &ControlGroupItem| div().size(px(16.0)).into_any_element(),
                            ))
                            .with_template_modifier(|root, _| root.m(px(8.0)));
                        if vertical {
                            builder = builder.vertical();
                        }
                        let group = builder.spawn(cx);
                        group.read(cx).focus_handle(cx).focus(window, cx);
                        Harness { group, changes: 0 }
                    });
                    cx.run_until_parked();
                    let bounds = cx.debug_bounds(["position-a", "position-b", "position-c"][index]).unwrap();
                    cx.update(|window, _| {
                        let width = px(metrics.focus.width);
                        let outer = bounds.dilate(width).scale(window.scale_factor());
                        let radius = (px(metrics.radius.md) + width).scale(window.scale_factor());
                        let zero = px(0.0).scale(window.scale_factor());
                        let first = index == 0;
                        let last = index + 1 == count;
                        let expected = gpui::Corners {
                            top_left: if first { radius } else { zero },
                            top_right: if if vertical { first } else { last } {
                                radius
                            } else {
                                zero
                            },
                            bottom_left: if if vertical { last } else { first } {
                                radius
                            } else {
                                zero
                            },
                            bottom_right: if last { radius } else { zero },
                        };
                        let color = look.resolve_role(SemanticRole::Focus).hsla();
                        let quads = window.painted_quads();
                        let rings: Vec<_> = quads.iter().filter(|quad| quad.border_color == color).collect();
                        assert!(!rings.is_empty(), "focused item has an outline");
                        for ring in rings {
                            assert_eq!(ring.bounds, outer, "outline sits outside only the focused face");
                            assert_eq!(
                                ring.corner_radii, expected,
                                "count={count}, index={index}, vertical={vertical}"
                            );
                        }
                    });
                }
            }
        }
    }

    #[test]
    fn selected_toggle_colors_match_soft_text_fields() {
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            let look = Look::built_in();
            look.set_mode(mode);
            for hovered in [false, true] {
                let toggle = icon_group_palette(&look, true, InteractionState { hovered, ..Default::default() });
                let field = crate::textfield::resolve_text_chrome_palette(
                    &look,
                    crate::TextFieldVariant::Soft,
                    hovered,
                    false,
                    true,
                );
                assert_eq!(toggle.background, field.background);
                assert_eq!(toggle.focus_border, field.focus_border);
            }
        }
    }
}
