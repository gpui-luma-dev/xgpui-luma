use gpui::{App, KeyBinding, actions};

actions!(
    luma_controls,
    [
        ActivateControl,
        OpenControl,
        CommitSelection,
        SelectNextItem,
        SelectPreviousItem,
        SelectNextRow,
        SelectPreviousRow,
        SelectFirstItem,
        SelectLastItem,
        OpenSubmenu,
        CloseSubmenu,
        IncreaseValue,
        DecreaseValue,
        IncreaseValueLarge,
        DecreaseValueLarge,
        MoveToStart,
        MoveToEnd,
        RemoveValue,
        OpenContextMenu
    ]
);

const COMMAND_CONTEXT: &str = "LumaCommandControl";
const CHOICE_CONTEXT: &str = "LumaChoiceControl";
const RANGE_VALUE_CONTEXT: &str = "LumaRangeValue";
const SCROLL_OFFSET_CONTEXT: &str = "LumaScrollOffset";
const MENU_CONTROL_CONTEXT: &str = "LumaMenuControl";
const SELECTOR_CONTROL_CONTEXT: &str = "LumaSelectorControl";
const CONTEXT_MENU_CONTROL_CONTEXT: &str = "LumaContextMenuControl";
const NAVIGATION_CONTEXT: &str = "LumaNavigationControl";
const TAB_LIST_CONTEXT: &str = "LumaTabList";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ControlKeyProfile {
    Command,
    Choice,
    RangeValue,
    ScrollOffset,
    Menu,
    Selector,
    TreeView,
    TreeViewExtended,
    ContextMenu,
    Navigation,
    TabList,
}

impl ControlKeyProfile {
    pub const fn context(self) -> &'static str {
        match self {
            Self::Command => COMMAND_CONTEXT,
            Self::Choice => CHOICE_CONTEXT,
            Self::RangeValue => RANGE_VALUE_CONTEXT,
            Self::ScrollOffset => SCROLL_OFFSET_CONTEXT,
            Self::Menu => MENU_CONTROL_CONTEXT,
            Self::Selector => SELECTOR_CONTROL_CONTEXT,
            Self::TreeView => "LumaTreeView",
            Self::TreeViewExtended => "LumaTreeViewExtended",
            Self::ContextMenu => CONTEXT_MENU_CONTROL_CONTEXT,
            Self::Navigation => NAVIGATION_CONTEXT,
            Self::TabList => TAB_LIST_CONTEXT,
        }
    }

    pub fn default_bindings(self) -> Vec<KeyBinding> {
        let context = self.context();

        match self {
            Self::Command => vec![
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::Choice => vec![KeyBinding::new("space", ActivateControl, Some(context))],
            Self::RangeValue => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("down", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("up", IncreaseValue, Some(context)),
                KeyBinding::new("pagedown", DecreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", IncreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
                KeyBinding::new("delete", RemoveValue, Some(context)),
                KeyBinding::new("backspace", RemoveValue, Some(context)),
            ],
            Self::ScrollOffset => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("up", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("down", IncreaseValue, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
            ],
            Self::Menu => vec![
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-up", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-down", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::Selector => vec![
                KeyBinding::new("escape", crate::focus::EscapeFocus, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-up", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-down", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::TreeView => vec![
                KeyBinding::new("escape", crate::focus::EscapeFocus, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-up", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-down", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("right", crate::controls::tree_view::ExpandNode, Some(context)),
                KeyBinding::new("left", crate::controls::tree_view::CollapseNode, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::TreeViewExtended => vec![
                KeyBinding::new("escape", crate::focus::EscapeFocus, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("right", crate::controls::tree_view::ExpandNode, Some(context)),
                KeyBinding::new("left", crate::controls::tree_view::CollapseNode, Some(context)),
            ],
            Self::ContextMenu => vec![
                KeyBinding::new("shift-f10", OpenContextMenu, Some(context)),
                KeyBinding::new("menu", OpenContextMenu, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-up", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-down", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::Navigation => vec![
                KeyBinding::new("escape", crate::focus::EscapeFocus, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-up", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-down", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::TabList => vec![
                KeyBinding::new("escape", crate::focus::EscapeFocus, Some(context)),
                KeyBinding::new("left", SelectPreviousItem, Some(context)),
                KeyBinding::new("right", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousRow, Some(context)),
                KeyBinding::new("down", SelectNextRow, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("cmd-left", SelectFirstItem, Some(context)),
                KeyBinding::new("cmd-right", SelectLastItem, Some(context)),
                KeyBinding::new("ctrl-a", SelectFirstItem, Some(context)),
                KeyBinding::new("ctrl-e", SelectLastItem, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
        }
    }
}

pub fn bind_default_control_keys(cx: &mut App) {
    cx.bind_keys(default_control_key_bindings());
}

pub fn default_control_key_bindings() -> Vec<KeyBinding> {
    [
        ControlKeyProfile::Command,
        ControlKeyProfile::Choice,
        ControlKeyProfile::RangeValue,
        ControlKeyProfile::ScrollOffset,
        ControlKeyProfile::Menu,
        ControlKeyProfile::Selector,
        ControlKeyProfile::TreeView,
        ControlKeyProfile::TreeViewExtended,
        ControlKeyProfile::ContextMenu,
        ControlKeyProfile::Navigation,
        ControlKeyProfile::TabList,
    ]
    .into_iter()
    .flat_map(ControlKeyProfile::default_bindings)
    .collect()
}

#[cfg(test)]
mod tests {
    use gpui::{Action, KeyBinding, Keystroke};

    use super::{
        ActivateControl, CloseSubmenu, ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue,
        IncreaseValueLarge, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem, SelectNextRow,
        SelectPreviousItem, SelectPreviousRow, default_control_key_bindings,
    };

    #[test]
    fn default_control_key_bindings_are_parseable() {
        assert_eq!(default_control_key_bindings().len(), 106);
    }

    #[test]
    fn profile_binding_counts_are_stable() {
        assert_eq!(ControlKeyProfile::Command.default_bindings().len(), 2);
        assert_eq!(ControlKeyProfile::Choice.default_bindings().len(), 1);
        assert_eq!(ControlKeyProfile::RangeValue.default_bindings().len(), 10);
        assert_eq!(ControlKeyProfile::ScrollOffset.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::Menu.default_bindings().len(), 12);
        assert_eq!(ControlKeyProfile::Selector.default_bindings().len(), 13);
        assert_eq!(ControlKeyProfile::TreeView.default_bindings().len(), 15);
        assert_eq!(ControlKeyProfile::TreeViewExtended.default_bindings().len(), 5);
        assert_eq!(ControlKeyProfile::ContextMenu.default_bindings().len(), 14);
        assert_eq!(ControlKeyProfile::Navigation.default_bindings().len(), 13);
        assert_eq!(ControlKeyProfile::TabList.default_bindings().len(), 13);
    }

    #[test]
    fn range_value_and_scroll_offset_profiles_have_different_vertical_semantics() {
        let range = ControlKeyProfile::RangeValue.default_bindings();
        let scroll = ControlKeyProfile::ScrollOffset.default_bindings();

        assert!(has_binding::<DecreaseValue>(&range, "down"));
        assert!(has_binding::<IncreaseValueLarge>(&range, "pageup"));
        assert!(has_binding::<IncreaseValue>(&scroll, "down"));
        assert!(has_binding::<DecreaseValueLarge>(&scroll, "pageup"));
    }

    #[test]
    fn navigation_profile_binds_collection_accelerators() {
        let bindings = ControlKeyProfile::Navigation.default_bindings();

        assert!(has_binding::<SelectPreviousItem>(&bindings, "up"));
        assert!(has_binding::<SelectNextItem>(&bindings, "down"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "home"));
        assert!(has_binding::<SelectLastItem>(&bindings, "end"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "cmd-up"));
        assert!(has_binding::<SelectLastItem>(&bindings, "cmd-down"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "ctrl-a"));
        assert!(has_binding::<SelectLastItem>(&bindings, "ctrl-e"));
        assert!(has_binding::<OpenSubmenu>(&bindings, "right"));
        assert!(has_binding::<CloseSubmenu>(&bindings, "left"));
        assert!(has_binding::<ActivateControl>(&bindings, "enter"));
        assert!(has_binding::<ActivateControl>(&bindings, "space"));
    }

    #[test]
    fn tab_list_profile_binds_roving_tab_accelerators() {
        let bindings = ControlKeyProfile::TabList.default_bindings();

        assert!(has_binding::<SelectPreviousItem>(&bindings, "left"));
        assert!(has_binding::<SelectPreviousRow>(&bindings, "up"));
        assert!(has_binding::<SelectNextItem>(&bindings, "right"));
        assert!(has_binding::<SelectNextRow>(&bindings, "down"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "home"));
        assert!(has_binding::<SelectLastItem>(&bindings, "end"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "cmd-left"));
        assert!(has_binding::<SelectLastItem>(&bindings, "cmd-right"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "ctrl-a"));
        assert!(has_binding::<SelectLastItem>(&bindings, "ctrl-e"));
        assert!(has_binding::<ActivateControl>(&bindings, "enter"));
        assert!(has_binding::<ActivateControl>(&bindings, "space"));
    }

    fn has_binding<A: Action>(bindings: &[KeyBinding], keystrokes: &'static str) -> bool {
        let expected = Keystroke::parse(keystrokes).expect("valid test keystroke");
        bindings.iter().any(|binding| {
            binding.keystrokes().len() == 1
                && {
                    let actual = binding.keystrokes()[0].inner();
                    actual.key == expected.key && actual.modifiers == expected.modifiers
                }
                && binding.action().as_any().is::<A>()
        })
    }
}
