use xgpui_luma::controls::scrollbar;
use xgpui_luma::controls::control_group::ControlGroupEventExt;
use xgpui_luma::controls::button::ButtonEvent;
use xgpui_luma_plot::CartesianStyleProvider;
use super::*;

struct SelectorHarness {
    gallery: super::super::LineGallery,
    look: std::sync::Arc<AppTheme>,
    scroll: xgpui_luma::controls::scroll_container::ScrollContainer,
    plot: Entity<InterpolationExample>,
    bounds: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, gpui::Bounds<gpui::Pixels>>>>,
}

impl Render for SelectorHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scroll.sync_scrollbar(cx);
        theme::scroll_surface(&self.look).size_full().render(cx).child(
            self.scroll.render(
                vstack! { super::super::gallery(&self.gallery, &self.look, 1.0, cx) }
                    .w(px(900.0))
                    .into_any_element(),
            ),
        )
    }
}

#[gpui::test]
fn selector_clicks_change_interpolation_on_one_unchanged_series(cx: &mut gpui::TestAppContext) {
    use xgpui_luma::controls::control_group::ControlGroupEvent;
    let (host, cx) = cx.add_window_view(|_, cx| {
        let look = theme::default_theme();
        let gallery = super::super::examples(&look, cx).expect("gallery");
        let plot = gallery.interpolation.clone();
        let scroll = xgpui_luma::controls::scroll_container::ScrollContainer::new(
            "selector-test-scroll",
            scrollbar::default_scrollbar_template(),
            cx,
        );
        let group = plot.read(cx).selector.read(cx).inner_group();
        let bounds = std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
        let captured = bounds.clone();
        group.clone().on_event(cx, move |_, event: &ControlGroupEvent, _| {
            if let ControlGroupEvent::ItemBoundsChanged { item_id, bounds } = event {
                captured.borrow_mut().insert(item_id.to_string(), *bounds);
            }
        });
        SelectorHarness { gallery, look, scroll, plot, bounds }
    });
    let y = host.read_with(cx, |host, _| f32::from(host.bounds.borrow().get("monotone").expect("bounds").origin.y));
    host.update(cx, |host, cx| host.scroll.set_vertical_offset(y - 100.0, cx));
    cx.run_until_parked();
    let plot = host.read_with(cx, |host, _| host.plot.clone());
    plot.read_with(cx, |plot, cx| {
        assert_eq!(plot.interpolation, Interpolation::Monotone);
        assert_eq!(plot.selector.read(cx).selected_id(cx).as_deref(), Some("monotone"));
        assert_eq!(plot.data.series().len(), 1);
        assert_eq!(plot.data.series()[0].samples().len(), 11);
    });
    let dark_axis = host
        .read_with(cx, |host, _| theme::SdkCartesianStyle::new(&host.look).resolve().axes[0].presentation.stroke.color);
    host.update(cx, |host, cx| {
        host.look.set_mode(xgpui_luma::theme::ThemeMode::Light);
        cx.refresh_windows();
    });
    cx.run_until_parked();
    let light_axis = host
        .read_with(cx, |host, _| theme::SdkCartesianStyle::new(&host.look).resolve().axes[0].presentation.stroke.color);
    assert_ne!(dark_axis, light_axis);
    plot.read_with(cx, |plot, cx| {
        assert_eq!(plot.look.mode(), xgpui_luma::theme::ThemeMode::Light);
        assert_eq!(plot.interpolation, Interpolation::Monotone);
        assert_eq!(plot.selector.read(cx).selected_id(cx).as_deref(), Some("monotone"));
        assert!(plot.data.series()[0].samples()[6].y.is_nan());
    });
    for (id, expected) in [("linear", Interpolation::Linear), ("monotone", Interpolation::Monotone)] {
        let bounds = host.read_with(cx, |host, _| *host.bounds.borrow().get(id).expect("selector bounds"));
        assert!(bounds.size.width > px(0.0) && bounds.size.height > px(0.0));
        assert!(bounds.origin.y >= px(0.0));
        cx.simulate_click(bounds.center(), gpui::Modifiers::default());
        plot.read_with(cx, |plot, cx| {
            assert_eq!(plot.interpolation, expected);
            assert_eq!(plot.selector.read(cx).selected_id(cx).as_deref(), Some(id));
            assert_eq!(plot.data.series().len(), 1);
            assert_eq!(plot.data.series()[0].samples()[2], Sample::new(2.5, 8.0));
            assert!(plot.data.series()[0].samples()[6].y.is_nan());
        });
    }
    host.update(cx, |host, cx| {
        host.look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
        cx.refresh_windows();
    });
    cx.run_until_parked();
    assert_eq!(
        host.read_with(cx, |host, _| theme::SdkCartesianStyle::new(&host.look).resolve().axes[0]
            .presentation
            .stroke
            .color),
        dark_axis
    );
}

#[gpui::test]
fn replay_restarts_sdk_motion_without_changing_observations_or_interpolation(cx: &mut gpui::TestAppContext) {
    let plot = cx.update(|cx| InterpolationExample::spawn(&theme::default_theme(), cx).expect("example"));
    let replay = plot.read_with(cx, |plot, _| plot.replay.clone());
    plot.read_with(cx, |plot, _| {
        assert_eq!(plot.motion.progress(), 1.0);
        assert!(!plot.motion.is_animating());
    });
    replay.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    plot.read_with(cx, |plot, _| {
        assert_eq!(plot.motion.progress(), 0.0);
        assert!(plot.motion.is_animating());
        assert_eq!(plot.interpolation, Interpolation::Monotone);
        assert_eq!(plot.data.series().len(), 1);
        assert!(plot.data.series()[0].samples()[6].y.is_nan());
    });
    plot.update(cx, |plot, _| plot.motion.snap_to(0.6));
    replay.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
    cx.run_until_parked();
    plot.read_with(cx, |plot, _| {
        assert_eq!(plot.motion.progress(), 0.0);
        assert!(plot.motion.is_animating());
    });
}
