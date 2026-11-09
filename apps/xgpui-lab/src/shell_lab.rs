//! Embedded port of xgpui's shell-lab: shared renderer, desktop readings and overrides.
use std::{collections::VecDeque, sync::Arc};
use gpui::{prelude::*, *};
use xgpui_shell::{ClientFramePreview, ShellOptions, ShellTypography, TitlebarAction};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_radix::Look;
use crate::{
    lab_controls::{Choice, labels},
    lab_ui,
    theme::ColorVizLookExt,
};

pub(crate) const PAGES: &[(&str, &str)] = &[("Shell Lab", "Shell Lab"), ("Desktop Preferences", "Desktop Preferences")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum Setting {
    State,
    Appearance,
    Active,
    Focus,
    Hover,
    Layout,
    Contrast,
    Accent,
    Scale,
    Typography,
    Motion,
    DoubleClick,
    MiddleClick,
    RightClick,
}
const SETTINGS: [Setting; 14] = [
    Setting::State,
    Setting::Appearance,
    Setting::Active,
    Setting::Focus,
    Setting::Hover,
    Setting::Layout,
    Setting::Contrast,
    Setting::Accent,
    Setting::Scale,
    Setting::Typography,
    Setting::Motion,
    Setting::DoubleClick,
    Setting::MiddleClick,
    Setting::RightClick,
];
impl Setting {
    fn spec(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::State => ("Window state", &["Floating", "Maximized", "Tiled", "Fullscreen"]),
            Self::Appearance => ("Appearance", &["Native", "Light", "Dark"]),
            Self::Active => ("Activation", &["Inactive", "Active"]),
            Self::Focus => ("Close focus", &["Off", "On"]),
            Self::Hover => ("Close hover", &["Off", "On"]),
            Self::Layout => ("Buttons", &["Platform / fallback", "Close only", "On left"]),
            Self::Contrast => ("Contrast", &["System", "Normal", "High contrast"]),
            Self::Accent => ("Focus accent", &["System", "Blue", "Orange", "Black"]),
            Self::Scale => ("Logical scale", &["System", "100%", "125%", "150%", "200%"]),
            Self::Typography => ("Title typography", &["System", "System UI 14px", "Large bold 24px", "Georgia 18px"]),
            Self::Motion => ("Reduced motion", &["System", "Normal", "Reduce"]),
            Self::DoubleClick => ("Double click", &["System", "Maximize / restore", "Minimize", "Menu", "None"]),
            Self::MiddleClick => ("Middle click", &["System", "Maximize / restore", "Minimize", "Menu", "None"]),
            Self::RightClick => ("Right click", &["System", "Maximize / restore", "Minimize", "Menu", "None"]),
        }
    }
}

pub(crate) struct ShellPage {
    look: Arc<Look>,
    active_page: SharedString,
    values: [usize; 14],
    choices: Vec<(Setting, Choice)>,
    reset: Entity<xgpui_luma::controls::button::Button>,
    events: VecDeque<String>,
    subscriptions: Vec<Subscription>,
    observing_window: bool,
}
impl ShellPage {
    pub(crate) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let values = Self::defaults();
        let choices = SETTINGS
            .into_iter()
            .map(|setting| {
                let (label, options) = setting.spec();
                let choice = Choice::new(
                    format!("shell-{setting:?}"),
                    label,
                    &look,
                    labels(options),
                    values[setting as usize],
                    cx,
                    move |page: &mut Self, index, cx| {
                        page.values[setting as usize] = index;
                        page.sync_preview_controls(cx);
                        page.record(format!("{}: {} (preview only)", label, options[index]));
                        cx.notify();
                    },
                );
                (setting, choice)
            })
            .collect();
        let reset = xgpui_luma_look_radix::Button::new("shell-reset")
            .look(&look)
            .content(|_, _| "Reset preview to desktop defaults")
            .on_click(cx, |page: &mut Self, cx| {
                page.values = Self::defaults();
                for (setting, choice) in &page.choices {
                    choice.sync(labels(setting.spec().1), page.values[*setting as usize], cx);
                }
                page.sync_preview_controls(cx);
                page.record("Reset preview to desktop defaults");
                cx.notify();
            })
            .spawn(cx);
        let weak = cx.entity().downgrade();
        let desktop = cx.observe_desktop_preferences(move |app| {
            let _ = weak.update(app, |page, cx| {
                page.record("Detected desktop preferences changed");
                cx.notify();
            });
        });
        Self {
            look,
            active_page: PAGES[0].0.into(),
            values,
            choices,
            reset,
            events: VecDeque::from(["Following desktop preferences; overrides affect the preview only".into()]),
            subscriptions: vec![desktop],
            observing_window: false,
        }
    }
    fn sync_preview_controls(&self, cx: &mut App) {
        let visible_chrome = self.values[Setting::State as usize] != 3;
        for (setting, choice) in &self.choices {
            if matches!(setting, Setting::Focus | Setting::Hover) {
                choice.selector.update(cx, |selector, cx| selector.set_enabled(visible_chrome, cx));
            }
        }
    }
    fn defaults() -> [usize; 14] {
        let mut values = [0; 14];
        values[Setting::Active as usize] = 1;
        values
    }
    pub(crate) fn activate(&mut self, page: SharedString, cx: &mut Context<Self>) {
        self.active_page = page;
        cx.notify();
    }
    fn record(&mut self, message: impl Into<String>) {
        let message = message.into();
        if self.events.back() != Some(&message) {
            self.events.push_back(message);
            while self.events.len() > 12 {
                self.events.pop_front();
            }
        }
    }
    fn options(&self) -> ShellOptions {
        let value = |setting| self.values[setting as usize];
        let boolean = |setting| match value(setting) {
            1 => Some(false),
            2 => Some(true),
            _ => None,
        };
        let action = |setting| match value(setting) {
            1 => Some(TitlebarAction::ToggleMaximize),
            2 => Some(TitlebarAction::Minimize),
            3 => Some(TitlebarAction::Menu),
            4 => Some(TitlebarAction::None),
            _ => None,
        };
        ShellOptions {
            high_contrast: boolean(Setting::Contrast),
            accent: match value(Setting::Accent) {
                1 => Some(0x3584e4),
                2 => Some(0xf97316),
                3 => Some(0),
                _ => None,
            },
            text_scale: match value(Setting::Scale) {
                1 => Some(1.),
                2 => Some(1.25),
                3 => Some(1.5),
                4 => Some(2.),
                _ => None,
            },
            typography: match value(Setting::Typography) {
                1 => Some(ShellTypography::default()),
                2 => Some(ShellTypography { font: font(".SystemUIFont").bold(), size: 24. }),
                3 => Some(ShellTypography { font: font("Georgia"), size: 18. }),
                _ => None,
            },
            reduced_motion: boolean(Setting::Motion),
            double_click_action: action(Setting::DoubleClick),
            middle_click_action: action(Setting::MiddleClick),
            right_click_action: action(Setting::RightClick),
        }
    }
    fn appearance(&self, window: &Window) -> WindowAppearance {
        match self.values[Setting::Appearance as usize] {
            1 => WindowAppearance::Light,
            2 => WindowAppearance::Dark,
            _ => window.appearance(),
        }
    }
    fn preview(&self, window: &Window) -> ClientFramePreview {
        let mut preview = ClientFramePreview::new(
            "XGPUI Shell — preview",
            self.options(),
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(self.look.chrome().content_background)
                .text_color(self.look.chrome().body_text)
                .child("Application content keeps its Radix theme"),
        );
        preview.appearance = self.appearance(window);
        preview.active = self.values[Setting::Active as usize] == 1;
        preview.maximized = self.values[Setting::State as usize] == 1;
        preview.fullscreen = self.values[Setting::State as usize] == 3;
        preview.tiling = if self.values[Setting::State as usize] == 2 {
            Tiling { top: true, left: true, bottom: true, right: false }
        } else {
            Tiling::default()
        };
        preview.show_focus = self.values[Setting::Focus as usize] == 1;
        preview.show_hover = self.values[Setting::Hover as usize] == 1;
        preview.button_layout = match self.values[Setting::Layout as usize] {
            1 => Some(WindowButtonLayout { left: [None; 3], right: [Some(WindowButton::Close), None, None] }),
            2 => Some(WindowButtonLayout {
                left: [Some(WindowButton::Close), Some(WindowButton::Minimize), Some(WindowButton::Maximize)],
                right: [None; 3],
            }),
            _ => None,
        };
        preview
    }
    fn rows(&self, preview: bool) -> Div {
        lab_ui::control_row().children(
            self.choices
                .iter()
                .filter(|(setting, _)| ((*setting as usize) < Setting::Contrast as usize) == preview)
                .map(|(_, choice)| choice.clone()),
        )
    }
    fn observe_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.observing_window {
            return;
        }
        self.observing_window = true;
        self.subscriptions.push(cx.observe_window_bounds(window, |page, _, cx| {
            page.record("Host window bounds/state changed");
            cx.notify();
        }));
        self.subscriptions.push(cx.observe_window_activation(window, |page, _, cx| {
            page.record("Host window activation changed");
            cx.notify();
        }));
        let weak = cx.entity().downgrade();
        self.subscriptions.push(window.observe_window_appearance(move |_, app| {
            let _ = weak.update(app, |_, cx| cx.notify());
        }));
        let weak = cx.entity().downgrade();
        self.subscriptions.push(window.observe_button_layout_changed(move |_, app| {
            let _ = weak.update(app, |_, cx| cx.notify());
        }));
        let weak = cx.entity().downgrade();
        self.subscriptions.push(window.observe_window_display(move |_, app| {
            let _ = weak.update(app, |_, cx| cx.notify());
        }));
    }
    fn inspector(&self, window: &Window, cx: &App) -> Div {
        let resolved = self.options().resolved_preferences(&cx.desktop_preferences());
        let look = resolved.resolved_look(self.appearance(window), self.values[Setting::Active as usize] == 1);
        let swatches = lab_ui::control_row().children(
            [
                ("Header", look.colors.header),
                ("Text", look.colors.text),
                ("Border", look.colors.border),
                ("Hover", look.colors.hover),
                ("Focus", look.focus),
            ]
            .into_iter()
            .map(|(label, color)| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size(px(18.)).bg(rgb(color)).border_1().border_color(self.look.chrome().border))
                    .child(format!("{label} #{color:06X}"))
            }),
        );
        lab_ui::panel(&self.look)
            .child(heading("Resolved client-frame look"))
            .child(swatches)
            .child(property("Title font", format!("{} · {:.1}px · weight {}", look.typography.font.family, look.typography.size, look.typography.font.weight)))
            .child(property("Geometry", format!("Header {:.1}px · button {:.1}px · logical scale {:.0}%", look.titlebar_height, look.button_size, resolved.text_scale * 100.)))
            .child(property("Titlebar actions", format!("Double {:?} · middle {:?} · right {:?}", resolved.double_click_action, resolved.middle_click_action, resolved.right_click_action)))
            .child(property("Reduced motion", format!("{} (policy; frame has no animation)", look.reduced_motion)))
            .child(lab_ui::caption(&self.look, "Requested font may be substituted. Logical text scale is separate from device DPI. Native controls and compositor decorations remain platform-owned."))
    }
    fn readings(&self, window: &Window, cx: &App) -> Div {
        let desktop = cx.desktop_preferences();
        let linux = matches!(cx.platform_info().os, OperatingSystem::Linux | OperatingSystem::FreeBsd);
        lab_ui::panel(&self.look).child(heading("Platform preferences — actual readings"))
            .child(lab_ui::caption(&self.look, if linux {
                "Linux portal preferences with optional GNOME/KDE fallbacks. Not reported can mean unavailable, unsupported or pending."
            } else {
                "Desktop preference detection is currently Linux-only. Missing readings here are not native OS defaults; preview values may be fallbacks."
            }))
            .child(property("Platform", format!("{:?}", cx.platform_info())))
            .child(property("Appearance", format!("{:?}", window.appearance())))
            .child(property("Button layout", format!("{:?}", cx.button_layout())))
            .child(property("Accent", detected(desktop.accent_color)))
            .child(property("High contrast", detected(desktop.high_contrast)))
            .child(property("Reduced motion", detected(desktop.reduced_motion)))
            .child(property("Font", detected(desktop.font)))
            .child(property("Text scale", detected(desktop.text_scale)))
            .child(property("Titlebar actions", format!("double {} · middle {} · right {}", detected(desktop.titlebar_double_click), detected(desktop.titlebar_middle_click), detected(desktop.titlebar_right_click))))
            .child(property("Decorations", format!("{:?}", window.window_decorations())))
            .child(property("Window state", format!("Active {} · maximized {} · fullscreen {} · resizable {}", window.is_window_active(), window.is_maximized(), window.is_fullscreen(), window.is_resizable())))
            .child(property("Capabilities", format!("{:?}", window.window_controls())))
            .child(property("Device scale", format!("{:.2}×", window.scale_factor())))
    }
}
fn heading(text: &'static str) -> Div {
    div().text_lg().font_weight(FontWeight::SEMIBOLD).child(text)
}
fn property(label: &'static str, value: String) -> Div {
    div()
        .flex()
        .flex_wrap()
        .gap_2()
        .child(div().font_weight(FontWeight::MEDIUM).child(label))
        .child(value)
}
fn detected<T: std::fmt::Debug>(value: Option<T>) -> String {
    value.map(|v| format!("{v:?}")).unwrap_or_else(|| "not reported".into())
}
impl Render for ShellPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.observe_window(window, cx);
        let resolved = self.options().resolved_preferences(&cx.desktop_preferences());
        let look = resolved.resolved_look(self.appearance(window), self.values[Setting::Active as usize] == 1);
        let mut content = lab_ui::container().child(lab_ui::header(
            &self.look,
            if self.active_page == "Desktop Preferences" {
                "Desktop Preferences"
            } else {
                "Shell Lab"
            },
            "Desktop-aware window chrome, preferences and client-frame diagnostics.",
        ));
        if self.active_page == "Desktop Preferences" {
            content = content.child(self.readings(window, cx));
        } else {
            content = content
                .child(lab_ui::caption(&self.look, "Simulation only: frame buttons, dragging and resizing never operate the Lab window. Preview overrides do not change desktop settings or the host titlebar."))
                .child(self.rows(true))
                .child(div().id("shell-client-frame-preview").debug_selector(|| "shell-client-frame-preview".into())
                    .w_full().h(px(look.titlebar_height + 150.)).flex_shrink_0().child(self.preview(window)));
            content = content.child(
                lab_ui::panel(&self.look)
                    .child(heading("Preview overrides"))
                    .child(self.rows(false))
                    .child(self.reset.clone()),
            );
            content = content.child(self.inspector(window, cx));
        }
        content = content.child(
            lab_ui::panel(&self.look)
                .child(heading("Recent events"))
                .children(self.events.iter().rev().map(|event| lab_ui::caption(&self.look, event.clone()))),
        );
        lab_ui::page("shell-lab-page", &self.look).child(content)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use ::core::prelude::v1::test;

    #[test]
    fn sdk_choices_drive_preview_and_reset_without_mutating_host_or_other_pages() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| ShellPage::new(crate::theme::default_look(), cx));
        let other = cx.new(|cx| ShellPage::new(crate::theme::default_look(), cx));
        let before = cx.update(|window, cx| {
            (
                window.window_bounds(),
                window.window_decorations(),
                window.is_maximized(),
                window.is_fullscreen(),
                cx.desktop_preferences(),
            )
        });
        for (setting, index) in [
            (Setting::State, 2),
            (Setting::Appearance, 2),
            (Setting::Scale, 3),
            (Setting::Accent, 3),
            (Setting::Contrast, 2),
            (Setting::DoubleClick, 4),
            (Setting::Typography, 2),
            (Setting::Motion, 1),
            (Setting::Layout, 2),
            (Setting::Hover, 1),
            (Setting::Focus, 1),
        ] {
            let choice =
                cx.update(|_, cx| page.read(cx).choices.iter().find(|(key, _)| *key == setting).unwrap().1.clone());
            cx.update(|_, cx| choice.select(index, cx));
            cx.run_until_parked();
        }
        cx.update(|window, cx| {
            let page = page.read(cx);
            let preview = page.preview(window);
            assert_eq!(preview.appearance, WindowAppearance::Dark);
            assert!(preview.tiling.left && preview.show_hover && preview.show_focus);
            assert_eq!(preview.button_layout.unwrap().left[0], Some(WindowButton::Close));
            let resolved = page.options().resolved_preferences(&cx.desktop_preferences());
            assert_eq!(resolved.text_scale, 1.5);
            assert_eq!(resolved.accent, 0);
            assert!(resolved.high_contrast);
            assert!(!resolved.reduced_motion);
            assert_eq!(resolved.double_click_action, TitlebarAction::None);
            assert_eq!(resolved.typography.size, 24.);
            assert_eq!(other.read(cx).values, ShellPage::defaults());
            assert_eq!(
                before,
                (
                    window.window_bounds(),
                    window.window_decorations(),
                    window.is_maximized(),
                    window.is_fullscreen(),
                    cx.desktop_preferences()
                )
            );
        });
        // Navigation keeps overrides; returning to the page uses the same retained controls.
        page.update(cx, |page, cx| page.activate("Desktop Preferences".into(), cx));
        cx.run_until_parked();
        page.update(cx, |page, cx| page.activate("Shell Lab".into(), cx));
        cx.run_until_parked();
        let reset = cx.update(|_, cx| page.read(cx).reset.clone());
        reset.update(cx, |_, cx| cx.emit(xgpui_luma::controls::button::ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.values, ShellPage::defaults());
            let resolved = page.options().resolved_preferences(&cx.desktop_preferences());
            assert_eq!(resolved, ShellOptions::default().resolved_preferences(&cx.desktop_preferences()));
            for (_, choice) in &page.choices {
                assert!(choice.selector.read(cx).selected_id().is_some());
            }
        });
    }
}
