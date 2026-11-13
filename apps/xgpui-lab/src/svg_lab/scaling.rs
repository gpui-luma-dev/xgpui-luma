//! Dedicated texture magnification versus zoom-aware SVG rasterization probe.
use super::*;
use gpui::{prelude::*, AnyElement, div};
use xgpui_svg::svg_image;
use crate::screen::{text, TextRole};
impl SvgPage {
    pub(super) fn scaling(&self, cx: &mut Context<Self>) -> anyhow::Result<AnyElement> {
        let asset = self.asset.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
        self.policy.resolution(asset.logical, self.camera.zoom, self.density)?;
        let baseline = asset.fixed.size(0);
        let backdrop = self.backdrop.color();
        let fixed = self.viewport(0, cx).bg(backdrop).child(self.texture_or_native(self.camera, false));
        let sharp = self.viewport(1, cx).bg(backdrop).child(
            svg_image("svg-scaling-progressive", asset.document.clone())
                .logical_size(asset.logical)
                .zoom(self.camera.zoom)
                .pan(self.camera.pan)
                .policy(self.policy)
                .on_status_change(cx.listener(|this, status: &SvgStatus, _, cx| {
                    this.status = Some(status.clone());
                    cx.notify();
                })),
        );
        let column = |name: &str, detail: String, visual: gpui::AnyElement| {
            div()
                .flex_1()
                .min_w(gpui::px(280.))
                .flex()
                .flex_col()
                .gap_2()
                .child(text(&self.look, TextRole::Name, name.to_string()))
                .child(text(&self.look, TextRole::Caption, detail))
                .child(
                    radix::Card::new(&self.look)
                        .style_override(|style| {
                            style.padding = 1.;
                            style.radius = 0.;
                            style.background = None;
                        })
                        .child(visual)
                        .into_element()
                        .w_full()
                        .flex_shrink_0()
                        .overflow_hidden(),
                )
        };
        Ok(div()
            .id("svg-scaling-comparison")
            .debug_selector(|| "svg-scaling-comparison".into())
            .w_full()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(column(
                "Texture zoom",
                format!("Fixed {}×{} source px", i32::from(baseline.width), i32::from(baseline.height)),
                fixed.into_any_element(),
            ))
            .child(column("Sharp zoom", "Progressive rerasterization".into(), sharp.into_any_element()))
            .into_any_element())
    }
}
