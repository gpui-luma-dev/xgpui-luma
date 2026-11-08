use xgpui_luma_plot::{Dataset, DatasetError, Sample, Series};

pub(super) const LAST_DAY: usize = 90;

/// Deterministic synthetic daily visitor counts, April 1 through June 30.
/// Independent sequences keep Desktop and Mobile distinct.
pub(super) fn dataset() -> Result<Dataset, DatasetError> {
    let mut desktop = Vec::new();
    let mut mobile = Vec::new();
    let mut a = 17_u32;
    let mut b = 103_u32;
    for day in 0..=LAST_DAY {
        a = a.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        b = b.wrapping_mul(22_695_477).wrapping_add(1);
        desktop.push(Sample::new(day as f64, 150.0 + f64::from((a >> 8) % 390)));
        mobile.push(Sample::new(day as f64, 25.0 + f64::from((b >> 8) % 160)));
    }
    Dataset::new()
        .with_series(Series::new("desktop", "Desktop", desktop))?
        .with_series(Series::new("mobile", "Mobile", mobile))
}

pub(super) fn date_label(day: usize) -> String {
    match day {
        0..=29 => format!("Apr {}", day + 1),
        30..=60 => format!("May {}", day - 29),
        _ => format!("Jun {}", day - 60),
    }
}

pub(super) fn first_day(days: usize) -> usize {
    (LAST_DAY + 1).saturating_sub(days)
}

pub(super) fn ticks(days: usize) -> Vec<(f64, String)> {
    let start = first_day(days);
    let step = if days > 30 {
        7
    } else if days > 7 {
        4
    } else {
        1
    };
    let mut ticks: Vec<_> = (start..=LAST_DAY).step_by(step).map(|day| (day as f64, date_label(day))).collect();
    if ticks.last().is_none_or(|(day, _)| *day != LAST_DAY as f64) {
        // Keep the endpoint, replacing a nearby regular tick instead of crowding it.
        if ticks.len() > 1 && ticks.last().is_some_and(|(day, _)| (LAST_DAY as f64 - *day) * 2.0 < step as f64) {
            ticks.pop();
        }
        ticks.push((LAST_DAY as f64, date_label(LAST_DAY)));
    }
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn date_ranges_keep_numeric_day_positions_and_independent_series() {
        assert_eq!(date_label(0), "Apr 1");
        assert_eq!(date_label(30), "May 1");
        assert_eq!(date_label(61), "Jun 1");
        assert_eq!(date_label(90), "Jun 30");
        assert_eq!(first_day(91), 0);
        assert_eq!(first_day(30), 61);
        assert_eq!(ticks(7).len(), 7);
        let month = ticks(30);
        assert_eq!(&month[month.len() - 2..], &[(85.0, "Jun 25".to_owned()), (90.0, "Jun 30".to_owned())]);
        assert!(!month.iter().any(|(_, label)| label == "Jun 29"));
        assert_eq!(ticks(91).last().expect("last tick").1, "Jun 30");
        let data = dataset().expect("dataset");
        assert_eq!(data.series()[0].samples().len(), 91);
        assert_eq!(data.series()[1].samples().len(), 91);
        assert_ne!(data.series()[0].samples(), data.series()[1].samples());
    }
}
