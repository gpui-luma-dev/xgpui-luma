use std::any::{Any, TypeId};
use std::collections::HashMap;

use gpui::{BorrowAppContext, Global};

use crate::theme::{ControlSize, MetricTokens};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LayoutCacheKey {
    pub size: ControlSize,
    pub scale_factor_bits: u32,
}

pub trait LumaLayoutCacheExt: BorrowAppContext {
    fn use_cached_layout<S, F>(&mut self, metrics: MetricTokens, key: LayoutCacheKey, compute: F) -> S
    where
        S: Clone + Send + Sync + 'static,
        F: FnOnce(&MetricTokens) -> S,
    {
        let metrics_key = ThemeMetricsKey::from(&metrics);
        let mut compute = Some(compute);

        self.update_default_global(|registry: &mut LumaLayoutCacheRegistry, _| {
            registry.activate(metrics_key);

            let entry_key = CacheEntryKey { metrics_key, layout_key: key, value_type: TypeId::of::<S>() };
            if let Some(cached) = registry.get::<S>(entry_key) {
                return cached;
            }

            let value = compute.take().expect("layout cache compute closure should only run once")(&metrics);
            registry.insert(entry_key, value.clone());
            value
        })
    }
}

impl<C> LumaLayoutCacheExt for C where C: BorrowAppContext {}

#[derive(Default)]
struct LumaLayoutCacheRegistry {
    active_metrics_key: Option<ThemeMetricsKey>,
    entries: HashMap<CacheEntryKey, Box<dyn Any + Send + Sync>>,
}

impl Global for LumaLayoutCacheRegistry {}

impl LumaLayoutCacheRegistry {
    fn activate(&mut self, metrics_key: ThemeMetricsKey) {
        if self.active_metrics_key != Some(metrics_key) {
            self.entries.clear();
            self.active_metrics_key = Some(metrics_key);
        }
    }

    fn get<S>(&self, key: CacheEntryKey) -> Option<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.entries.get(&key).and_then(|value| value.downcast_ref::<S>()).cloned()
    }

    fn insert<S>(&mut self, key: CacheEntryKey, value: S)
    where
        S: Clone + Send + Sync + 'static,
    {
        self.entries.insert(key, Box::new(value));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct CacheEntryKey {
    metrics_key: ThemeMetricsKey,
    layout_key: LayoutCacheKey,
    value_type: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct ThemeMetricsKey {
    spacing: [u32; 7],
    radius: [u32; 6],
    border_width: [u32; 3],
    focus: [u32; 3],
    control_sm: [u32; 6],
    control_md: [u32; 6],
    control_lg: [u32; 6],
}

impl From<&MetricTokens> for ThemeMetricsKey {
    fn from(metrics: &MetricTokens) -> Self {
        Self {
            spacing: [
                metrics.spacing.s0.to_bits(),
                metrics.spacing.s1.to_bits(),
                metrics.spacing.s2.to_bits(),
                metrics.spacing.s3.to_bits(),
                metrics.spacing.s4.to_bits(),
                metrics.spacing.s5.to_bits(),
                metrics.spacing.s6.to_bits(),
            ],
            radius: [
                metrics.radius.none.to_bits(),
                metrics.radius.sm.to_bits(),
                metrics.radius.md.to_bits(),
                metrics.radius.lg.to_bits(),
                metrics.radius.xl.to_bits(),
                metrics.radius.pill.to_bits(),
            ],
            border_width: [
                metrics.border_width.hairline.to_bits(),
                metrics.border_width.default.to_bits(),
                metrics.border_width.strong.to_bits(),
            ],
            focus: [metrics.focus.gap.to_bits(), metrics.focus.width.to_bits(), metrics.focus.offset.to_bits()],
            control_sm: control_metric_key(metrics.control.sm),
            control_md: control_metric_key(metrics.control.md),
            control_lg: control_metric_key(metrics.control.lg),
        }
    }
}

fn control_metric_key(tokens: crate::theme::ControlMetricTokens) -> [u32; 6] {
    [
        tokens.radius.to_bits(),
        tokens.height.to_bits(),
        tokens.control_height.to_bits(),
        tokens.padding_x.to_bits(),
        tokens.padding_y.to_bits(),
        tokens.gap.to_bits(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{CacheEntryKey, LayoutCacheKey, LumaLayoutCacheRegistry, ThemeMetricsKey};
    use crate::theme::{ControlSize, MetricTokens};
    use std::any::TypeId;

    #[test]
    fn focus_gap_participates_in_the_metrics_signature() {
        let mut metrics = MetricTokens::default();
        let initial = ThemeMetricsKey::from(&metrics);
        metrics.focus.gap += 1.0;
        assert_ne!(ThemeMetricsKey::from(&metrics), initial);
    }

    #[test]
    fn registry_returns_cached_value_for_same_metric_signature() {
        let metrics = MetricTokens::default();
        let metrics_key = ThemeMetricsKey::from(&metrics);
        let entry_key = CacheEntryKey {
            metrics_key,
            layout_key: LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: 1.0f32.to_bits() },
            value_type: TypeId::of::<u32>(),
        };

        let mut registry = LumaLayoutCacheRegistry::default();
        registry.activate(metrics_key);
        registry.insert(entry_key, 7_u32);

        assert_eq!(registry.get::<u32>(entry_key), Some(7));
    }

    #[test]
    fn registry_clears_entries_when_metrics_change() {
        let metrics = MetricTokens::default();
        let mut other_metrics = MetricTokens::default();
        other_metrics.control.md.control_height += 2.0;

        let metrics_key = ThemeMetricsKey::from(&metrics);
        let other_key = ThemeMetricsKey::from(&other_metrics);
        let entry_key = CacheEntryKey {
            metrics_key,
            layout_key: LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: 2.0f32.to_bits() },
            value_type: TypeId::of::<u32>(),
        };

        let mut registry = LumaLayoutCacheRegistry::default();
        registry.activate(metrics_key);
        registry.insert(entry_key, 11_u32);
        registry.activate(other_key);

        assert_eq!(registry.get::<u32>(entry_key), None);
    }
}
