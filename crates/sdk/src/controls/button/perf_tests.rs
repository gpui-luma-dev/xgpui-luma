use super::*;
use crate::{perf_measure::measure, theme::*};

struct ShadowButtons {
    buttons: Vec<gpui::Entity<Button>>,
}

impl Render for ShadowButtons {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_wrap().children(self.buttons.iter().cloned())
    }
}

#[test]
#[ignore = "manual headless performance measurement"]
fn measure_shadow_controls() {
    for layers in [0, 1, 2, 8] {
        let layer = ThemeTokens::default().elevation.control.to_box_shadows()[0].clone();
        let shadow = crate::style::lss::ShadowLayer {
            x: f32::from(layer.offset.x),
            y: f32::from(layer.offset.y),
            blur: f32::from(layer.blur_radius),
            spread: f32::from(layer.spread_radius),
            color: crate::style::lss_color(layer.color),
            inset: layer.inset,
        };
        let layers_value: Arc<[crate::style::lss::ShadowLayer]> = vec![shadow; layers].into();
        let source = crate::style::lss::Overrides::new().set(
            crate::style::lss::PartId::ROOT,
            crate::style::lss::Prop::<Arc<[crate::style::lss::ShadowLayer]>>::new("shadow"),
            layers_value,
        );
        let mut app = gpui::TestAppContext::single();
        let (page, cx) = app.add_window_view(|_, cx| ShadowButtons {
            buttons: (0..100)
                .map(|index| {
                    let mut builder = ButtonBuilder::new(format!("button-{index}"));
                    builder.model.overrides = source.clone();
                    builder.spawn(cx)
                })
                .collect(),
        });
        cx.run_until_parked();
        measure(&format!("Button/100 headless redraw/{layers} layers"), 5, || {
            cx.update(|_, app| {
                page.update(app, |page, cx| {
                    for button in &page.buttons {
                        button.update(cx, |_, cx| cx.notify());
                    }
                });
            });
            cx.run_until_parked();
        });
        measure(&format!("Button/construct entity/{layers} layers"), 100, || {
            cx.update(|_, app| {
                let mut builder = ButtonBuilder::new("cold-button");
                builder.model.overrides = source.clone();
                page.update(app, |_, cx| builder.spawn(cx))
            })
        });
    }
}
