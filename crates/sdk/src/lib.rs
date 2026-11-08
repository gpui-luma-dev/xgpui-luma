//! XGPUI-Luma controls, layouts, themes, and shared look contracts.
//!
//! Pick a look crate in your app: `xgpui-luma-look-shadcn` or
//! `xgpui-luma-look-radix`.

#[macro_use]
pub mod macros;

pub mod infra;
pub mod color;
pub mod motion;
pub mod controls;
pub mod focus;
pub mod interaction;
pub mod init;
pub mod key_handling;
pub mod layouts;
pub mod shell;
pub mod theme;
pub mod prelude;

pub use motion::{
    ContinuousPhase, OVERLAY_ENTER_SCALE_MIN, OverlayPresence, PopupLifecycle, VisualTransition, overlay_enter_offset,
    overlay_enter_scale,
};
pub use init::init;
pub use theme::provenance::*;
pub use layouts::{DockPanel, GridLayout, GridTrack, LayerStack, WideMiddle, WideMiddleLayout, spawn_wide_middle};

#[cfg(all(test, feature = "test-support"))]
mod interaction_tests;

#[cfg(all(test, feature = "test-support"))]
mod perf_measure;

#[cfg(all(test, feature = "test-support"))]
#[test]
fn performance_counter_includes_zeroed_allocations_and_reallocations() {
    use std::{
        alloc::{Layout, alloc_zeroed, dealloc, realloc},
        hint::black_box,
    };
    let (_, allocations, bytes) = perf_measure::allocations_for(|| {
        // SAFETY: Matching nonzero layouts are used for each operation, and
        // null is checked before the allocation is accessed or deallocated.
        unsafe {
            let layout = Layout::from_size_align(16, 8).unwrap();
            let ptr = black_box(alloc_zeroed(black_box(layout)));
            assert!(!ptr.is_null());
            let ptr = black_box(realloc(black_box(ptr), layout, 32));
            assert!(!ptr.is_null());
            dealloc(ptr, Layout::from_size_align(32, 8).unwrap());
        }
    });
    assert_eq!((allocations, bytes), (2, 48));
    let (_, allocations, bytes) = perf_measure::allocations_for(|| ());
    assert_eq!((allocations, bytes), (0, 0));
}
