//! The SDK toolbar on the design surface page (issue #376), shell only. The toolbar's `.lss`
//! (`toolbar`) styles its shell, whose one state is `:disabled`; the buttons, toggles, menus and
//! selectors inside it keep their own `.lss` and stay live, but aren't picked or inspected here (states
//! that belong to items are #302's to design). The page has no options: the toolbar's focus strategy
//! is keyboard behavior, not part of its look.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, SharedString, Subscription, Window, div, prelude::*};
use lucide_svg_static::Icon as LucideIcon;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::selector::SelectorItem;
use xgpui_luma::controls::toolbar::{
    StyledToolbarTheme, Toolbar, ToolbarBuilder, ToolbarEvent, ToolbarItem, ToolbarItemKind, ToolbarRenderModel,
    ToolbarRenderedItem, ToolbarTheme, ToolbarValue, ToolbarVariant, toolbar_template_with_theme,
};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::style::lss::{Resolved, Theme};
use xgpui_luma::style::{StyleBinding, ThemeSource};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

pub(super) struct ToolbarSurface {
    toolbar: Toolbar,
}

impl ToolbarSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        Box::new(Self { toolbar: editing_toolbar(cx) })
    }
}

/// The toolbar shell from a base theme, enabled as the toolbar is or as Design mode picks it.
struct PinnedToolbarTheme {
    base: Arc<dyn ToolbarTheme>,
    /// In Design mode, whether the shell draws enabled (`:disabled` not picked), whatever the live
    /// toolbar is.
    enabled: Option<bool>,
}

impl PinnedToolbarTheme {
    /// The `toolbar` control in `theme`.
    fn new(theme: Theme, enabled: Option<bool>) -> Self {
        let base = StyledToolbarTheme::new(StyleBinding::new("toolbar", ThemeSource::Fixed(theme)));
        Self { base: Arc::new(base), enabled }
    }
}

impl ToolbarTheme for PinnedToolbarTheme {
    fn style(&self) -> StyleBinding {
        self.base.style()
    }

    fn resolve_style(&self, enabled: bool, variant: ToolbarVariant) -> Arc<Resolved> {
        self.base.resolve_style(self.enabled.unwrap_or(enabled), variant)
    }
}

impl SurfaceControl for ToolbarSurface {
    fn title(&self) -> &'static str {
        "Toolbar"
    }

    fn view(&self) -> AnyView {
        self.toolbar.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let enabled = pinned.map(|states| !states.has("disabled"));
        let template = toolbar_template_with_theme(Arc::new(PinnedToolbarTheme::new(theme, enabled)));
        self.toolbar.update(cx, |toolbar, cx| toolbar.set_template(template, cx));
    }

    /// The toolbar stays enabled in Design mode too: disabling it disables its items, which would draw their
    /// own `:disabled` though only the shell is designed here; the shell's `:disabled` is picked (#397).
    fn set_enabled(&self, _enabled: bool, _cx: &mut App) {}

    /// The shell's one state.
    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    /// The toolbar's events are its items' clicks and changes; none sets a shell state.
    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.toolbar, move |side: &mut ControlSide, _, event: &ToolbarEvent, cx| {
            if let Some(line) = format_toolbar_event(event) {
                on_event(side, SurfaceEvent { line, state: None }, cx);
            }
        })
    }

    /// The shell drawn around placeholder items.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let template = toolbar_template_with_theme(Arc::new(PinnedToolbarTheme::new(theme.clone(), None)));
        let model = ToolbarRenderModel {
            id: &id,
            items: Vec::new(),
            enabled: !states.has("disabled"),
            variant: ToolbarVariant::Outline,
        };
        let item = |text: &'static str| ToolbarRenderedItem {
            kind: ToolbarItemKind::Hosted,
            element: div().px_1().child(text).into_any_element(),
        };
        let separator = || ToolbarRenderedItem { kind: ToolbarItemKind::Separator, element: div().into_any_element() };
        let items = vec![item("Heading"), item("Quote"), separator(), item("Typography")];
        template.render(&model, items, window, cx).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// An editing toolbar built only from SDK builders and their default themes.
fn editing_toolbar(cx: &mut Context<ControlSide>) -> Toolbar {
    ToolbarBuilder::new("workshop-sdk-toolbar")
        .item(toolbar_menu(
            "assistant",
            "Assistant",
            LucideIcon::WandSparkles,
            [
                MenuItem::new("rewrite").label("Rewrite"),
                MenuItem::new("summarize").label("Summarize"),
                MenuItem::new("continue").label("Continue writing"),
            ],
            cx,
        ))
        .separator("assistant-separator")
        .item(toolbar_selector(
            "style",
            "Paragraph",
            [
                SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                SelectorItem::new("heading").label("Heading").icon(LucideIcon::TypeIcon),
                SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
            ],
            "paragraph",
            cx,
        ))
        .separator("style-separator")
        .item(toolbar_toggle("bold", LucideIcon::Bold, cx))
        .item(toolbar_toggle("italic", LucideIcon::Italic, cx))
        .item(toolbar_toggle("underline", LucideIcon::Underline, cx))
        .item(toolbar_label_menu(
            "color",
            "A",
            [
                MenuItem::new("default").label("Default color"),
                MenuItem::new("muted").label("Muted"),
                MenuItem::new("accent").label("Accent"),
            ],
            cx,
        ))
        .separator("format-separator")
        .item(toolbar_menu(
            "alignment",
            "Alignment",
            LucideIcon::TextAlignStart,
            [
                MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
            ],
            cx,
        ))
        .separator("insert-separator")
        .item(toolbar_button("link", LucideIcon::Link, cx))
        .item(toolbar_button("image", LucideIcon::Image, cx))
        .item(toolbar_button("video", LucideIcon::Video, cx))
        .item(toolbar_menu(
            "more",
            "More",
            LucideIcon::Ellipsis,
            [
                MenuItem::new("clear").label("Clear formatting"),
                MenuItem::new("divider").label("Insert divider"),
                MenuItem::new("comment").label("Add comment"),
            ],
            cx,
        ))
        .separator("code-separator")
        .item(toolbar_search(cx))
        .item(toolbar_button("code", LucideIcon::Code, cx))
        .spawn(cx)
}

fn format_toolbar_event(event: &ToolbarEvent) -> Option<String> {
    match event {
        ToolbarEvent::Click { id } => Some(format!("ToolbarEvent::Click {{ id: \"{id}\" }}")),
        ToolbarEvent::Change { id, value } => {
            let payload = match value {
                ToolbarValue::Bool(on) => format!("bool({on})"),
                ToolbarValue::String(value) => format!("string(\"{value}\")"),
            };
            Some(format!("ToolbarEvent::Change {{ id: \"{id}\", value: {payload} }}"))
        }
        _ => None,
    }
}

fn toolbar_menu<T: 'static>(
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    items: impl IntoIterator<Item = MenuItem>,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let menu = xgpui_luma::controls::popup_menu::PopupMenuBuilder::new(id)
        .label(label)
        .icon(icon)
        .items(items)
        .spawn(cx);
    ToolbarItem::menu_control(id, menu, cx).label(label)
}

fn toolbar_label_menu<T: 'static>(
    id: &'static str,
    label: &'static str,
    items: impl IntoIterator<Item = MenuItem>,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let menu = xgpui_luma::controls::popup_menu::PopupMenuBuilder::new(id).label(label).items(items).spawn(cx);
    ToolbarItem::menu_control(id, menu, cx).label(label)
}

fn toolbar_selector<T: 'static>(
    id: &'static str,
    label: &'static str,
    items: impl IntoIterator<Item = SelectorItem>,
    selected: &'static str,
    cx: &mut Context<T>,
) -> ToolbarItem {
    let selector = xgpui_luma::controls::selector::SelectorBuilder::new(id)
        .label(label)
        .items(items)
        .selected_id(selected)
        .spawn(cx);
    ToolbarItem::selector_control(id, selector, cx).label(label)
}

fn toolbar_toggle<T: 'static>(id: &'static str, icon: LucideIcon, cx: &mut Context<T>) -> ToolbarItem {
    let toggle = xgpui_luma::controls::toggle::new(id)
        .content(move |model, _| {
            xgpui_luma::infra::icon::lucide_icon(icon, model.look.foreground, model.look.icon_size)
        })
        .spawn(cx);
    ToolbarItem::toggle_button(id, toggle, cx).label(id)
}

fn toolbar_button<T: 'static>(id: &'static str, icon: LucideIcon, cx: &mut Context<T>) -> ToolbarItem {
    let button = Button::new(id)
        .role(xgpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .icon(icon)
        .spawn(cx);
    ToolbarItem::command_button(id, button, cx).label(id)
}

fn toolbar_search<T: 'static>(cx: &mut Context<T>) -> ToolbarItem {
    let field = xgpui_luma::controls::textfield::TextFieldBuilder::new("search").placeholder("Search...").spawn(cx);
    ToolbarItem::textfield_control("search", field, cx)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Entity, TestAppContext, VisualTestContext};

    use super::*;
    use xgpui_luma::controls::toolbar::resolve_toolbar;
    use xgpui_luma::style::PartRead;
    use xgpui_luma::style::lss::{Overrides, PartId, Prop};
    use crate::workshop::surface_page::{SurfacePage, WorkMode};
    use crate::workshop::{Sample, Workshop};

    fn open(app: &mut TestAppContext) -> (Entity<Workshop>, Entity<SurfacePage>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |v, cx| v.select("toolbar", cx));
        cx.run_until_parked();
        let page = cx.update(|_, cx| match &workshop.read(cx).samples["toolbar"] {
            Sample::Surface(v) => v.clone(),
            _ => panic!("the toolbar page is a design surface page"),
        });
        (workshop, page, cx)
    }

    #[test]
    fn the_toolbar_page_shows_the_shell_without_options() {
        let mut app = TestAppContext::single();
        let (_, page, cx) = open(&mut app);
        for id in ["workshop-surface-pane", "workshop-surface-control", "style-inspector"] {
            assert!(cx.debug_bounds(id).is_some(), "{id} rendered");
        }
        assert!(cx.debug_bounds("workshop-style-column").is_none(), "no old inspector column");
        // Focus strategy is keyboard behavior, not look, so the page offers no options.
        assert!(cx.debug_bounds("workshop-surface-options").is_none(), "no focus option");
        let side = cx.update(|_, cx| page.read(cx).side());

        // Shell only: one state, :disabled, picked in Design.
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        assert_eq!(cx.update(|_, cx| side.read(cx).pinned()), Some(StateSet::new(["disabled"])));
        let (columns, rows) = cx.update(|_, cx| side.read(cx).matrix_labels());
        assert_eq!(columns, ["—", ":disabled"]);
        assert_eq!(rows, ["base"]);
        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        let toolbar =
            cx.update(|_, cx| side.read(cx).control_as::<ToolbarSurface>().expect("a toolbar").toolbar.clone());
        assert!(
            cx.update(|_, cx| toolbar.read(cx).is_enabled()),
            "Design keeps the items enabled, not drawn :disabled"
        );
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-1").is_some(), "matrix cells draw the shell");
        crate::workshop::surface_page::assert_matrix_controls_apart(cx);
    }

    #[test]
    fn a_picked_state_decides_the_shell_whatever_the_live_toolbar_is() {
        // A base whose enabled and disabled shells differ (the SDK palette's happen to share a fill).
        struct Marked;
        impl ToolbarTheme for Marked {
            fn resolve_style(&self, enabled: bool, variant: ToolbarVariant) -> Arc<Resolved> {
                let radius = if enabled { 1. } else { 0. };
                let overrides = Overrides::new().set(PartId::ROOT, Prop::<f32>::new("radius"), radius);
                resolve_toolbar(&self.style().with_overrides(&overrides), true, variant)
            }
        }
        let shell = |enabled: Option<bool>, live: bool| {
            PinnedToolbarTheme { base: Arc::new(Marked), enabled }
                .resolve_style(live, ToolbarVariant::Outline)
                .root()
                .number("radius")
        };
        assert_eq!((shell(None, true), shell(None, false)), (1., 0.), "Test: drawn as the toolbar is");
        assert_eq!(shell(Some(true), false), 1., "Design without :disabled draws enabled though input is off");
        assert_eq!(shell(Some(false), true), 0., "Design with :disabled draws disabled");
    }
}
