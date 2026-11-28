//! Every trigger drawn as a button face paints the resolved `:disabled { opacity }` (#430). Headless.

use std::sync::Arc;

use gpui::{AnyView, App, Context, Hsla, IntoElement, Render, TestAppContext, Window, div, prelude::*};

use crate::controls::button::StyledButtonTemplate;
use crate::controls::popup_menu::{PopupMenu, StyledPopupMenuTheme, ThemedPopupMenuTemplate};
use crate::controls::selector::{Selector, SelectorItem, StyledSelectorTheme, ThemedSelectorTemplate};
use crate::controls::trigger_button::{self, TriggerData};
use crate::style::{PartRead as _, ThemeSource};

/// The SDK with every trigger faded to half when disabled, and its fill made opaque so the fade is measurable.
fn faded_theme() -> crate::style::lss::Theme {
    let theme = crate::style::default_theme();
    let lss = "trigger_button { part root { background := #336699  :disabled { opacity = 0.5 } } }
               popup_menu { part root { background := #336699  :disabled { opacity = 0.5 } } }";
    let sheet = theme
        .stylesheet()
        .with_source("app", crate::style::lss::Source::new("fade.lss", lss.to_string()))
        .expect("compiles");
    theme.with_stylesheet(&sheet).expect("binds")
}

struct Host(AnyView);

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.0.clone())
    }
}

/// The alpha of the fill painted at the `{id}-face` box.
fn face_alpha(
    cx: &mut TestAppContext,
    id: &str,
    build: impl FnOnce(&mut Window, &mut Context<Host>) -> AnyView,
) -> f32 {
    face_fill(cx, id, build).a
}

/// The fill painted at the `{id}-face` box.
fn face_fill(
    cx: &mut TestAppContext,
    id: &str,
    build: impl FnOnce(&mut Window, &mut Context<Host>) -> AnyView,
) -> Hsla {
    let (_, cx) = cx.add_window_view(|window, cx| Host(build(window, cx)));
    cx.run_until_parked();
    let selector: &'static str = Box::leak(format!("{id}-face").into_boxed_str());
    let bounds = cx.debug_bounds(selector).expect("the face is drawn");
    cx.update(|window, _: &mut App| {
        let scaled = bounds.scale(window.scale_factor());
        let fills: Vec<Hsla> = window
            .painted_quads()
            .into_iter()
            .filter(|quad| quad.bounds == scaled)
            .filter_map(|quad| quad.background.as_solid())
            .collect();
        fills.first().copied().expect("the face paints a fill")
    })
}

#[gpui::test]
fn a_disabled_trigger_button_fades(cx: &mut TestAppContext) {
    let template = || Arc::new(StyledButtonTemplate::<TriggerData>::new(faded_theme()).control("trigger_button"));
    let enabled = face_alpha(cx, "sort", |_, cx| trigger_button::new("sort").template(template()).spawn(cx).into());
    let disabled = face_alpha(cx, "sort", |_, cx| {
        trigger_button::new("sort").template(template()).enabled(false).spawn(cx).into()
    });
    assert!((enabled - 1.0).abs() < 1e-3, "{enabled}");
    assert!((disabled - 0.5).abs() < 1e-3, "{disabled}");
}

#[gpui::test]
fn a_disabled_selector_fades(cx: &mut TestAppContext) {
    let template = || {
        let panel = Arc::new(|| crate::controls::selector_list::default_selector_items_panel_look(&Default::default()));
        Arc::new(ThemedSelectorTemplate::new(
            Arc::new(StyledSelectorTheme::new(ThemeSource::Fixed(faded_theme()), panel)),
            crate::controls::selector_list::default_selector_items_template(),
        ))
    };
    let selector = |enabled: bool| {
        move |_: &mut Window, cx: &mut Context<Host>| -> AnyView {
            Selector::new("pick")
                .items([SelectorItem::new("a").label("A")])
                .template(template())
                .enabled(enabled)
                .spawn(cx)
                .into()
        }
    };
    let enabled = face_alpha(cx, "trigger", selector(true));
    let disabled = face_alpha(cx, "trigger", selector(false));
    assert!((enabled - 1.0).abs() < 1e-3, "{enabled}");
    assert!((disabled - 0.5).abs() < 1e-3, "{disabled}");
}

#[gpui::test]
fn a_disabled_popup_menu_fades(cx: &mut TestAppContext) {
    let template = || {
        let menu = Arc::new(|| crate::style::StyleBinding::sdk("floating_menu"));
        Arc::new(ThemedPopupMenuTemplate::new(Arc::new(StyledPopupMenuTheme::new(
            ThemeSource::Fixed(faded_theme()),
            menu,
        ))))
    };
    let menu = |enabled: bool| {
        move |_: &mut Window, cx: &mut Context<Host>| -> AnyView {
            PopupMenu::new("menu").label("Actions").template(template()).enabled(enabled).spawn(cx).into()
        }
    };
    let enabled = face_alpha(cx, "trigger", menu(true));
    let disabled = face_alpha(cx, "trigger", menu(false));
    assert!((enabled - 1.0).abs() < 1e-3, "{enabled}");
    assert!((disabled - 0.5).abs() < 1e-3, "{disabled}");
}

/// The SDK chevron dims with the label when disabled; at rest it is the muted foreground.
#[test]
fn the_sdk_chevron_dims_when_disabled() {
    use crate::controls::selector::{SelectorTriggerStyle, default_selector_theme};
    use crate::controls::selector::{SelectorLook, SelectorVisualState, resolve_selector_trigger};
    use crate::theme::InteractionState;
    let theme = default_selector_theme();
    let look = |disabled| {
        let visual = SelectorVisualState {
            interaction: InteractionState { disabled, ..Default::default() },
            ..Default::default()
        };
        let resolved = resolve_selector_trigger(&theme.style(), SelectorTriggerStyle::Outline, visual, 1.0);
        SelectorLook::from_resolved(&resolved, theme.items_panel(), false)
    };
    let (resting, disabled) = (look(false), look(true));
    assert_ne!(resting.trigger_icon, disabled.trigger_icon);
    assert_eq!(disabled.trigger_icon, disabled.trigger_foreground, "the disabled label color");
    assert_eq!(disabled.trigger_opacity, 1.0, "the SDK draws colors, no fade");
}

/// The SDK with an open trigger fill (`:hover:open`: an open trigger draws hovered), and a green chevron, on the trigger button (so its specializations).
fn open_theme() -> crate::style::lss::Theme {
    let theme = crate::style::default_theme();
    let lss = "trigger_button { icon = #00ff00  part root { :hover:open { background = #ff0000 } } }";
    let sheet = theme
        .stylesheet()
        .with_source("app", crate::style::lss::Source::new("open.lss", lss.to_string()))
        .expect("compiles");
    theme.with_stylesheet(&sheet).expect("binds")
}

fn red() -> Hsla {
    gpui::rgb(0xff0000).into()
}

fn close(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
}

/// `trigger_button { :open {…} }` reaches an open trigger button, selector and popup menu alike.
#[gpui::test]
fn the_open_state_reaches_every_trigger(cx: &mut TestAppContext) {
    let template = || Arc::new(StyledButtonTemplate::<TriggerData>::new(open_theme()).control("trigger_button"));
    let fill = |open: bool, cx: &mut TestAppContext| {
        face_fill(cx, "sort", move |_, cx| {
            trigger_button::new("sort")
                .template(template())
                .with_data(TriggerData::default().open(open))
                .spawn(cx)
                .into()
        })
    };
    let open_fill = fill(true, cx);
    assert!(close(open_fill, red()), "an open trigger button: {open_fill:?}");
    assert!(!close(fill(false, cx), red()), "a closed trigger button");

    use crate::controls::popup_menu::PopupMenuTriggerStyle;
    use crate::controls::selector::{SelectorTheme, SelectorTriggerStyle, SelectorVisualState, resolve_selector_trigger};
    use crate::theme::InteractionState;
    let panel = Arc::new(|| crate::controls::selector_list::default_selector_items_panel_look(&Default::default()));
    let selector = StyledSelectorTheme::new(ThemeSource::Fixed(open_theme()), panel);
    let at = |open| {
        let visual = SelectorVisualState { open, ..Default::default() };
        resolve_selector_trigger(&selector.style(), SelectorTriggerStyle::Outline, visual, 1.0)
            .root()
            .color("background")
    };
    assert!(close(at(true), red()), "an open selector");
    assert!(!close(at(false), red()), "a closed selector");

    let menu = Arc::new(|| crate::style::StyleBinding::sdk("floating_menu"));
    let popup = StyledPopupMenuTheme::new(ThemeSource::Fixed(open_theme()), menu);
    // The template draws an open popup menu hovered, from the resolved trigger style (#433).
    let hovered = InteractionState { hovered: true, ..Default::default() };
    let trigger = |open| {
        crate::controls::popup_menu::styled::resolve_trigger(
            &crate::controls::popup_menu::PopupMenuTheme::style(&popup),
            PopupMenuTriggerStyle::Outline,
            hovered,
            open,
            1.0,
        )
    };
    let (open_trigger, closed_trigger) = (trigger(true), trigger(false));
    assert!(close(open_trigger.root().color("background"), red()), "an open popup menu");
    assert!(!close(closed_trigger.root().color("background"), red()), "a closed popup menu");
    assert!(
        close(closed_trigger.root().color("icon"), gpui::rgb(0x00ff00).into()),
        "the popup chevron is `icon`"
    );
}

/// The popup menu's chevron is the label color unless a look changes `icon`, as before (#430).
#[gpui::test]
fn the_sdk_popup_chevron_is_the_label_color(_cx: &mut TestAppContext) {
    use crate::controls::popup_menu::{PopupMenuTriggerStyle, default_popup_menu_theme};
    use crate::theme::InteractionState;
    let theme = default_popup_menu_theme();
    for disabled in [false, true] {
        let state = InteractionState { disabled, ..Default::default() };
        let resolved = crate::controls::popup_menu::styled::resolve_trigger(
            &theme.style(),
            PopupMenuTriggerStyle::Outline,
            state,
            false,
            1.0,
        );
        let root = resolved.root();
        let icon = root.color_opt("icon").unwrap_or(root.color("foreground"));
        assert_eq!(icon, root.color("foreground"), "disabled {disabled}");
    }
}
