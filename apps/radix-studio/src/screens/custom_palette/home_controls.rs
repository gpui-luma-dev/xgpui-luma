//! Home-screen control construction. Bind every example explicitly to the draft look.
use gpui::{Context, Entity, div, prelude::*, px};
use xgpui_luma::controls::{
    button::Button,
    popup_menu::PopupMenu,
    tabs::{Tabs, TabsItem},
    textfield::TextField,
    toolbar::Toolbar,
    tree_view::TreeView,
};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::prelude::TooltipEntityExt;
use xgpui_luma_look_radix::{self as radix, Look};
use super::super::shared::tree_view;

#[derive(Clone)]
pub struct PreviewControls {
    pub tabs: Entity<Tabs>,
    pub toolbar: Toolbar,
    pub actions: Entity<PopupMenu>,
    pub tree: TreeView<()>,
    pub search_field: TextField,
    pub search_submit: Entity<Button>,
    pub tooltip_slider: xgpui_luma::controls::slider::Slider,
    pub sign_up_name: TextField,
    pub sign_up_email: TextField,
    pub sign_up_password: TextField,
    pub create_account: Entity<Button>,
    pub continue_github: Entity<Button>,
    pub icon_samples: super::icon_samples::IconSamples,
    pub task_samples: super::task_samples::TaskSamples,
}

impl PreviewControls {
    pub fn spawn<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let preview_tree = tree_view::spawn("palette-tree", look, true, cx);
        let (preview_toolbar, preview_actions) = super::preview_toolbar::spawn(look, cx);
        let preview_tabs = radix::Tabs::new("palette-preview-tabs")
            .look(look)
            .line()
            .with_template_modifier(|root, _| root.w_full())
            .items([
                TabsItem::new("themes").label("Themes"),
                TabsItem::new("primitives").label("Primitives"),
                TabsItem::new("icons").label("Icons"),
                TabsItem::new("colors").label("Colors"),
            ])
            .active("themes")
            .spawn(cx);
        let search_field = radix::TextField::new("preview-search").look(look).placeholder("Search…").spawn(cx);
        let search_submit = radix::Button::new("preview-search-submit")
            .look(look)
            .solid()
            .label("Submit")
            .spawn(cx)
            .tooltip(super::tooltip_examples::search_help(), cx);
        let tooltip_slider = radix::Slider::new("preview-tooltip-slider")
            .look(look)
            .value(50.0)
            .with_template_modifier(|root, _| root.w_full())
            .spawn(cx)
            .help("Adjust the preview value with the arrow keys", cx);

        let sign_up_name = radix::TextField::new("signup-name").look(look).placeholder("Full name").spawn(cx);
        let sign_up_email = radix::TextField::new("signup-email").look(look).placeholder("Email").spawn(cx);
        let sign_up_password = radix::TextField::new("signup-password").look(look).placeholder("Password").spawn(cx);
        let create_account = radix::Button::new("signup-create").look(look).solid().label("Create account").spawn(cx);
        let continue_github = radix::Button::new("signup-github")
            .look(look)
            .outline()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(model.look.gap))
                    .child(
                        gpui::svg()
                            .path("assets/react-icons/github-logo.svg")
                            .size(px(model.look.icon_size))
                            .text_color(model.look.foreground),
                    )
                    .child("Continue with GitHub")
            })
            .spawn(cx);
        let icon_samples = super::icon_samples::IconSamples::spawn(look, cx);
        let task_samples = super::task_samples::TaskSamples::spawn(look, cx);

        Self {
            tabs: preview_tabs,
            toolbar: preview_toolbar,
            actions: preview_actions,
            tree: preview_tree,
            search_field,
            search_submit,
            tooltip_slider,
            sign_up_name,
            sign_up_email,
            sign_up_password,
            create_account,
            continue_github,
            icon_samples,
            task_samples,
        }
    }

    pub fn notify<M: 'static>(&self, cx: &mut Context<M>) {
        self.icon_samples.notify(cx);
        self.task_samples.notify(cx);
        self.search_submit.update(cx, |_, cx| cx.notify());
        self.tooltip_slider.update(cx, |_, cx| cx.notify());
        self.toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.actions.update(cx, |_, cx| cx.notify());
        self.tabs.update(cx, |_, cx| cx.notify());
    }
}
