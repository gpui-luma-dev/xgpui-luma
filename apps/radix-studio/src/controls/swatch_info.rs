//! Palette-step details and equivalent alpha colors over the selected background.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use gpui::{Animation, AnimationExt};
use super::copy_icon::{CopyIcon, success_displacement};

#[derive(Clone, Default)]
pub(crate) struct CopyFeedback {
    generation: u64,
    copied: bool,
}

use gpui::{AnyElement, Context, Entity, FocusHandle, Hsla, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::prelude::{HasPresenter, TooltipEntityExt};
use xgpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode, OverlayWindowPosition};
use xgpui_luma_look_radix as radix;
use xgpui_luma::color::{ColorValue, DisplayP3Color, GamutMapping};
use xgpui_luma::controls::button::Button;
use xgpui_luma_look_radix::{Look, LookControlExt, ScaleFamily, SemanticRole};
use palette::convert::FromColorUnclamped;

use crate::color_hex::format_hex;

#[derive(Clone, Debug)]
pub(crate) struct SwatchSelection {
    pub title: String,
    pub step: u8,
    pub color: Hsla,
    pub solid: String,
    pub alpha: String,
    pub hsl: String,
    pub hsla: String,
    pub p3: String,
    pub p3_alpha: String,
}

impl SwatchSelection {
    pub fn new(look: &Look, family: ScaleFamily, step: u8, background: ColorValue) -> anyhow::Result<Self> {
        let name = look.palette_label(family);
        let name: String = if name == "custom" {
            (if family == ScaleFamily::Gray { "Gray" } else { "Accent" }).into()
        } else {
            let mut chars = name.chars();
            chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
        };
        Self::from_source(&name, step, look.resolve_step_source(family, step).value, background)
    }

    pub fn from_alpha(name: &str, step: u8, source: ColorValue, background: ColorValue) -> anyhow::Result<Self> {
        let rgb = source.to_srgba_unclamped()?;
        let bg = background.to_srgba_unclamped()?;
        let solid = ColorValue::srgb(
            rgb.red * rgb.alpha + bg.red * (1.0 - rgb.alpha),
            rgb.green * rgb.alpha + bg.green * (1.0 - rgb.alpha),
            rgb.blue * rgb.alpha + bg.blue * (1.0 - rgb.alpha),
            1.0,
        );
        let mut selection = Self::from_source(name, step, solid, background)?;
        // Keep the original translucent color for the preview. The solid value
        // above is only the equivalent color shown in the details text.
        selection.color = xgpui_luma::color::gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde)?;
        selection.hsla = format_hsl(selection.color, true);
        selection.alpha =
            format!("#{:02X}{:02X}{:02X}{:02X}", byte(rgb.red), byte(rgb.green), byte(rgb.blue), byte(rgb.alpha));
        let p3 = DisplayP3Color::from_color_unclamped(rgb);
        selection.p3_alpha =
            format!("color(display-p3 {:.4} {:.4} {:.4} / {:.4})", p3.red, p3.green, p3.blue, p3.alpha);
        Ok(selection)
    }

    pub fn from_source(name: &str, step: u8, source: ColorValue, background: ColorValue) -> anyhow::Result<Self> {
        let color = xgpui_luma::color::gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde)?;
        let rgb = source.to_srgba_fallback(GamutMapping::CssLocalMinde)?;
        let bg = background.to_srgba_fallback(GamutMapping::CssLocalMinde)?;
        let (alpha_rgb, alpha) = equivalent_alpha([rgb.red, rgb.green, rgb.blue], [bg.red, bg.green, bg.blue]);
        let p3 = DisplayP3Color::from_color_unclamped(source.to_srgba_unclamped()?);
        let p3_bg = DisplayP3Color::from_color_unclamped(background.to_srgba_unclamped()?);
        let (p3_rgb, p3_alpha) = equivalent_alpha([p3.red, p3.green, p3.blue], [p3_bg.red, p3_bg.green, p3_bg.blue]);
        let oklch = source.to_oklcha_unclamped()?;
        let alpha_color = xgpui_luma::color::gpui_bridge::to_hsla(
            ColorValue::srgb(alpha_rgb[0], alpha_rgb[1], alpha_rgb[2], alpha),
            GamutMapping::CssLocalMinde,
        )?;
        Ok(Self {
            title: format!("{name} {step}"),
            step,
            color,
            solid: format!("#{}", format_hex(color)),
            hsl: format_hsl(color, false),
            hsla: format_hsl(alpha_color, true),
            alpha: format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                byte(alpha_rgb[0]),
                byte(alpha_rgb[1]),
                byte(alpha_rgb[2]),
                byte(alpha)
            ),
            p3: format!("oklch({:.1}% {:.4} {:.1})", oklch.l * 100.0, oklch.chroma, oklch.hue.into_positive_degrees()),
            p3_alpha: format!("color(display-p3 {:.4} {:.4} {:.4} / {:.4})", p3_rgb[0], p3_rgb[1], p3_rgb[2], p3_alpha),
        })
    }
}

/// Local control owning the reusable modal, selected color and close action.
pub(crate) struct ColorDetails {
    selection: Arc<Mutex<Option<SwatchSelection>>>,
    overlay: OverlayWindow,
    close: Entity<Button<Hsla>>,
    copies: Vec<Entity<Button<CopyFeedback>>>,
}

impl ColorDetails {
    pub fn new(look: &Arc<Look>, cx: &mut Context<Self>) -> Self {
        let selection = Arc::new(Mutex::new(None::<SwatchSelection>));
        let close = super::icon_button::ghost_no_hover("swatch-info-close", look, gpui::white())
            .content(|model, _| {
                crate::assets::icon_named("cross-2")
                    .map(|icon| crate::assets::react_icon(icon, model.data, 20.0))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .with_template_modifier(|root, _| {
                root.aria_label("Close color details").debug_selector(|| "swatch-info-close".into())
            })
            .on_click(cx, |this, cx| {
                this.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            })
            .spawn(cx)
            .help("Close color details", cx);

        let copies: Vec<Entity<Button<CopyFeedback>>> = ["solid", "alpha", "hsl", "hsla", "p3", "p3-alpha"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let button_look = Arc::clone(look);
                let id = format!("swatch-info-copy-{name}");
                let button = super::icon_button::ghost_no_hover(id.clone(), look, CopyFeedback::default())
                    .compact()
                    .content(move |model, _| {
                        let color = button_look.resolve_role(SemanticRole::MutedForeground).hsla();
                        let icon = CopyIcon { color, displacement: 0.0 };
                        let icon = if model.data.copied {
                            icon.with_animation(
                                ("copy-success", model.data.generation),
                                Animation::new(Duration::from_millis(1400)),
                                |mut icon, progress| {
                                    icon.displacement = success_displacement(progress);
                                    icon
                                },
                            )
                            .into_any_element()
                        } else {
                            icon.into_any_element()
                        };
                        icon
                    })
                    .with_template_modifier(move |root, model| {
                        root.aria_label(if model.data.copied {
                            "Copied"
                        } else {
                            "Copy to clipboard"
                        })
                        .debug_selector({
                            let id = id.clone();
                            move || id.clone()
                        })
                    })
                    .spawn(cx)
                    .help("Copy to clipboard", cx);
                {
                    let callback_button = (&button).downgrade();
                    button.clone().on_click(cx, move |this, cx| {
                        let Some(button) = callback_button.upgrade() else {
                            return;
                        };

                        if let Some(selection) = this.selection.lock().ok().and_then(|info| info.clone()) {
                            let values = [
                                &selection.solid,
                                &selection.alpha,
                                &selection.hsl,
                                &selection.hsla,
                                &selection.p3,
                                &selection.p3_alpha,
                            ];
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(values[index].clone()));
                            button.update(cx, |button, cx| {
                                let mut feedback = button.data().clone();
                                feedback.generation = feedback.generation.wrapping_add(1);
                                feedback.copied = true;
                                let generation = feedback.generation;
                                button.set_data(feedback, cx);
                                cx.spawn(async move |button, cx| {
                                    cx.background_executor().timer(Duration::from_millis(1400)).await;
                                    let _ = button.update(cx, |button, cx| {
                                        if button.data().generation == generation {
                                            let mut feedback = button.data().clone();
                                            feedback.copied = false;
                                            button.set_data(feedback, cx);
                                        }
                                    });
                                })
                                .detach();
                            });
                        }
                    })
                };
                button
            })
            .collect();
        let popup_copies = copies.clone();
        let popup_look = Arc::clone(look);
        let shell_look = Arc::clone(look);
        let info = Arc::clone(&selection);
        let popup_close = close.clone();
        let overlay = look
            .overlay_window("swatch-info")
            .mode(OverlayWindowMode::Modal)
            .position(OverlayWindowPosition::Center)
            .width(600.0)
            .with_template_modifier(move |root, _| {
                root.bg(shell_look.resolve_role(SemanticRole::Background).hsla()).cursor_default().occlude()
            })
            .content(move |_, window, _| {
                let selection = info.lock().ok().and_then(|guard| guard.clone());
                div()
                    .id("swatch-info-content")
                    .max_h((window.viewport_size().height - px(64.0)).max(px(100.0)))
                    .overflow_y_scroll()
                    .when_some(selection, |root, selection| {
                        root.child(render(&selection, &popup_look, popup_close.clone(), &popup_copies))
                    })
                    .into_any_element()
            })
            .theme_child(close.clone())
            .theme_children(copies.clone())
            .spawn(cx);
        Self { selection, overlay, close, copies }
    }

    pub fn open(&mut self, selection: SwatchSelection, focus: Option<FocusHandle>, cx: &mut Context<Self>) {
        for copy in &self.copies {
            copy.update(cx, |button, cx| {
                let generation = button.data().generation.wrapping_add(1);
                button.set_data(CopyFeedback { generation, copied: false }, cx);
            });
        }
        let foreground = close_foreground(selection.color);
        self.close.update(cx, |close, cx| close.set_data(foreground, cx));
        if let Ok(mut info) = self.selection.lock() {
            *info = Some(selection);
        }
        self.overlay.update(cx, |overlay, cx| overlay.open_from(focus, cx));
        cx.notify();
    }

    #[cfg(all(test, feature = "test-support"))]
    pub fn is_copied(&self, index: usize, cx: &gpui::App) -> bool {
        self.copies[index].read(cx).data().copied
    }
    #[cfg(all(test, feature = "test-support"))]
    pub fn selected(&self) -> Option<SwatchSelection> {
        self.selection.lock().ok().and_then(|guard| guard.clone())
    }
    #[cfg(all(test, feature = "test-support"))]
    pub fn is_open(&self, cx: &gpui::App) -> bool {
        self.overlay.read(cx).is_open()
    }
}

impl Render for ColorDetails {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.overlay.clone()
    }
}

/// SDK-backed palette swatch with a constant face and border-only hover.
pub(crate) fn swatch<M: 'static>(
    id: String,
    look: &Look,
    width: Option<f32>,
    height: f32,
    color: impl Fn() -> Hsla + Send + Sync + 'static,
    cx: &mut Context<M>,
) -> Entity<Button> {
    let geometry = radix::button_look_for(
        look,
        radix::ButtonVariant::GhostQuiet,
        radix::Paint::accent(),
        radix::ButtonSize::Two,
        radix::Radius::None,
    );
    let swatch_look = look.clone();
    let color = Arc::new(color);
    let content_color = Arc::clone(&color);
    let debug_id = id.clone();
    let preview_id = format!("{id}-preview");
    Button::new(id.clone())
        .template(radix::button_template(look, radix::ButtonVariant::GhostQuiet, radix::Paint::accent()))
        .with_look(move |model| {
            let mut look = geometry(model);
            look.background = gpui::transparent_black();
            look.border = None;
            look.radius = 0.0;
            look.height = height;
            look.padding_x = 0.0;
            look.padding_y = 0.0;
            look
        })
        .content(move |_, _| {
            let color = content_color();
            div()
                .debug_selector({
                    let id = preview_id.clone();
                    move || id.clone()
                })
                .when(color.a < 1.0, |root| {
                    // Button presenters are measured intrinsically. Give the
                    // percentage-width canvas a concrete catalog-cell width.
                    root.when_some(width, |root, width| root.w(px((width - 2.0).max(0.0)))).child(
                        xgpui_luma_color::ColorSwatch::new(xgpui_luma::color::gpui_bridge::from_hsla(color))
                            .height(px(height - 2.0))
                            .rounded(px(0.0))
                            .bordered(false)
                            .checkerboard(true),
                    )
                })
        })
        .with_template_modifier(move |root, _| {
            let color = color();
            root.w_full()
                .when_some(width, |root, width| root.w(px(width)).flex_none())
                .aria_label(id.clone())
                .debug_selector({
                    let id = debug_id.clone();
                    move || id.clone()
                })
                .h(px(height))
                .rounded(px(0.0))
                .bg(if color.a < 1.0 {
                    gpui::transparent_black()
                } else {
                    color
                })
                .border_1()
                .border_color(gpui::transparent_black())
                .hover(|style| style.border_color(swatch_look.resolve_role(SemanticRole::Foreground).hsla()))
        })
        .spawn(cx)
}

// Choose the greater black/white contrast from linear sRGB relative luminance.
fn close_foreground(color: Hsla) -> Hsla {
    let rgb = color.to_rgb();
    let linear = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b);
    if (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) {
        gpui::black()
    } else {
        gpui::white()
    }
}

fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn format_hsl(color: Hsla, include_alpha: bool) -> String {
    let hue = color.h * 360.0;
    let saturation = color.s * 100.0;
    let lightness = color.l * 100.0;
    if include_alpha {
        format!("hsla({hue:.1}, {saturation:.1}%, {lightness:.1}%, {:.4})", color.a)
    } else {
        format!("hsl({hue:.1}, {saturation:.1}%, {lightness:.1}%)")
    }
}

// Smallest opacity whose foreground can reproduce the solid color over this background.
fn equivalent_alpha(color: [f32; 3], background: [f32; 3]) -> ([f32; 3], f32) {
    let alpha = color
        .iter()
        .zip(background)
        .fold(0.0_f32, |alpha, (&color, bg)| {
            let required = if color > bg {
                (color - bg) / (1.0 - bg)
            } else if color < bg {
                (bg - color) / bg
            } else {
                0.0
            };
            alpha.max(required)
        })
        .clamp(0.0, 1.0);
    let foreground = if alpha > 0.0 {
        std::array::from_fn(|i| ((color[i] - background[i]) / alpha + background[i]).clamp(0.0, 1.0))
    } else {
        color
    };
    (foreground, alpha)
}

fn usage(step: u8) -> (&'static str, &'static str) {
    match step {
        1..=2 => ("Backgrounds", "Steps 11 and 12 text"),
        3..=5 => ("Interactive components", "Step 12 labels"),
        6..=8 => ("Borders and separators", "Steps 1–5 backgrounds"),
        9..=10 => ("Solid colors", "Contrasting labels"),
        _ => ("Accessible text", "Steps 1–5 backgrounds"),
    }
}

pub(crate) fn render(
    selection: &SwatchSelection,
    look: &Look,
    close: Entity<Button<Hsla>>,
    copies: &[Entity<Button<CopyFeedback>>],
) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();
    let (usage, pairs) = usage(selection.step);
    let row = |label: &'static str, value: String| {
        div()
            .flex()
            .items_center()
            .gap(px(16.0))
            .child(div().w(px(104.0)).flex_none().text_color(muted).child(label))
            .child(div().min_w_0().flex_1().child(value))
    };
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .text_color(fg)
        .child(
            div()
                .relative()
                .w_full()
                .h(px(240.0))
                .debug_selector(|| "swatch-info-preview".into())
                .child(
                    xgpui_luma_color::ColorSwatch::new(xgpui_luma::color::gpui_bridge::from_hsla(selection.color))
                        .height(px(240.0))
                        .rounded(px(0.0))
                        .bordered(false)
                        .checkerboard(true),
                )
                .child(div().absolute().top(px(4.0)).right(px(4.0)).child(close)),
        )
        .child(div().text_lg().font_weight(gpui::FontWeight::BOLD).child(selection.title.clone()))
        .child(row("Usage", usage.into()))
        .child(row("Pairs with", pairs.into()))
        .child(div().w_full().h(px(1.0)).bg(border))
        .children(
            [
                ("Solid color", &selection.solid),
                ("Alpha color", &selection.alpha),
                ("HSL", &selection.hsl),
                ("HSLA", &selection.hsla),
                ("P3 color", &selection.p3),
                ("P3 alpha", &selection.p3_alpha),
            ]
            .into_iter()
            .zip(copies)
            .map(|((label, value), copy)| {
                row(label, value.clone())
                    .when(label.starts_with("P3"), |row| row.line_through())
                    .child(copy.clone())
            }),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn close_icon_contrasts_with_selected_color() {
        for color in [gpui::black(), gpui::rgb(0x203a6e).into(), gpui::rgb(0x0000ff).into()] {
            assert_eq!(close_foreground(color), gpui::white());
        }
        for color in [gpui::white(), gpui::rgb(0xffff00).into(), gpui::rgb(0x00ff00).into()] {
            assert_eq!(close_foreground(color), gpui::black());
        }
    }

    #[test]
    fn alpha_details_retain_original_transparency() {
        let source = ColorValue::parse_css("#ffffff80").unwrap();
        let selection = SwatchSelection::from_alpha("White", 5, source, ColorValue::srgb(0.0, 0.0, 0.0, 1.0)).unwrap();
        assert_eq!(selection.alpha, "#FFFFFF80");
        assert_eq!(selection.solid, "#808080");
        assert_eq!(selection.hsl, "hsl(0.0, 0.0%, 50.2%)");
        assert_eq!(selection.hsla, "hsla(0.0, 0.0%, 100.0%, 0.5020)");
        assert_eq!(
            selection.color,
            xgpui_luma::color::gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap()
        );
        assert!(selection.p3_alpha.ends_with("/ 0.5020)"));
    }

    #[test]
    fn alpha_equivalent_recomposes_on_light_dark_and_colored_backgrounds() {
        for background in [[0.0; 3], [1.0; 3], [0.1, 0.2, 0.3]] {
            for color in [[0.12, 0.23, 0.43], background, [0.9, 0.4, 0.7]] {
                let (foreground, alpha) = equivalent_alpha(color, background);
                for i in 0..3 {
                    assert!((foreground[i] * alpha + background[i] * (1.0 - alpha) - color[i]).abs() < 1e-6);
                }
            }
        }
    }
}
