//! Gamut-reveal demonstration using the shared lab page and SDK controls.
use std::sync::Arc;
use gpui::{prelude::*, *};
use xgpui_luma::{controls::button::Button, prelude::HasPresenter, vstack, hstack};
use xgpui_luma_look_radix::{self as radix, Look};
use crate::{
    gamut_reveal::Reveal,
    screen::{Hero, LabScreen, TextRole, text},
};

const VIEWS: [&str; 3] = ["sRGB preview", "Native Display P3", "Out-of-sRGB mask"];

pub(crate) struct HiddenSymbolLab {
    look: Arc<Look>,
    reveal: Result<Reveal, String>,
    selected: usize,
    presentation_pending: bool,
    presentation_attempted: bool,
    presentation_error: Option<crate::l10n::UiCopy>,
    attribution: Entity<Button>,
    thumbnails: Vec<Entity<Button>>,
    thumbnail_wide: bool,
}

impl HiddenSymbolLab {
    pub(crate) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let attribution = radix::Button::new("hidden-symbol-reference")
            .look(&look)
            .outline()
            .content(crate::l10n::label("WebKit reference ↗"))
            .on_click(cx, |_: &mut Self, cx| cx.open_url("https://webkit.org/blog-files/color-gamut/"))
            .spawn(cx);
        let thumbnails = VIEWS
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let owner = cx.weak_entity();
                radix::Button::new(format!("hidden-symbol-sample-{index}"))
                    .look(&look)
                    .content_only()
                    .content(move |_, cx| {
                        let mut preview = div().w(px(140.)).h(px(140.)).flex_shrink_0();
                        if let Some(page) = owner.upgrade() {
                            let page = page.read(cx);
                            if let Ok(reveal) = &page.reveal {
                                preview = preview.child(reveal.picture(index, page.thumbnail_wide));
                            }
                        }
                        preview
                    })
                    .with_template_modifier(move |row, _| {
                        row.w(px(140.))
                            .h(px(140.))
                            .flex_shrink_0()
                            .p_0()
                            .items_start()
                            .debug_selector(move || format!("hidden-symbol-sample-{index}"))
                    })
                    .on_click(cx, move |this: &mut Self, cx| this.select(index, cx))
                    .spawn(cx)
            })
            .collect();
        Self {
            look,
            reveal: Reveal::new().map_err(|error| error.to_string()),
            selected: 1,
            presentation_pending: false,
            presentation_attempted: false,
            presentation_error: None,
            attribution,
            thumbnails,
            thumbnail_wide: false,
        }
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = index;
        for thumbnail in &self.thumbnails {
            thumbnail.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
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
                let desired = WindowColorMode::WideGamutSdr;
                this.presentation_error = match window.set_color_mode(desired) {
                    Ok(status) if status.effective() == desired => None,
                    Ok(_) => Some("Wide-gamut output is unavailable; showing an sRGB preview.".into()),
                    Err(error) => Some(crate::l10n::UiCopy::formatted(
                        |args| crate::l10n::messages::msg_ui_527707d951e0(args[0].clone()),
                        vec![format!("{}", error)],
                    )),
                };
                this.presentation_pending = false;
                this.presentation_attempted = true;
                cx.notify();
            });
        }
        let images_changed = if let Ok(reveal) = &mut self.reveal {
            reveal.poll(window)
        } else {
            false
        };
        let wide = window.color_status().effective() == WindowColorMode::WideGamutSdr;
        if images_changed || self.thumbnail_wide != wide {
            self.thumbnail_wide = wide;
            for thumbnail in &self.thumbnails {
                thumbnail.update(cx, |_, cx| cx.notify());
            }
        }
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
            .hero(Hero::fixed(vstack! { gap=16;
                picture,
                hstack! { gap=12 align=start justify=center; }
                    .w_full().flex_shrink_0()
                    .debug_selector(|| "hidden-symbol-samples".into())
                    .children(self.thumbnails.iter().enumerate().map(|(index, image)| {
                        vstack! { gap=8; image.clone(), text(&self.look, TextRole::Caption, VIEWS[index]) }.w(px(140.))
                    }))
            }.w_full(), px(520.)))
            .caption(crate::l10n::UiCopy::Separated(" · ", vec![VIEWS[self.selected].into(), (if wide { "Wide-gamut output (Display P3)" } else { "sRGB output" }).into(), display.into()]))
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
    fn sample_images_select_the_main_image() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| HiddenSymbolLab::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        for selected in 0..3 {
            let button = cx.update(|_, cx| page.read(cx).thumbnails[selected].clone());
            button.update(cx, |_, cx| cx.emit(xgpui_luma::controls::button::ButtonEvent::Click));
            cx.run_until_parked();
            cx.update(|_, cx| {
                let page = page.read(cx);
                assert_eq!(page.selected, selected);
            });
        }
        let main = cx.debug_bounds("hidden-symbol-picture").unwrap();
        let samples = cx.debug_bounds("hidden-symbol-samples").unwrap();
        let caption = cx.debug_bounds("hidden-wide-gamut-symbol-caption").unwrap();
        assert!(samples.top() >= main.bottom() + px(15.));
        assert!(caption.top() >= samples.bottom());
        let mut previous = None;
        for selector in ["hidden-symbol-sample-0", "hidden-symbol-sample-1", "hidden-symbol-sample-2"] {
            let sample = cx.debug_bounds(selector).unwrap();
            assert_eq!(sample.size.width, px(140.));
            assert_eq!(sample.size.height, px(140.));
            assert!(sample.bottom() <= samples.bottom());
            assert_eq!(sample.top(), samples.top());
            if let Some(right) = previous {
                assert!(sample.left() > right);
            }
            previous = Some(sample.right());
        }
    }
}
