use xgpui_luma::controls::tabs::TabsEventExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, KeyDownEvent, MouseButton, Pixels, Render, ScrollHandle,
    ScrollEvent, Window, div, point, prelude::*, px,
};
use xgpui_luma::controls::sidebar::SidebarControl;
use xgpui_luma::controls::tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode};
use xgpui_luma::infra::ElementExt;
use xgpui_luma::{declare_form};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook};
use crate::studio::panels::sidebar::{INITIAL_PROPERTY_SELECTION_ID, property_sidebar};
use crate::studio::style::sections;
use crate::studio::style::shared::callout::render_sparse_catalog_callout;
use crate::studio::doc_shell::{
    StickySectionHeadingTracker, render_sticky_section_heading_lane, with_sticky_heading_tracker,
};

const STYLE_INDEX_WIDTH: f32 = 148.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StyleGuideSection {
    Sidebar,
    Feedback,
    Buttons,
    IconButtons,
    Checkbox,
    Radio,
    Switch,
    Toggle,
    Menus,
    Pager,
    Selectors,
    Tabs,
    Toolbar,
    Tooltips,
    Scrollbar,
    Slider,
    SplitButton,
    TextField,
    TextArea,
    Table,
    TreeView,
    Accordion,
    Typography,
    ShadowTokens,
}

impl StyleGuideSection {
    const ALL: [Self; 24] = [
        Self::Accordion,
        Self::Buttons,
        Self::Checkbox,
        Self::Feedback,
        Self::IconButtons,
        Self::Table,
        Self::Menus,
        Self::Pager,
        Self::Radio,
        Self::Scrollbar,
        Self::Selectors,
        Self::Sidebar,
        Self::Slider,
        Self::SplitButton,
        Self::Switch,
        Self::Tabs,
        Self::TextArea,
        Self::TextField,
        Self::Toggle,
        Self::Toolbar,
        Self::Tooltips,
        Self::TreeView,
        Self::Typography,
        Self::ShadowTokens,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Accordion => "Accordion",
            Self::Buttons => "Buttons",
            Self::Checkbox => "Checkbox",
            Self::Feedback => "Feedback",
            Self::IconButtons => "Icon Button",
            Self::Table => "Table",
            Self::Menus => "Menus",
            Self::Pager => "Pager",
            Self::Radio => "Radio",
            Self::Scrollbar => "Scrollbar",
            Self::Selectors => "Selectors",
            Self::Sidebar => "Sidebar",
            Self::Slider => "Slider",
            Self::SplitButton => "Split Button",
            Self::Switch => "Switch",
            Self::Tabs => "Tabs",
            Self::TextArea => "Text Area",
            Self::TextField => "Text Field",
            Self::Toggle => "Toggles",
            Self::Toolbar => "Toolbar",
            Self::Tooltips => "Tooltips",
            Self::TreeView => "Tree View",
            Self::Typography => "Typography",
            Self::ShadowTokens => "Shadow Tokens",
        }
    }
}

declare_form! {
    pub struct StyleGuidePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {
            scroll_handle: ScrollHandle = ScrollHandle::new(),
            sticky_heading_tracker: Rc<RefCell<StickySectionHeadingTracker>> =
                Rc::new(RefCell::new(StickySectionHeadingTracker::default())),
            last_scroll_offset: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            last_max_scroll: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            sidebar_preview: Option<Entity<SidebarControl>> = None,
            buttons_preview_tabs: Option<Entity<Tabs>> = None,
            icon_buttons_preview_tabs: Option<Entity<Tabs>> = None,
            checkbox_preview_tabs: Option<Entity<Tabs>> = None,
            radio_preview_tabs: Option<Entity<Tabs>> = None,
            switch_preview_tabs: Option<Entity<Tabs>> = None,
            switch_customization_preview: Option<Entity<sections::switch::customization::SwitchCustomizationPreview>> =
                None,
            toggles_preview_tabs: Option<Entity<Tabs>> = None,
            menus_preview_tabs: Option<Entity<Tabs>> = None,
            pager_preview: Option<Entity<sections::pager::PagerPreview>> = None,
            selectors_preview_tabs: Option<Entity<Tabs>> = None,
            scrollbar_preview_tabs: Option<Entity<Tabs>> = None,
            textfield_preview_tabs: Option<Entity<Tabs>> = None,
            textarea_preview_tabs: Option<Entity<Tabs>> = None,
            slider_preview_tabs: Option<Entity<Tabs>> = None,
            slider_customization_preview: Option<Entity<sections::slider::customization::SliderCustomizationPreview>> =
                None,
            table_preview_tabs: Option<Entity<Tabs>> = None,
            table_preview: Option<Entity<sections::table::TablePreview>> = None,
            tree_view_preview_tabs: Option<Entity<Tabs>> = None,
            tree_view_preview: Option<Entity<sections::tree_view::TreeViewPreview>> = None,
            accordion_preview_tabs: Option<Entity<Tabs>> = None,
            accordion_preview: Option<Entity<sections::accordion::AccordionPreview>> = None,
            toolbar_preview_tabs: Option<Entity<Tabs>> = None,
            tooltip_preview: Option<Entity<sections::tooltips::TooltipPreview>> = None,
            toolbar_preview: Option<Entity<sections::toolbar::ToolbarPreview>> = None,
        }
    }
}

impl StyleGuidePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        if let Some(preview) = &self.tooltip_preview {
            preview.update(cx, |preview, cx| preview.sync_look(self.look.clone(), cx));
        }
        self.sync_sidebar_preview(cx);
        self.sync_buttons_preview_tabs(cx);
        self.sync_icon_buttons_preview_tabs(cx);
        self.sync_checkbox_preview_tabs(cx);
        self.sync_radio_preview_tabs(cx);
        self.sync_switch_preview_tabs(cx);
        self.sync_switch_customization_preview(cx);
        self.sync_toggles_preview_tabs(cx);
        self.sync_menus_preview_tabs(cx);
        self.sync_pager_preview(cx);
        self.sync_selectors_preview_tabs(cx);
        self.sync_scrollbar_preview_tabs(cx);
        self.sync_textfield_preview_tabs(cx);
        self.sync_textarea_preview_tabs(cx);
        self.sync_table_preview_tabs(cx);
        self.sync_table_preview(cx);
        self.sync_tree_view_preview_tabs(cx);
        self.sync_tree_view_preview(cx);
        self.sync_accordion_preview_tabs(cx);
        self.sync_accordion_preview(cx);
        self.sync_toolbar_preview_tabs(cx);
        self.sync_toolbar_preview(cx);
        self.sync_slider_preview_tabs(cx);
        self.sync_slider_customization_preview(cx);
        self.sticky_heading_tracker.borrow_mut().reset();
        cx.notify();
    }

    fn set_vertical_offset(&self, value: f32, cx: &mut Context<Self>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    fn scroll_vertical_by(&self, delta: Pixels, cx: &mut Context<Self>) -> bool {
        let current = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let target = (current + delta.as_f32()).clamp(0.0, max);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    fn sidebar_preview(&mut self, cx: &mut Context<Self>) -> Entity<SidebarControl> {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            return sidebar;
        }

        let look = self.look.clone();
        let sidebar = shadcn::Sidebar::new("luma-studio-style-guide-sidebar-preview")
            .look(look.as_ref())
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .sidebar(property_sidebar(
                &look,
                "luma-studio-style-guide-sidebar-preview-panel",
                "Properties",
                "Task workspace",
            ))
            .spawn(cx);

        self.sidebar_preview = Some(sidebar.clone());
        sidebar
    }

    fn sync_sidebar_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            let look = self.look.clone();
            sidebar.update(cx, move |sidebar, cx| {
                sidebar.set_panel_template(look.sidebar_panel_template(), cx);
                sidebar.set_scrollbar_template(look.scrollbar_template(), cx);
                sidebar.set_sidebar(
                    property_sidebar(
                        &look,
                        "luma-studio-style-guide-sidebar-preview-panel",
                        "Properties",
                        "Task workspace",
                    ),
                    cx,
                );
                sidebar.set_selected_id(INITIAL_PROPERTY_SELECTION_ID, cx);
            });
        }
    }

    fn sync_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_template(), cx);
            });
        }
    }

    fn buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-buttons-preview-tabs")
            .look(self.look.as_ref())
            .items([TabsItem::new("template-preview").label("Template Preview"), TabsItem::new("sizes").label("Sizes")])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_template(), cx);
            });
        }
    }

    fn icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-icon-buttons-preview-tabs")
            .look(self.look.as_ref())
            .items([TabsItem::new("template-preview").label("Template Preview"), TabsItem::new("sizes").label("Sizes")])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.icon_buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_toggles_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.toggles_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_template(), cx);
            });
        }
    }

    fn toggles_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.toggles_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-toggles-preview-tabs")
            .look(self.look.as_ref())
            .items([TabsItem::new("template-preview").label("Template Preview"), TabsItem::new("sizes").label("Sizes")])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.toggles_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_checkbox_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.checkbox_preview_tabs.clone(), cx);
    }

    fn checkbox_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.checkbox_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-checkbox-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.checkbox_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_radio_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.radio_preview_tabs.clone(), cx);
    }

    fn radio_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.radio_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-radio-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.radio_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_switch_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.switch_preview_tabs.clone(), cx);
    }

    fn switch_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.switch_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-switch-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("customization").label("Customization"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.switch_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn switch_customization_preview(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Entity<sections::switch::customization::SwitchCustomizationPreview> {
        if let Some(preview) = self.switch_customization_preview.clone() {
            return preview;
        }

        let preview =
            cx.new(|cx| sections::switch::customization::SwitchCustomizationPreview::new(cx, self.look.clone()));
        self.switch_customization_preview = Some(preview.clone());
        preview
    }

    fn sync_switch_customization_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.switch_customization_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_menus_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.menus_preview_tabs.clone(), cx);
    }

    fn menus_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.menus_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-menus-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("menu-trigger").label("Menu Trigger"),
                TabsItem::new("trigger-sizes").label("Trigger Sizes"),
                TabsItem::new("floating-menu").label("Floating Menu"),
                TabsItem::new("sizes").label("Sizes"),
            ])
            .active("menu-trigger")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.menus_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn pager_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::pager::PagerPreview> {
        if let Some(preview) = self.pager_preview.clone() {
            return preview;
        }

        let preview = cx.new(|cx| sections::pager::PagerPreview::new(cx, self.look.clone()));
        self.pager_preview = Some(preview.clone());
        preview
    }

    fn sync_pager_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.pager_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_scrollbar_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.scrollbar_preview_tabs.clone(), cx);
    }

    fn sync_textfield_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.textfield_preview_tabs.clone(), cx);
    }

    fn sync_textarea_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.textarea_preview_tabs.clone(), cx);
    }

    fn textarea_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.textarea_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-textarea-style-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("textarea-shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.textarea_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_selectors_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.selectors_preview_tabs.clone(), cx);
    }

    fn selectors_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.selectors_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-selectors-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.selectors_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn scrollbar_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.scrollbar_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-scrollbar-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.scrollbar_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn textfield_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.textfield_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-textfield-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("shadows").label("Shadows"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.textfield_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_slider_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.slider_preview_tabs.clone(), cx);
    }

    fn slider_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.slider_preview_tabs.clone() {
            return tabs;
        }

        let tabs = shadcn::Tabs::new("luma-studio-slider-preview-tabs")
            .look(self.look.as_ref())
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("sizes").label("Sizes"),
                TabsItem::new("customization").label("Customization"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        self.slider_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn slider_customization_preview(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Entity<sections::slider::customization::SliderCustomizationPreview> {
        if let Some(preview) = self.slider_customization_preview.clone() {
            return preview;
        }

        let preview =
            cx.new(|cx| sections::slider::customization::SliderCustomizationPreview::new(cx, self.look.clone()));
        self.slider_customization_preview = Some(preview.clone());
        preview
    }

    fn sync_slider_customization_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.slider_customization_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_preview_tabs(&self, tabs: Option<Entity<Tabs>>, cx: &mut Context<Self>) {
        if let Some(tabs) = tabs {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_template(), cx);
            });
        }
    }

    fn spawn_template_sizes_tabs(&self, id: &'static str, cx: &mut Context<Self>) -> Entity<Tabs> {
        let tabs = shadcn::Tabs::new(id)
            .look(self.look.as_ref())
            .items([TabsItem::new("template-preview").label("Template Preview"), TabsItem::new("sizes").label("Sizes")])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .template(self.look.tabs_template())
            .spawn(cx);

        tabs.clone().on_event(cx, |_, _: &TabsEvent, cx| {
            cx.notify();
        });

        tabs
    }

    fn table_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.table_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("luma-studio-table-preview-tabs", cx);
        self.table_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_table_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.table_preview_tabs.clone(), cx);
    }

    fn table_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::table::TablePreview> {
        if let Some(preview) = self.table_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::table::TablePreview::new(cx, self.look.clone()));
        self.table_preview = Some(preview.clone());
        preview
    }

    fn sync_table_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.table_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn tree_view_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.tree_view_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("luma-studio-tree-view-preview-tabs", cx);
        self.tree_view_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_tree_view_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.tree_view_preview_tabs.clone(), cx);
    }

    fn tree_view_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::tree_view::TreeViewPreview> {
        if let Some(preview) = self.tree_view_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::tree_view::TreeViewPreview::new(cx, self.look.clone()));
        self.tree_view_preview = Some(preview.clone());
        preview
    }

    fn sync_tree_view_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.tree_view_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn accordion_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.accordion_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("luma-studio-accordion-preview-tabs", cx);
        self.accordion_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_accordion_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.accordion_preview_tabs.clone(), cx);
    }

    fn accordion_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::accordion::AccordionPreview> {
        if let Some(preview) = self.accordion_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::accordion::AccordionPreview::new(cx, self.look.clone()));
        self.accordion_preview = Some(preview.clone());
        preview
    }

    fn sync_accordion_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.accordion_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn toolbar_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.toolbar_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("luma-studio-toolbar-preview-tabs", cx);
        self.toolbar_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_toolbar_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.toolbar_preview_tabs.clone(), cx);
    }

    fn toolbar_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::toolbar::ToolbarPreview> {
        if let Some(preview) = self.toolbar_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::toolbar::ToolbarPreview::new(cx, self.look.clone()));
        self.toolbar_preview = Some(preview.clone());
        preview
    }

    fn sync_toolbar_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.toolbar_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }
}

impl Render for StyleGuidePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tooltip_preview.is_none() {
            self.tooltip_preview = Some(cx.new(|cx| sections::tooltips::TooltipPreview::new(cx, self.look.clone())));
        }
        let _ = self.sidebar_preview(cx);
        let _ = self.buttons_preview_tabs(cx);
        let _ = self.icon_buttons_preview_tabs(cx);
        let _ = self.checkbox_preview_tabs(cx);
        let _ = self.radio_preview_tabs(cx);
        let _ = self.switch_preview_tabs(cx);
        let _ = self.switch_customization_preview(cx);
        let _ = self.toggles_preview_tabs(cx);
        let _ = self.menus_preview_tabs(cx);
        let _ = self.pager_preview(cx);
        let _ = self.selectors_preview_tabs(cx);
        let _ = self.scrollbar_preview_tabs(cx);
        let _ = self.textfield_preview_tabs(cx);
        let _ = self.textarea_preview_tabs(cx);
        let _ = self.slider_preview_tabs(cx);
        let _ = self.slider_customization_preview(cx);
        let _ = self.table_preview_tabs(cx);
        let _ = self.table_preview(cx);
        let _ = self.tree_view_preview_tabs(cx);
        let _ = self.tree_view_preview(cx);
        let _ = self.accordion_preview_tabs(cx);
        let _ = self.accordion_preview(cx);
        let _ = self.toolbar_preview_tabs(cx);
        let _ = self.toolbar_preview(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let scroll_handle = self.scroll_handle.clone();
            let last_scroll_offset = self.last_scroll_offset.clone();
            let last_max_scroll = self.last_max_scroll.clone();
            let sticky_heading_tracker = self.sticky_heading_tracker.clone();
            let scroll_y = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
            let sticky_snapshot = sticky_heading_tracker.borrow().snapshot(scroll_y);
            let active_section_title = sticky_heading_tracker.borrow().active_title(scroll_y);

            with_sticky_heading_tracker(sticky_heading_tracker.clone(), || {
                div()
                    .id("luma-studio-typography")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .bg(chrome.content_background)
                    .px(px(28.0))
                    .pt(px(12.0))
                    .pb(px(28.0))
                    .child(
                        div()
                            .w_full()
                            .min_h(px(0.0))
                            .flex_1()
                            .flex()
                            .items_stretch()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .child(
                                        div()
                                            .id("style-guide-content")
                                            .size_full()
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(0.0))
                                            .track_scroll(&self.scroll_handle)
                                            .on_prepaint(move |_, window: &mut Window, cx: &mut App| {
                                                let offset = (-scroll_handle.offset().y.as_f32()).max(0.0);
                                                let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                                                let offset_changed = (last_scroll_offset.get() - offset).abs() > 0.5;
                                                let max_changed = (last_max_scroll.get() - max_scroll).abs() > 0.5;

                                                if offset_changed || max_changed {
                                                    last_scroll_offset.set(offset);
                                                    last_max_scroll.set(max_scroll);
                                                    cx.notify(window.current_view());
                                                }
                                            })
                                            .child(
                                                div().w_full().flex().justify_start().pb(px(12.0)).child(
                                                    div()
                                                        .w_full()
                                                        .max_w(px(980.0))
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(14.0))
                                                        .on_prepaint({
                                                            let sticky_heading_tracker = sticky_heading_tracker.clone();
                                                            move |bounds, window, cx| {
                                                                let changed = sticky_heading_tracker
                                                                    .borrow_mut()
                                                                    .set_content_origin_y(bounds.origin.y.as_f32());
                                                                if changed {
                                                                    cx.notify(window.current_view());
                                                                }
                                                            }
                                                        })
                                                        .when_some(
                                                            render_sparse_catalog_callout(self.look.as_ref()),
                                                            |panel, callout| panel.child(callout),
                                                        )
                                                        .children(StyleGuideSection::ALL.into_iter().map(|section| {
                                                            self.render_section_shell(section, window, cx)
                                                        })),
                                                ),
                                            ),
                                    )
                                    .child(render_sticky_section_heading_lane(
                                        sticky_snapshot,
                                        chrome.content_background,
                                        980.0,
                                    )),
                            )
                            .child(self.render_style_index(active_section_title, cx)),
                    )
            })
        })
    }
}

impl StyleGuidePanel {
    fn render_section_shell(
        &self,
        section: StyleGuideSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = self.render_section(section, window, cx);
        let shell = div().relative().w_full().child(content);

        if Self::section_allows_interaction(section) {
            return shell.on_scroll(cx.listener(Self::handle_section_overlay_scroll_wheel)).into_any_element();
        }

        shell
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_mouse_down(MouseButton::Right, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_scroll(cx.listener(Self::handle_section_overlay_scroll_wheel))
                    .on_key_down(cx.listener(Self::handle_section_overlay_key_down)),
            )
            .into_any_element()
    }

    fn section_allows_interaction(section: StyleGuideSection) -> bool {
        matches!(
            section,
            StyleGuideSection::Buttons
                | StyleGuideSection::IconButtons
                | StyleGuideSection::Checkbox
                | StyleGuideSection::Radio
                | StyleGuideSection::Switch
                | StyleGuideSection::Toggle
                | StyleGuideSection::Menus
                | StyleGuideSection::Pager
                | StyleGuideSection::Selectors
                | StyleGuideSection::Scrollbar
                | StyleGuideSection::Slider
                | StyleGuideSection::TextField
                | StyleGuideSection::Table
                | StyleGuideSection::TreeView
                | StyleGuideSection::Accordion
                | StyleGuideSection::Toolbar
                | StyleGuideSection::Tooltips
        )
    }

    fn render_section(&self, section: StyleGuideSection, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match section {
            StyleGuideSection::Tooltips => sections::tooltips::render_tooltip_section(
                &self.look,
                self.tooltip_preview.clone().expect("tooltip preview"),
            ),
            StyleGuideSection::Sidebar => sections::sidebar::render_sidebar_template_section(
                self.sidebar_preview.clone().expect("sidebar preview"),
                self.look.as_ref(),
                cx,
            ),
            StyleGuideSection::Feedback => {
                sections::feedback::render_feedback_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Buttons => sections::buttons::render_button_template_matrix_section(
                self.look.clone(),
                self.buttons_preview_tabs.clone().expect("buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::IconButtons => sections::icon_buttons::render_icon_button_template_matrix_section(
                self.look.clone(),
                self.icon_buttons_preview_tabs.clone().expect("icon buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Checkbox => sections::checkbox::render_checkbox_template_matrix_section(
                self.look.clone(),
                self.checkbox_preview_tabs.clone().expect("checkbox preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Radio => sections::radio::render_radio_template_matrix_section(
                self.look.clone(),
                self.radio_preview_tabs.clone().expect("radio preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Switch => sections::switch::render_switch_template_matrix_section(
                self.look.clone(),
                self.switch_preview_tabs.clone().expect("switch preview tabs"),
                self.switch_customization_preview.clone().expect("switch customization preview"),
                window,
                cx,
            ),
            StyleGuideSection::Toggle => sections::toggle::render_toggle_template_matrix_section(
                self.look.clone(),
                self.toggles_preview_tabs.clone().expect("toggles preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Menus => sections::menus::render_menu_template_state_section(
                self.look.clone(),
                self.menus_preview_tabs.clone().expect("menus preview tabs"),
                Arc::new(cx.listener(Self::handle_section_overlay_scroll_wheel)),
                window,
                cx,
            ),
            StyleGuideSection::Pager => sections::pager::render_pager_template_section(
                self.look.clone(),
                self.pager_preview.clone().expect("pager preview"),
            ),
            StyleGuideSection::Selectors => sections::selectors::render_selector_templates_section(
                self.look.clone(),
                self.selectors_preview_tabs.clone().expect("selectors preview tabs"),
                Arc::new(cx.listener(Self::handle_section_overlay_scroll_wheel)),
                window,
                cx,
            ),
            StyleGuideSection::Tabs => sections::tabs::render_tabs_template_section(self.look.clone(), window, cx),
            StyleGuideSection::Scrollbar => sections::scrollbar::render_scrollbar_template_section(
                self.look.clone(),
                self.scrollbar_preview_tabs.clone().expect("scrollbar preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Slider => sections::slider::render_slider_template_section(
                self.look.clone(),
                self.slider_preview_tabs.clone().expect("slider preview tabs"),
                self.slider_customization_preview.clone().expect("slider customization preview"),
                window,
                cx,
            ),
            StyleGuideSection::SplitButton => {
                sections::split_button::render_split_button_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::TextField => sections::textfield::render_textfield_template_section(
                self.look.clone(),
                self.textfield_preview_tabs.clone().expect("textfield preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::TextArea => sections::textarea::render_textarea_template_section(
                self.look.clone(),
                self.textarea_preview_tabs.clone().expect("textarea preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Table => sections::table::render_table_template_section(
                self.look.clone(),
                self.table_preview_tabs.clone().expect("table preview tabs"),
                self.table_preview.clone().expect("table preview"),
                cx,
            ),
            StyleGuideSection::TreeView => sections::tree_view::render_tree_view_template_section(
                self.look.clone(),
                self.tree_view_preview_tabs.clone().expect("tree view preview tabs"),
                self.tree_view_preview.clone().expect("tree view preview"),
                cx,
            ),
            StyleGuideSection::Accordion => sections::accordion::render_accordion_template_section(
                self.look.clone(),
                self.accordion_preview_tabs.clone().expect("accordion preview tabs"),
                self.accordion_preview.clone().expect("accordion preview"),
                window,
                cx,
            ),
            StyleGuideSection::Toolbar => sections::toolbar::render_toolbar_template_section(
                self.look.clone(),
                self.toolbar_preview_tabs.clone().expect("toolbar preview tabs"),
                self.toolbar_preview.clone().expect("toolbar preview"),
                cx,
            ),
            StyleGuideSection::Typography => sections::typography::render_typography_section(self.look.as_ref()),
            StyleGuideSection::ShadowTokens => sections::shadows::render_shadow_tokens_section(self.look.as_ref()),
        }
    }

    fn render_style_index(&self, active_title: Option<&'static str>, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();

        div()
            .id("style-guide-style-index")
            .w(px(STYLE_INDEX_WIDTH))
            .flex_shrink_0()
            .child(
                div().w_full().h_full().flex().items_center().justify_center().child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .line_height(px(24.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(chrome.title_text)
                                .child("Style Index"),
                        )
                        .child(div().flex().flex_col().gap(px(5.0)).children(StyleGuideSection::ALL.into_iter().map(
                            |section| {
                                render_style_index_item(
                                    section,
                                    section.label(),
                                    active_title == Some(section.label()),
                                    chrome.title_text,
                                    chrome.muted_text,
                                    cx,
                                )
                            },
                        ))),
                ),
            )
            .into_any_element()
    }

    fn handle_style_index_mouse_down(
        &mut self,
        section: StyleGuideSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.prevent_default();
        cx.stop_propagation();

        let Some(top) = self.sticky_heading_tracker.borrow().anchor_top(section.label()) else {
            return;
        };

        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        self.set_vertical_offset(top.clamp(0.0, max), cx);
    }

    fn handle_section_overlay_scroll_wheel(
        &mut self,
        event: &ScrollEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(px(40.0)).y.as_f32();
        if !delta.is_finite() || delta.abs() <= f32::EPSILON {
            return;
        }

        if self.scroll_vertical_by(px(-delta), cx) {
            window.prevent_default();
            cx.stop_propagation();
        }
    }

    fn handle_section_overlay_key_down(&mut self, _: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.prevent_default();
        cx.stop_propagation();
    }
}

fn render_style_index_item(
    section: StyleGuideSection,
    label: &'static str,
    active: bool,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    cx: &mut Context<StyleGuidePanel>,
) -> AnyElement {
    let color = if active { title_text } else { muted_text };
    div()
        .id(format!("style-guide-style-index-item-{label}"))
        .w_full()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(6.0))
        .text_size(px(13.0))
        .line_height(px(17.0))
        .font_weight(if active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(gpui::hsla(color.h, color.s, color.l, if active { 1.0 } else { 0.82 }))
        .when(active, |item| item.bg(gpui::hsla(muted_text.h, muted_text.s, muted_text.l, 0.10)))
        .cursor_pointer()
        .hover(move |style| style.bg(gpui::hsla(muted_text.h, muted_text.s, muted_text.l, 0.12)).text_color(title_text))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| this.handle_style_index_mouse_down(section, window, cx)),
        )
        .child(label)
        .into_any_element()
}
