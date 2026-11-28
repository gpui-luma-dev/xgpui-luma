use super::*;
use crate::perf_measure::measure;
use gpui::prelude::*;

#[test]
#[ignore = "manual headless performance measurement"]
fn measure_textarea_hot_paths() {
    for count in [100, 10_000] {
        let mut app = gpui::TestAppContext::single();
        let (editor, cx) = app.add_window_view(|_, cx| {
            TextArea::from_builder(
                TextAreaBuilder::new("perf-editor")
                    .value("office é中🙂 e\u{301} العربية\n".repeat(count))
                    .rows(5)
                    .with_template_modifier(|control, _| control.w(px(300.0))),
                cx,
            )
        });
        cx.run_until_parked();
        for case in ["warm", "cold layout", "edit", "resize", "scroll", "selection", "IME", "warm decoration"] {
            // Reset between workloads so selection and IME cases stay independent.
            editor.update(cx, |view, cx| {
                view.model.overrides = crate::style::lss::Overrides::new();
                view.state.selection_anchor = None;
                view.state.cursor = 0;
                view.marked_range = None;
                view.vertical_scroll = px(0.0);
                cx.notify();
            });
            cx.run_until_parked();
            let mut step = 0;
            measure(&format!("TextArea/headless {case}/{count} lines"), 5, || {
                step += 1;
                editor.update(cx, |view, cx| {
                    match case {
                        "cold layout" => view.layout_cache = None,
                        "edit" => {
                            let mut value = view.model.value.to_string();
                            if step % 2 == 0 {
                                value.pop();
                            } else {
                                value.push('z');
                            }
                            view.model.value = value.into();
                        }
                        "resize" => {
                            let width = if step % 2 == 0 { 280.0 } else { 300.0 };
                            view.model.overrides = crate::style::lss::Overrides::new().set(
                                crate::style::lss::PartId::ROOT,
                                crate::style::lss::Prop::<f32>::new("padding_x"),
                                width - 275.0,
                            );
                        }
                        "scroll" => view.vertical_scroll = px(if step % 2 == 0 { 0.0 } else { 200.0 }),
                        "selection" | "warm decoration" => {
                            view.state.selection_anchor = Some(0);
                            view.state.cursor = if case == "warm decoration" { 5 } else { 3 + step % 3 };
                        }
                        "IME" => view.marked_range = Some(0..3 + step % 3),
                        _ => {}
                    }
                    cx.notify();
                });
                cx.run_until_parked();
            });
        }
    }
}
