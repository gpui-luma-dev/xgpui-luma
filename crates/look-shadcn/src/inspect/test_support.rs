#![cfg(test)]

use std::collections::BTreeMap;

use crate::CssTokenMap;

pub fn sample_catalog() -> CssTokenMap {
    CssTokenMap::from_map(BTreeMap::from([
        ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("primary-foreground".into(), "oklch(1 0 0)".into()),
        ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
        ("secondary-foreground".into(), "oklch(1 0 0)".into()),
        ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("card".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
        ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ("accent-foreground".into(), "oklch(1 0 0)".into()),
        ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("sidebar".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("sidebar-primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("sidebar-primary-foreground".into(), "oklch(1 0 0)".into()),
        ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ("sidebar-accent-foreground".into(), "oklch(1 0 0)".into()),
        ("sidebar-border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("sidebar-ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
    ]))
}
