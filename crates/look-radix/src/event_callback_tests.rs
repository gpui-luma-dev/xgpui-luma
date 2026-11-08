//! Headless checks for forwarding callbacks through Look-owned builders.
use gpui::{AppContext, TestAppContext};
#[derive(Default)]
struct Owner {
    all: usize,
    specific: usize,
}
macro_rules! check_look {
    ($test:ident, $builder:expr, $method:ident($($arg:tt),*), $event:expr) => {
        #[test]
        fn $test() {
            let app = TestAppContext::single();
            let owner = app.update(|cx| cx.new(|_| Owner::default()));
            let control = app.update(|cx| owner.update(cx, |_, cx| {
                $builder
                    .on_event(cx, |owner: &mut Owner, _, _| owner.all += 1)
                    .$method(cx, |owner, $($arg,)* _| owner.specific += 1)
                    .spawn(cx)
            }));
            app.update(|cx| control.update(cx, |_, cx| cx.emit($event)));
            app.read(|cx| {
                assert_eq!(owner.read(cx).all, 1);
                assert_eq!(owner.read(cx).specific, 1);
            });
        }
    };
}
check_look!(
    checkbox_group_forwards_callbacks,
    crate::CheckboxGroup::<xgpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_, _, _),
    xgpui_luma::controls::control_group::ControlGroupEvent::Change {
        changed_id: "payload".into(),
        selected: true,
        selected_ids: Vec::new()
    }
);
check_look!(
    toolbar_forwards_callbacks,
    crate::Toolbar::new("callbacks"),
    on_value_changed(_, _),
    xgpui_luma::controls::toolbar::ToolbarEvent::Change {
        id: "payload".into(),
        value: xgpui_luma::controls::toolbar::ToolbarValue::Bool(true)
    }
);
check_look!(
    tree_view_forwards_callbacks,
    crate::TreeView::<()>::new("callbacks"),
    on_focus_changed(_),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::FocusChanged { focused: true }
);
check_look!(
    selector_forwards_callbacks,
    crate::Selector::new("callbacks"),
    on_value_changed(_, _),
    xgpui_luma::controls::selector::SelectorEvent::Change { item_id: "payload".into(), label: "payload".into() }
);
check_look!(
    radio_group_forwards_callbacks,
    crate::RadioGroup::<xgpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_),
    xgpui_luma::controls::radio_group::RadioGroupEvent::Change { value: None }
);
check_look!(
    popup_menu_forwards_callbacks,
    crate::PopupMenu::new("callbacks"),
    on_focus_changed(_),
    xgpui_luma::controls::popup_menu::PopupMenuEvent::FocusChanged { focused: true }
);
check_look!(
    text_area_forwards_callbacks,
    crate::TextArea::new("callbacks"),
    on_value_changed(_),
    xgpui_luma::controls::textarea::TextAreaEvent::Change { value: "payload".into() }
);
check_look!(
    radio_forwards_callbacks,
    crate::Radio::new("callbacks"),
    on_value_changed(_),
    xgpui_luma::controls::radio_button::RadioButtonEvent::Change { selected: true }
);
check_look!(
    text_field_forwards_callbacks,
    crate::TextField::new("callbacks"),
    on_value_changed(_),
    xgpui_luma::controls::textfield::TextFieldEvent::Change { value: "payload".into() }
);
check_look!(
    tabs_forwards_callbacks,
    crate::Tabs::new("callbacks"),
    on_value_changed(_, _),
    xgpui_luma::controls::tabs::TabsEvent::Change { tab_id: "payload".into(), label: "payload".into() }
);
check_look!(
    slider_forwards_callbacks,
    crate::Slider::new("callbacks"),
    on_value_changed(_, _),
    xgpui_luma::controls::slider::SliderEvent::Change {
        thumb_id: xgpui_luma::controls::slider::ThumbId::next(),
        value: 0.75
    }
);
check_look!(
    context_menu_forwards_callbacks,
    crate::ContextMenu::new("callbacks"),
    on_focus_changed(_),
    xgpui_luma::controls::context_menu::ContextMenuEvent::FocusChanged { focused: true }
);

check_look!(
    tree_drag_started_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drag_started(_, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::Started {
            source: gpui::EntityId::from(1),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()]
        }
    )
);

check_look!(
    tree_dropped_forwards,
    crate::TreeView::<()>::new("drag"),
    on_dropped(_, _, _, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::Dropped {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: xgpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: xgpui_luma::controls::tree_view::TreeDropPosition::Before
            },
            changed: false
        }
    )
);

check_look!(
    tree_drop_rejected_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drop_rejected(_, _, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::Rejected {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            error: "denied".into()
        }
    )
);

check_look!(
    tree_drag_ended_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drag_ended(_, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::Ended {
            source: gpui::EntityId::from(1),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            reason: xgpui_luma::controls::tree_view::TreeDragEndReason::Unchanged
        }
    )
);

check_look!(
    tree_items_removed_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_removed(_, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsRemoved {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()]
        }
    )
);

check_look!(
    tree_items_added_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_added(_, _, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsAdded {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: xgpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: xgpui_luma::controls::tree_view::TreeDropPosition::Before
            }
        }
    )
);

check_look!(
    tree_items_reordered_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_reordered(_, _, _, _),
    xgpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsReordered {
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: xgpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: xgpui_luma::controls::tree_view::TreeDropPosition::Before
            }
        }
    )
);

check_look!(
    autocomplete_factory_forwards_callbacks,
    crate::autocomplete_builder(&crate::Look::built_in(), "callbacks", []),
    on_select(_, _),
    xgpui_luma::controls::autocomplete::AutocompleteEvent::Select { item_id: "a".into(), label: "A".into() }
);
check_look!(
    combobox_factory_forwards_callbacks,
    crate::combobox_builder(&crate::Look::built_in(), "callbacks", []),
    on_value_changed(_),
    xgpui_luma::controls::combobox::ComboBoxEvent::Change { query: "a".into() }
);
check_look!(
    search_selector_factory_forwards_callbacks,
    crate::search_selector_builder(&crate::Look::built_in(), "callbacks", []),
    on_select(_, _),
    xgpui_luma::controls::search_selector::SearchSelectorEvent::Select { item_id: "a".into(), label: "A".into() }
);
check_look!(
    selection_panel_factory_forwards_callbacks,
    crate::selection_panel_builder(
        &crate::Look::built_in(),
        "callbacks",
        Vec::<xgpui_luma::controls::selection_panel::SelectionPanelItem>::new()
    ),
    on_activate_row(_, _, _),
    xgpui_luma::controls::selection_panel::SelectionPanelEvent::ActivateRow {
        source_index: 2,
        visible_index: 1,
        item_id: "a".into()
    }
);

check_look!(
    control_group_factory_forwards_callbacks,
    crate::control_group_builder::<xgpui_luma::controls::control_group::ControlGroupItem>(
        &crate::Look::built_in(),
        "callbacks",
        xgpui_luma::theme::ControlSize::Md
    ),
    on_value_changed(_, _, _),
    xgpui_luma::controls::control_group::ControlGroupEvent::Change {
        changed_id: "a".into(),
        selected: true,
        selected_ids: vec!["a".into()]
    }
);
check_look!(
    icon_group_factory_forwards_callbacks,
    crate::icon_group_builder::<xgpui_luma::controls::control_group::ControlGroupItem>(
        &crate::Look::built_in(),
        "callbacks",
        xgpui_luma::theme::ControlSize::Md
    ),
    on_activate(_),
    xgpui_luma::controls::control_group::ControlGroupEvent::Activate { activated_id: "a".into() }
);
check_look!(
    accordion_factory_forwards_callbacks,
    crate::accordion_builder(&crate::Look::built_in(), "callbacks"),
    on_expanded_changed(_, _),
    xgpui_luma::controls::accordion::AccordionEvent::ExpandedChanged { item_id: "a".into(), expanded: true }
);
check_look!(
    pager_factory_forwards_callbacks,
    crate::pager_builder(&crate::Look::built_in(), "callbacks"),
    on_page_changed(_),
    xgpui_luma::controls::pager::PagerEvent::PageChanged { page: 3 }
);
check_look!(
    sidebar_factory_forwards_callbacks,
    crate::sidebar_builder(&crate::Look::built_in(), "callbacks"),
    on_select(_),
    xgpui_luma::controls::sidebar::SidebarEvent::Select { id: "a".into() }
);

check_look!(
    table_factory_forwards_callbacks,
    crate::table_builder::<xgpui_luma::controls::table::TableLabel>(&crate::Look::built_in(), "callbacks"),
    on_selection_changed(_),
    xgpui_luma::controls::table::TableEvent::SelectionChanged { selected_indices: vec![1] }
);

check_look!(
    dock_splitter_factory_forwards_callbacks,
    crate::dock_splitter_builder(
        &crate::Look::built_in(),
        "callbacks",
        xgpui_luma::controls::dock_splitter::SplitterOrientation::Vertical
    ),
    on_resize_delta(_),
    xgpui_luma::controls::dock_splitter::DockSplitterEvent::Resize { total_delta: 17.0 }
);
check_look!(
    split_view_factory_forwards_callbacks,
    crate::split_view_builder(&crate::Look::built_in(), "callbacks"),
    on_collapsed_changed(_),
    xgpui_luma::controls::split_view::SplitViewEvent::CollapsedChanged { collapsed: true }
);

check_look!(
    resizable_panels_factory_forwards_callbacks,
    crate::resizable_panels_builder(&crate::Look::built_in(), "callbacks"),
    on_sizes_changed(_),
    xgpui_luma::controls::resizable_panels::ResizablePanelsEvent::SizesChanged { sizes_px: vec![80.0, 160.0] }
);

check_look!(
    split_button_factory_forwards_action_callbacks,
    crate::split_button_builder(
        &crate::Look::built_in(),
        "callbacks",
        crate::PopupMenuVariant::Solid,
        crate::Tone::Accent
    ),
    on_action_click(),
    xgpui_luma::controls::split_button::SplitButtonEvent::ActionClick
);
check_look!(
    split_button_factory_forwards_menu_callbacks,
    crate::split_button_builder(
        &crate::Look::built_in(),
        "callbacks",
        crate::PopupMenuVariant::Soft,
        crate::Tone::Gray
    ),
    on_select(_, _),
    xgpui_luma::controls::split_button::SplitButtonEvent::Select { item_id: "save-as".into(), label: "Save as".into() }
);
