//! Luma owner for the original direct/legacy Hermite editor model.
use super::*;
use super::numeric::Number;
use crate::screen::{
    ComparePane, Hero, LabScreen, PanelShell, ParametersPanel, TextRole, labeled_control, properties, text,
};
pub(super) use super::mesh_tangent::MeshTangent;
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Control {
    HermiteX,
    HermiteY,
    Red,
    Green,
    Blue,
    HermiteUAngle,
    HermiteULength,
    HermiteVAngle,
    HermiteVLength,
}
impl Control {
    fn label(self) -> &'static str {
        match self {
            Self::HermiteX => "Point X",
            Self::HermiteY => "Point Y",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::HermiteUAngle => "U angle (°)",
            Self::HermiteULength => "U length",
            Self::HermiteVAngle => "V angle (°)",
            Self::HermiteVLength => "V length",
        }
    }

    pub fn range(self) -> (f32, f32, f32) {
        match self {
            Self::HermiteUAngle | Self::HermiteVAngle => (0., 360., 1.),
            Self::HermiteULength | Self::HermiteVLength => (0., 2., 0.01),
            _ => (0., 1., 0.01),
        }
    }
}
pub(super) struct MeshPane {
    look: Arc<Look>,
    editor: super::hermite::HermiteEditor,
    numbers: Vec<Number>,
    buttons: Vec<Entity<Button>>,
    sliders: Vec<(Control, Slider)>,
    choices: Vec<crate::lab_controls::Choice>,
}
impl MeshPane {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let buttons = [(0, "Grid 2×2 / 3×3"), (3, "Reset point"), (4, "Reset mesh"), (5, "Legacy round-trip")]
            .into_iter()
            .map(|(id, label)| {
                radix::Button::new(format!("hermite-{id}"))
                    .look(look.as_ref())
                    .outline()
                    .label(label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        match id {
                            0 => this.editor.set_side(if this.editor.side == 2 { 3 } else { 2 }),
                            3 => this.editor.reset_selected(),
                            4 => this.editor.load_preset(this.editor.preset),
                            _ => {
                                let points = this.editor.legacy_points();
                                let tangents = this.editor.tangents.clone();
                                this.editor.import_legacy(&points, &tangents);
                            }
                        }
                        this.sync(cx);
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let sliders = [
            Control::HermiteX,
            Control::HermiteY,
            Control::Red,
            Control::Green,
            Control::Blue,
            Control::HermiteUAngle,
            Control::HermiteULength,
            Control::HermiteVAngle,
            Control::HermiteVLength,
        ]
        .into_iter()
        .map(|control| {
            let (min, max, step) = control.range();
            let slider = radix::Slider::new(format!("hermite-{control:?}"))
                .look(look.as_ref())
                .range(min..max)
                .step(step)
                .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                    this.editor.set(control, value);
                    cx.notify();
                })
                .spawn(cx);
            (control, slider)
        })
        .collect();
        let choices = ["Preset", "Point"]
            .into_iter()
            .enumerate()
            .map(|(id, label)| {
                crate::lab_controls::Choice::new(
                    format!("hermite-choice-{id}"),
                    label,
                    &look,
                    (0..4).map(|i| format!("{label} {}", i + 1)).collect(),
                    0,
                    cx,
                    move |this: &mut Self, index, cx| {
                        if id == 0 {
                            this.editor.load_preset(index as u8);
                        } else {
                            this.editor.selected = index;
                        }
                        this.sync(cx);
                        cx.notify();
                    },
                )
            })
            .collect();
        let mut pane = Self {
            look: look.clone(),
            editor: super::hermite::HermiteEditor::new(0),
            buttons,
            sliders,
            choices,
            numbers: Vec::new(),
        };
        pane.numbers = pane
            .sliders
            .iter()
            .map(|(control, slider)| {
                let control = *control;
                Number::attach(
                    &format!("hermite-{control:?}"),
                    control.label(),
                    control.range(),
                    pane.editor.value(control),
                    slider.clone(),
                    &look,
                    cx,
                    move |this: &mut Self, value, _| {
                        this.editor.set(control, value);
                        this.editor.value(control)
                    },
                )
            })
            .collect();
        pane.sync(cx);
        pane
    }
    fn sync(&mut self, cx: &mut Context<Self>) {
        self.choices[0].sync(
            (0..self.editor.preset_count()).map(|i| format!("Preset {}", i + 1)).collect(),
            self.editor.preset as usize,
            cx,
        );
        self.choices[1].sync(
            (0..self.editor.points.len())
                .map(|i| {
                    format!(
                        "Point {} (row {}, column {})",
                        i + 1,
                        i / self.editor.side as usize + 1,
                        i % self.editor.side as usize + 1
                    )
                })
                .collect(),
            self.editor.selected,
            cx,
        );
        for (control, slider) in &self.sliders {
            slider.update(cx, |slider, cx| slider.set_value(self.editor.value(*control), cx));
        }
        for (number, (control, _)) in self.numbers.iter().zip(&self.sliders) {
            number.sync(self.editor.value(*control), cx);
        }
    }
}
impl Render for MeshPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use xgpui_shader::{painted, Paint};
        self.editor.refresh();
        for (number, (control, _)) in self.numbers.iter().zip(&self.sliders) {
            number.observe(self.editor.value(*control), cx);
        }
        let legacy = painted().w_full().h(px(280.)).bg_paint(Paint::from(self.editor.legacy.clone()));
        let (direct, error) = match &self.editor.brush {
            Ok(brush) => (painted().w_full().h(px(280.)).bg_brush(brush.clone()).into_any_element(), None),
            Err(error) => (
                text(&self.look, TextRole::Caption, "Direct preview unavailable").h(px(280.)).into_any_element(),
                Some(format!(
                    "Direct brush rejected this edit: {error}. Legacy preview stays current; adjust controls or reset."
                )),
            ),
        };
        let mut screen = LabScreen::new("shader-mesh", self.look.clone())
            .crumb("xgpui-shader", "Mesh Editor")
            .title("Mesh Editor")
            .lede("Compare two ways to draw the same mesh while editing its points, colors and curves.")
            .hero(Hero::compare(
                ComparePane::new("Legacy tessellation", legacy),
                ComparePane::new("Direct Hermite brush", direct),
            ))
            .caption(format!(
                "{}×{} · preset {} · point {} (row {}, column {}) · {}",
                self.editor.side,
                self.editor.side,
                self.editor.preset + 1,
                self.editor.selected + 1,
                self.editor.selected / self.editor.side as usize + 1,
                self.editor.selected % self.editor.side as usize + 1,
                if self.editor.edited { "edited" } else { "preset" }
            ))
            .notice(error);
        for choice in &self.choices {
            screen = screen.control(labeled_control(&self.look, choice.clone().into_element()));
        }
        for button in &self.buttons {
            screen = screen.action(button.clone());
        }
        screen = screen.panels(
            ParametersPanel::new("Selected point").params(self.numbers.iter().map(Number::param)),
            PanelShell::new("Mesh structure").child(properties(
                &self.look,
                [
                    ("Shared points", self.editor.points.len().to_string()),
                    ("Axes", "U and V tangent vectors".into()),
                    ("Legacy grid", "64 tessellation steps".into()),
                ],
            )),
        );
        if let Ok(brush) = &self.editor.brush {
            screen = screen.section_panel(
                PanelShell::new("The same patch on borders and glyphs")
                    .child(painted().w_full().h(px(72.)).border_8().border_brush(brush.clone()))
                    .child(
                        TextRole::Title
                            .apply(&self.look, painted().w_full().text_brush(brush.clone()))
                            .child("Hermite patch across glyphs"),
                    ),
            );
        }
        screen.note("Angles are clockwise from right. Lengths use normalized mesh space; changing an angle at zero length preserves it for the next length edit. Both previews use the same shared points.")
            .credits("Adapted from xgpui paint-lab/hermite_editor.rs. Direct-brush errors leave the legacy preview current. Point edits rebuild the retained material once per rendered frame.")
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{MeshPane, Control};
    use xgpui_luma::controls::button::ButtonEvent;
    #[test]
    fn mounted_luma_mesh_controls_edit_the_direct_and_legacy_model() {
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| MeshPane::new(crate::theme::default_look(), cx));
        let (side, point, red) = cx.update(|_, cx| {
            let page = page.read(cx);
            (
                page.buttons[0].clone(),
                page.choices[1].clone(),
                page.sliders.iter().find(|(control, _)| matches!(control, Control::Red)).unwrap().1.clone(),
            )
        });
        side.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|_, cx| point.select(1, cx));
        red.update(cx, |slider, cx| slider.set_thumb_position(slider.primary_thumb_id(), 0.23, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let editor = &page.read(cx).editor;
            assert_eq!(editor.side, 3);
            assert_eq!(editor.selected, 1);
            assert!((editor.points[1].color[0] - 0.23).abs() < 0.001);
            assert!(!editor.dirty);
            assert!(editor.brush.is_ok());
        });
    }
    #[test]
    fn numeric_edits_selection_and_reset_keep_the_same_shared_points() {
        use xgpui_luma::controls::textfield::TextFieldEvent;
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| MeshPane::new(crate::theme::default_look(), cx));
        let x = cx.update(|_, cx| page.read(cx).numbers[0].field.clone());
        x.update(cx, |field, cx| {
            field.set_value("0.23", cx);
            cx.emit(TextFieldEvent::Change { value: "0.23".into() });
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.editor.points[0].position[0], 0.23);
            assert_eq!(page.editor.legacy_points()[0].position[0], 0.23);
            // Moving a boundary corner inward is rejected by the direct brush.
            assert!(page.editor.brush.is_err());
            assert!(!page.editor.dirty);
        });
        let reset = cx.update(|_, cx| page.read(cx).buttons[1].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(x.read(cx).value(), format!("{:.3}", page.editor.value(Control::HermiteX)));
            assert!(page.editor.brush.is_ok());
            assert!(!page.editor.dirty);
        });
    }
}
