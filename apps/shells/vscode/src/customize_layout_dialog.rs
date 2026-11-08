use xgpui_luma::controls::overlay_window::OverlayWindowEventExt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, EventEmitter, FocusHandle, Hsla, IntoElement, Render, SharedString, Window, div,
    prelude::*, px,
};
use xgpui_luma::controls::icon_button::IconButton;
use xgpui_luma::controls::button::{ButtonContentContext, ControlIcon, ControlPresenter};
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::overlay_window::{
    OverlayWindow, OverlayWindowDismissPolicy, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition,
    OverlayWindowRenderModel,
};
use xgpui_luma::controls::control_group::{
    ControlGroup, ControlGroupItem, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupItemVisualContext,
};
use xgpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle, observe_theme_revision};
use xgpui_luma::{GridTrack, grid_layout, hstack, vstack};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole};
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::layout_config::{LayoutConfig, LayoutRegion, PanelAlignment, PrimarySideBarPosition};

const DIALOG_W: f32 = 600.0;
const HEADER_H: f32 = 38.0;
const ROW_H: f32 = 27.0;
const CONTENT_PAD_X: f32 = 18.0;
const CONTENT_PAD_TOP: f32 = 8.0;
const SECTION_GAP: f32 = 4.0;
const ROW_RADIUS: f32 = 4.0;
const ICON_COL_W: f32 = 30.0;
const SHORTCUT_COL_W: f32 = 96.0;
const EYE_COL_W: f32 = 30.0;
const POSITION_LABEL_COL_W: f32 = 250.0;
const LAYOUT_ICON_BORDER_W: f32 = 1.0;

pub struct CustomizeLayoutDialog {
    look: Arc<ShadcnLook>,
    overlay: OverlayWindow,
    close_button: IconButton,
    reset_button: IconButton,
    visibility_group: ControlGroup<ControlGroupItem>,
    primary_side_bar_position_group: ControlGroup<ControlGroupItem>,
    panel_alignment_group: ControlGroup<ControlGroupItem>,
}

impl EventEmitter<OverlayWindowEvent> for CustomizeLayoutDialog {}

impl CustomizeLayoutDialog {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let dialog = cx.entity().clone();
        let config = LayoutConfig::default();
        let close_button = shadcn::Button::icon_button("customize-layout-close", LucideIcon::X)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);
        let reset_button = shadcn::Button::icon_button("customize-layout-reset", LucideIcon::RotateCcw)
            .look(look.as_ref())
            .content_only()
            .size(shadcn::ShadcnSize::Sm)
            .spawn(cx);
        sync_header_icon_presenters(&close_button, &reset_button, &look, cx);

        let visibility_group = shadcn::MenuChoiceGroup::new("customize-layout-visibility")
            .look(look.as_ref())
            .multiple()
            .items(visibility_group_items())
            .selected_ids(config.visible_region_ids())
            .with_menu_row_item_content_sized(
                shadcn::ShadcnSize::Sm,
                ROW_H,
                ROW_RADIUS,
                |model, visual, _window, _cx| visibility_row_content(model, visual),
            )
            .spawn(cx);

        let primary_side_bar_position_group =
            shadcn::MenuChoiceGroup::new("customize-layout-primary-side-bar-position")
                .look(look.as_ref())
                .single_required()
                .items([
                    ControlGroupItem::new(PrimarySideBarPosition::Left.id())
                        .label(PrimarySideBarPosition::Left.label()),
                    ControlGroupItem::new(PrimarySideBarPosition::Right.id())
                        .label(PrimarySideBarPosition::Right.label()),
                ])
                .selected(PrimarySideBarPosition::Left.id())
                .with_menu_row_item_content_sized(
                    shadcn::ShadcnSize::Sm,
                    ROW_H,
                    ROW_RADIUS,
                    |model, visual, _window, _cx| position_row_content(model, visual),
                )
                .spawn(cx);

        let panel_alignment_group = shadcn::MenuChoiceGroup::new("customize-layout-panel-alignment")
            .look(look.as_ref())
            .single_required()
            .items([
                ControlGroupItem::new(PanelAlignment::Left.id()).label(PanelAlignment::Left.label()),
                ControlGroupItem::new(PanelAlignment::Right.id()).label(PanelAlignment::Right.label()),
                ControlGroupItem::new(PanelAlignment::Center.id()).label(PanelAlignment::Center.label()),
                ControlGroupItem::new(PanelAlignment::Justify.id()).label(PanelAlignment::Justify.label()),
            ])
            .selected(PanelAlignment::Center.id())
            .with_menu_row_item_content_sized(
                shadcn::ShadcnSize::Sm,
                ROW_H,
                ROW_RADIUS,
                |model, visual, _window, _cx| panel_alignment_row_content(model, visual),
            )
            .spawn(cx);

        let overlay_look = look.clone();

        let overlay = look
            .overlay_window("customize-layout")
            .mode(OverlayWindowMode::Modeless)
            .position(OverlayWindowPosition::Center)
            .draggable(true)
            .size(ControlSize::Sm)
            .dismiss_policy(OverlayWindowDismissPolicy::KeepOpen)
            .width(DIALOG_W)
            .with_template_modifier(move |shell, _| {
                let dialog_background = overlay_look
                    .resolve_secondary_button(Default::default(), ControlSize::Sm, InteractionState::default())
                    .background;
                shell.p_0().overflow_hidden().bg(dialog_background)
            })
            .content(move |overlay_model, window, app| dialog.read(app).render_body(overlay_model, window, app))
            .theme_child(close_button.clone())
            .theme_child(reset_button.clone())
            .theme_child(visibility_group.clone())
            .theme_child(primary_side_bar_position_group.clone())
            .theme_child(panel_alignment_group.clone())
            .spawn(cx);

        overlay.clone().on_event(cx, |_, event: &OverlayWindowEvent, cx| {
            cx.emit(event.clone());
        });

        let this = Self {
            look,
            overlay,
            close_button,
            reset_button,
            visibility_group,
            primary_side_bar_position_group,
            panel_alignment_group,
        };
        observe_theme_revision(cx, |this, cx| {
            sync_header_icon_presenters(&this.close_button, &this.reset_button, &this.look, cx);
            cx.notify();
        })
        .detach();
        this
    }

    pub fn close_button(&self) -> IconButton {
        self.close_button.clone()
    }

    pub fn reset_button(&self) -> IconButton {
        self.reset_button.clone()
    }

    pub fn visibility_group(&self) -> ControlGroup<ControlGroupItem> {
        self.visibility_group.clone()
    }

    pub fn primary_side_bar_position_group(&self) -> ControlGroup<ControlGroupItem> {
        self.primary_side_bar_position_group.clone()
    }

    pub fn panel_alignment_group(&self) -> ControlGroup<ControlGroupItem> {
        self.panel_alignment_group.clone()
    }

    pub fn sync_from_config(&self, config: &LayoutConfig, cx: &mut App) {
        self.sync_visibility_group(config, cx);
        self.sync_position_group(config, cx);
        self.sync_panel_alignment_group(config, cx);
    }

    pub fn open(&mut self, opener: Option<FocusHandle>, cx: &mut App) {
        self.overlay.update(cx, |overlay, cx| overlay.open_from(opener, cx));
    }

    pub fn dismiss(&mut self, cx: &mut App) {
        self.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
    }

    fn sync_visibility_group(&self, config: &LayoutConfig, cx: &mut App) {
        self.visibility_group.update(cx, |group, cx| {
            group.set_selected_ids(config.visible_region_ids(), cx);
        });
    }

    fn sync_position_group(&self, config: &LayoutConfig, cx: &mut App) {
        self.primary_side_bar_position_group.update(cx, |group, cx| {
            group.set_selected_ids([config.primary_side_bar_position.id()], cx);
        });
    }

    fn sync_panel_alignment_group(&self, config: &LayoutConfig, cx: &mut App) {
        self.panel_alignment_group.update(cx, |group, cx| {
            group.set_selected_ids([config.panel_alignment.id()], cx);
        });
    }

    fn render_body(
        &self,
        _overlay_model: &OverlayWindowRenderModel<'_>,
        _window: &mut Window,
        _app: &App,
    ) -> AnyElement {
        let chrome = self.look.chrome();

        vstack! {
            dialog_header(&self.look, self.reset_button.clone(), self.close_button.clone()),
            vstack! {
                gap=SECTION_GAP;
                visibility_section(self.visibility_group.clone()),
                section_divider(chrome.border),
                positions_section(
                    self.primary_side_bar_position_group.clone(),
                    self.panel_alignment_group.clone(),
                    chrome.border,
                ),
            },
        }
        .w_full()
        .into_any_element()
    }
}

impl Render for CustomizeLayoutDialog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.overlay.clone()
    }
}

fn visibility_group_items() -> [ControlGroupItem; 6] {
    [
        visibility_group_item(LayoutRegion::ActivityBar),
        visibility_group_item(LayoutRegion::SecondaryActivityBar),
        visibility_group_item(LayoutRegion::PrimarySideBar),
        visibility_group_item(LayoutRegion::SecondarySideBar),
        visibility_group_item(LayoutRegion::Panel),
        visibility_group_item(LayoutRegion::StatusBar),
    ]
}

fn visibility_group_item(region: LayoutRegion) -> ControlGroupItem {
    ControlGroupItem::new(region.id()).label(region.label())
}

fn region_from_id(id: &SharedString) -> LayoutRegion {
    match id.as_ref() {
        "secondary-activity-bar" => LayoutRegion::SecondaryActivityBar,
        "primary-side-bar" => LayoutRegion::PrimarySideBar,
        "secondary-side-bar" => LayoutRegion::SecondarySideBar,
        "panel" => LayoutRegion::Panel,
        "status-bar" => LayoutRegion::StatusBar,
        _ => LayoutRegion::ActivityBar,
    }
}

fn section_divider(color: Hsla) -> impl IntoElement {
    div().w_full().h(px(1.0)).mx(px(CONTENT_PAD_X)).bg(color)
}

fn dialog_header(look: &ShadcnLook, reset_button: IconButton, close_button: IconButton) -> impl IntoElement {
    let chrome = look.chrome();
    let title_style = look.typography_role(ShadcnTextRole::H4);

    div()
        .h(px(HEADER_H))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(chrome.panel_background)
        .text_color(chrome.title_text)
        .relative()
        .child(div().typography_style(title_style).child("Customize Layout"))
        .child(
            hstack! {
                gap=2;
                reset_button,
                close_button,
            }
            .absolute()
            .right(px(10.0))
            .top(px(5.0)),
        )
}

fn sync_header_icon_presenters(close_button: &IconButton, reset_button: &IconButton, look: &ShadcnLook, cx: &mut App) {
    let color = look
        .resolve_ghost_button(ButtonFamilyRole::Icon, ControlSize::Sm, InteractionState::default())
        .foreground;
    close_button.update(cx, |button, cx| {
        button.set_presenter(header_icon_presenter(ControlIcon::Lucide(LucideIcon::X), color), cx);
    });
    reset_button.update(cx, |button, cx| {
        button.set_presenter(header_icon_presenter(ControlIcon::Lucide(LucideIcon::RotateCcw), color), cx);
    });
}

fn header_icon_presenter(icon: ControlIcon, color: Hsla) -> ControlPresenter<ButtonContentContext<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(icon) => xgpui_luma::infra::icon::lucide_icon(*icon, color, 14.0),
        ControlIcon::SvgPath(path) => {
            gpui::svg().size(px(14.0)).text_color(color).path(path.clone()).into_any_element()
        }
    })
}

fn visibility_section(group: ControlGroup<ControlGroupItem>) -> impl IntoElement {
    div().w_full().px(px(CONTENT_PAD_X)).pt(px(CONTENT_PAD_TOP)).child(group)
}

fn positions_section(
    primary_group: ControlGroup<ControlGroupItem>,
    panel_alignment_group: ControlGroup<ControlGroupItem>,
    border: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=SECTION_GAP;
        primary_group,
        div().w_full().h(px(1.0)).bg(border),
        panel_alignment_group,
        div().w_full().h(px(1.0)).bg(border),
    }
    .w_full()
    .px(px(CONTENT_PAD_X))
}

fn visibility_row_content(
    model: &ControlGroupItemRenderModel<ControlGroupItem>,
    visual: &ControlGroupItemVisualContext,
) -> AnyElement {
    let region = region_from_id(model.item.id());

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(SHORTCUT_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_region_icon(region, visual.foreground),
            [0, 1] => div()
                .min_w_0()
                .typography_style(visual.typography)
                .text_color(visual.foreground)
                .truncate()
                .child(ControlGroupItemLike::label(model.item).to_string()),
            [0, 2] => shortcut_column(
                region,
                model.index == 0,
                visual.muted_foreground,
                visual.muted_foreground,
                visual.typography,
            ),
            [0, 3] => visibility_eye(model.selected, visual.muted_foreground),
        })
        .into_any_element()
}

fn position_row_content(
    model: &ControlGroupItemRenderModel<ControlGroupItem>,
    visual: &ControlGroupItemVisualContext,
) -> AnyElement {
    let position = primary_side_bar_position_from_id(model.item.id());
    let icon = primary_side_bar_position_icon(position);

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(POSITION_LABEL_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_option_icon(icon, visual.foreground),
            [0, 1] => hstack! {
                gap=8 align=center;
                div()
                    .typography_style(visual.typography)
                    .text_color(visual.foreground)
                    .child(ControlGroupItemLike::label(model.item).to_string()),
                selected_check(model.selected, visual.foreground),
            },
            [0, 2] => position_header(
                position == PrimarySideBarPosition::Left,
                "Primary Side Bar Position",
                visual.typography,
                visual.muted_foreground,
            ),
            [0, 3] => div().w(px(EYE_COL_W)),
        })
        .into_any_element()
}

fn panel_alignment_row_content(
    model: &ControlGroupItemRenderModel<ControlGroupItem>,
    visual: &ControlGroupItemVisualContext,
) -> AnyElement {
    let alignment = panel_alignment_from_id(model.item.id());
    let icon = panel_alignment_icon(alignment);

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(POSITION_LABEL_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_option_icon(icon, visual.foreground),
            [0, 1] => hstack! {
                gap=8 align=center;
                div()
                    .typography_style(visual.typography)
                    .text_color(visual.foreground)
                    .child(ControlGroupItemLike::label(model.item).to_string()),
                selected_check(model.selected, visual.foreground),
            },
            [0, 2] => position_header(
                alignment == PanelAlignment::Left,
                "Panel Alignment",
                visual.typography,
                visual.muted_foreground,
            ),
            [0, 3] => div().w(px(EYE_COL_W)),
        })
        .into_any_element()
}

fn primary_side_bar_position_from_id(id: &SharedString) -> PrimarySideBarPosition {
    match id.as_ref() {
        "right" => PrimarySideBarPosition::Right,
        _ => PrimarySideBarPosition::Left,
    }
}

fn panel_alignment_from_id(id: &SharedString) -> PanelAlignment {
    match id.as_ref() {
        "left" => PanelAlignment::Left,
        "right" => PanelAlignment::Right,
        "justify" => PanelAlignment::Justify,
        _ => PanelAlignment::Center,
    }
}

fn position_header(show: bool, label: &'static str, style: LumaTextStyle, color: Hsla) -> impl IntoElement {
    hstack! {
        justify=end align=center;
        div()
            .typography_style(style)
            .text_color(color)
            .child(if show {
                label
            } else {
                ""
            }),
    }
    .w_full()
}

fn selected_check(selected: bool, color: Hsla) -> impl IntoElement {
    div()
        .w(px(18.0))
        .h(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(15.0))
        .text_color(color)
        .child(if selected {
            xgpui_luma::infra::icon::lucide_icon(LucideIcon::Check, color, 15.0)
        } else {
            div().into_any_element()
        })
}

fn visibility_eye(visible: bool, muted_text: Hsla) -> impl IntoElement {
    let eye_icon = if visible { LucideIcon::Eye } else { LucideIcon::EyeOff };

    div()
        .w(px(EYE_COL_W))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.0))
        .text_color(muted_text)
        .child(xgpui_luma::infra::icon::lucide_icon(eye_icon, muted_text, 13.0))
}

fn shortcut_column(
    region: LayoutRegion,
    show_visibility_header: bool,
    border: Hsla,
    muted_text: Hsla,
    muted_style: LumaTextStyle,
) -> AnyElement {
    if show_visibility_header {
        return hstack! {
            justify=end align=center;
            div().typography_style(muted_style).text_color(muted_text).child("Visibility"),
        }
        .w_full()
        .into_any_element();
    }

    if let Some(keys) = region.shortcut_keys() {
        return hstack! {
            justify=end align=center gap=2;
            shortcut_key(border, muted_text, keys[0]),
            shortcut_key(border, muted_text, keys[1]),
        }
        .w_full()
        .into_any_element();
    }

    div().w_full().into_any_element()
}

fn shortcut_key(border: Hsla, text_color: Hsla, label: &str) -> impl IntoElement {
    div()
        .min_w(px(18.0))
        .h(px(18.0))
        .px(px(4.0))
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .rounded(px(3.0))
        .border_1()
        .border_color(border)
        .text_size(px(11.0))
        .text_color(text_color)
        .child(label.to_string())
}

fn layout_region_icon(region: LayoutRegion, foreground: Hsla) -> impl IntoElement {
    let icon = div()
        .w(px(18.0))
        .h(px(18.0))
        .border(px(LAYOUT_ICON_BORDER_W))
        .border_color(foreground)
        .rounded(px(2.0))
        .overflow_hidden();

    match region {
        LayoutRegion::ActivityBar => icon.child(div().h_full().w(px(4.0)).bg(foreground)),
        LayoutRegion::SecondaryActivityBar => icon.flex().justify_end().child(div().h_full().w(px(4.0)).bg(foreground)),
        LayoutRegion::PrimarySideBar => icon.child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutRegion::SecondarySideBar => icon.flex().justify_end().child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutRegion::Panel => icon.flex().flex_col().justify_end().child(div().w_full().h(px(5.0)).bg(foreground)),
        LayoutRegion::StatusBar => icon.flex().flex_col().justify_end().child(div().w_full().h(px(3.0)).bg(foreground)),
    }
}

#[derive(Clone, Copy)]
enum LayoutOptionIcon {
    PrimaryLeft,
    PrimaryRight,
    PanelLeft,
    PanelRight,
    PanelCenter,
    PanelJustify,
}

fn primary_side_bar_position_icon(position: PrimarySideBarPosition) -> LayoutOptionIcon {
    match position {
        PrimarySideBarPosition::Left => LayoutOptionIcon::PrimaryLeft,
        PrimarySideBarPosition::Right => LayoutOptionIcon::PrimaryRight,
    }
}

fn panel_alignment_icon(alignment: PanelAlignment) -> LayoutOptionIcon {
    match alignment {
        PanelAlignment::Left => LayoutOptionIcon::PanelLeft,
        PanelAlignment::Right => LayoutOptionIcon::PanelRight,
        PanelAlignment::Center => LayoutOptionIcon::PanelCenter,
        PanelAlignment::Justify => LayoutOptionIcon::PanelJustify,
    }
}

fn layout_option_icon(option: LayoutOptionIcon, foreground: Hsla) -> impl IntoElement {
    let icon = div()
        .w(px(18.0))
        .h(px(18.0))
        .border(px(LAYOUT_ICON_BORDER_W))
        .border_color(foreground)
        .rounded(px(2.0))
        .overflow_hidden();

    match option {
        LayoutOptionIcon::PrimaryLeft => icon.child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutOptionIcon::PrimaryRight => icon.flex().justify_end().child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutOptionIcon::PanelLeft => {
            icon.flex().flex_col().justify_end().child(div().w(px(8.0)).h(px(5.0)).bg(foreground))
        }
        LayoutOptionIcon::PanelRight => {
            icon.flex().flex_col().justify_end().items_end().child(div().w(px(8.0)).h(px(5.0)).bg(foreground))
        }
        LayoutOptionIcon::PanelCenter => icon
            .flex()
            .flex_col()
            .justify_end()
            .items_center()
            .child(div().w(px(8.0)).h(px(5.0)).bg(foreground)),
        LayoutOptionIcon::PanelJustify => {
            icon.flex().flex_col().justify_end().child(div().w_full().h(px(5.0)).bg(foreground))
        }
    }
}
