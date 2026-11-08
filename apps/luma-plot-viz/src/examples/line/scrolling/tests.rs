use super::*;
use xgpui_luma::controls::scrollbar::ScrollbarViewport;
use xgpui_luma_plot::{Sample, Series};

#[test]
fn scrollbar_extent_and_thumb_follow_dataset_coordinates() {
    let data = Dataset::new()
        .with_series(Series::new(
            "a",
            "A",
            [Sample::new(-100.0, 1.0), Sample::new(100.0, 2.0), Sample::new(900.0, f64::NAN)],
        ))
        .expect("unique series");
    let (min, max) = time_domain(&data).expect("time range");
    assert_eq!((min, max), (-100.0, 900.0));
    let viewport = ScrollbarViewport::new(min as f32, max as f32, min as f32, (min + WINDOW_SECONDS) as f32);
    assert!((viewport.thumb_fraction() - 0.12).abs() < f32::EPSILON);
    assert_eq!(viewport.scroll_range().start, -100.0);
    assert_eq!(viewport.scroll_range().end, 780.0);
    assert!(time_domain(&Dataset::new()).is_none());
}
