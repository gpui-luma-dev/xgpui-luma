//! Isolated headless costs, not application frame times. Run with:
//! `cargo bench -p xgpui-luma --bench render_hot_paths -- --measure`
use std::{collections::HashMap, hint::black_box, rc::Rc, sync::Arc};

#[path = "../src/perf_measure.rs"]
mod perf_measure;
use perf_measure::measure;

use gpui::{BoxShadow, ElementId, SharedString};
use xgpui_luma::{
    controls::{
        button_family::{ButtonFamilyRole, compose_button_family_look, default_button_family_theme},
        sidebar::SidebarPanelTemplateHandlers,
        tree_view::TreeViewTemplateHandlers,
    },
    theme::{ControlSize, InteractionState, MetricTokens, StandardBoxScale, ThemeTokens},
};
use smallvec::SmallVec;

fn main() {
    // `cargo test --all-targets` should compile this target without benchmarking.
    if !std::env::args().any(|argument| argument == "--measure") {
        return;
    }
    println!(
        "Isolated costs (median of five batches); fixtures excluded unless construction is named. Ranges show batch min/max; allocation counts include reallocations and zeroed allocations on the measuring thread."
    );
    let iterations = 100_000;
    let metrics = MetricTokens::default();
    measure("StandardBoxScale/direct", iterations, || {
        StandardBoxScale::compute(black_box(ControlSize::Md), black_box(&metrics), black_box(2.0))
    });

    let id = SharedString::from("benchmark-table");
    measure("ElementId/formatted", iterations, || {
        ElementId::from(format!("{}-row-{}", black_box(&id), black_box(42usize)))
    });
    measure("ElementId/composite", iterations, || ElementId::from((black_box(&id).clone(), black_box(42usize))));

    for count in [100usize, 10_000] {
        let ids: Vec<SharedString> = (0..count).map(|index| format!("node-{index}").into()).collect();
        let indices: HashMap<SharedString, usize> =
            ids.iter().cloned().enumerate().map(|(index, id)| (id, index)).collect();
        let target = &ids[count - 1];
        measure(&format!("Tree lookup/linear/{count}"), 10_000, || {
            black_box(&ids).iter().position(|id| id == black_box(target))
        });
        measure(&format!("Tree lookup/indexed/{count}"), iterations, || {
            black_box(&indices).get(black_box(target)).copied()
        });
    }

    let handlers = TreeViewTemplateHandlers {
        hover: Rc::new(|_, _, _| {}),
        mouse_down: Rc::new(|_, _, _| {}),
        mouse_up: Rc::new(|_, _, _| {}),
        click: Rc::new(|_, _, _| {}),
        disclosure: Rc::new(|_, _, _| {}),
    };
    measure("Tree handlers/shared clone", iterations, || black_box(&handlers).clone());

    for count in [100usize, 10_000] {
        let mut sidebar = SidebarPanelTemplateHandlers::default();
        sidebar.row_hovers = Rc::new((0..count).map(|_| handlers.hover.clone()).collect());
        sidebar.row_mouse_downs = Rc::new((0..count).map(|_| handlers.mouse_down.clone()).collect());
        sidebar.row_mouse_ups = Rc::new((0..count).map(|_| handlers.mouse_up.clone()).collect());
        sidebar.row_mouse_up_outs = Rc::clone(&sidebar.row_mouse_ups);
        sidebar.row_clicks = Rc::new((0..count).map(|_| handlers.click.clone()).collect());
        // Mirrors the cached row-container clone; excludes the O(N) signature check.
        measure(&format!("Sidebar/shared row containers/{count}"), iterations, || {
            let sidebar = black_box(&sidebar);
            (
                Rc::clone(&sidebar.row_bounds),
                Rc::clone(&sidebar.row_hovers),
                Rc::clone(&sidebar.row_mouse_downs),
                Rc::clone(&sidebar.row_mouse_ups),
                Rc::clone(&sidebar.row_mouse_up_outs),
                Rc::clone(&sidebar.row_clicks),
                Rc::clone(&sidebar.children_height_reports),
            )
        });
    }

    let theme = default_button_family_theme();
    let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
    let scale = StandardBoxScale::compute(ControlSize::Md, &metrics, 2.0);
    let mut look = compose_button_family_look(&palette, ButtonFamilyRole::Text, &scale, metrics.radius.pill);
    measure("Button look/no shadows clone", iterations, || black_box(&look).clone());
    let layer = ThemeTokens::default().elevation.control.to_box_shadows()[0].clone();
    for count in [0, 1, 2, 8] {
        let shadows = vec![layer.clone(); count];
        look.shadow = Some(shadows.clone());
        let inline: SmallVec<[BoxShadow; 2]> = shadows.iter().cloned().collect();
        let shared: Arc<[BoxShadow]> = shadows.clone().into();
        measure(&format!("Button look/Vec clone/{count}"), iterations, || black_box(&look).clone());
        measure(&format!("Shadow/Vec construct/{count}"), iterations, || black_box(&shadows).to_vec());
        measure(&format!("Shadow/SmallVec construct/{count}"), iterations, || {
            black_box(&shadows).iter().cloned().collect::<SmallVec<[BoxShadow; 2]>>()
        });
        measure(&format!("Shadow/Arc construct/{count}"), iterations, || {
            Arc::<[BoxShadow]>::from(black_box(&shadows).as_slice())
        });
        measure(&format!("Shadow/SmallVec clone/{count}"), iterations, || black_box(&inline).clone());
        measure(&format!("Shadow/Arc clone/{count}"), iterations, || Arc::clone(black_box(&shared)));
        // GPUI 1.24.2 takes Vec<BoxShadow>; include that ownership conversion.
        use gpui::{Styled, div};
        measure(&format!("Shadow/Vec clone+GPUI/{count}"), iterations, || div().shadow(black_box(&shadows).clone()));
        measure(&format!("Shadow/SmallVec clone+GPUI/{count}"), iterations, || {
            div().shadow(black_box(&inline).clone().into_vec())
        });
        measure(&format!("Shadow/Arc clone+GPUI/{count}"), iterations, || {
            let cloned = Arc::clone(black_box(&shared));
            div().shadow(cloned.to_vec())
        });
        measure(&format!("Shadow/Vec construct+GPUI/{count}"), iterations, || {
            div().shadow(black_box(&shadows).to_vec())
        });
        measure(&format!("Shadow/SmallVec construct+GPUI/{count}"), iterations, || {
            div().shadow(black_box(&shadows).iter().cloned().collect::<SmallVec<[BoxShadow; 2]>>().into_vec())
        });
        measure(&format!("Shadow/Arc construct+GPUI/{count}"), iterations, || {
            let constructed: Arc<[BoxShadow]> = black_box(&shadows).as_slice().into();
            div().shadow(constructed.to_vec())
        });
    }
}
