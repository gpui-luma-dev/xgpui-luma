//! Screen composition; shaping, fallback resolution and CPU masks stay in the owner.
use super::*;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, ParametersPanel, TextRole, properties, text};
impl ParleyPage {
    pub(super) fn backend_error(&self, id: &'static str, title: &'static str) -> LabScreen {
        LabScreen::new(id, self.look.clone()).crumb("xgpui-parley", title).title(title).notice(Some({
            let __l10n_0 = &(self.engine.as_ref().err());
            crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_b952f649fe0a(args[0].clone()),
                vec![format!("{:?}", __l10n_0)],
            )
        }))
    }
    pub(super) fn row_screen(&self, cx: &mut Context<Self>) -> LabScreen {
        let (id, title, lede) = match self.active_tab.as_ref() {
            "Fonts & Fallback" => (
                "parley-fonts",
                "Fonts & Fallback",
                "Explore how a line finds the fonts it needs across different scripts.",
            ),
            "Rasterization" => (
                "parley-raster",
                "Rasterization",
                "Inspect the pixels produced from shaped text at different raster scales.",
            ),
            _ => (
                "parley-shaping",
                "Text Shaping",
                "Explore how letters become glyphs, clusters and resolved font runs.",
            ),
        };
        self.numbers[0].observe(self.font_size, cx);
        self.numbers[1].observe(self.raster_scale, cx);
        let mut screen = LabScreen::new(id, self.look.clone()).crumb("xgpui-parley", title).title(title).lede(lede)
            .caption({ let __l10n_0 = &(self.font_size); let __l10n_1 = &(self.raster_scale); crate::l10n::UiCopy::formatted(|args| crate::l10n::messages::msg_ui_442726a94666(args[0].clone(), args[1].clone()), vec![format!("{:.0}", __l10n_0), format!("{:.2}", __l10n_1)]) })
            .panels(ParametersPanel::new("Text parameters").params(self.numbers.iter().take(if title == "Rasterization" { 2 } else { 1 }).map(|n|n.param())),
                PanelShell::new("Diagnostics").child(properties(&self.look, [
                    ("CPU run", if self.measuring { "Running" } else if self.report.is_some() { "Finished" } else { "Not measured" }.to_owned()),
                    ("Iterations", "20 warm samples / median".into()),
                    ("Seed size", "28 px (independent of preview)".into()),
                ])))
            .note("Preview controls rebuild the retained CPU masks. CPU diagnostics use the original font fixtures and a separate layout cache.")
            .credits("CPU layout and direct glyph rasterization only; excludes frame and GPU timing. Native reference rows use the shell’s existing backend. Mask subjects use fixed paper and ink in both themes.");
        for button in self.buttons.iter().take(4) {
            screen = screen.action(button.clone());
        }
        if let Some(error) = &self.child_error {
            screen = screen.notice(Some(crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_e1912577b320(args[0].clone()),
                vec![error.to_string()],
            )));
        }
        if let Some(Err(error)) = &self.report {
            screen = screen.notice(Some(crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_23f15f03bf33(args[0].clone()),
                vec![format!("{:#}", error)],
            )));
        }
        match self.rows.as_ref().expect("rows prepared before screen") {
            Err(error) => screen.notice(Some(crate::l10n::UiCopy::formatted(
                |args| crate::l10n::messages::msg_ui_1a1f34d132b1(args[0].clone()),
                vec![format!("{:#}", error)],
            ))),
            Ok(rows) => {
                let first = if title == "Fonts & Fallback" { 2 } else { 0 };
                let row = &rows[first];
                screen = screen.hero(Hero::stage(ComparePane::new(row.label.clone(), self.mask(row))));
                for (index, row) in rows.iter().enumerate().skip(first) {
                    let mut panel = PanelShell::new(row.label.clone()).child(properties(
                        &self.look,
                        [
                            ("Source", crate::l10n::UiCopy::from(row.text.clone())),
                            ("Layout", {
                                let __l10n_0 = &(row.width - 4.);
                                let __l10n_1 = &(row.faces);
                                crate::l10n::UiCopy::formatted(
                                    |args| crate::l10n::messages::msg_ui_44949aeaeb1f(args[0].clone(), args[1].clone()),
                                    vec![format!("{:.2}", __l10n_0), format!("{}", __l10n_1)],
                                )
                            }),
                            ("Runs / clusters", crate::l10n::UiCopy::from(row.details.clone())),
                        ],
                    ));
                    if title == "Fonts & Fallback" {
                        panel = panel
                            .child(text(&self.look, TextRole::Detail, {
                                let __l10n_0 = &(row.font.family);
                                let __l10n_1 = &(row.font.weight.0);
                                let __l10n_2 = &(row.font.fallbacks);
                                crate::l10n::UiCopy::formatted(
                                    |args| {
                                        crate::l10n::messages::msg_ui_af9f6786d0b6(
                                            args[0].clone(),
                                            args[1].clone(),
                                            args[2].clone(),
                                        )
                                    },
                                    vec![format!("{}", __l10n_0), format!("{}", __l10n_1), format!("{:?}", __l10n_2)],
                                )
                            }))
                            .child(div().text_size(px(self.font_size)).font(row.font.clone()).child({
                                let __l10n_0 = &(row.text);
                                crate::l10n::UiCopy::formatted(
                                    |args| crate::l10n::messages::msg_ui_7d7494c67be2(args[0].clone()),
                                    vec![__l10n_0.to_string()],
                                )
                            }));
                    }
                    if index != first {
                        panel = panel.child(self.mask(row));
                    }
                    if title == "Rasterization" {
                        let pixels = row.image.size(0);
                        let bytes = row.image.as_bytes(0).unwrap();
                        let coverage = bytes.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
                        panel = panel.child(text(&self.look, TextRole::Value, {
                            let __l10n_0 = &(i32::from(pixels.width));
                            let __l10n_1 = &(i32::from(pixels.height));
                            let __l10n_2 = &(bytes.len());
                            crate::l10n::UiCopy::formatted(
                                |args| {
                                    crate::l10n::messages::msg_ui_9e375930ec55(
                                        args[0].clone(),
                                        args[1].clone(),
                                        args[2].clone(),
                                        args[3].clone(),
                                    )
                                },
                                vec![
                                    format!("{}", __l10n_0),
                                    format!("{}", __l10n_1),
                                    format!("{}", coverage),
                                    format!("{}", __l10n_2),
                                ],
                            )
                        }));
                    }
                    if let Some(Ok(report)) = &self.report {
                        let t = &report[index];
                        panel = panel.child(text(&self.look, TextRole::Value, {
                            let __l10n_0 = &(t.layout_first);
                            let __l10n_1 = &(t.layout_warm);
                            let __l10n_2 = &(t.raster_first);
                            let __l10n_3 = &(t.raster_warm);
                            let __l10n_4 = &(t.width);
                            let __l10n_5 = &(t.faces);
                            let __l10n_6 = &(t.glyphs);
                            crate::l10n::UiCopy::compose(
                                |args| {
                                    crate::l10n::messages::msg_ui_3a27000cb2e2(
                                        args[0].clone(),
                                        args[1].clone(),
                                        args[2].clone(),
                                        args[3].clone(),
                                        args[4].clone(),
                                        args[5].clone(),
                                        args[6].clone(),
                                    )
                                },
                                vec![
                                    crate::l10n::UiCopy::numeric(format!("{:.1}", __l10n_0)),
                                    crate::l10n::UiCopy::numeric(format!("{:.2}", __l10n_1)),
                                    crate::l10n::UiCopy::numeric(format!("{:.1}", __l10n_2)),
                                    crate::l10n::UiCopy::numeric(format!("{:.1}", __l10n_3)),
                                    crate::l10n::UiCopy::numeric(format!("{:.2}", __l10n_4)),
                                    crate::l10n::UiCopy::counted("faces", format!("{}", __l10n_5)),
                                    crate::l10n::UiCopy::counted("glyphs", format!("{}", __l10n_6)),
                                ],
                            )
                        }));
                    }
                    screen = screen.section_panel(panel);
                }
                screen
            }
        }
    }
    fn mask(&self, row: &Row) -> impl IntoElement + use<> {
        div()
            .id(format!("parley-mask-{}", row.label))
            .w_full()
            .overflow_x_scroll()
            .bg(rgb(palette::PAPER))
            .p_3()
            .child(img(row.image.clone()).w(px(row.width)).h(px(row.height)))
    }
}
