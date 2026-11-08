use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    div, App, AppContext, Application, AnyView, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions,
};
use xgpui_luma_look_radix as radix;
use wasm_bindgen::prelude::*;
use xgpui_luma::theme::ThemeMode;

type Refresh = Rc<dyn Fn(&mut App)>;

struct Sample {
    view: AnyView,
    refresh: Refresh,
}

struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let path = if path == "icons/open.svg" {
            lucide_svg_static::Icon::FolderOpen.asset_path()
        } else {
            path
        };
        Ok(lucide_svg_static::asset_bytes(path).map(Cow::Borrowed))
    }

    fn list(&self, _: &str) -> anyhow::Result<Vec<gpui::SharedString>> {
        Ok(Vec::new())
    }
}

// Each module compiles its documentation block unchanged. The adapter only
// renders the retained control and exposes callback state beneath it.
macro_rules! sample {
    ($name:ident, $format:literal, $($field:ident),+) => {
        sample!($name, control, state, format!($format, $(state.$field),+));
    };
    ($name:ident, $control:ident, $state:ident, $status:expr) => {
        // Some documentation imports also support later examples on the same page.
        #[allow(unused_imports)]
        mod $name {
            use super::*;
            include!(concat!(env!("OUT_DIR"), "/", stringify!($name), "_sample.rs"));

            impl Render for Example {
                fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                    let frame = div().w(gpui::px(320.0)).max_w(gpui::relative(1.0)).flex().flex_col();
                    let frame = match stringify!($name) {
                        "table" => frame.w(gpui::px(600.0)).h(gpui::px(300.0)),
                        "tree_view" => frame.w(gpui::px(440.0)).h(gpui::px(280.0)),
                        "textarea" | "selection_panel" => frame.h(gpui::px(200.0)),
                        _ => frame,
                    };
                    let frame = match stringify!($name) {
                        "slider" | "textfield" | "textarea" | "selector" | "combobox" | "autocomplete"
                        | "search_selector" | "selection_panel" | "table" | "tree_view" | "accordion" | "toolbar" => frame,
                        _ => frame.items_center(),
                    };
                    let $state = &*self;
                    div().w_full().flex().flex_col().items_center().gap_4()
                        .child(frame.child(self.$control.clone()))
                        .child($status)
                }
            }

            pub(super) fn spawn(cx: &mut App) -> Sample {
                let view = cx.new(Example::new);
                let control = view.read(cx).$control.clone();
                Sample {
                    view: view.into(),
                    refresh: Rc::new(move |cx| control.update(cx, |_, cx| cx.notify())),
                }
            }
        }
    };
}

sample!(button, "Save requests: {} · Focused: {}", save_requests, focused);
sample!(checkbox, "Notifications enabled: {}", notifications_enabled);
sample!(switch, "Notifications enabled: {}", notifications_enabled);
sample!(slider, "Preview: {:.0} · Committed: {:.0}", preview, committed);
sample!(tabs, "Active tab: {}", active_tab);

sample!(radio_button, "Compact: {}", compact);
sample!(radio_group, "Density: {:?}", density);
sample!(toggle, "Bold: {}", bold);
sample!(textfield, "Name: {}", name);
sample!(textarea, "Notes: {}", notes);
sample!(selector, "Status: {}", status);
sample!(combobox, "Query: {}", query);
sample!(autocomplete, "Query: {}", query);
sample!(search_selector, "Query: {}", query);
sample!(selection_panel, "Last action: {}", last_action);

sample!(icon_button, "Open requests: {}", open_requests);
sample!(split_button, "Last action: {}", last_action);
sample!(popover_button, "Details open: {}", details_open);
sample!(popup_menu, "Last action: {}", last_action);
sample!(context_menu, "Last action: {}", last_action);
sample!(toolbar, "Last command: {}", last_command);
sample!(accordion, "Expanded: {}", details_expanded);
sample!(tree_view, "Selected: {:?}", selected_nodes);
sample!(table, "Selected rows: {:?}", selected_rows);
sample!(icon_group, "Selected: {:?}", selected_tools);
sample!(tooltip, button, state, {
    let _ = state.help_shown.load(std::sync::atomic::Ordering::Relaxed);
    "Hover Save to show its tooltip"
});

// Color pages include their own Render implementations. Retain those too,
// and observe the sample to redraw the host's preview/commit readout.
macro_rules! rendered_sample {
    ($name:ident, $control:ident, $state:ident, $status:expr $(, $extra:ident)*) => {
        mod $name {
            use super::{Sample, Rc, App};
            include!(concat!(env!("OUT_DIR"), "/", stringify!($name), "_sample.rs"));

            struct Host {
                example: Entity<Example>,
                _subscription: gpui::Subscription,
            }

            impl Render for Host {
                fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                    let $state = self.example.read(cx);
                    gpui::div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(self.example.clone())
                        .child($status)
                }
            }

            pub(super) fn spawn(cx: &mut App) -> Sample {
                let example = gpui::AppContext::new(cx, Example::new);
                let control = example.read(cx).$control.clone();
                $(let $extra = example.read(cx).$extra.clone();)*
                let view = gpui::AppContext::new(cx, |cx| Host {
                    _subscription: cx.observe(&example, |_, _, cx| cx.notify()),
                    example,
                });
                Sample { view: view.into(), refresh: Rc::new(move |cx| {
                    control.update(cx, |_, cx| cx.notify());
                    $($extra.update(cx, |_, cx| cx.notify());)*
                }) }
            }
        }
    };
}

rendered_sample!(
    color_slider,
    control,
    state,
    format!("Hue: {:.0}° · Committed: {:.0}°", state.preview, state.committed)
);
rendered_sample!(
    color_arc,
    control,
    state,
    format!("Hue: {:.0}° · Committed: {:.0}°", state.preview, state.committed)
);
rendered_sample!(
    color_ring,
    control,
    state,
    format!("Hue: {:.0}° · Committed: {:.0}°", state.preview, state.committed)
);
rendered_sample!(
    color_field,
    field,
    state,
    gpui::div()
        .text_center()
        .child(gpui::div().child(format!("Preview S: {:.2} · V: {:.2}", state.preview.s, state.preview.v)))
        .child(gpui::div().child(format!("Committed S: {:.2} · V: {:.2}", state.committed.s, state.committed.v)))
);

rendered_sample!(pager, control, _state, gpui::div(), numeric, page_size_select);
rendered_sample!(progress, progress, _state, gpui::div(), circular, loading_bar, loading_ring, advance, reset);
rendered_sample!(stepper, stepper, _state, gpui::div());
rendered_sample!(color_swatch, picker, _state, gpui::div());

mod listbox {
    use super::{App, AppContext, Sample, Rc};
    include!(concat!(env!("OUT_DIR"), "/listbox_sample.rs"));

    pub(super) fn spawn(cx: &mut App) -> Sample {
        match Example::initial_state() {
            Ok(state) => {
                let view = cx.new(|cx| Example::new(state, cx));
                let refresh_view = view.clone();
                Sample { view: view.into(), refresh: Rc::new(move |cx| refresh_view.update(cx, |_, cx| cx.notify())) }
            }
            Err(error) => {
                struct ErrorView(String);
                impl Render for ErrorView {
                    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                        div().child(self.0.clone())
                    }
                }
                let view = cx.new(|_| ErrorView(format!("Unable to build list: {error}")));
                Sample { view: view.into(), refresh: Rc::new(|_| {}) }
            }
        }
    }
}

struct Preview {
    sample: Sample,
    look: radix::Look,
}

// Plot code blocks expose builder functions instead of retained controls.
// Their host supplies only bounds and the datasets requested by secondary examples.
macro_rules! chart_sample {
    ($name:ident, $variant:ident, $render:expr) => {
        mod $name {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($name), "_sample.rs"));

            struct Host {
                variant: bool,
            }

            impl gpui::Render for Host {
                fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                    use gpui::{IntoElement as _, ParentElement as _, Styled as _};
                    let $variant = self.variant;
                    let chart: Result<gpui::AnyElement, DatasetError> = $render;
                    let content = match chart {
                        Ok(chart) => chart,
                        Err(error) => gpui::div().child(format!("Unable to build chart: {error}")).into_any_element(),
                    };
                    gpui::div().w_full().h(gpui::px(360.0)).child(content)
                }
            }

            pub(super) fn spawn(cx: &mut gpui::App, variant: bool) -> super::Sample {
                let view = gpui::AppContext::new(cx, |_| Host { variant });
                let refresh_view = view.clone();
                super::Sample {
                    view: view.into(),
                    refresh: std::rc::Rc::new(move |cx| refresh_view.update(cx, |_, cx| cx.notify())),
                }
            }
        }
    };
}

chart_sample!(plot_cartesian, bars, {
    if bars {
        (|| {
            let data = Dataset::new()
                .with_series(Series::new(
                    "desktop",
                    "Desktop",
                    [Sample::new(0.0, 12.0), Sample::new(1.0, 24.0), Sample::new(2.0, 18.0)],
                ))?
                .with_series(Series::new(
                    "mobile",
                    "Mobile",
                    [Sample::new(0.0, 8.0), Sample::new(1.0, 16.0), Sample::new(2.0, 12.0)],
                ))?;
            Ok(bar_chart(data).into_any_element())
        })()
    } else {
        combined_chart().map(|chart| chart.into_any_element())
    }
});
chart_sample!(plot_pie, rings, {
    if rings {
        (|| {
            let data = Dataset::new()
                .with_series(Series::new(
                    "current",
                    "Current",
                    [Sample::new(0.0, 275.0), Sample::new(1.0, 200.0), Sample::new(2.0, 125.0)],
                ))?
                .with_series(Series::new(
                    "previous",
                    "Previous",
                    [Sample::new(0.0, 240.0), Sample::new(1.0, 210.0), Sample::new(2.0, 150.0)],
                ))?;
            Ok(comparison_rings(data).into_any_element())
        })()
    } else {
        browser_share().map(|chart| chart.into_any_element())
    }
});
chart_sample!(plot_radar, _variant, comparison().map(|chart| chart.into_any_element()));
chart_sample!(plot_radial, _variant, completion().map(|chart| chart.into_any_element()));

thread_local! {
    static DARK: Cell<bool> = const { Cell::new(false) };
    static VIEW: RefCell<Option<(Entity<Preview>, gpui::AsyncApp)>> = const { RefCell::new(None) };
}

/// Mirror the documentation theme, including changes during graphics startup.
#[wasm_bindgen]
pub fn set_theme(dark: bool) {
    DARK.set(dark);
    VIEW.with_borrow_mut(|state| {
        if let Some((view, app)) = state {
            let _ = view.update(app, |view, cx| {
                view.look.set_mode(if dark { ThemeMode::Dark } else { ThemeMode::Light });
                if let Some(look) = cx.try_global::<xgpui_luma_look_shadcn::ShadcnLook>() {
                    look.set_mode(if dark { ThemeMode::Dark } else { ThemeMode::Light });
                }
                (view.sample.refresh)(cx);
                cx.refresh_windows();
                cx.notify();
            });
        }
    });
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let look = &self.look;
        div()
            .size_full()
            .bg(look.page_background())
            .text_color(look.resolve_role(radix::SemanticRole::Foreground).hsla())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .p_6()
            .font_family("Inter 24pt")
            .child(div().w_full().child(self.sample.view.clone()))
    }
}

#[wasm_bindgen]
pub fn start(example_id: &str) -> Result<(), JsValue> {
    let spawn: fn(&mut App) -> Sample = match example_id {
        "button-basic" => button::spawn,
        "checkbox-basic" => checkbox::spawn,
        "switch-basic" => switch::spawn,
        "slider-basic" => slider::spawn,
        "tabs-basic" => tabs::spawn,
        "listbox-basic" => listbox::spawn,
        "radio-button-basic" => radio_button::spawn,
        "radio-group-basic" => radio_group::spawn,
        "toggle-basic" => toggle::spawn,
        "textfield-basic" => textfield::spawn,
        "textarea-basic" => textarea::spawn,
        "selector-basic" => selector::spawn,
        "combobox-basic" => combobox::spawn,
        "autocomplete-basic" => autocomplete::spawn,
        "search-selector-basic" => search_selector::spawn,
        "selection-panel-basic" => selection_panel::spawn,
        "pager-basic" => pager::spawn,
        "icon-button-basic" => icon_button::spawn,
        "split-button-basic" => split_button::spawn,
        "tooltip-basic" => tooltip::spawn,
        "popover-button-basic" => popover_button::spawn,
        "popup-menu-basic" => popup_menu::spawn,
        "context-menu-basic" => context_menu::spawn,
        "toolbar-basic" => toolbar::spawn,
        "accordion-basic" => accordion::spawn,
        "tree-view-basic" => tree_view::spawn,
        "table-basic" => table::spawn,
        "icon-group-basic" => icon_group::spawn,
        "progress-basic" => progress::spawn,
        "stepper-basic" => stepper::spawn,
        "color-swatch-basic" => color_swatch::spawn,

        "color-slider-basic" => color_slider::spawn,
        "color-field-basic" => color_field::spawn,
        "color-arc-basic" => color_arc::spawn,
        "color-ring-basic" => color_ring::spawn,
        "plot-cartesian-basic" => |cx| plot_cartesian::spawn(cx, false),
        "plot-cartesian-bars" => |cx| plot_cartesian::spawn(cx, true),
        "plot-pie-basic" => |cx| plot_pie::spawn(cx, false),
        "plot-pie-rings" => |cx| plot_pie::spawn(cx, true),
        "plot-radar-basic" => |cx| plot_radar::spawn(cx, false),
        "plot-radial-basic" => |cx| plot_radial::spawn(cx, false),
        _ => return Err(JsValue::from_str("Unknown example")),
    };
    gpui_web::init_logging();
    Application::with_platform(Rc::new(gpui_web::WebPlatform::new(false))).with_assets(Assets).run(
        move |cx: &mut App| {
            let result = xgpui_luma::init(cx).and_then(|_| {
                xgpui_luma::key_handling::bind_default_control_keys(cx);
                cx.text_system().add_fonts(vec![
                    Cow::Borrowed(include_bytes!("../../common/assets/google/inter/Inter-Variable.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/ibm-plex-sans/IBMPlexSans-Regular.ttf")),
                ])?;
                let look = radix::Look::built_in();
                // Use the embedded font's declared family instead of a desktop alias.
                let mut stylesheet = look.common_stylesheet();
                stylesheet.tokens.typography.font.sans.family = "Inter 24pt".into();
                look.set_common_stylesheet(stylesheet);
                look.set_mode(if DARK.get() { ThemeMode::Dark } else { ThemeMode::Light });
                cx.set_global(look.clone());
                let picker_look = xgpui_luma_look_shadcn::ShadcnLook::built_in();
                picker_look.set_mode(if DARK.get() { ThemeMode::Dark } else { ThemeMode::Light });
                cx.set_global(picker_look);
                cx.open_window(WindowOptions::default(), |_, cx| {
                    let sample = spawn(cx);
                    let view = cx.new(|_| Preview { sample, look });
                    VIEW.with_borrow_mut(|state| *state = Some((view.clone(), cx.to_async())));
                    view
                })?;
                if let Some(window) = web_sys::window() {
                    if let Ok(event) = web_sys::Event::new("luma-ready") {
                        let _ = window.dispatch_event(&event);
                    }
                }
                Ok(())
            });
            if let Err(error) = result {
                web_sys::console::error_1(&format!("Luma embed failed: {error:#}").into());
            }
        },
    );
    Ok(())
}
