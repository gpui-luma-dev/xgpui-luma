//! Pure stop-list model: ordering, count limits, and edit operations.
//!
//! The model has no GPUI dependencies so every edit rule is unit-testable.
//! Positioned lists (gradient stops) are kept sorted by position; unpositioned
//! lists (mesh points, palette colors) keep insertion order.

use xgpui_luma::color::ColorValue;

/// Stable identity for one stop. Keys are allocated by the model and never reused.
pub(crate) type StopKey = u64;

/// Smallest distance kept between neighbouring positioned stops when inserting.
const MIN_INSERT_GAP: f32 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StopData {
    pub key: StopKey,
    pub color: ColorValue,
    /// Normalized `0.0..=1.0`; `None` for unpositioned lists.
    pub position: Option<f32>,
}

/// Inclusive bounds on how many stops the list may hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CountLimits {
    pub min: usize,
    pub max: usize,
}

impl Default for CountLimits {
    fn default() -> Self {
        Self { min: 1, max: usize::MAX }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StopModel {
    stops: Vec<StopData>,
    next_key: StopKey,
    limits: CountLimits,
    positioned: bool,
}

impl StopModel {
    pub fn new(
        entries: impl IntoIterator<Item = (ColorValue, Option<f32>)>,
        limits: CountLimits,
        positioned: bool,
    ) -> Self {
        let mut model = Self { stops: Vec::new(), next_key: 1, limits, positioned };
        model.replace(entries);
        model
    }

    pub fn stops(&self) -> &[StopData] {
        &self.stops
    }

    pub fn len(&self) -> usize {
        self.stops.len()
    }

    pub fn get(&self, key: StopKey) -> Option<&StopData> {
        self.stops.iter().find(|stop| stop.key == key)
    }

    pub fn index_of(&self, key: StopKey) -> Option<usize> {
        self.stops.iter().position(|stop| stop.key == key)
    }

    pub fn can_add(&self) -> bool {
        self.stops.len() < self.limits.max
    }

    pub fn can_remove(&self) -> bool {
        self.stops.len() > self.limits.min
    }

    /// Replace all stops. Keys are freshly allocated; positions are clamped and sorted.
    pub fn replace(&mut self, entries: impl IntoIterator<Item = (ColorValue, Option<f32>)>) {
        let positioned = self.positioned;
        self.stops = entries
            .into_iter()
            .map(|(color, position)| {
                let key = self.next_key;
                self.next_key += 1;
                let position = positioned.then(|| position.unwrap_or(0.0).clamp(0.0, 1.0));
                StopData { key, color, position }
            })
            .collect();
        self.sort();
    }

    /// Replace all stops with caller-owned keys (e.g. a host's own ids).
    ///
    /// Duplicate keys keep their first entry. Later [`Self::add`] calls
    /// allocate keys above every supplied key.
    pub fn replace_keyed(&mut self, stops: impl IntoIterator<Item = StopData>) {
        let positioned = self.positioned;
        let mut next: Vec<StopData> = Vec::new();
        for stop in stops {
            if next.iter().any(|existing| existing.key == stop.key) {
                continue;
            }
            let position = positioned.then(|| stop.position.unwrap_or(0.0).clamp(0.0, 1.0));
            next.push(StopData { position, ..stop });
        }
        if let Some(max) = next.iter().map(|stop| stop.key).max() {
            self.next_key = self.next_key.max(max.saturating_add(1));
        }
        self.stops = next;
        self.sort();
    }

    /// Insert a stop next to `anchor` (or into the widest gap when `None`).
    ///
    /// Positioned lists insert at the midpoint of the chosen gap with the
    /// interpolated color; unpositioned lists duplicate the anchor color after it.
    pub fn add(&mut self, anchor: Option<StopKey>) -> Option<StopKey> {
        if !self.can_add() {
            return None;
        }
        let key = self.next_key;
        let (index, color, position) = if self.positioned {
            let (left, right) = self.insert_gap(anchor)?;
            let (a, b) = (&self.stops[left], &self.stops[right]);
            let (pa, pb) = (a.position.unwrap_or(0.0), b.position.unwrap_or(1.0));
            if pb - pa < MIN_INSERT_GAP * 2.0 {
                return None;
            }
            (right, mix_colors(a.color, b.color, 0.5), Some((pa + pb) * 0.5))
        } else {
            let anchor_index = anchor.and_then(|key| self.index_of(key)).unwrap_or(self.stops.len().saturating_sub(1));
            let color =
                self.stops.get(anchor_index).map(|stop| stop.color).unwrap_or(ColorValue::srgb(0.5, 0.5, 0.5, 1.0));
            ((anchor_index + 1).min(self.stops.len()), color, None)
        };
        self.next_key += 1;
        self.stops.insert(index, StopData { key, color, position });
        Some(key)
    }

    pub fn remove(&mut self, key: StopKey) -> bool {
        if !self.can_remove() {
            return false;
        }
        let Some(index) = self.index_of(key) else {
            return false;
        };
        self.stops.remove(index);
        true
    }

    pub fn set_color(&mut self, key: StopKey, color: ColorValue) -> bool {
        match self.stops.iter_mut().find(|stop| stop.key == key) {
            Some(stop) if stop.color != color => {
                stop.color = color;
                true
            }
            _ => false,
        }
    }

    /// Move a positioned stop; returns the clamped position actually applied.
    // Unused while stop positions are display-only (#131).
    #[allow(dead_code)]
    pub fn set_position(&mut self, key: StopKey, position: f32) -> Option<f32> {
        if !self.positioned {
            return None;
        }
        let position = position.clamp(0.0, 1.0);
        let stop = self.stops.iter_mut().find(|stop| stop.key == key)?;
        if stop.position == Some(position) {
            return None;
        }
        stop.position = Some(position);
        self.sort();
        Some(position)
    }

    /// Apply a new order (a permutation of the current keys).
    ///
    /// Positioned lists keep their sorted position slots and move colors
    /// through them, so dragging a row changes which color sits at each stop.
    pub fn reorder(&mut self, order: &[StopKey]) -> bool {
        if order.len() != self.stops.len() || order.iter().any(|key| self.get(*key).is_none()) {
            return false;
        }
        let current: Vec<StopKey> = self.stops.iter().map(|stop| stop.key).collect();
        if current == order {
            return false;
        }
        let slots: Vec<Option<f32>> = self.stops.iter().map(|stop| stop.position).collect();
        let mut next: Vec<StopData> = order.iter().filter_map(|key| self.get(*key).copied()).collect();
        if next.len() != slots.len() {
            return false;
        }
        for (stop, position) in next.iter_mut().zip(slots) {
            stop.position = position;
        }
        self.stops = next;
        true
    }

    /// Mirror the list: positions become `1 - p` and order reverses.
    pub fn reverse(&mut self) -> bool {
        if self.stops.len() < 2 {
            return false;
        }
        self.stops.reverse();
        for stop in &mut self.stops {
            stop.position = stop.position.map(|p| 1.0 - p);
        }
        true
    }

    /// Space positioned stops evenly across `0..=1`, keeping their order.
    pub fn distribute(&mut self) -> bool {
        if !self.positioned || self.stops.len() < 2 {
            return false;
        }
        let last = (self.stops.len() - 1) as f32;
        let mut changed = false;
        for (index, stop) in self.stops.iter_mut().enumerate() {
            let position = Some(index as f32 / last);
            changed |= stop.position != position;
            stop.position = position;
        }
        changed
    }

    fn sort(&mut self) {
        if self.positioned {
            // Stable: equal positions keep their current relative order.
            self.stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    /// Neighbouring indices `(left, right)` bounding the insertion gap.
    fn insert_gap(&self, anchor: Option<StopKey>) -> Option<(usize, usize)> {
        if self.stops.len() < 2 {
            return None;
        }
        if let Some(index) = anchor.and_then(|key| self.index_of(key)) {
            return Some(if index + 1 < self.stops.len() {
                (index, index + 1)
            } else {
                (index - 1, index)
            });
        }
        (0..self.stops.len() - 1)
            .max_by(|&a, &b| {
                let gap = |i: usize| self.stops[i + 1].position.unwrap_or(1.0) - self.stops[i].position.unwrap_or(0.0);
                gap(a).partial_cmp(&gap(b)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|left| (left, left + 1))
    }
}

/// Seed an inserted stop's color: straight interpolation of encoded RGB, with
/// no gamut mapping. Two sRGB-tagged stops mix in encoded sRGB (staying sRGB);
/// any other pair mixes in encoded Display P3, the gradient demos' wide working
/// space, so P3 stops are never clipped to sRGB here.
pub(crate) fn mix_colors(a: ColorValue, b: ColorValue, t: f32) -> ColorValue {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    if let (ColorValue::Srgb(a), ColorValue::Srgb(b)) = (a, b) {
        return ColorValue::srgb(
            lerp(a.red, b.red),
            lerp(a.green, b.green),
            lerp(a.blue, b.blue),
            lerp(a.alpha, b.alpha),
        );
    }
    let (Ok(pa), Ok(pb)) = (a.to_display_p3_unclamped(), b.to_display_p3_unclamped()) else {
        return a;
    };
    ColorValue::display_p3(
        lerp(pa.red, pb.red),
        lerp(pa.green, pb.green),
        lerp(pa.blue, pb.blue),
        lerp(pa.alpha, pb.alpha),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray(v: f32) -> ColorValue {
        ColorValue::srgb(v, v, v, 1.0)
    }

    fn gradient() -> StopModel {
        StopModel::new(
            [(gray(0.0), Some(0.0)), (gray(0.5), Some(0.5)), (gray(1.0), Some(1.0))],
            CountLimits { min: 2, max: 4 },
            true,
        )
    }

    fn positions(model: &StopModel) -> Vec<f32> {
        model.stops().iter().map(|stop| stop.position.unwrap()).collect()
    }

    #[test]
    fn replace_clamps_sorts_and_allocates_unique_keys() {
        let model = StopModel::new([(gray(1.0), Some(1.4)), (gray(0.0), Some(-0.2))], CountLimits::default(), true);
        assert_eq!(positions(&model), vec![0.0, 1.0]);
        assert_ne!(model.stops()[0].key, model.stops()[1].key);
    }

    #[test]
    fn replace_keyed_keeps_host_keys_sorts_and_allocates_above_them() {
        let mut model = gradient();
        let stop = |key, position| StopData { key, color: gray(0.5), position: Some(position) };
        model.replace_keyed([stop(40, 1.0), stop(7, 0.0), stop(7, 0.5), stop(12, 0.5)]);
        let keys: Vec<StopKey> = model.stops().iter().map(|stop| stop.key).collect();
        assert_eq!(keys, vec![7, 12, 40]);
        assert_eq!(positions(&model), vec![0.0, 0.5, 1.0]);
        let added = model.add(Some(7)).expect("room to add");
        assert!(added > 40);
    }

    #[test]
    fn add_inserts_midpoint_after_anchor_and_respects_max() {
        let mut model = gradient();
        let first = model.stops()[0].key;
        let added = model.add(Some(first)).expect("room for a fourth stop");
        assert_eq!(positions(&model), vec![0.0, 0.25, 0.5, 1.0]);
        assert_eq!(model.index_of(added), Some(1));
        assert!(model.add(None).is_none(), "max count is 4");
    }

    #[test]
    fn add_without_anchor_uses_widest_gap() {
        let mut model = StopModel::new(
            [(gray(0.0), Some(0.0)), (gray(0.2), Some(0.2)), (gray(1.0), Some(1.0))],
            CountLimits::default(),
            true,
        );
        model.add(None);
        assert_eq!(positions(&model), vec![0.0, 0.2, 0.6, 1.0]);
    }

    #[test]
    fn remove_respects_min_count() {
        let mut model = gradient();
        let keys: Vec<_> = model.stops().iter().map(|stop| stop.key).collect();
        assert!(model.remove(keys[1]));
        assert!(!model.remove(keys[0]), "min count is 2");
        assert_eq!(model.len(), 2);
    }

    #[test]
    fn set_position_resorts_and_reports_clamped_value() {
        let mut model = gradient();
        let first = model.stops()[0].key;
        assert_eq!(model.set_position(first, 0.75), Some(0.75));
        assert_eq!(model.index_of(first), Some(1));
        assert_eq!(model.set_position(first, 2.0), Some(1.0));
        assert_eq!(model.set_position(first, 1.0), None, "unchanged position is not an edit");
    }

    #[test]
    fn reverse_mirrors_positions_and_order() {
        let mut model = StopModel::new([(gray(0.0), Some(0.0)), (gray(1.0), Some(0.3))], CountLimits::default(), true);
        let first = model.stops()[0].key;
        assert!(model.reverse());
        assert_eq!(positions(&model), vec![0.7, 1.0]);
        assert_eq!(model.index_of(first), Some(1));
    }

    #[test]
    fn distribute_spaces_evenly_and_ignores_unpositioned_lists() {
        let mut model = StopModel::new(
            [(gray(0.0), Some(0.1)), (gray(0.5), Some(0.2)), (gray(1.0), Some(0.9))],
            CountLimits::default(),
            true,
        );
        assert!(model.distribute());
        assert_eq!(positions(&model), vec![0.0, 0.5, 1.0]);
        assert!(!model.distribute(), "already distributed");

        let mut palette = StopModel::new([(gray(0.0), None), (gray(1.0), None)], CountLimits::default(), false);
        assert!(!palette.distribute());
        assert!(palette.stops().iter().all(|stop| stop.position.is_none()));
    }

    #[test]
    fn reorder_moves_colors_through_fixed_position_slots() {
        let mut model = gradient();
        let keys: Vec<_> = model.stops().iter().map(|stop| stop.key).collect();
        assert!(model.reorder(&[keys[2], keys[0], keys[1]]));
        assert_eq!(positions(&model), vec![0.0, 0.5, 1.0], "slots are unchanged");
        assert_eq!(model.stops()[0].color, gray(1.0), "last color moved to the first slot");
        assert!(!model.reorder(&[keys[0], keys[1]]), "must be a full permutation");
        assert!(!model.reorder(&[keys[2], keys[0], keys[1]]), "same order is not an edit");
    }

    #[test]
    fn unpositioned_add_duplicates_anchor_after_it() {
        let mut palette = StopModel::new([(gray(0.0), None), (gray(1.0), None)], CountLimits::default(), false);
        let first = palette.stops()[0].key;
        let added = palette.add(Some(first)).unwrap();
        assert_eq!(palette.index_of(added), Some(1));
        assert_eq!(palette.get(added).unwrap().color, gray(0.0));
    }

    #[test]
    fn inserted_stop_between_p3_stops_stays_unclipped() {
        let red = ColorValue::display_p3(1.0, 0.0, 0.0, 1.0);
        let mut model = StopModel::new([(red, Some(0.0)), (red, Some(1.0))], CountLimits::default(), true);
        let key = model.add(None).unwrap();
        let color = model.get(key).unwrap().color;
        assert_eq!(color, ColorValue::display_p3(1.0, 0.0, 0.0, 1.0));
        assert!(!color.is_in_gamut(xgpui_luma::color::Gamut::Srgb).unwrap());
        // Two sRGB stops keep producing sRGB.
        assert!(matches!(mix_colors(gray(0.0), gray(1.0), 0.5), ColorValue::Srgb(_)));
    }
}
