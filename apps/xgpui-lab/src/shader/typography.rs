//! Shared native/browser acceptance demo for issue #28's static and motion milestones,
//! laid out with `LabScreen` (see `docs/mockups/typography-mockup.html`).
use gpui::{prelude::*, *};
use lucide_svg_static::Icon;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use xgpui_luma::{
    controls::{button::Button, slider::Slider, switch::Switch, textfield::TextField},
    focus::LumaFocusScopeExt,
    prelude::HasPresenter,
    theme::ThemeMode,
};
use xgpui_luma_look_radix::{self as radix, Look};
use xgpui_shader::typography::{KineticEffect, SelectableText, TextShadow};
use xgpui_shader::{Brush, BrushStop};

use crate::{
    lab_controls::{Choice, labels},
    screen::{
        CardWidth, ComparePane, Hero, LabCard, LabScreen, PanelShell, Param, ParametersPanel, TextRole, card_button,
        labeled_control, properties, text, value_field_look,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MotionMode {
    Static,
    Typewriter,
    Scramble,
    Wave,
}
impl MotionMode {
    fn label(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Typewriter => "Typewriter",
            Self::Scramble => "Scramble",
            Self::Wave => "Wave",
        }
    }
}
const MOTIONS: [MotionMode; 4] = [MotionMode::Static, MotionMode::Typewriter, MotionMode::Scramble, MotionMode::Wave];
const FILLS: [&str; 3] = ["Native", "Cool gradient", "Warm gradient"];
const SHADOWS: [&str; 3] = ["Off", "Glow", "Drop"];
const SOFTNESS: [f32; 3] = [4., 8., 12.];

/// Name, text, what to try, what the card says it tests, and its icon.
const SAMPLES: [(&str, &str, &str, &str, Icon); 4] = [
    (
        "Editorial",
        "Good typography keeps its meaning.",
        "A shaped line, an authentic source, and a selection you can trust.",
        "Shaped line",
        Icon::CaseSensitive,
    ),
    (
        "Ligatures",
        "office · affine · efficient · café",
        "Select inside fi / ffi: caret positions interpolate within the shaped ligature.",
        "Carets inside fi / ffi",
        Icon::Ligature,
    ),
    (
        "Unicode",
        "Cafe\u{301} · naïve · 👩🏽‍💻 · 🇺🇸",
        "Combining accents, skin tones, joined emoji and flags stay together when selecting.",
        "Grapheme clusters",
        Icon::Languages,
    ),
    (
        "Overflow",
        "The whole sentence remains selectable, even when the viewport is deliberately narrow.",
        "Drag beyond the edge, or use Shift + End. The caret scrolls into view.",
        "Caret scrolls into view",
        Icon::MoveHorizontal,
    ),
];
/// A card shows at most this many characters of its sample, so long samples keep cards even.
const CARD_SAMPLE_CHARS: usize = 34;

fn clip(text: &str, chars: usize) -> String {
    if text.chars().count() <= chars {
        text.to_owned()
    } else {
        format!("{}…", text.chars().take(chars - 1).collect::<String>().trim_end())
    }
}

/// Stage type size (normal, large) and its line height.
const STAGE_TYPE: [(f32, f32); 2] = [(36., 52.), (48., 64.)];

const TRY_IT: &str = "Try it: Tab to focus · Shift + arrows to select · Home / End · Cmd or Ctrl + A / C · double-click a word · triple-click the line.";
const SCOPE: &str = "Read-only text field with the full source, a text run and selection endpoints; motion does not change the accessible source or selection. Single-line LTR text: wrapping, bidirectional selection and rich-text editing are outside this milestone. Compare glyph glow and shadow in default or wide-gamut SDR; the previous color mode is restored on exit.";

pub struct TypographyDemo {
    focus_scope: FocusHandle,
    presentation_buttons: Vec<Entity<Button>>,
    window_handle: Option<AnyWindowHandle>,
    previous_color_mode: Option<WindowColorMode>,
    presentation_error: Option<String>,
    look: Arc<Look>,
    buttons: Vec<(Action, String, Entity<Button>)>,
    choices: Vec<(Action, Choice)>,
    motion_buttons: Vec<(MotionMode, Entity<Button>)>,
    sample_cards: Vec<Entity<Button>>,
    selected_sample: Arc<AtomicUsize>,
    switches: Vec<(Action, Switch)>,
    progress_field: TextField,
    progress_slider: Slider,
    seed_field: TextField,
    frame_field: TextField,
    /// Values last written to the fields and slider, so typing is never overwritten.
    synced: (u32, u64, u64),
    focus_pending: bool,
    samples: Vec<Entity<SelectableText>>,
    active: usize,
    narrow: bool,
    large: bool,
    light: bool,
    status: SharedString,
    _subscriptions: Vec<Subscription>,
    motion: MotionMode,
    playing: bool,
    progress: f32,
    phase: f32,
    reduced: bool,
    last_tick: Instant,
    seed: u64,
    fill: usize,
    shadow: usize,
    softness: usize,
}
impl TypographyDemo {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let samples: Vec<_> = SAMPLES
            .iter()
            .map(|(_, text, ..)| cx.new(|cx| SelectableText::new(*text, cx).expect("LTR demo sample")))
            .collect();
        let subscriptions = samples.iter().map(|sample| cx.observe(sample, |_, _, cx| cx.notify())).collect();
        let selected_sample = Arc::new(AtomicUsize::new(0));
        let mut demo = Self {
            focus_scope: cx.focus_handle(),
            presentation_buttons: [
                ("typography-presentation-default", "Default SDR", WindowColorMode::Default),
                ("typography-presentation-wide", "Wide-gamut SDR", WindowColorMode::WideGamutSdr),
            ]
            .into_iter()
            .map(|(id, label, mode)| {
                radix::Button::new(id)
                    .look(look.as_ref())
                    .outline()
                    .content(crate::l10n::label(label))
                    .on_click(cx, move |this: &mut Self, cx| this.set_presentation(mode, cx))
                    .spawn(cx)
            })
            .collect(),
            window_handle: None,
            previous_color_mode: None,
            presentation_error: None,
            buttons: Self::make_buttons(&look, cx),
            choices: Self::make_choices(&look, cx),
            motion_buttons: MOTIONS
                .into_iter()
                .map(|mode| {
                    let button = radix::Button::new(format!("typography-motion-{mode:?}"))
                        .look(look.as_ref())
                        .outline()
                        .content(crate::l10n::label(mode.label()))
                        .on_click(cx, move |this: &mut Self, cx| this.action(Action::Motion(mode), cx))
                        .spawn(cx);
                    (mode, button)
                })
                .collect(),
            sample_cards: Self::make_sample_cards(&look, &selected_sample, cx),
            selected_sample,
            switches: Self::make_switches(&look, cx),
            progress_field: Self::value_field("typography-progress", &look, "45", cx, |this, text, _| {
                let value = text.trim().parse::<f32>().ok().filter(|v| (0. ..=100.).contains(v))?;
                this.set_progress(value / 100.);
                Some(())
            }),
            progress_slider: radix::Slider::new("typography-progress-slider")
                .look(look.as_ref())
                .size(radix::SliderSize::One)
                .range(0. ..1.)
                .value(0.45)
                .step(0.01)
                .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                    this.set_progress(value);
                    cx.notify();
                })
                .spawn(cx),
            seed_field: Self::value_field("typography-seed", &look, "42", cx, |this, text, _| {
                this.seed = text.trim().parse::<u64>().ok()?;
                this.playing = false;
                Some(())
            }),
            frame_field: Self::value_field("typography-frame", &look, "0", cx, |this, text, _| {
                this.phase = text.trim().parse::<u64>().ok()? as f32;
                this.playing = false;
                Some(())
            }),
            synced: (45, 42, 0),
            look,
            focus_pending: false,
            samples,
            active: 0,
            narrow: false,
            large: false,
            light: false,
            status: "Select a phrase to inspect its authentic source.".into(),
            _subscriptions: subscriptions,
            motion: MotionMode::Static,
            playing: false,
            progress: 0.45,
            phase: 0.,
            reduced: false,
            last_tick: Instant::now(),
            seed: 42,
            fill: 1,
            shadow: 1,
            softness: 1,
        };
        demo.apply_style(cx);
        demo
    }
    fn set_presentation(&mut self, mode: WindowColorMode, cx: &mut Context<Self>) {
        if let Some(handle) = self.window_handle {
            let result = handle
                .update(cx, |_, window, _| {
                    let previous = window.color_status().requested();
                    window.set_color_mode(mode).map(|_| previous)
                })
                .and_then(|result| result);
            match result {
                Ok(previous) => {
                    self.previous_color_mode.get_or_insert(previous);
                    self.presentation_error = None;
                }
                Err(error) => self.presentation_error = Some(error.to_string()),
            }
            cx.notify();
        }
    }

    fn apply_style(&mut self, cx: &mut Context<Self>) {
        // Sample colors follow the look's mode: deep inks on light, bright inks on dark.
        let gradient = |light: [[f32; 4]; 2], dark: [[f32; 4]; 2]| {
            let [from, to] = if self.light { light } else { dark };
            Brush::linear(0., vec![BrushStop::new(0., from), BrushStop::new(1., to)]).unwrap()
        };
        let fill = match self.fill {
            1 => Some(gradient(
                [[0.02, 0.15, 0.6, 1.], [0.0, 0.45, 0.25, 1.]],
                [[0.25, 0.65, 1., 1.], [0.5, 1., 0.8, 1.]],
            )),
            2 => Some(gradient(
                [[0.65, 0.02, 0.25, 1.], [0.5, 0.2, 0., 1.]],
                [[1., 0.35, 0.6, 1.], [1., 0.85, 0.35, 1.]],
            )),
            _ => None,
        };
        let radius = px(SOFTNESS[self.softness]);
        let shadow = match self.shadow {
            1 => Some(TextShadow::glow(rgba(if self.light { 0x286de580 } else { 0x56dac8e0 }).into(), radius).unwrap()),
            2 => Some(TextShadow::new(rgba(0x000000b0).into(), point(px(3.), px(5.)), radius).unwrap()),
            _ => None,
        };
        for sample in &self.samples {
            sample.update(cx, |text, cx| {
                text.set_fill(fill.clone(), cx).unwrap();
                text.set_shadow(shadow.clone(), cx);
            });
        }
        cx.notify();
    }
    fn choose(&mut self, index: usize, cx: &mut Context<Self>) {
        self.active = index;
        self.selected_sample.store(index, Ordering::Relaxed);
        for card in &self.sample_cards {
            card.update(cx, |_, cx| cx.notify());
        }
        self.status = "Sample changed. Selection is retained independently for each sample.".into();
        self.focus_pending = true;
        cx.notify();
    }
    fn select_all(&mut self, cx: &mut Context<Self>) {
        self.samples[self.active].update(cx, |text, cx| text.select_all(cx));
        self.focus_pending = true;
    }
    fn copy(&mut self, cx: &mut Context<Self>) {
        let copied = self.samples[self.active].update(cx, |text, cx| text.copy_selection(cx));
        self.status = if copied {
            "Copy requested for the selected source. Paste into another app to verify."
        } else {
            "Select some text first; the clipboard was left unchanged."
        }
        .into();
        cx.notify();
    }
    /// Reveal modes always replay from the start, including repeated clicks.
    fn set_motion(&mut self, mode: MotionMode) {
        self.motion = mode;
        if matches!(mode, MotionMode::Typewriter | MotionMode::Scramble) {
            self.progress = 0.;
            self.phase = 0.;
        }
        self.playing = mode != MotionMode::Static;
        self.last_tick = Instant::now();
    }
    /// A reveal at 0% is blank, so Restart must also resume playback.
    fn restart(&mut self) {
        self.progress = 0.;
        self.phase = 0.;
        if matches!(self.motion, MotionMode::Typewriter | MotionMode::Scramble) {
            self.playing = true;
        }
        self.last_tick = Instant::now();
    }
    /// Seek a reveal effect; static and wave seek through the typewriter.
    fn set_progress(&mut self, value: f32) {
        if self.motion != MotionMode::Scramble {
            self.motion = MotionMode::Typewriter;
        }
        self.playing = false;
        self.progress = value.clamp(0., 1.);
    }
}

/// Live state for the caption line.
fn motion_line(motion: MotionMode, reduced: bool, progress: f32, seed: u64, frame: u64) -> crate::l10n::UiCopy {
    if reduced {
        return "reduced motion: complete static source, no animation requests".into();
    }
    match motion {
        MotionMode::Static => "static reference · select the unchanged source".into(),
        MotionMode::Typewriter => {
            let __l10n_0 = &(progress * 100.);
            crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_22978acebeaa(args[0].clone()),
                vec![format!("{:.0}", __l10n_0)],
            )
        }
        MotionMode::Scramble => {
            let __l10n_0 = &(progress * 100.);
            crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_b4db23dc5d3e(args[0].clone(), args[1].clone(), args[2].clone()),
                vec![format!("{:.0}", __l10n_0), format!("{}", seed), format!("{}", frame)],
            )
        }
        MotionMode::Wave => "wave · 8 px amplitude · selection follows the original baseline".into(),
    }
}

impl Render for TypographyDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.window_handle = Some(window.window_handle());
        if self.focus_pending {
            self.focus_pending = false;
            window.focus(&self.samples[self.active].focus_handle(cx), cx);
        }
        let light = self.look.mode() == ThemeMode::Light;
        if light != self.light {
            self.light = light;
            self.apply_style(cx);
        }
        let reduced = self.reduced || cx.reduce_motion();
        let now = Instant::now();
        if self.playing && !reduced && self.motion != MotionMode::Static {
            let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.1);
            if matches!(self.motion, MotionMode::Typewriter | MotionMode::Scramble) {
                if self.motion == MotionMode::Scramble {
                    self.phase += dt * 12.;
                }
                self.progress = (self.progress + dt / 4.).min(1.);
                if self.progress >= 1. {
                    self.playing = false;
                }
            } else {
                self.phase = (self.phase + dt * 2.2).rem_euclid(std::f32::consts::TAU);
            }
            if self.playing {
                window.request_animation_frame();
            }
        }
        self.last_tick = now;
        self.sync_controls(cx);
        let frame = self.phase.floor() as u64;
        let effect = if reduced {
            KineticEffect::Static
        } else {
            match self.motion {
                MotionMode::Static => KineticEffect::Static,
                MotionMode::Typewriter => KineticEffect::Typewriter { progress: self.progress },
                MotionMode::Scramble => {
                    KineticEffect::Scramble { progress: self.progress, seed: self.seed, tick: frame }
                }
                MotionMode::Wave => KineticEffect::Wave { amplitude: px(8.), phase: self.phase, cluster_phase: 0.65 },
            }
        };
        self.samples[self.active].update(cx, |text, cx| text.set_effect(effect, cx).unwrap());
        let text_state = self.samples[self.active].read(cx);
        let range = text_state.selection();
        let source = SAMPLES[self.active].1;
        let selected = text_state.selected_text().to_owned();
        let selected_count = selected.chars().count();
        let total = source.chars().count();
        let caption = crate::l10n::UiCopy::Separated(
            " · ",
            vec![
                SAMPLES[self.active].0.into(),
                motion_line(self.motion, reduced, self.progress, self.seed, frame),
                crate::l10n::UiCopy::compose(
                    |a| crate::l10n::messages::msg_lab_scalar_selection(a[0].clone(), a[1].clone()),
                    vec![
                        crate::l10n::UiCopy::numeric(selected_count.to_string()),
                        crate::l10n::UiCopy::numeric(total.to_string()),
                    ],
                ),
            ],
        );
        let screen = LabScreen::new("typography", self.look.clone())
            .crumb("xgpui-shader", "Typography")
            .title("Typography")
            .lede("Explore painted glyph effects on selectable text. Motion, fill and shadow change the pixels; selection and the clipboard always read the original source.")
            .hero(Hero::stage(self.stage(reduced)))
            .caption(caption);
        let screen = [Action::Fill, Action::Shadow, Action::Softness]
            .into_iter()
            .filter_map(|action| self.choices.iter().find(|(id, _)| *id == action).map(|(_, choice)| choice.clone()))
            .fold(
                self.motion_buttons.iter().fold(screen, |screen, (_, button)| screen.control(button.clone())),
                |screen, choice| screen.control(labeled_control(&self.look, choice.into_element())),
            );
        let screen = screen.control(self.button(Action::Reset));
        let screen = self.presentation_buttons.iter().fold(screen, |screen, button| screen.control(button.clone()));
        let color_status = window.color_status();
        let presentation_status = {
            let __l10n_0 = &(color_status.requested());
            let __l10n_1 = &(color_status.effective());
            let __l10n_2 = &(color_status.display_wide_gamut());
            let __l10n_3 = &(window.scale_factor());
            crate::l10n::UiCopy::formatted(
                |args| {
                    crate::l10n::messages::msg_ui_42dbcf47180e(
                        args[0].clone(),
                        args[1].clone(),
                        args[2].clone(),
                        args[3].clone(),
                    )
                },
                vec![
                    format!("{:?}", __l10n_0),
                    format!("{:?}", __l10n_1),
                    format!("{:?}", __l10n_2),
                    format!("{}", __l10n_3),
                ],
            )
        };
        let presentation_note = self.presentation_error.clone().unwrap_or_else(|| {
            if color_status.requested() == WindowColorMode::WideGamutSdr
                && color_status.effective() != WindowColorMode::WideGamutSdr
            {
                "Wide gamut is not effective; this run does not validate wide-gamut text rendering.".into()
            } else {
                "Compare both presentation modes; wide-gamut validation requires Effective: WideGamutSdr.".into()
            }
        });
        let screen = screen.control(div().id("typography-presentation-status").role(Role::Status).child(text(
            &self.look,
            TextRole::Caption,
            presentation_status,
        )));
        let screen = self.switches.iter().fold(screen, |screen, (_, switch)| screen.control(switch.clone()));
        let logical = self.logical_layer(source, &selected, range, selected_count, total);
        div()
            .id("typography-demo")
            .role(Role::Application)
            .aria_label(crate::l10n::resolve("Selectable typography demo", cx))
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .child(
                self.sample_cards
                    .iter()
                    .fold(screen, |screen, card| screen.card(card.clone()))
                    .panels(self.motion_panel(), logical)
                    .note(crate::l10n::UiCopy::Joined(vec![presentation_note.into(), TRY_IT.into()]))
                    .credits(SCOPE),
            )
    }
}

impl TypographyDemo {
    /// The hero: the selectable sample on a stage, with the effect and width as badges.
    fn stage(&self, reduced: bool) -> ComparePane {
        let (size, line_height) = STAGE_TYPE[usize::from(self.large)];
        let surface = div()
            .id("typography-surface")
            .debug_selector(|| "typography-surface".into())
            .w(if self.narrow {
                Length::from(px(320.))
            } else {
                Length::from(relative(1.))
            })
            .max_w_full()
            .text_size(px(size))
            .line_height(px(line_height))
            .child(self.samples[self.active].clone());
        let meta = div()
            .flex()
            .justify_between()
            .gap_4()
            .child(text(&self.look, TextRole::Label, SAMPLES[self.active].2))
            .child(text(&self.look, TextRole::Label, {
                let __l10n_0 = &(FILLS[self.fill].to_lowercase());
                let __l10n_1 = &(SHADOWS[self.shadow].to_lowercase());
                let __l10n_2 = &(SOFTNESS[self.softness]);
                crate::l10n::UiCopy::formatted(
                    |args| {
                        crate::l10n::messages::msg_ui_a89d3ea3f2b6(
                            args[0].clone(),
                            args[1].clone(),
                            args[2].clone(),
                            args[3].clone(),
                        )
                    },
                    vec![
                        format!("{:.0}", size),
                        __l10n_0.to_string(),
                        __l10n_1.to_string(),
                        format!("{:.0}", __l10n_2),
                    ],
                )
            }));
        let visual = div()
            .min_h(px(200.))
            .px_6()
            .py_4()
            .flex()
            .flex_col()
            .justify_center()
            .gap_4()
            .child(surface)
            .child(meta);
        let effect = if reduced {
            "Reduced motion".to_owned()
        } else {
            format!("{:?}", self.motion)
        };
        let width = if self.narrow { "320 px" } else { "Fluid width" };
        let badge = |label: String, positive: bool| {
            radix::Badge::new(&self.look, label).variant(if positive {
                radix::BadgeVariant::Soft
            } else {
                radix::BadgeVariant::Outline
            })
        };
        ComparePane::new("Visual layer", visual).subtitle("Painted glyphs · canonical selection").badge(
            div()
                .flex()
                .gap_2()
                .child(badge(effect, self.motion != MotionMode::Static && !reduced))
                .child(badge(width.into(), false)),
        )
    }

    /// Wide panel: what the motion is doing, and how to seek or step it.
    fn motion_panel(&self) -> ParametersPanel {
        let transport = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .child(self.button(Action::Restart))
            .child(self.button(Action::Back))
            .child(self.button(Action::Next));
        ParametersPanel::new("Motion")
            .subtitle("Effects repaint glyphs only; the source never changes")
            .action(self.button(Action::Play))
            .params([
                Param::new("Progress", self.progress_field.clone(), self.progress_slider.clone()).range("0–100%"),
                Param::new("Seed", self.seed_field.clone(), self.button(Action::Seed)).range("scramble"),
                Param::new("Frame", self.frame_field.clone(), self.button(Action::Frame)).range("scramble · wave"),
            ])
            .footer(transport)
    }

    /// Narrow panel: what selection and the clipboard read.
    fn logical_layer(
        &self,
        source: &str,
        selected: &str,
        range: std::ops::Range<usize>,
        selected_count: usize,
        total: usize,
    ) -> PanelShell {
        let selection = if selected.is_empty() {
            "—".to_owned()
        } else {
            selected.to_owned()
        };
        PanelShell::new("Logical layer")
            .subtitle("What selection and the clipboard read")
            .action(self.button(Action::Copy))
            .child(properties(
                &self.look,
                [
                    ("Source", crate::l10n::UiCopy::from(source.to_owned())),
                    ("Selection", crate::l10n::UiCopy::from(selection)),
                    ("Range", crate::l10n::UiCopy::from(format!("UTF-8 {}..{}", range.start, range.end))),
                    (
                        "Scalars",
                        crate::l10n::UiCopy::formatted(
                            |args| crate::l10n::messages::msg_ui_8750b29a9d6d(args[0].clone(), args[1].clone()),
                            vec![format!("{}", selected_count), format!("{}", total)],
                        ),
                    ),
                ],
            ))
            .child(div().flex().child(self.button(Action::SelectAll)))
            .child(div().id("copy-status").role(Role::Status).child(text(
                &self.look,
                TextRole::Caption,
                self.status.clone(),
            )))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Sample(usize),
    Motion(MotionMode),
    Play,
    Restart,
    Reset,
    Back,
    Next,
    Seed,
    Frame,
    Reduced,
    Fill,
    Shadow,
    Softness,
    SelectAll,
    Copy,
    Width,
    Size,
}
impl TypographyDemo {
    fn make_buttons(look: &Arc<Look>, cx: &mut Context<Self>) -> Vec<(Action, String, Entity<Button>)> {
        use self::Action::*;
        [Play, Restart, Reset, Back, Next, Seed, Frame, SelectAll, Copy]
            .into_iter()
            .map(|action| {
                let button = radix::Button::new(format!("typography-{action:?}"))
                    .look(look.as_ref())
                    .outline()
                    .content(crate::l10n::label("…"))
                    .on_click(cx, move |this: &mut Self, cx| this.action(action, cx))
                    .spawn(cx);
                (action, String::new(), button)
            })
            .collect()
    }
    /// The four samples as selectable cards; each card describes what the sample tests.
    fn make_sample_cards(look: &Arc<Look>, selected: &Arc<AtomicUsize>, cx: &mut Context<Self>) -> Vec<Entity<Button>> {
        SAMPLES
            .iter()
            .enumerate()
            .map(|(index, &(name, sample, _, tests, icon))| {
                let card_look = look.clone();
                let selected = selected.clone();
                card_button(
                    look,
                    format!("typography-sample-{index}"),
                    CardWidth::Flexible,
                    move || selected.load(Ordering::Relaxed) == index,
                    move |_| {
                        LabCard::new(name)
                            .icon(icon)
                            .subtitle(tests)
                            .body(text(&card_look, TextRole::Field, clip(sample, CARD_SAMPLE_CHARS)))
                            .values([
                                ("scalars", sample.chars().count().to_string()),
                                ("bytes", sample.len().to_string()),
                            ])
                            .contents(&card_look)
                    },
                    cx,
                    move |this: &mut Self, cx| this.action(Action::Sample(index), cx),
                )
            })
            .collect()
    }
    fn make_switches(look: &Arc<Look>, cx: &mut Context<Self>) -> Vec<(Action, Switch)> {
        [(Action::Width, "Narrow"), (Action::Size, "Large type"), (Action::Reduced, "Reduce motion")]
            .into_iter()
            .map(|(action, label)| {
                let switch = radix::Switch::new(format!("typography-{action:?}"))
                    .look(look.as_ref())
                    .content(crate::l10n::label(label))
                    .checked(false)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        match action {
                            Action::Width => this.narrow = on,
                            Action::Size => this.large = on,
                            _ => this.reduced = on,
                        }
                        cx.notify();
                    })
                    .spawn(cx);
                (action, switch)
            })
            .collect()
    }
    /// A mono value field; `apply` parses and applies a valid edit, `None` rejects it.
    fn value_field(
        id: &str,
        look: &Arc<Look>,
        value: &str,
        cx: &mut Context<Self>,
        apply: impl Fn(&mut Self, &str, &mut Context<Self>) -> Option<()> + 'static,
    ) -> TextField {
        crate::l10n::localize_field(
            radix::TextField::new(id.to_owned())
                .look(look.as_ref())
                .surface()
                .look_override(value_field_look(look))
                .full_width(true)
                .value(value.to_owned())
                .select_all_on_tab_focus(true)
                .on_value_changed(cx, move |this: &mut Self, text, cx| {
                    if apply(this, text, cx).is_some() {
                        cx.notify();
                    }
                })
                .spawn(cx),
            cx,
        )
    }
    fn make_choices(look: &Look, cx: &mut Context<Self>) -> Vec<(Action, Choice)> {
        [
            (Action::Fill, "Fill", labels(&FILLS), 1),
            (Action::Shadow, "Shadow", labels(&SHADOWS), 1),
            (Action::Softness, "Softness", labels(&["4 px", "8 px", "12 px"]), 1),
        ]
        .into_iter()
        .map(|(action, label, options, selected)| {
            (
                action,
                Choice::new(
                    format!("typography-{action:?}"),
                    label,
                    look,
                    options,
                    selected,
                    cx,
                    move |this: &mut Self, index, cx| {
                        match action {
                            Action::Fill => this.fill = index,
                            Action::Shadow => this.shadow = index,
                            Action::Softness => this.softness = index,
                            _ => unreachable!(),
                        }
                        this.apply_style(cx);
                        cx.notify();
                    },
                ),
            )
        })
        .collect()
    }
    fn button(&self, action: Action) -> AnyElement {
        self.buttons
            .iter()
            .find(|(key, _, _)| *key == action)
            .expect("registered typography control")
            .2
            .clone()
            .into_any_element()
    }
    /// Keep every control in step with state changed elsewhere (actions, playback, reduced motion).
    fn sync_controls(&mut self, cx: &mut Context<Self>) {
        for (action, choice) in &self.choices {
            let (options, index) = match action {
                Action::Fill => (labels(&FILLS), self.fill),
                Action::Shadow => (labels(&SHADOWS), self.shadow),
                Action::Softness => (labels(&["4 px", "8 px", "12 px"]), self.softness),
                _ => unreachable!(),
            };
            choice.sync(options, index, cx);
        }
        for (action, switch) in &self.switches {
            let on = match action {
                Action::Width => self.narrow,
                Action::Size => self.large,
                _ => self.reduced,
            };
            if *switch.read(cx).data() != on {
                switch.update(cx, |switch, cx| switch.set_data(on, cx));
            }
        }
        let values = ((self.progress * 100.).round() as u32, self.seed, self.phase.floor() as u64);
        if values != self.synced {
            if values.0 != self.synced.0 {
                let progress = self.progress;
                self.progress_field.update(cx, |field, cx| field.set_value(values.0.to_string(), cx));
                self.progress_slider.update(cx, |slider, cx| slider.set_value(progress, cx));
            }
            if values.1 != self.synced.1 {
                self.seed_field.update(cx, |field, cx| field.set_value(values.1.to_string(), cx));
            }
            if values.2 != self.synced.2 {
                self.frame_field.update(cx, |field, cx| field.set_value(values.2.to_string(), cx));
            }
            self.synced = values;
        }
        for (action, previous, button) in &mut self.buttons {
            let label: String = match *action {
                Action::Play => if self.playing { "Pause" } else { "Play" }.into(),
                Action::Restart => "Restart".into(),
                Action::Reset => "Reset".into(),
                Action::Back => "−10%".into(),
                Action::Next => "+10%".into(),
                Action::Seed => "New seed".into(),
                Action::Frame => "Next frame".into(),
                Action::SelectAll => "Select all".into(),
                Action::Copy => "Copy".into(),
                _ => continue,
            };
            if *previous != label {
                *previous = label.clone();
                button
                    .update(cx, |button, cx| button.set_presenter(std::sync::Arc::new(crate::l10n::label(label)), cx));
            }
        }
    }
    fn action(&mut self, action: Action, cx: &mut Context<Self>) {
        use self::Action::*;
        match action {
            Fill | Shadow | Softness => unreachable!("selector setting"),
            Sample(index) => self.choose(index, cx),
            Motion(mode) => self.set_motion(mode),
            Play => {
                if self.motion == MotionMode::Static {
                    self.motion = MotionMode::Wave;
                }
                if self.progress >= 1. && matches!(self.motion, MotionMode::Typewriter | MotionMode::Scramble) {
                    self.progress = 0.;
                }
                self.playing = !self.playing;
                self.last_tick = Instant::now();
            }
            Reset => self.reset(cx),
            Restart => self.restart(),
            Back | Next => self.set_progress(self.progress + if action == Back { -0.1 } else { 0.1 }),
            Seed => {
                self.seed = self.seed.wrapping_add(1);
                self.playing = false;
            }
            Frame => {
                self.phase += 1.;
                self.playing = false;
            }
            Reduced => self.reduced = !self.reduced,
            SelectAll => self.select_all(cx),
            Copy => self.copy(cx),
            Width => self.narrow = !self.narrow,
            Size => self.large = !self.large,
        }
        cx.notify();
    }
    fn reset(&mut self, cx: &mut Context<Self>) {
        self.presentation_error = None;
        self.suspend(cx);
        self.choose(0, cx);
        self.focus_pending = false;
        self.narrow = false;
        self.large = false;
        self.reduced = false;
        self.motion = MotionMode::Static;
        self.progress = 0.45;
        self.phase = 0.;
        self.seed = 42;
        self.fill = 1;
        self.shadow = 1;
        self.softness = 1;
        self.status = "Select a phrase to inspect its authentic source.".into();
        for sample in &self.samples {
            sample.update(cx, |text, cx| {
                text.clear_selection(cx);
                text.set_effect(KineticEffect::Static, cx).unwrap();
            });
        }
        // Reset invalid edits too, even if their parsed value never changed.
        self.synced = (u32::MAX, u64::MAX, u64::MAX);
        self.sync_controls(cx);
        self.apply_style(cx);
    }

    pub(super) fn suspend(&mut self, cx: &mut Context<Self>) {
        if let Some(previous) = self.previous_color_mode.take()
            && let Some(handle) = self.window_handle
        {
            self.presentation_error = handle
                .update(cx, |_, window, _| window.set_color_mode(previous))
                .and_then(|result| result)
                .err()
                .map(|error| error.to_string());
        }
        self.playing = false;
        self.focus_pending = false;
        self.last_tick = Instant::now();
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{Action, MotionMode, SAMPLES, TypographyDemo};
    use gpui::{Focusable, TestAppContext};
    use xgpui_luma::controls::button::ButtonEvent;
    use xgpui_shader::typography::KineticEffect;

    #[test]
    fn presentation_buttons_report_unsupported_wide_output_and_restore_on_exit() {
        use gpui::WindowColorMode;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let buttons = cx.update(|_, cx| page.read(cx).presentation_buttons.clone());
        buttons[1].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|window, cx| {
            let demo = page.read(cx);
            assert!(demo.presentation_error.as_ref().unwrap().contains("unsupported"));
            assert_eq!(window.color_status().requested(), WindowColorMode::Default);
            assert_eq!(demo.previous_color_mode, None);
        });
        buttons[0].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let demo = page.read(cx);
            assert!(demo.presentation_error.is_none());
            assert_eq!(demo.previous_color_mode, Some(WindowColorMode::Default));
        });
        page.update(cx, |demo, cx| demo.suspend(cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let demo = page.read(cx);
            assert_eq!(demo.previous_color_mode, None);
            assert!(demo.presentation_error.is_none());
        });
    }

    #[test]
    fn reset_button_restores_settings_selections_and_invalid_fields() {
        use xgpui_luma::controls::textfield::TextFieldEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        page.update(cx, |demo, cx| {
            demo.choose(2, cx);
            demo.presentation_error = Some("previous failed request".into());
            demo.select_all(cx);
            demo.motion = MotionMode::Wave;
            demo.playing = true;
            demo.narrow = true;
            demo.large = true;
            demo.reduced = true;
            demo.progress = 0.9;
            demo.phase = 12.;
            demo.seed = 99;
            demo.fill = 2;
            demo.shadow = 2;
            demo.softness = 2;
            demo.apply_style(cx);
        });
        cx.run_until_parked();
        let field = cx.update(|_, cx| page.read(cx).progress_field.clone());
        field.update(cx, |field, cx| {
            field.set_value("invalid", cx);
            cx.emit(TextFieldEvent::Change { value: "invalid".into() });
        });
        let reset = cx.update(|_, cx| {
            page.read(cx).buttons.iter().find(|(action, _, _)| *action == Action::Reset).unwrap().2.clone()
        });
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let demo = page.read(cx);
            assert_eq!(demo.active, 0);
            assert!(demo.presentation_error.is_none());
            assert_eq!(demo.motion, MotionMode::Static);
            assert!(!demo.playing && !demo.narrow && !demo.large && !demo.reduced);
            assert_eq!((demo.progress, demo.phase, demo.seed), (0.45, 0., 42));
            assert_eq!((demo.fill, demo.shadow, demo.softness), (1, 1, 1));
            assert_eq!(demo.progress_field.read(cx).value(), "45");
            assert_eq!(demo.seed_field.read(cx).value(), "42");
            assert_eq!(demo.frame_field.read(cx).value(), "0");
            assert!(demo.switches.iter().all(|(_, switch)| !*switch.read(cx).data()));
            assert!(demo.samples.iter().all(|sample| {
                sample.read(cx).selection().is_empty() && sample.read(cx).effect() == KineticEffect::Static
            }));
        });
    }

    #[test]
    fn tab_and_shift_tab_reach_and_leave_selectable_text() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::focus::bind_default_focus_keys(cx);
        });
        let (page, cx) = app.add_window_view(|window, cx| {
            let page = TypographyDemo::new(crate::theme::default_look(), cx);
            window.focus(&page.focus_scope, cx);
            page
        });
        cx.run_until_parked();
        let text_focus = cx.update(|_, cx| page.read(cx).samples[0].focus_handle(cx));
        let mut reached = false;
        for _ in 0..64 {
            cx.simulate_keystrokes("tab");
            if cx.update(|window, _| text_focus.is_focused(window)) {
                reached = true;
                break;
            }
        }
        assert!(reached, "Tab must reach selectable text through the Luma controls");
        cx.simulate_keystrokes("shift-tab");
        assert!(!cx.update(|window, _| text_focus.is_focused(window)));
        cx.simulate_keystrokes("tab");
        assert!(cx.update(|window, _| text_focus.is_focused(window)));
        cx.simulate_keystrokes("tab");
        assert!(!cx.update(|window, _| text_focus.is_focused(window)));
        cx.simulate_keystrokes("shift-tab");
        assert!(cx.update(|window, _| text_focus.is_focused(window)));
    }

    #[test]
    fn luma_effects_preserve_canonical_selection_and_unicode_clipboard() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let click = |action, cx: &mut gpui::VisualTestContext| {
            cx.update(|_, cx| {
                let demo = page.read(cx);
                if let Some((_, choice)) = demo.choices.iter().find(|(key, _)| *key == action) {
                    let index = match action {
                        Action::Fill => (demo.fill + 1) % 3,
                        Action::Shadow => 1,
                        Action::Softness => 2,
                        _ => unreachable!(),
                    };
                    choice.clone().select(index, cx);
                } else if let Action::Sample(index) = action {
                    let card = demo.sample_cards[index].clone();
                    card.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
                } else if let Some((_, _, button)) = demo.buttons.iter().find(|(key, _, _)| *key == action) {
                    let button = button.clone();
                    button.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
                } else {
                    // Drive motion and toggle actions directly in this selection/clipboard test.
                    page.update(cx, |demo, cx| demo.action(action, cx));
                }
            });
            cx.run_until_parked();
        };
        click(Action::Sample(2), cx);
        click(Action::SelectAll, cx);
        let text = cx.update(|_, cx| page.read(cx).samples[2].clone());
        let identity = text.entity_id();
        for action in [
            Action::Motion(MotionMode::Typewriter),
            Action::Restart,
            Action::Next,
            Action::Motion(MotionMode::Scramble),
            Action::Seed,
            Action::Frame,
            Action::Fill,
            Action::Fill,
            Action::Fill,
            Action::Shadow,
            Action::Softness,
            Action::Width,
            Action::Size,
        ] {
            click(action, cx);
            cx.update(|_, cx| {
                assert_eq!(page.read(cx).samples[2].entity_id(), identity);
                assert_eq!(text.read(cx).text(), SAMPLES[2].1);
                assert_eq!(text.read(cx).selected_text(), SAMPLES[2].1);
            });
        }
        click(Action::Copy, cx);
        assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), SAMPLES[2].1);
        click(Action::Reduced, cx);
        cx.update(|_, cx| assert_eq!(text.read(cx).effect(), KineticEffect::Static));
        click(Action::Reduced, cx);
        cx.update(|window, cx| window.focus(&text.focus_handle(cx), cx));
        cx.simulate_keystrokes("end shift-left");
        cx.update(|_, cx| assert_eq!(text.read(cx).selected_text(), "🇺🇸"));
        cx.simulate_keystrokes(if cfg!(target_os = "macos") { "cmd-c" } else { "ctrl-c" });
        assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), "🇺🇸");
        click(Action::Sample(1), cx);
        click(Action::Sample(2), cx);
        cx.update(|_, cx| assert_eq!(text.read(cx).selected_text(), "🇺🇸"));
        page.update(cx, |page, cx| {
            page.action(Action::Motion(MotionMode::Wave), cx);
            assert!(page.playing);
            page.suspend(cx);
            assert!(!page.playing);
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(matches!(text.read(cx).effect(), KineticEffect::Wave { .. }));
            assert_eq!(text.read(cx).selected_text(), "🇺🇸");
        });
    }
    #[test]
    fn screen_slots_value_fields_and_switches_stay_in_sync() {
        use xgpui_luma::controls::textfield::TextFieldEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        for slot in [
            "typography-crumb",
            "typography-hero",
            "typography-controls",
            "typography-cards",
            "typography-panels",
        ] {
            assert!(cx.debug_bounds(slot).is_some(), "{slot} is rendered");
        }
        assert!(cx.debug_bounds("typography-sample-3").is_some(), "sample cards render");
        let progress = cx.update(|_, cx| page.read(cx).progress_field.clone());
        for (text, expected) in [("80", 0.8), ("200", 0.8), ("x", 0.8)] {
            progress.update(cx, |field, cx| {
                field.set_value(text, cx);
                cx.emit(TextFieldEvent::Change { value: text.into() });
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let demo = page.read(cx);
                assert!((demo.progress - expected).abs() < 1e-6);
                assert_eq!(demo.motion, MotionMode::Typewriter, "seeking reveals through the typewriter");
            });
        }
        page.update(cx, |demo, cx| {
            demo.action(Action::Next, cx);
            demo.action(Action::Reduced, cx);
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let demo = page.read(cx);
            assert_eq!(demo.progress_field.read(cx).value(), "90");
            let reduced = &demo.switches.iter().find(|(action, _)| *action == Action::Reduced).unwrap().1;
            assert!(*reduced.read(cx).data(), "switch follows state changed elsewhere");
            assert_eq!(demo.samples[0].read(cx).effect(), KineticEffect::Static, "reduced motion shows the source");
        });
    }

    #[test]
    fn motion_buttons_and_restart_replay_reveals_instead_of_leaving_blank_text() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let restart = cx.update(|_, cx| {
            page.read(cx).buttons.iter().find(|(action, _, _)| *action == Action::Restart).unwrap().2.clone()
        });
        for mode in [MotionMode::Typewriter, MotionMode::Scramble] {
            let button =
                cx.update(|_, cx| page.read(cx).motion_buttons.iter().find(|(key, _)| *key == mode).unwrap().1.clone());
            // Exercise the actual button callbacks twice: repeating a mode replays it.
            for _ in 0..2 {
                page.update(cx, |demo, _| {
                    demo.progress = 1.;
                    demo.phase = 48.;
                    demo.playing = false;
                });
                button.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
                cx.update(|_, cx| {
                    let demo = page.read(cx);
                    assert_eq!(demo.motion, mode);
                    assert!(
                        demo.progress < 0.05 && demo.phase < 1.3,
                        "reveal starts near zero and may already advance a frame"
                    );
                    assert!(demo.playing);
                });
            }
            page.update(cx, |demo, cx| {
                demo.choose(1, cx);
                demo.choose(0, cx);
                assert_eq!(demo.motion, mode);
                demo.progress = 0.6;
                demo.playing = false;
            });
            restart.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            cx.update(|_, cx| {
                let demo = page.read(cx);
                assert!(demo.progress < 0.05 && demo.phase < 1.3, "restarted reveal may already advance a frame");
                assert!(demo.playing, "Restart resumes a paused reveal");
            });
        }
        page.update(cx, |demo, cx| {
            demo.action(Action::Motion(MotionMode::Wave), cx);
            assert!(demo.playing);
            demo.playing = false;
            demo.action(Action::Restart, cx);
            assert!(!demo.playing, "Restart preserves a paused Wave");
            demo.action(Action::Motion(MotionMode::Static), cx);
            assert!(!demo.playing);
            demo.action(Action::Restart, cx);
            assert!(!demo.playing, "Restart does not animate Static");
        });
        cx.run_until_parked();
    }

    #[test]
    fn pointer_drag_word_and_line_selection_use_the_original_text() {
        use gpui::{Modifiers, MouseButton, MouseDownEvent, VisualTestContext, div, point, prelude::*, px, size};
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let window =
            app.open_window(size(px(1400.), px(1800.)), |_, cx| TypographyDemo::new(crate::theme::default_look(), cx));
        let page = window.root(&mut app).unwrap();
        let mut cx = VisualTestContext::from_window(window.into(), &app);
        let draw = |cx: &mut VisualTestContext| {
            cx.draw(point(px(0.), px(0.)), size(px(1400.), px(1800.)), |_, _| div().size_full().child(page.clone()));
        };
        draw(&mut cx);
        let bounds = cx.debug_bounds("typography-surface").expect("mounted selectable surface");
        let start = point(bounds.left() + px(30.), bounds.center().y);
        cx.simulate_event(MouseDownEvent {
            position: start,
            button: MouseButton::Left,
            click_count: 2,
            modifiers: Modifiers::default(),
            first_mouse: false,
        });
        cx.simulate_mouse_up(start, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.samples[0].read(cx).selected_text(), "Good");
        });
        cx.simulate_event(MouseDownEvent {
            position: start,
            button: MouseButton::Left,
            click_count: 3,
            modifiers: Modifiers::default(),
            first_mouse: false,
        });
        cx.simulate_mouse_up(start, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| assert_eq!(page.read(cx).samples[0].read(cx).selected_text(), SAMPLES[0].1));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        let end = start + point(px(130.), px(0.));
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            let page = page.read(cx);
            let text = page.samples[0].read(cx);
            assert!(!text.selection().is_empty());
            assert!(text.selection().len() < SAMPLES[0].1.len());
            assert_eq!(text.selected_text(), &SAMPLES[0].1[text.selection()]);
        });
    }
}
