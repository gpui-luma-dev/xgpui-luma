use super::*;
use crate::{controls::button_family::*, perf_measure::measure, theme::*};

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
        let metrics = MetricTokens::default();
        let theme = default_button_family_theme();
        let palette = theme.resolve(ButtonFamilyRole::Text, InteractionState::default());
        let scale = StandardBoxScale::compute(&metrics, 2.0);
        let mut look = compose_button_family_look(&palette, ButtonFamilyRole::Text, &scale, metrics.radius.pill);
        let layer = ThemeTokens::default().elevation.control.to_box_shadows()[0].clone();
        look.shadow = Some(vec![layer; layers]);
        let source: super::super::model::ButtonLookSource<()> = Arc::new(move |_| look.clone());
        let mut app = gpui::TestAppContext::single();
        let (page, cx) = app.add_window_view(|_, cx| ShadowButtons {
            buttons: (0..100)
                .map(|index| {
                    let mut builder = ButtonBuilder::new(format!("button-{index}"));
                    builder.model.look = Some(source.clone());
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
                builder.model.look = Some(source.clone());
                page.update(app, |_, cx| builder.spawn(cx))
            })
        });
    }
}
