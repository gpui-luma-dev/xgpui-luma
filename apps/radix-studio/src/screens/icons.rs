//! Icons screen with SDK buttons and popup icon details.

use std::sync::Arc;
use gpui::{
    Bounds, ClipboardItem, Context, Entity, Hsla, IntoElement, PathBuilder, Pixels, Point, Render, Subscription,
    WeakEntity, Window, canvas, div, point, prelude::*, px,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{vstack, hstack};
use xgpui_luma::controls::button::Button;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_radix::{self as radix, Look, SemanticRole};
use crate::assets::{ICON_GROUPS, icon_named, react_icon};
use crate::assets::Icon;

const MINT_LINE: Hsla = Hsla { h: 168.0 / 360.0, s: 0.80, l: 0.60, a: 0.24 };
const PINK_LINE: Hsla = Hsla { h: 325.0 / 360.0, s: 0.90, l: 0.65, a: 0.28 };

pub struct State {
    groups: Vec<(&'static str, Vec<Entity<radix::HoverCard>>)>,
    _details: Entity<IconDetails>,
    _subscriptions: Vec<Subscription>,
}
impl State {
    pub fn new<M: 'static>(look: &Arc<Look>, cx: &mut Context<M>) -> Self {
        let details = cx.new(|cx| IconDetails::new(look.clone(), cx));
        let mut subscriptions = Vec::new();
        let groups = ICON_GROUPS
            .iter()
            .map(|(title, names)| {
                let buttons = names
                    .iter()
                    .filter_map(|name| {
                        let icon = icon_named(name)?;
                        let button = radix::Button::new(format!("inspect-icon-{name}"))
                            .look(look)
                            .ghost_quiet()
                            .content(move |model, _| react_icon(icon, model.look.foreground, 15.0))
                            .with_template_modifier({
                                let name = *name;
                                move |root, _| root.debug_selector(move || format!("inspect-icon-{name}"))
                            })
                            .spawn(cx);
                        let content = details.clone();
                        let name = *name;
                        let group = *title;
                        let popup = radix::HoverCard::new(format!("icon-info-{name}"), button)
                            .look(look)
                            .content(move |_, cx| {
                                if content.read(cx).selected.is_some_and(|(selected, _, _)| selected == name) {
                                    content.clone().into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            })
                            .spawn(cx);
                        let panel = details.clone();
                        subscriptions.push(cx.subscribe(&popup, move |_, popup, event: &radix::HoverCardEvent, cx| {
                            if matches!(event, radix::HoverCardEvent::OpenChanged { open: true }) {
                                let previous = panel.read(cx).popup.as_ref().and_then(WeakEntity::upgrade);
                                if let Some(previous) = previous
                                    && previous != popup
                                {
                                    previous.update(cx, |popup, cx| popup.dismiss(cx));
                                }
                                panel.update(cx, |panel, cx| {
                                    panel.selected = Some((name, group, icon));
                                    panel.copy.update(cx, |button, cx| button.set_data(false, cx));
                                    panel.popup = Some(popup.downgrade());
                                    cx.notify();
                                });
                            }
                        }));
                        Some(popup)
                    })
                    .collect();
                (*title, buttons)
            })
            .collect();
        Self { groups, _details: details, _subscriptions: subscriptions }
    }
    pub fn render(&self, fg: Hsla, muted: Hsla, surface: Hsla) -> gpui::AnyElement {
        let mut groups = vstack! { gap=28; };
        for (title, buttons) in &self.groups {
            groups = groups.child(
                vstack! { gap=16;
                    div().text_sm().font_weight(gpui::FontWeight::MEDIUM).text_color(fg).child(*title),
                    div().w_full().flex().flex_wrap().gap(px(8.0)).children(buttons.iter().cloned()),
                }
                .w_full(),
            );
        }
        vstack! { gap=16 align=center;
            div().text_2xl().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child("Icons"),
            div().text_sm().text_color(muted).child("Hover over an icon to inspect its details."),
            div().w_full().rounded_xl().bg(surface).px_8().py_8().child(groups),
        }
        .w_full()
        .px_6()
        .py_12()
        .into_any_element()
    }
}

struct IconDetails {
    look: Arc<Look>,
    popup: Option<WeakEntity<radix::HoverCard>>,
    selected: Option<(&'static str, &'static str, Icon)>,
    copy: Entity<Button<bool>>,
}
impl IconDetails {
    fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let copy = radix::Button::new("icon-details-copy")
            .look(&look)
            .typed(false)
            .content(|model, _| div().child(if model.data { "Copied" } else { "Copy SVG" }).into_any_element())
            .with_template_modifier(|root, _| root.debug_selector(|| "icon-details-copy".into()))
            .spawn(cx);

        copy.clone().on_click(cx, |this, cx| {
            if let Some((_, _, icon)) = this.selected {
                match std::str::from_utf8(icon.svg_bytes()) {
                    Ok(svg) => {
                        cx.write_to_clipboard(ClipboardItem::new_string(svg.to_owned()));
                        this.copy.update(cx, |button, cx| button.set_data(true, cx));
                    }
                    Err(error) => eprintln!("failed to copy icon SVG: {error}"),
                }
            }
        });

        Self { look, popup: None, selected: None, copy }
    }
}
impl Render for IconDetails {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let Some((name, group, icon)) = self.selected else {
            return div().into_any_element();
        };
        let fg = self.look.resolve_role(SemanticRole::Foreground).hsla();
        hstack! { gap=20 align=center;
            react_icon(icon, fg, 60.0),
            vstack! { gap=6;
                div().font_weight(gpui::FontWeight::SEMIBOLD).child(name),
                div().text_sm().child(format!("{group} · 15 × 15 · SVG · {} bytes", icon.svg_bytes().len())),
                div().text_xs().child(icon.asset_path()),
                div().text_xs().child("Radix Icons · MIT license"),
                self.copy.clone(),
            },
        }
        .debug_selector(|| "icon-details".into())
        .w(px(440.0))
        .text_color(fg)
        .into_any_element()
    }
}

pub fn hero_decoration(fg: Hsla) -> impl IntoElement {
    div().absolute().inset_0().overflow_hidden().child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                paint_hero_construction(bounds, fg, window);
            },
        )
        .absolute()
        .size_full(),
    )
}

fn paint_hero_construction(bounds: Bounds<Pixels>, fg: Hsla, window: &mut Window) {
    let s = 80.0_f32;
    let x0 = f32::from(bounds.left()) + (f32::from(bounds.size.width) - s) * 0.5 - 200.0;
    let y0 = f32::from(bounds.top()) + 230.0;

    let span = (f32::from(bounds.size.width) + f32::from(bounds.size.height)) * 2.0;

    let mut paint_ray = |px_off: f32, py_off: f32, deg: f32, color: Hsla| {
        let angle = deg.to_radians();
        let dx = angle.cos() * span;
        let dy = angle.sin() * span;
        let cx = x0 + px_off * s;
        let cy = y0 + py_off * s;

        let mut b = PathBuilder::stroke(px(1.0));
        b.move_to(point(px(cx - dx), px(cy - dy)));
        b.line_to(point(px(cx + dx), px(cy + dy)));
        if let Ok(path) = b.build() {
            window.paint_path(path, color);
        }
    };

    paint_ray(0.0, 0.0, 0.0, MINT_LINE);
    paint_ray(0.0, 1.0, 0.0, MINT_LINE);
    paint_ray(0.0, 0.0, 90.0, MINT_LINE);
    paint_ray(1.0, 0.0, 90.0, MINT_LINE);

    paint_ray(0.0, 0.467, 0.0, PINK_LINE);
    paint_ray(0.0, 0.533, 0.0, PINK_LINE);
    paint_ray(-0.288, 0.0, 45.0, PINK_LINE);
    paint_ray(-0.288, 0.090, 45.0, PINK_LINE);
    paint_ray(0.622, 0.0, -45.0, PINK_LINE);
    paint_ray(0.622, 0.090, -45.0, PINK_LINE);
    paint_ray(0.242, 0.0, 45.0, PINK_LINE);
    paint_ray(0.242, 1.0, -45.0, PINK_LINE);

    paint_hero_arrow(Point::new(px(x0), px(y0)), s, fg, window);
}

fn paint_hero_arrow(origin: Point<Pixels>, s: f32, fg: Hsla, window: &mut Window) {
    let scale = s / 15.0;
    let mut b = PathBuilder::fill();
    let p = |x: f32, y: f32| point(origin.x + px(x * scale), origin.y + px(y * scale));

    b.move_to(p(6.854, 3.146));
    b.line_to(p(3.707, 7.0));
    b.line_to(p(12.5, 7.0));
    b.line_to(p(13.0, 7.5));
    b.line_to(p(12.5, 8.0));
    b.line_to(p(3.707, 8.0));
    b.line_to(p(6.854, 11.854));
    b.line_to(p(6.146, 11.854));
    b.line_to(p(2.146, 7.854));
    b.line_to(p(2.0, 7.5));
    b.line_to(p(2.146, 7.146));
    b.line_to(p(6.146, 3.146));
    b.close();

    if let Ok(path) = b.build() {
        window.paint_path(path, fg);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    struct Harness {
        state: State,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(div().flex().gap(px(500.0)).children(self.state.groups[0].1.iter().take(2).cloned()))
        }
    }
    #[test]
    fn selecting_an_icon_opens_details_and_copies_svg() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Harness { state: State::new(&Arc::new(Look::built_in()), cx) }
        });
        cx.simulate_resize(gpui::size(px(700.0), px(400.0)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("icon-details").is_none());
        let bounds = cx.debug_bounds("inspect-icon-font-style").unwrap();
        cx.simulate_click(bounds.center(), Default::default());
        cx.run_until_parked();
        assert!(cx.debug_bounds("icon-details").is_some());
        cx.update(|_, app| {
            let selected = view.read(app).state._details.read(app).selected.unwrap();
            assert_eq!(selected.0, "font-style");
            assert_eq!(selected.1, "Typography");
        });
        let bounds = cx.debug_bounds("icon-details-copy").unwrap();
        cx.simulate_click(bounds.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, app| {
            assert_eq!(
                app.read_from_clipboard().and_then(|item| item.text()),
                Some(std::str::from_utf8(icon_named("font-style").unwrap().svg_bytes()).unwrap().to_owned())
            )
        });
        cx.update(|_, app| assert!(*view.read(app).state._details.read(app).copy.read(app).data()));
        assert!(cx.debug_bounds("icon-details-close").is_none());
        cx.simulate_click(point(px(650.0), px(350.0)), Default::default());
        cx.run_until_parked();
        cx.update(|_, app| assert!(!view.read(app).state.groups[0].1[0].read(app).is_open(app)));
    }
    #[test]
    fn switching_icons_closes_previous_popup_and_click_away_dismisses() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Harness { state: State::new(&Arc::new(Look::built_in()), cx) }
        });
        cx.simulate_resize(gpui::size(px(700.0), px(400.0)));
        cx.run_until_parked();
        for selector in ["inspect-icon-font-style", "inspect-icon-font-italic"] {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        }
        cx.update(|_, app| {
            let state = &view.read(app).state;
            assert!(!state.groups[0].1[0].read(app).is_open(app));
            assert!(state.groups[0].1[1].read(app).is_open(app));
            assert_eq!(state._details.read(app).selected.unwrap().0, "font-italic");
        });
        cx.simulate_click(point(px(650.0), px(350.0)), Default::default());
        cx.run_until_parked();
        cx.update(|_, app| assert!(!view.read(app).state.groups[0].1[1].read(app).is_open(app)));
    }
    #[test]
    fn hover_opens_card_and_keeps_it_open_while_using_content() {
        use std::time::Duration;
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Harness { state: State::new(&Arc::new(Look::built_in()), cx) }
        });
        cx.simulate_resize(gpui::size(px(700.0), px(400.0)));
        cx.run_until_parked();
        let bounds = cx.debug_bounds("inspect-icon-font-style").unwrap();
        cx.simulate_mouse_move(bounds.center(), None, Default::default());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(199));
        cx.run_until_parked();
        cx.update(|_, app| assert!(!view.read(app).state.groups[0].1[0].read(app).is_open(app)));
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        cx.update(|_, app| assert!(view.read(app).state.groups[0].1[0].read(app).is_open(app)));
        let bounds = cx.debug_bounds("icon-details-copy").unwrap();
        cx.simulate_mouse_move(bounds.center(), None, Default::default());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(300));
        cx.run_until_parked();
        cx.update(|_, app| assert!(view.read(app).state.groups[0].1[0].read(app).is_open(app)));
        cx.simulate_mouse_move(point(px(650.0), px(350.0)), None, Default::default());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(201));
        cx.run_until_parked();
        cx.update(|_, app| assert!(!view.read(app).state.groups[0].1[0].read(app).is_open(app)));
    }
}
