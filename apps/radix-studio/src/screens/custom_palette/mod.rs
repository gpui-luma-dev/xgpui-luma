//! Custom Palette tab — seed editor, scale grid, and control preview canvas.

mod home_controls;
mod icon_samples;
mod preview_toolbar;
mod tooltip_examples;
mod signup_mesh;
mod task_samples;

pub(crate) use signup_mesh::{MeshCache, MeshRequest};

use std::sync::Arc;

use gpui::{App, Context, Entity, Hsla, ImageSource, IntoElement, RenderImage, div, img, prelude::*, px};
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::popup_menu::PopupMenu;
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use xgpui_luma::controls::tree_view::TreeView;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::infra::ElementExt;
use xgpui_luma::{GridLayout, GridTrack, WideMiddle, WideMiddleLayout, hstack, vstack};
use xgpui_luma_look_radix::{Look, SCALE_LEN, ScaleFamily};

use crate::app::RadixStudioApp;
use self::signup_mesh::SignupStage;
use crate::controls::ColorTextField;

/// Side columns stay compact; center is wider for the signup stage.
const PREVIEW_SIDE_COLUMN_MIN: f32 = 240.0;
const PREVIEW_SIDE_COLUMN_PREFERRED: f32 = 280.0;
/// Middle column hard cap in the three-column row (also signup glass width).
pub const PREVIEW_CENTER_COLUMN_MAX: f32 = 435.0;
const PREVIEW_COLUMN_GAP: f32 = 24.0;
const SIGNUP_MESH_MAX_SIDE: f32 = 960.0;
const SIGNUP_INNER_FORM_CARD_W: f32 = 340.0;
/// Form face alpha so mesh grads show through.
const SIGNUP_PANEL_ALPHA: f32 = 0.55;

/// Gutter between shade cells and between Color / Gray rows (Radix Colors grid).
const SCALE_SWATCH_GAP: f32 = 3.0;

const SCALE_LEGEND_GROUPS: &[(&str, usize)] = &[
    ("Backgrounds", 2),
    ("Interactive components", 3),
    ("Borders and separators", 3),
    ("Solid colors", 2),
    ("Accessible text", 2),
];

pub use home_controls::PreviewControls;

pub struct PageArgs<'a> {
    pub look: &'a Arc<Look>,
    pub mode_selector: RadioGroup<RadioGroupItem>,
    pub accent_field: Entity<ColorTextField>,
    pub gray_field: Entity<ColorTextField>,
    pub background_field: Entity<ColorTextField>,
    pub copy_menu: Entity<PopupMenu>,
    pub palette_swatches: Vec<Entity<Button>>,
    pub preview_layout: Entity<WideMiddle>,
    pub mesh_image: Option<Arc<RenderImage>>,
    pub app: Entity<RadixStudioApp>,
    pub controls: PreviewControls,
    pub surface: Hsla,
    pub border: Hsla,
    pub muted: Hsla,
    pub fg: Hsla,
    pub accent: Hsla,
}

pub fn page(args: PageArgs<'_>, cx: &mut Context<RadixStudioApp>) -> gpui::AnyElement {
    let PageArgs {
        look,
        mode_selector,
        accent_field,
        gray_field,
        background_field,
        copy_menu,
        palette_swatches,
        preview_layout,
        mesh_image,
        app,
        controls,
        surface,
        border,
        muted,
        fg,
        accent,
    } = args;

    vstack! {
        gap=28;
        header_block(fg),
        div().mx_auto().child(mode_selector),
        seed_row(SeedRow {
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            accent,
            gray: look.resolve_step(ScaleFamily::Gray, 8).hsla(),
            background: look.resolve_step(ScaleFamily::Gray, 1).hsla(),
            border,
            muted,
        }),
        scale_section(muted, &palette_swatches),
        preview_section(look, mesh_image, app, preview_layout, cx, controls, surface, border, muted, fg),
    }
    .into_any_element()
}

pub fn preview_layout_defaults() -> WideMiddle {
    WideMiddle::new()
        .gap(PREVIEW_COLUMN_GAP)
        .side_min_width(PREVIEW_SIDE_COLUMN_MIN)
        .side_preferred_width(PREVIEW_SIDE_COLUMN_PREFERRED)
        .middle_min_width(PREVIEW_CENTER_COLUMN_MAX)
        .middle_max_width(PREVIEW_CENTER_COLUMN_MAX)
}

fn header_block(fg: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div().text_3xl().font_weight(gpui::FontWeight::BOLD).text_color(fg).child("Create a custom palette"),
    }
    .w_full()
}

struct SeedRow {
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    accent: Hsla,
    gray: Hsla,
    background: Hsla,
    border: Hsla,
    muted: Hsla,
}

fn seed_row(row: SeedRow) -> impl IntoElement {
    let SeedRow { accent_field, gray_field, background_field, copy_menu, accent, gray, background, border, muted } =
        row;

    vstack! {
        gap=10 align=center;
        hstack! {
            gap=16 align=end justify=center;
            seed_field("Accent", accent_field, accent, border, muted),
            seed_field("Gray", gray_field, gray, border, muted),
            seed_field("Background", background_field, background, border, muted),
            // Align the visible button face; its SDK root reserves extra focus-ring space.
            div().h(px(ColorTextField::HEIGHT)).flex().items_center().child(copy_menu),
        },
    }
    .w_full()
}

fn seed_field(
    label: &'static str,
    field: Entity<ColorTextField>,
    _swatch: Hsla,
    _border: Hsla,
    muted: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=6;
        div().text_xs().text_color(muted).child(label),
        div().w(px(180.0)).child(field),
    }
}

fn scale_section(muted: Hsla, swatches: &[Entity<Button>]) -> impl IntoElement {
    let columns = vec![GridTrack::Star(1.0); SCALE_LEN];
    let mut grid = GridLayout::new().columns(columns).gap_x(SCALE_SWATCH_GAP).gap_y(SCALE_SWATCH_GAP);

    let mut col = 0usize;
    for &(label, span) in SCALE_LEGEND_GROUPS {
        grid = grid.child_with_span(legend_group(label, muted), 0, col, span);
        col += span;
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_step_number(step, muted), 1, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(swatches[usize::from(step - 1)].clone(), 2, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(swatches[12 + usize::from(step - 1)].clone(), 3, (step - 1) as usize);
    }

    grid.into_element().w_full()
}

fn legend_group(label: &'static str, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4 align=center;
        div()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .text_center()
            .truncate()
            .child(label),
        div().w_full().h(px(2.0)).rounded_full().bg(muted.opacity(0.45)),
    }
    .w_full()
}

fn scale_step_number(step: u8, muted: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .child(format!("{step}")),
    }
    .w_full()
}

#[allow(clippy::too_many_arguments)]
fn preview_section(
    look: &Look,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    preview_layout: Entity<WideMiddle>,
    cx: &App,
    controls: PreviewControls,
    surface: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
) -> impl IntoElement {
    WideMiddleLayout::new(
        preview_layout,
        preview_column_left(
            look,
            controls.tree,
            controls.search_field,
            controls.search_submit,
            controls.tooltip_slider,
            controls.icon_samples,
        ),
        vstack! {
            gap=14;
            hstack! {
                gap=12 align=center;
                controls.toolbar,
                controls.actions,
            }.w_full().flex_wrap(),
            preview_column_center(
                look,
                mesh_image,
                app,
                controls.sign_up_name,
                controls.sign_up_email,
                controls.sign_up_password,
                controls.create_account,
                controls.continue_github,
                border,
                muted,
                fg,
            ),
        }
        .w_full(),
        preview_column_right(look, controls.tabs, controls.task_samples, surface),
    )
    .build(cx)
}

#[allow(clippy::too_many_arguments)]
fn preview_column_left(
    look: &Look,
    tree: TreeView<()>,
    search_field: TextField,
    search_submit: Entity<Button>,
    tooltip_slider: xgpui_luma::controls::slider::Slider,
    icon_samples: icon_samples::IconSamples,
) -> impl IntoElement {
    vstack! {
        gap=14;
        hstack! {
            gap=8 align=center;
            div().flex_1().min_w_0().child(search_field),
            search_submit,
        }
        .w_full(),
        div().flex().flex_col().gap(px(8.0))
            .child(div().text_size(px(12.0)).child("Tooltip experiment · hover or focus"))
            .child(tooltip_slider),
        alert_card(look),
        super::shared::tree_view::panel(tree, look),
        div().flex().flex_wrap().gap(px(12.0)).children([
            xgpui_luma_look_radix::Badge::new(look, "Fully-featured").pill().into_any_element(),
            xgpui_luma_look_radix::Badge::new(look, "Built with Radix").variant(xgpui_luma_look_radix::BadgeVariant::Surface).pill().into_any_element(),
            xgpui_luma_look_radix::Badge::new(look, "Open source").variant(xgpui_luma_look_radix::BadgeVariant::Outline).pill().into_any_element(),
        ]),
        icon_samples.render(),
        super::shared::card_samples::home(look),
    }
    .w_full()
}

#[allow(clippy::too_many_arguments)]
fn preview_column_center(
    look: &Look,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    name: TextField,
    email: TextField,
    password: TextField,
    create: Entity<Button>,
    github: Entity<Button>,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
) -> impl IntoElement {
    let stage = SignupStage::default();
    let stage_fill = stage.fill(look);
    let panel_fill = {
        let mut c = stage.card(look);
        c.a = SIGNUP_PANEL_ALPHA;
        c
    };
    let mesh_opacity = stage.mesh_opacity;

    let inner_form_card = div()
        .id("inner_form_card")
        .flex_none()
        .w(px(SIGNUP_INNER_FORM_CARD_W))
        .min_w(px(SIGNUP_INNER_FORM_CARD_W))
        .max_w(px(SIGNUP_INNER_FORM_CARD_W))
        .overflow_hidden()
        .rounded_xl()
        .border_1()
        .border_color(border)
        .bg(panel_fill)
        .p_6()
        .child(
            vstack! {
                gap=16;
                labeled_field("Full name", name, muted),
                labeled_field("Email", email, muted),
                labeled_field("Password", password, muted),
                create,
                hstack! {
                    gap=12 align=center;
                    div().flex_1().h(px(1.0)).bg(border),
                    div().text_xs().text_color(muted).child("OR"),
                    div().flex_1().h(px(1.0)).bg(border),
                }
                .w_full(),
                github,
            }
            .w_full(),
        );

    let content = vstack! {
        gap=16 align=center justify=center;
        div()
            .flex_none()
            .text_xl()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(fg)
            .child("Sign up"),
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(inner_form_card),
    }
    .relative()
    .w_full()
    .px_4()
    .pt_6()
    .pb(px(45.0));

    div()
        .id("signup_outer_card")
        .w_full()
        .overflow_hidden()
        .relative()
        .bg(stage_fill)
        .on_prepaint(move |bounds, _window, cx| {
            let width_px = bounds.size.width.as_f32().max(1.0);
            let height_px = bounds.size.height.as_f32().max(1.0);
            let scale = (SIGNUP_MESH_MAX_SIDE / width_px.max(height_px)).min(1.0);
            let width = (width_px * scale).round().max(1.0) as u32;
            let height = (height_px * scale).round().max(1.0) as u32;
            app.update(cx, |this, cx| {
                this.request_signup_mesh(width, height, cx);
            });
        })
        .when_some(mesh_image, |this, image| {
            this.child(
                img(ImageSource::Render(image))
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover)
                    .opacity(mesh_opacity),
            )
        })
        .child(content)
}

fn preview_column_right(
    look: &Look,
    tabs: Entity<Tabs>,
    tasks: task_samples::TaskSamples,
    surface: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=14;
        tabs,
        avatar_samples(look),
        typography_block(look),
        tasks.render(surface),
    }
    .w_full()
}

/// Local sample composition; Avatar owns its geometry and palette treatment.
fn avatar_samples(look: &Look) -> impl IntoElement {
    use xgpui_luma::controls::button::ControlIcon;
    use xgpui_luma_look_radix::{Avatar, AvatarVariant, Radius};

    let rows = [AvatarVariant::Solid, AvatarVariant::Soft];
    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .max_w(px(300.0))
        .mx_auto()
        .children(rows.into_iter().map(|variant| {
            let avatar = |text| Avatar::new(look, text).variant(variant).radius(Radius::Full);
            let people = || ControlIcon::SvgPath("assets/avatars/people.svg".into());
            // Spacing contracts in narrow side columns while preserving the avatar size.
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(avatar("V").image("assets/avatars/portrait.jpg"))
                .child(avatar("V").image("assets/avatars/portrait.jpg"))
                .child(avatar("V"))
                .child(avatar("BG"))
                .child(avatar("").icon(people()))
                .child(avatar("").icon(people()).high_contrast(true))
        }))
}

fn labeled_field(label: &'static str, field: TextField, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4;
        div().text_xs().text_color(muted).child(label),
        field,
    }
    .w_full()
}

fn alert_card(look: &Look) -> impl IntoElement {
    xgpui_luma_look_radix::callout(
        look,
        "Please upgrade to the new version.",
        xgpui_luma::controls::button::ControlIcon::SvgPath("assets/react-icons/info-circled.svg".into()),
    )
}

/// Local typography sample: identical content in accent and neutral treatments.
fn typography_block(look: &Look) -> impl IntoElement {
    vstack! {
        gap=16;
        biography_quote(look, ScaleFamily::Color),
        biography_quote(look, ScaleFamily::Gray),
    }
    .w_full()
}

fn biography_quote(look: &Look, family: ScaleFamily) -> impl IntoElement {
    const TEXT: &str = "Susan Kare is an American graphic designer and artist, who contributed interface elements and typefaces for the first Apple Macintosh personal computer from 1983 to 1986.";
    let (body, emphasis) = match family {
        ScaleFamily::Gray => {
            (look.resolve_step(ScaleFamily::Gray, 11).hsla(), look.resolve_step(ScaleFamily::Gray, 12).hsla())
        }
        _ => (look.resolve_step(ScaleFamily::Gray, 12).hsla(), look.resolve_step(family, 11).hsla()),
    };
    let highlights =
        ["graphic designer", "interface", "typefaces", "Apple Macintosh"].into_iter().filter_map(|phrase| {
            TEXT.find(phrase).map(|start| {
                (start..start + phrase.len(), gpui::HighlightStyle { color: Some(emphasis), ..Default::default() })
            })
        });
    div()
        .w_full()
        .border_l(px(3.0))
        .border_color(look.resolve_step(family, 6).hsla())
        .pl(px(12.0))
        .text_sm()
        .text_color(body)
        .child(gpui::StyledText::new(TEXT).with_highlights(highlights))
}
