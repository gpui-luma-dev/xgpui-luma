//! Legacy standalone presentation; pixel generation is shared with the current page.
use super::*;
pub(super) use crate::gamut_reveal::Reveal;

impl Lab {
    pub(super) fn render_reveal(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(reveal) = &mut self.reveal {
            reveal.poll(window);
        }
        let labels = ["sRGB preview", "Display P3 — native wide", "Out-of-sRGB mask"];
        let mut body = div().id("gamut-reveal-scroll").size_full().overflow_y_scroll().p_6().bg(rgb(0x858585)).text_color(rgb(0x101010)).flex().flex_col().gap_4()
            .child(div().flex().flex_wrap().gap_2()
                .when(self.standalone_navigation, |row| row.child(self.button("← Chromaticity & metadata".into(), 18, cx)))
                .child(self.window_mode_controls(window, cx)))
            .child(div().text_2xl().child(crate::l10n::ui("Hidden wide-gamut symbol")))
            .child(div().id("webkit-gamut-attribution").cursor_pointer().text_color(rgb(0x003d76))
                .child(crate::l10n::ui("Inspired by WebKit — Examples of various wide-gamut images ↗"))
                .on_click(|_, _, cx| cx.open_url("https://webkit.org/blog-files/color-gamut/")))
            .child(crate::l10n::ui("Choose a thumbnail below. In native P3, look for a ring and X in the red field; in the clipped sRGB preview, the symbol should disappear."))
            .child(crate::l10n::ui(format!("Currently showing: {} • effective {:?} • display wide gamut {:?}", labels[self.reveal_selection], window.color_status().effective(), window.color_status().display_wide_gamut())));
        if self.reveal_selection == 1 && !self.wide {
            body = body.child(crate::l10n::ui("Native wide output is unavailable or disabled: showing the sRGB fallback. The symbol is not expected to appear."));
        }
        if let Some(reveal) = &self.reveal {
            body = body
                .child(
                    div()
                        .w_full()
                        .max_w(px(620.))
                        .aspect_ratio(1.)
                        .flex_shrink_0()
                        .child(reveal.picture(self.reveal_selection, self.wide)),
                )
                .child(div().flex().flex_wrap().gap_4().children((0..3).map(|selection| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .id(ElementId::Name(format!("reveal-thumbnail-{selection}").into()))
                                .w(px(150.))
                                .h(px(150.))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.reveal_selection = selection;
                                    cx.notify();
                                }))
                                .child(reveal.picture(selection, self.wide)),
                        )
                        .child(self.button(labels[selection].into(), 19 + selection, cx))
                })))
                .child(reveal.error.clone().unwrap_or_default());
        }
        body.child(crate::l10n::ui("What this shows: the background is P3 red; the symbol is sRGB red converted into P3 coordinates. Clipping both to sRGB makes them the same red. The mask marks source colors outside sRGB in magenta; red is inside. Edge coverage is generated in linear light."))
            .child(crate::l10n::ui("Use a wide-gamut display and native wide output. The OS display profile and panel determine the visible contrast. A screenshot, an sRGB preview, or a capability flag alone cannot establish physical wide-gamut delivery. No HDR brightness is involved."))
            .child(crate::l10n::ui("Original generated ring/X motif and implementation; WebKit is credited for the demonstration concept. No WebKit artwork is embedded, and no image file or ICC decoder is used."))
            .child(self.error.clone().unwrap_or_default())
    }
}
