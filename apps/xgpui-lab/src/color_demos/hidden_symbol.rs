//! Gamut-reveal demonstration using the shared lab page and SDK controls.
use std::sync::Arc;
use gpui::{prelude::*, *};
use xgpui_luma::{controls::button::Button, prelude::HasPresenter};
use xgpui_luma_look_radix::{self as radix, Look};
use crate::{
    chromaticity::reveal::Reveal,
    lab_controls::{Choice, labels},
    screen::{Hero, LabScreen, labeled_control},
};

const VIEWS: [&str; 3] = ["sRGB preview", "Native Display P3", "Out-of-sRGB mask"];

pub(crate) struct HiddenSymbolLab {
    look: Arc<Look>,
    reveal: Result<Reveal, String>,
    selected: usize,
    wide_requested: bool,
    presentation_pending: bool,
    presentation_attempted: bool,
    presentation_error: Option<String>,
    view_choice: Choice,
    output_choice: Choice,
    attribution: Entity<Button>,
}

impl HiddenSymbolLab {
    pub(crate) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let view_choice =
            Choice::new("hidden-symbol-view", "View", &look, labels(&VIEWS), 1, cx, |this: &mut Self, index, cx| {
                this.selected = index;
                cx.notify();
            });
        let output_choice = Choice::new(
            "hidden-symbol-output",
            "Window output",
            &look,
            labels(&["Wide gamut", "sRGB"]),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.wide_requested = index == 0;
                this.presentation_attempted = false;
                this.presentation_error = None;
                cx.notify();
            },
        );
        let attribution = radix::Button::new("hidden-symbol-reference")
            .look(&look)
            .outline()
            .label("WebKit reference ↗")
            .on_click(cx, |_: &mut Self, cx| cx.open_url("https://webkit.org/blog-files/color-gamut/"))
            .spawn(cx);
        Self {
            look,
            reveal: Reveal::new().map_err(|error| error.to_string()),
            selected: 1,
            wide_requested: true,
            presentation_pending: false,
            presentation_attempted: false,
            presentation_error: None,
            view_choice,
            output_choice,
            attribution,
        }
    }

    pub(crate) fn activate(&mut self, cx: &mut Context<Self>) {
        self.presentation_attempted = false;
        cx.notify();
    }
}

impl Render for HiddenSymbolLab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.presentation_attempted && !self.presentation_pending {
            self.presentation_pending = true;
            cx.defer_in(window, |this, window, cx| {
                let desired = if this.wide_requested {
                    WindowColorMode::WideGamutSdr
                } else {
                    WindowColorMode::Default
                };
                this.presentation_error = match window.set_color_mode(desired) {
                    Ok(status) if status.effective() == desired => None,
                    Ok(_) => Some("Wide-gamut output is unavailable; showing an sRGB preview.".into()),
                    Err(error) => Some(format!("Window output unavailable: {error}")),
                };
                this.presentation_pending = false;
                this.presentation_attempted = true;
                cx.notify();
            });
        }
        if let Ok(reveal) = &mut self.reveal {
            reveal.poll(window);
        }
        let wide = window.color_status().effective() == WindowColorMode::WideGamutSdr;
        let mut picture =
            div().w_full().aspect_ratio(1.).flex_shrink_0().debug_selector(|| "hidden-symbol-picture".into());
        let error = match &self.reveal {
            Ok(reveal) => {
                picture = picture.child(reveal.picture(self.selected, wide));
                reveal.error.clone()
            }
            Err(error) => Some(error.clone()),
        };
        let display = match window.color_status().display_wide_gamut() {
            Some(true) => "Display reports wide gamut",
            Some(false) => "Display reports sRGB / SDR",
            None => "Display gamut unknown",
        };
        LabScreen::new("hidden-wide-gamut-symbol", self.look.clone())
            .crumb("xgpui-gamut", "Hidden Wide-Gamut Symbol")
            .title("Hidden Wide-Gamut Symbol")
            .lede("Compare native Display P3 with its clipped sRGB preview. Look for a ring and X in the red field.")
            .hero(Hero::fixed(picture, px(520.)))
            .caption(format!("{} · {} · {display}", VIEWS[self.selected],
                if wide { "Wide-gamut output (Display P3)" } else { "sRGB output" }))
            .control(labeled_control(&self.look, self.view_choice.clone().into_element()))
            .control(labeled_control(&self.look, self.output_choice.clone().into_element()))
            .control(self.attribution.clone())
            .notice(self.presentation_error.clone())
            .notice(error)
            .note("The background is P3 red; the symbol is sRGB red converted into P3 coordinates. Clipping both to sRGB hides the symbol. The mask marks out-of-sRGB colors in magenta and in-gamut colors in red.")
            .credits("Original generated ring/X motif, inspired by WebKit’s wide-gamut examples. Native output and a wide-gamut display are needed to reveal the difference. Display profiles and panel capabilities determine visible contrast; screenshots do not establish physical wide-gamut delivery. This is SDR, not HDR.")
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::prelude::v1::test;

    #[test]
    fn view_and_window_output_controls_are_independent() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| HiddenSymbolLab::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let (view, output) = cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.selected, 1);
            assert!(page.wide_requested);
            assert!(page.presentation_attempted);
            (page.view_choice.clone(), page.output_choice.clone())
        });
        for selected in 0..3 {
            cx.update(|_, cx| view.select(selected, cx));
            cx.run_until_parked();
            cx.update(|_, cx| {
                let page = page.read(cx);
                assert_eq!(page.selected, selected);
                assert!(page.wide_requested);
            });
        }
        cx.update(|_, cx| output.select(1, cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            let page = page.read(cx);
            assert!(!page.wide_requested);
            assert_eq!(page.selected, 2);
            assert_eq!(window.color_status().effective(), WindowColorMode::Default);
            assert!(page.presentation_error.is_none());
        });
        assert!(cx.debug_bounds("hidden-symbol-picture").is_some());
    }
}
