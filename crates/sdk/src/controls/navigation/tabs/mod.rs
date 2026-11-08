//! Tab navigation with optional retained panels and incoming-content fades.
//!
//! Navigation-only `.items(...)` remains supported. Content-bearing tabs render
//! list and body together by default, or expose slots for a custom layout.
//!
//! ```no_run
//! use std::time::Duration;
//! use gpui::{Context, div, prelude::*};
//! use xgpui_luma::{controls::tabs::Tabs, motion::{Transition, Easing}};
//!
//! # fn build<M: 'static>(cx: &mut Context<M>) {
//! let tabs = Tabs::new("settings")
//!     .tab_with("account", "Account", |_, _| div().child("Account settings"))
//!     .tab_with("appearance", "Appearance", |_, _| div().child("Appearance settings"))
//!     .motion(|m| m.content = Transition::timed(Duration::from_millis(300), Easing::OutCubic))
//!     .active("account")
//!     .spawn(cx);
//!
//! // Or place these slots separately in your header and content viewport.
//! let list = tabs.read(cx).tab_list();
//! let body = tabs.read(cx).body();
//! # }
//! ```
//! Content callbacks should retain existing entities; recreating entities inside
//! the callback would reset their state. For a view entity, use `.tab(id, label, view)`.

//! # Builder and state contract
//!
//! Radix and Shadcn look builders forward the same content, selection, and
//! typed `.motion(...)` configuration to this control. Untouched roles inherit
//! the look snapshot. Use `Transition::instant()` for one role or
//! `motion_enabled(false)` to disable every role. Configure before `.spawn(cx)`;
//! existing controls retain their captured profile when the look changes.
//!
//! Selection belongs to the control. An app can mirror it by observing the
//! entity and calling `set_active`; setters do not emit selection events.
//! Selecting the current, disabled, or unknown ID does nothing. Mouse and
//! keyboard selection emit `Change`; activation emits `Activate`, preceded by
//! `Reactivate` when activating the current tab. Focus navigation follows the
//! underlying control group's selection policy: arrows skip disabled items and
//! wrap, selecting and activating the next item; Enter/Space activate the current
//! item. Programmatic selection also updates the keyboard navigation anchor.
//! Item replacement retains the
//! active enabled ID, otherwise chooses the first enabled item or none. It
//! finishes the fade and does not emit selection events.
//!
//! Only the selected presenter runs, on each body render (including fade frames).
//! It is not a once-only factory or a cached element tree. Captured entities
//! remain alive while their content is registered; removal/replacement releases
//! that registration. Keep scroll handles and other page state in retained views
//! or presenter captures. Tabs do not capture or restore page focus or scroll:
//! callers own restoration. Mouse/keyboard activation keeps focus on navigation;
//! programmatic selection does not move focus. A retained `ScrollHandle` preserves
//! its offset when the page returns, subject to normal viewport/content clamping.
//! There is no implicit lazy creation or eviction policy.
//!
//! # Retained view
//!
//! ```no_run
//! use gpui::{Context, Entity, Render};
//! use xgpui_luma::{controls::tabs::Tabs, motion::{Transition, Easing}};
//!
//! fn build<M: 'static, V: Render + 'static>(page: Entity<V>, cx: &mut Context<M>) {
//!     // Construct `page` once, outside the presenter. Its state survives switching.
//!     let tabs = Tabs::new("workspace")
//!         .tab("editor", "Editor", page.clone())
//!         .tab_with("help", "Help", |_, _| gpui::div())
//!         .spawn(cx);
//!     tabs.update(cx, |tabs, cx| tabs.set_active("help", cx));
//! }
//! ```
//!
//! # Explicit page focus restoration
//!
//! ```no_run
//! use gpui::{Context, Entity, Focusable, Render, Window};
//! use xgpui_luma::{controls::tabs::Tabs, motion::{Transition, Easing}};
//!
//! fn restore<V: Render + Focusable + 'static>(
//!     tabs: &Entity<Tabs>, page: &Entity<V>, window: &mut Window, cx: &mut Context<V>,
//! ) {
//!     tabs.update(cx, |tabs, cx| tabs.set_active("editor", cx));
//!     // Call from an app action when entering the page is desired.
//!     // Keep this focus handle and any ScrollHandle in the retained page entity.
//!     let focus = page.read(cx).focus_handle(cx);
//!     focus.focus(window, cx);
//! }
//! ```
//!
//! # App-owned presenter state
//!
//! ```no_run
//! use gpui::{Context, Entity, SharedString, div, prelude::*};
//! use xgpui_luma::{controls::tabs::Tabs, motion::{Transition, Easing}};
//!
//! struct Document { title: SharedString }
//! fn build<M: 'static>(document: Entity<Document>, cx: &mut Context<M>) {
//!     let source = document.clone();
//!     let tabs = Tabs::new("document")
//!         .tab_with("preview", "Preview", move |_, cx| {
//!             div().child(source.read(cx).title.clone())
//!         })
//!         .spawn(cx);
//!     document.update(cx, |document, cx| {
//!         document.title = "Updated".into();
//!         cx.notify();
//!     });
//!     // Or call this from an observer of the app-owned state entity.
//!     tabs.update(cx, |tabs, cx| tabs.refresh_content(cx));
//! }
//! ```
//! `refresh_content`, `set_content`, and `remove_content` invalidate the body
//! without changing selection or restarting a content transition. `set_items` prunes content
//! for removed IDs; new items can receive panels through `set_content`.
//!
//! # Custom content transitions
//! [`TabsTemplate`] styles navigation; [`TabsContentTransition`] renders panels.
//! Timing comes from [`TabsMotion::content`], independently of [`TabsContentLayout`].
//! Choose [`TabsContentLayout::Fill`] explicitly for a bounded sliding viewport.
//! Use [`TabsContentTransition::with_factory`] with [`TabsTransitionRenderer`] for
//! independent effect state per control. Ordered [`TabsTransitionChange`] callbacks
//! expose the prior progress for continuous retargeting across interrupted changes.
//! Retiring panels remain in [`TabsTransitionFrame::outgoing`] until completion,
//! cancellation or removal. Incoming panels stay interactive; inactive panels are
//! shielded and their focus returns to navigation (or the configured focus target).
//! Relative layouts need no measurement; `measure_viewport(true)` opts into bounds
//! and its post-layout redraw. Animation frames invalidate only the body slot.
//!
mod content;
mod transition;
mod control;
mod indicator;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{Tabs, TabsEvent};
pub use content::TabsContent;
pub use transition::{
    TabsContentLayout, TabsContentTransition, TabsTransitionChange, TabsTransitionDirection, TabsTransitionFrame,
    TabsTransitionPanel, TabsTransitionRenderer,
};
use content::{TabsBody, TabsList};
pub use indicator::{TabsIndicatorMotion, TabsIndicatorPaint, TabsIndicatorRect, indicator_rect_from_bounds};
pub use model::{
    TabsMotion, TabsBuilder, TabsItem, TabsItemAccessory, TabsModel, TabsRenderItem, TabsRenderModel, TabsTriggerKind,
    TabsWidthMode,
};
pub use template::{
    TabsBoundsHandler, TabsClickHandler, TabsHoverHandler, TabsItemButtonStyle, TabsItemOverlay, TabsMouseDownHandler,
    TabsMouseUpHandler, TabsOverlayPlacement, TabsOverlayState, TabsTemplate, TabsTemplateHandlers, ThemedTabsTemplate,
    default_tabs_template, render_tab_button, render_tab_button_with_style, render_tabs_overlay_host,
    resolve_tabs_uniform_item_width,
};

pub use theme::{DefaultTabsTheme, TabsBaseline, TabsItemLook, TabsListLook, TabsTheme, default_tabs_theme};

pub use crate::infra::state::{CompositeItemState as TabsItemState, ControlFocusState};

mod callback;
pub use callback::TabsEventExt;

mod drag;
pub use drag::{TabsDragEvent, TabsDropProposal};

#[cfg(all(test, feature = "test-support"))]
mod drag_tests;
