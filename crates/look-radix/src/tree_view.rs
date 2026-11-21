//! Radix scale colors for the SDK's interactive tree view.

use std::sync::Arc;

use gpui::{Context, Div, SharedString, div, prelude::*};
use xgpui_luma::controls::tree_view::{
    TreeNode, TreeViewBuilder, TreeViewPalette, TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate,
};
use xgpui_luma::theme::{InteractionState, MetricTokens};

use crate::{Look, ScaleFamily};

/// The Radix tree view: rows, outline and layout from `.lss` (`style/tree_view.lss`); icons and the drag
/// shadow from the look.
struct RadixTreeViewTheme {
    look: Look,
    styled: xgpui_luma::controls::tree_view::StyledTreeView,
}

/// Neutral frame for a tree viewport. The application supplies its dimensions.
pub fn tree_view_frame(look: &Look) -> Div {
    div()
        .border_1()
        .border_color(look.resolve_step(ScaleFamily::Gray, 6).hsla())
        .bg(look.resolve_step(ScaleFamily::Gray, 1).hsla())
        .overflow_hidden()
}

impl TreeViewTheme for RadixTreeViewTheme {
    fn drag_preview_shadow(&self) -> Vec<gpui::BoxShadow> {
        self.look
            .common_stylesheet()
            .theme_tokens(self.look.mode(), xgpui_luma::color::GamutMapping::CssLocalMinde)
            .map(|tokens| tokens.elevation.drag_preview.to_box_shadows())
            .unwrap_or_default()
    }

    fn icon(&self, role: xgpui_luma::theme::stylesheet::IconRole) -> xgpui_luma::infra::icon::IconSource {
        self.look.common_stylesheet().icons.resolve(role)
    }
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette {
        self.styled.row(state, selected)
    }

    fn row_outline(&self, state: InteractionState) -> Option<gpui::Hsla> {
        self.styled.outline(state)
    }

    fn resolve_scale(&self, scale_factor: f32) -> xgpui_luma::controls::tree_view::TreeViewScale {
        self.styled.scale(scale_factor)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics_for(self.look.size())
    }
}

/// Live theme adapter; rows resolve current colors when rendered.
/// Notify affected views after mutating the look; see [`Look`].
pub fn tree_view_theme(look: &Look) -> Arc<dyn TreeViewTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("tree_view", crate::textfield::live_source(look));
    let binding = if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(RadixTreeViewTheme {
        look: look.clone(),
        styled: xgpui_luma::controls::tree_view::StyledTreeView::new(binding),
    })
}

/// SDK row shells retain selection, disclosure, and keyboard behavior.
pub fn tree_view_template<T: Send + Sync + 'static>(look: &Look) -> Arc<dyn TreeViewTemplate<T>> {
    Arc::new(ThemedTreeViewTemplate::new(tree_view_theme(look)))
}

/// Look-owned tree builder using Radix colors and typography.
pub struct TreeView<T: Clone + Send + Sync + 'static> {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::tree_view::TreeViewEvent<T>>,

    look: Option<Look>,
    builder: TreeViewBuilder<T>,
}

impl<T: Clone + Send + Sync + 'static> TreeView<T> {
    xgpui_luma::forward_motion!(xgpui_luma::controls::tree_view::TreeViewMotion, builder builder);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(enabled);
        self
    }

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewEvent<T>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe the committed `ItemsRemoved` phase before successful completion.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsRemoved {
                source,
                target,
                node_id,
                node_ids,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, cx);
            }
        })
    }

    /// Observe the committed `ItemsAdded` phase before successful completion.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsAdded {
                source,
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe the committed `ItemsReordered` phase before successful completion.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsReordered {
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe `Started` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &gpui::SharedString, &Vec<gpui::SharedString>, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Started { source, node_id, node_ids } = event {
                callback(owner, *source, node_id, node_ids, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Dropped {
                source,
                target,
                node_id,
                node_ids,
                location,
                changed,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, *changed, cx);
            }
        })
    }

    /// Observe `Rejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Rejected {
                source,
                target,
                node_id,
                node_ids,
                error,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, error, cx);
            }
        })
    }

    /// Observe `Ended` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            xgpui_luma::controls::tree_view::TreeDragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Ended { source, node_id, node_ids, reason } =
                event
            {
                callback(owner, *source, node_id, node_ids, *reason, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::DragDrop` through the existing event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeActivated` through the existing event stream.
    pub fn on_node_activated<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeActivated { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionPolicyChanged` through the existing event stream.
    pub fn on_selection_policy_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewSelectionPolicy, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::SelectionPolicyChanged { policy } = event {
                callback(owner, policy, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeExpanded` through the existing event stream.
    pub fn on_node_expanded<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeExpanded { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeCollapsed` through the existing event stream.
    pub fn on_node_collapsed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeCollapsed { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionChanged` through the existing event stream.
    pub fn on_selection_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &std::collections::HashSet<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::SelectionChanged { selected_ids } = event {
                callback(owner, selected_ids, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ActiveNodeChanged` through the existing event stream.
    pub fn on_active_node_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::ActiveNodeChanged { node_id } = event {
                callback(owner, node_id, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ScrollChanged` through the existing event stream.
    pub fn on_scroll_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::RowHoverChanged` through the existing event stream.
    pub fn on_row_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::RowHoverChanged { node_id, index, hovered } = event {
                callback(owner, node_id, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { callbacks: Default::default(), look: None, builder: TreeViewBuilder::new(id) }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set initial nodes. Duplicate IDs retain the previous items, as in the SDK.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    /// Enable scoped, host-owned drag/drop transactions using the SDK policy.
    pub fn drag_drop(mut self, config: xgpui_luma::controls::tree_view::TreeViewDragDrop<T>) -> Self {
        self.builder = self.builder.drag_drop(config);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::tree_view::TreeView<T> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = crate::look::resolve_look(self.look.as_ref(), cx.try_global::<Look>());
            self.builder
                .stylesheet(&look.sized_stylesheet(look.size()))
                .motion_defaults(look.motion_defaults().tree_view)
                .template(tree_view_template(&look))
                .spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn rows_follow_mode_and_custom_accent_changes() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let selected = theme.resolve_row(Default::default(), true);
            crate::control_group::assert_close(
                selected.background.unwrap(),
                look.resolve_step(ScaleFamily::Color, 3).hsla(),
            );
            crate::control_group::assert_close(selected.foreground, look.resolve_step(ScaleFamily::Color, 12).hsla());
            let before = selected.background;
            look.set_accent_seed(xgpui_luma::color::gpui_bridge::from_rgba(gpui::rgb(0xd6409f))).unwrap();
            let custom = theme.resolve_row(Default::default(), true);
            assert_ne!(custom.background, before);
            crate::control_group::assert_close(
                custom.background.unwrap(),
                look.resolve_step(ScaleFamily::Color, 3).hsla(),
            );
            look.set_palettes(crate::Accent::Indigo, crate::Gray::Auto);
        }
    }

    #[test]
    fn hover_outlines_without_replacing_the_selection_fill() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let hovered = InteractionState { hovered: true, ..Default::default() };
            assert_eq!(theme.resolve_row(hovered, false).background, None);
            assert_eq!(
                theme.resolve_row(hovered, true).background,
                theme.resolve_row(Default::default(), true).background,
            );
            crate::control_group::assert_close(
                theme.row_outline(hovered).unwrap(),
                look.resolve_step(ScaleFamily::Color, 8).hsla(),
            );
            assert_eq!(theme.row_outline(Default::default()), None);
            assert_eq!(theme.row_outline(InteractionState { disabled: true, ..hovered }), None);
        }
    }

    #[test]
    fn disabled_overrides_selection_and_interaction() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let state =
                InteractionState { disabled: true, hovered: true, pressed: true, focused: true, invalid: false };
            let row = theme.resolve_row(state, true);
            assert_eq!(row.background, None);
            crate::control_group::assert_close(row.foreground, look.resolve_step(ScaleFamily::Gray, 11).hsla());
        }
    }
}
