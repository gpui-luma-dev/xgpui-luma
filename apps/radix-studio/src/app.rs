//! Radix Studio app shell — shared state, page routing, and window chrome.

use xgpui_luma::controls::radio_group::RadioGroupEventExt;
use xgpui_luma::controls::popup_menu::PopupMenuEventExt;
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::prelude::TooltipEntityExt;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

use std::sync::Arc;

use gpui::{
    Context, Entity, FocusHandle, Hsla, Render, RenderImage, AnyElement, ScrollHandle, Subscription, Task, Window, div,
    prelude::*, px,
};
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent};
use xgpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem, RadioGroupEvent};
use crate::controls::theme_mode;
use xgpui_luma::focus::LumaFocusScopeExt;
use xgpui_luma::infra::menu_item::{MenuItem, MenuItemIcon};
use xgpui_luma::shell::TitleBar;
use xgpui_luma::infra::element_ext::ElementExt;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::ThemeMode;
use xgpui_luma::{WideMiddle, dock_panel, spawn_wide_middle, vstack};
use xgpui_luma_look_radix::{Accent, Gray, Look, ScaleFamily, SemanticRole};
use xgpui_luma_look_radix as radix;

use crate::screens::custom_palette::{MeshCache, MeshRequest};
use crate::controls::{
    ControlBar, ClassicShadowEditor, ClassicShadowEditorEvent, ColorTextField, ColorTextFieldEvent, ScreenNav,
    ScreenNavEvent, SCREEN_TABS,
};
use crate::screens::{colors, custom_palette, developer, icons, style_guide};
use crate::tabs::RadixStudioTab;

const CONTENT_MAX_W: f32 = 1280.0;
const STARTUP_MODE: ThemeMode = ThemeMode::Dark;

use crate::controls::swatch_info::{self, ColorDetails, SwatchSelection};

// Resolve only paint fields to the current sRGB backend.
fn editor_preview(source: ColorValue) -> Hsla {
    gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap_or_else(|error| {
        eprintln!("failed to preview Radix editor color: {error}");
        gpui::black()
    })
}

/// Exact editor inputs, independent of the derived palette scale colors.
#[derive(Default)]
struct PaletteEdits {
    light: [Option<ColorValue>; 3],
    dark: [Option<ColorValue>; 3],
}

impl PaletteEdits {
    fn startup() -> Self {
        let mut edits = Self::default();
        edits.set(STARTUP_MODE, 2, ColorValue::srgb(0.0, 0.0, 0.0, 1.0));
        edits
    }

    fn for_mode(&self, mode: ThemeMode) -> &[Option<ColorValue>; 3] {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    fn colors(&self, theme: &Look) -> radix::CustomColors {
        // Defaults come from the named theme, never from the generated output.
        // Otherwise editing one input would silently change the other two seeds.
        let edits = self.for_mode(theme.mode());
        radix::CustomColors {
            accent: edits[0].unwrap_or_else(|| theme.resolve_role_source(SemanticRole::Primary).value),
            gray: edits[1].unwrap_or_else(|| theme.resolve_step_source(ScaleFamily::Gray, 8).value),
            background: edits[2].unwrap_or_else(|| theme.resolve_role_source(SemanticRole::Background).value),
        }
    }

    fn apply(&self, theme: &Look, draft: &Look) {
        if self.for_mode(theme.mode()).iter().any(Option::is_some) {
            if let Err(error) = draft.set_custom_colors(self.colors(theme)) {
                eprintln!("failed to generate Radix colors: {error}");
            }
        }
    }

    fn apply_app_theme(&self, defaults: &Look, theme: &Look) {
        // Other screens keep their standard background while sharing the edited seeds.
        if self.for_mode(defaults.mode())[..2].iter().any(Option::is_some) {
            let mut colors = self.colors(defaults);
            colors.background = defaults.resolve_role_source(SemanticRole::Background).value;
            if let Err(error) = theme.set_custom_colors(colors) {
                eprintln!("failed to generate app colors: {error}");
            }
        }
    }

    fn set_input(&mut self, mode: ThemeMode, index: usize, color: ColorValue, accent_linked: bool) {
        self.set(mode, index, color);
        if index == 0 && accent_linked {
            self.set(ThemeMode::Light, index, color);
            self.set(ThemeMode::Dark, index, color);
        }
    }

    fn set(&mut self, mode: ThemeMode, index: usize, color: ColorValue) {
        let values = match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        };
        values[index] = Some(color);
    }
}

pub struct RadixStudioApp {
    focus_scope: FocusHandle,
    /// Named palette defaults, independent of generated editor output.
    defaults: Arc<Look>,
    /// App-wide accent and gray, generated against the standard background.
    theme: Arc<Look>,
    /// Custom background palette shared by Custom Palette, Style Guide and Developer.
    draft: Arc<Look>,
    theme_accent: Accent,
    theme_gray: Gray,
    palette_edits: PaletteEdits,
    accent_linked: bool,
    screen_nav: Entity<ScreenNav>,
    control_bar: Entity<ControlBar>,
    page_scroll: ScrollHandle,
    screen_scrolls: Vec<(RadixStudioTab, ScrollHandle)>,
    screens: Entity<Tabs>,
    mode_selector: RadioGroup<RadioGroupItem>,
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    home: custom_palette::PreviewControls,
    style_guide: Option<style_guide::State>,
    shadow_editor: Entity<ClassicShadowEditor>,
    developer: Option<developer::State>,
    color_details: Entity<ColorDetails>,
    colors: Option<colors::State>,
    icons: Option<icons::State>,
    palette_swatches: Vec<Entity<Button>>,
    preview_layout: Entity<WideMiddle>,
    signup_mesh_cache: MeshCache,
    signup_mesh_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl RadixStudioApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        let theme_accent = Accent::Indigo;
        let theme_gray = Gray::Auto;
        let theme = Arc::new(Look::built_in());
        theme.set_mode(STARTUP_MODE);
        cx.set_global(theme.as_ref().clone());
        let defaults = Arc::new(theme.fork());
        let draft = Arc::new(theme.fork());
        let palette_edits = PaletteEdits::startup();
        let seed_colors = palette_edits.colors(&theme);
        palette_edits.apply(&theme, &draft);

        let mode_selector = theme_mode::spawn("palette-mode", &draft, cx);
        let mut screen_builder = radix::Tabs::new("screen-nav-tabs")
            .look(&theme)
            .line()
            .active("page-custom-palette")
            .content_layout(xgpui_luma::controls::tabs::TabsContentLayout::Fill)
            .content_transition(crate::screen_transition::directional_slide().focus_target({
                let app = cx.entity().downgrade();
                move |id, _, cx| {
                    app.upgrade().and_then(|app| app.read(cx).screen_nav.read(cx).page_focus_handle(id, cx))
                }
            }))
            .motion(|motion| {
                motion.content = xgpui_luma::motion::Transition::timed(
                    std::time::Duration::from_millis(400),
                    xgpui_luma::motion::Easing::OutCubic,
                )
            });
        for (screen, id, label) in SCREEN_TABS {
            let app = cx.entity().downgrade();
            screen_builder = screen_builder.tab_with(id, label, move |window, cx| {
                app.update(cx, |app, cx| app.render_screen(screen, window, cx))
                    .unwrap_or_else(|_| div().into_any_element())
            });
        }
        let screens = screen_builder.spawn(cx);
        let screen_nav = cx.new(|cx| ScreenNav::new(&theme, &draft, screens.clone(), cx));
        let control_bar = cx.new(|_| ControlBar::new(&theme, screen_nav.clone()));
        let screen_scrolls: Vec<_> = SCREEN_TABS.iter().map(|(tab, _, _)| (*tab, ScrollHandle::new())).collect();
        let page_scroll = screen_scrolls[0].1.clone();
        let home = custom_palette::PreviewControls::spawn(&draft, cx);
        let accent_color = seed_colors.accent;
        let gray_color = seed_colors.gray;
        let background_color = seed_colors.background;
        let accent_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-accent", accent_color, cx))
            .help("Accent seed color. Click the swatch to pick a color or enter a hex value.", cx);
        let gray_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-gray", gray_color, cx))
            .help("Gray seed color for neutral surfaces and text. Click the swatch or enter a hex value.", cx);
        let background_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-background", background_color, cx))
            .help("Background seed color. Click the swatch to pick a color or enter a hex value.", cx);

        accent_field.update(cx, |field, cx| field.enable_accent_link(cx));

        let mut subscriptions = Vec::new();
        // Screen presenters read app-owned state; refresh them when that state changes.
        subscriptions.push(cx.observe(&cx.entity(), {
            let screens = screens.downgrade();
            let control_bar = control_bar.downgrade();
            move |_, _, cx| {
                if let Some(bar) = control_bar.upgrade() {
                    bar.update(cx, |_, cx| cx.notify());
                }
                if let Some(screens) = screens.upgrade() {
                    screens.update(cx, |tabs, cx| tabs.refresh_content(cx));
                }
            }
        }));
        for (index, field) in [(0, &accent_field), (1, &gray_field), (2, &background_field)] {
            subscriptions.push(cx.subscribe(field, move |this, _, event: &ColorTextFieldEvent, cx| {
                let color = match event {
                    ColorTextFieldEvent::Change { color } => *color,
                    ColorTextFieldEvent::LinkChange { linked } => {
                        this.accent_linked = *linked;
                        if !linked {
                            cx.notify();
                            return;
                        }
                        this.palette_edits.colors(&this.defaults).accent
                    }
                };
                this.palette_edits.set_input(this.draft.mode(), index, color, this.accent_linked);
                if matches!(event, ColorTextFieldEvent::LinkChange { linked: true }) {
                    // Regenerate the other mode immediately, preserving its gray/background seeds.
                    let mode = this.defaults.mode();
                    let other = match mode {
                        ThemeMode::Light => ThemeMode::Dark,
                        ThemeMode::Dark => ThemeMode::Light,
                    };
                    let defaults = this.defaults.fork();
                    defaults.set_mode(other);
                    this.draft.set_mode(other);
                    this.theme.set_mode(other);
                    this.palette_edits.apply(&defaults, &this.draft);
                    this.palette_edits.apply_app_theme(&defaults, &this.theme);
                    this.draft.set_mode(mode);
                    this.theme.set_mode(mode);
                }
                this.palette_edits.apply(&this.defaults, &this.draft);
                this.palette_edits.apply_app_theme(&this.defaults, &this.theme);
                this.home.notify(cx);
                this.screen_nav.update(cx, |nav, cx| nav.theme_changed(cx));
                cx.refresh_windows();
                cx.notify();
            }));
        }

        let copy_menu = radix::PopupMenu::new("copy-palette")
            .look(&draft)
            .solid()
            .primary()
            .label("Copy")
            .items([
                MenuItem::new("copy-url")
                    .label("Copy palette URL")
                    .icon(MenuItemIcon::asset("assets/react-icons/share-2.svg")),
                MenuItem::separator("copy-url-separator"),
                MenuItem::new("copy-css")
                    .label("Copy CSS code")
                    .icon(MenuItemIcon::asset("assets/react-icons/copy.svg"))
                    .submenu([
                        MenuItem::new("copy-accent").label("Copy accent scale"),
                        MenuItem::new("copy-gray").label("Copy gray scale"),
                        MenuItem::new("copy-background").label("Copy background color"),
                    ]),
                MenuItem::new("copy-svg")
                    .label("Copy SVG object")
                    .icon(MenuItemIcon::asset("assets/react-icons/figma-logo.svg")),
            ])
            .spawn(cx);

        copy_menu.clone().on_event(cx, |this, event: &PopupMenuEvent, cx| {
            if let PopupMenuEvent::Select { item_id, .. } = event {
                if let Some(text) = this.palette_copy_text(item_id.as_ref()) {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                }
            }
        });

        let color_details = cx.new(|cx| ColorDetails::new(&draft, cx));
        let mut palette_swatches = Vec::new();
        for family in [ScaleFamily::Color, ScaleFamily::Gray] {
            for step in 1..=12 {
                let swatch_look = Arc::clone(&draft);
                let swatch = swatch_info::swatch(
                    format!("swatch-{}-{step}", family.as_str()),
                    &draft,
                    None,
                    44.0,
                    move || swatch_look.resolve_step(family, step).hsla(),
                    cx,
                )
                .on_click(cx, move |this, cx| {
                    this.open_swatch_info(family, step, cx);
                });
                palette_swatches.push(swatch);
            }
        }

        mode_selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
            if let RadioGroupEvent::Change { value: Some(value) } = event {
                this.set_mode(
                    if value.as_ref() == "light" {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    },
                    cx,
                );
            }
        });
        subscriptions.push(cx.subscribe(&screen_nav, |this, _, event: &ScreenNavEvent, cx| match event {
            ScreenNavEvent::Change { tab } => {
                if let Some((_, handle)) = this.screen_scrolls.iter().find(|(screen, _)| screen == tab) {
                    this.page_scroll = handle.clone();
                    let duration = match this.screens.read(cx).motion_profile().content.description() {
                        xgpui_luma::motion::TransitionDescription::Timed { duration, .. } => duration,
                        _ => std::time::Duration::from_millis(400),
                    };
                    this.control_bar.update(cx, |bar, cx| bar.select(*tab, f32::from(handle.offset().y), duration, cx));
                }
            }
            ScreenNavEvent::ModeChange { mode } => this.set_mode(*mode, cx),
            ScreenNavEvent::ResetTheme => this.reset_theme(cx),
        }));
        let shadow_editor = cx.new(|cx| ClassicShadowEditor::new(&theme, cx));
        subscriptions.push(cx.subscribe(&shadow_editor, |_this, _, event: &ClassicShadowEditorEvent, cx| {
            let ClassicShadowEditorEvent::Changed = event;
            cx.notify();
        }));

        let preview_layout = spawn_wide_middle(custom_palette::preview_layout_defaults(), cx);

        Self {
            focus_scope,
            defaults,
            theme,
            draft,
            theme_accent,
            theme_gray,
            palette_edits,
            accent_linked: true,
            screen_nav,
            control_bar,
            page_scroll,
            screen_scrolls,
            screens,
            mode_selector,
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            home,
            style_guide: None,
            developer: None,
            shadow_editor,
            color_details,
            colors: None,
            icons: None,
            palette_swatches,
            preview_layout,
            signup_mesh_cache: MeshCache::default(),
            signup_mesh_task: None,
            _subscriptions: subscriptions,
        }
    }

    #[cfg(all(test, feature = "test-support"))]
    fn active_tab(&self, cx: &gpui::App) -> RadixStudioTab {
        self.screen_nav.read(cx).active()
    }

    fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.defaults.set_mode(mode);
        self.theme.set_mode(mode);
        self.theme.set_palettes(self.theme_accent, self.theme_gray);
        self.draft.set_mode(mode);
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.palette_edits.apply(&self.defaults, &self.draft);
        self.palette_edits.apply_app_theme(&self.defaults, &self.theme);
        self.mode_selector.update(cx, |group, cx| group.set_selected(theme_mode::mode_id(mode), cx));
        self.screen_nav.update(cx, |nav, cx| nav.theme_changed(cx));
        self.sync_seed_fields(cx);
        cx.refresh_windows();
        cx.notify();
    }

    fn palette_copy_text(&self, action: &str) -> Option<String> {
        use crate::color_hex::format_hex;
        match action {
            "copy-url" => {
                let theme = self.defaults.fork();
                let mut params = Vec::new();
                for (mode, suffix) in [(ThemeMode::Light, "light"), (ThemeMode::Dark, "dark")] {
                    theme.set_mode(mode);
                    let colors = self.palette_edits.colors(&theme);
                    for (name, color) in
                        [("accent", colors.accent), ("gray", colors.gray), ("background", colors.background)]
                    {
                        params.push(format!("{name}-{suffix}={}", format_hex(editor_preview(color))));
                    }
                }
                Some(format!("https://www.radix-ui.com/colors/custom?{}", params.join("&")))
            }
            "copy-accent" | "copy-gray" => {
                let (family, name) = if action == "copy-accent" {
                    (ScaleFamily::Color, "accent")
                } else {
                    (ScaleFamily::Gray, "gray")
                };
                let declarations: Vec<String> = (1..=12)
                    .map(|step| {
                        format!("  --{name}-{step}: #{};", format_hex(self.draft.resolve_step(family, step).hsla()))
                    })
                    .collect();
                Some(format!(":root {{\n{}\n}}", declarations.join("\n")))
            }
            "copy-background" => Some(format!(
                "--background: #{};",
                format_hex(editor_preview(self.palette_edits.colors(&self.defaults).background))
            )),
            "copy-svg" => {
                let mut svg = String::from(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"200\" viewBox=\"0 0 1200 200\">\n",
                );
                for (row, family) in [ScaleFamily::Color, ScaleFamily::Gray].into_iter().enumerate() {
                    for step in 1..=12 {
                        svg.push_str(&format!(
                            "  <rect x=\"{}\" y=\"{}\" width=\"100\" height=\"100\" fill=\"#{}\"/>\n",
                            usize::from(step - 1) * 100,
                            row * 100,
                            format_hex(self.draft.resolve_step(family, step).hsla())
                        ));
                    }
                }
                svg.push_str("</svg>");
                Some(svg)
            }
            _ => None,
        }
    }

    /// Restores both looks and the theme editors to their startup values.
    fn reset_theme(&mut self, cx: &mut Context<Self>) {
        self.palette_edits = PaletteEdits::startup();
        self.theme.set_palettes(self.theme_accent, self.theme_gray);
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.draft.set_classic_params(Default::default());
        self.shadow_editor.update(cx, |editor, cx| editor.reset(cx));
        self.set_mode(STARTUP_MODE, cx);
    }

    /// Prepaint supplies dimensions only; all raster work runs off the UI thread.
    pub fn request_signup_mesh(&mut self, width: u32, height: u32, cx: &mut Context<Self>) {
        let Some(mut request) = self.signup_mesh_cache.request(MeshRequest::for_look(&self.draft, width, height))
        else {
            return;
        };
        self.signup_mesh_task = Some(cx.spawn(async move |this, cx| {
            loop {
                let image = cx.background_executor().spawn(async move { request.render() }).await;
                let next = this.update(cx, |this, cx| {
                    let next = this.signup_mesh_cache.complete(request.key, image);
                    if next.is_none() {
                        this.signup_mesh_task = None;
                        cx.notify();
                    }
                    next
                });
                match next {
                    Ok(Some(latest)) => request = latest,
                    _ => break,
                }
            }
        }));
    }

    fn cached_signup_mesh(&self) -> Option<Arc<RenderImage>> {
        self.signup_mesh_cache.image()
    }

    fn sync_seed_fields(&self, cx: &mut Context<Self>) {
        self.home.notify(cx);
        let colors = self.palette_edits.colors(&self.defaults);
        self.accent_field.update(cx, |field, cx| field.set_color(colors.accent, cx));
        self.gray_field.update(cx, |field, cx| field.set_color(colors.gray, cx));
        self.background_field.update(cx, |field, cx| field.set_color(colors.background, cx));
    }

    pub fn open_swatch_info(&mut self, family: ScaleFamily, step: u8, cx: &mut Context<Self>) {
        let selection =
            match SwatchSelection::new(&self.draft, family, step, self.palette_edits.colors(&self.defaults).background)
            {
                Ok(selection) => selection,
                Err(error) => {
                    eprintln!("failed to resolve swatch details: {error}");
                    return;
                }
            };
        let focus = self.focus_scope.clone();
        self.color_details.update(cx, |panel, cx| panel.open(selection, Some(focus), cx));
        cx.notify();
    }

    fn surface(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Surface).hsla()
    }

    fn border(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Border).hsla()
    }

    fn fg(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Foreground).hsla()
    }

    fn muted(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::MutedForeground).hsla()
    }

    /// Chrome colors for the Custom Palette screen, which follows the draft instead.
    fn draft_chrome(&self) -> DraftChrome {
        DraftChrome {
            surface: self.draft.resolve_role(SemanticRole::Surface).hsla(),
            border: self.draft.resolve_role(SemanticRole::Border).hsla(),
            fg: self.draft.resolve_role(SemanticRole::Foreground).hsla(),
            muted: self.draft.resolve_role(SemanticRole::MutedForeground).hsla(),
            accent: self.draft.resolve_role(SemanticRole::Primary).hsla(),
        }
    }
}

struct DraftChrome {
    surface: Hsla,
    border: Hsla,
    fg: Hsla,
    muted: Hsla,
    accent: Hsla,
}

impl RadixStudioApp {
    fn screen_background(&self, tab: RadixStudioTab) -> gpui::Background {
        match tab {
            RadixStudioTab::Colors => colors::page_background(self.theme.mode()),
            // These screens share the editor's background and gradient.
            RadixStudioTab::CustomPalette | RadixStudioTab::StyleGuide | RadixStudioTab::Developer => {
                let recipe = radix::PageBackground { settle_at: 0.5, ..Default::default() };
                let (start, _) = recipe.stops(&self.draft);
                let end = editor_preview(self.palette_edits.colors(&self.defaults).background);
                gpui::linear_gradient(
                    180.0,
                    gpui::linear_color_stop(start, 0.0),
                    gpui::linear_color_stop(end, recipe.settle_at),
                )
            }
            RadixStudioTab::Icons => self.defaults.page_background(),
        }
    }

    fn render_screen(&mut self, active_tab: RadixStudioTab, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let scroll = self
            .screen_scrolls
            .iter()
            .find(|(tab, _)| *tab == active_tab)
            .map(|(_, handle)| handle.clone())
            .unwrap_or_default();
        let observed_scroll = scroll.clone();
        let bar = self.control_bar.downgrade();
        let surface = self.surface();
        let fg = self.fg();
        let muted = self.muted();
        let draft_chrome = self.draft_chrome();
        let content = match active_tab {
            RadixStudioTab::CustomPalette => custom_palette::page(
                custom_palette::PageArgs {
                    look: &self.draft,
                    mode_selector: self.mode_selector.clone(),
                    accent_field: self.accent_field.clone(),
                    gray_field: self.gray_field.clone(),
                    background_field: self.background_field.clone(),
                    copy_menu: self.copy_menu.clone(),
                    palette_swatches: self.palette_swatches.clone(),
                    preview_layout: self.preview_layout.clone(),
                    mesh_image: self.cached_signup_mesh(),
                    app: cx.entity(),
                    controls: self.home.clone(),
                    surface: draft_chrome.surface,
                    border: draft_chrome.border,
                    muted: draft_chrome.muted,
                    fg: draft_chrome.fg,
                    accent: draft_chrome.accent,
                },
                cx,
            ),
            RadixStudioTab::Colors => self
                .colors
                .get_or_insert_with(|| colors::State::new(&self.theme, self.color_details.clone(), cx))
                .render(&self.theme, fg, muted),
            RadixStudioTab::Icons => {
                self.icons.get_or_insert_with(|| icons::State::new(&self.defaults, cx)).render(fg, muted, surface)
            }
            RadixStudioTab::StyleGuide => {
                let guide = self.style_guide.get_or_insert_with(|| style_guide::State::new(&self.draft, cx));
                guide.render(&self.draft, window, cx)
            }
            RadixStudioTab::Developer => self
                .developer
                .get_or_insert_with(|| developer::State::new(&self.draft, self.shadow_editor.clone(), cx))
                .render(cx),
        };
        let body = vstack! { gap=28; content };
        let viewport = div()
            .id("radix-studio-scroll")
            .debug_selector(move || format!("screen-scroll-{active_tab:?}"))
            .relative()
            .flex_1()
            .w_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .on_prepaint(move |_, window, cx| {
                if let Some(bar) = bar.upgrade() {
                    bar.update(cx, |bar, cx| {
                        if bar.scroll_changed(active_tab, f32::from(observed_scroll.offset().y)) {
                            cx.on_next_frame(window, |_, _, cx| cx.notify());
                        }
                    });
                }
            })
            .when(active_tab == RadixStudioTab::Icons, |d| d.child(icons::hero_decoration(fg)))
            .child(body.relative().w_full().max_w(px(CONTENT_MAX_W)).mx_auto().px_8().pt(px(20.0)).pb_8());
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(self.screen_background(active_tab))
            .debug_selector(move || format!("screen-panel-{active_tab:?}"))
            .child(viewport)
            .into_any_element()
    }
}

impl Render for RadixStudioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface();
        let border = self.border();
        let fg = self.fg();

        let title_bar = TitleBar::new().background_color(surface).border_color(border).text_color(fg).child(
            div()
                .id("radix-studio-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .text_color(fg)
                .child("Radix Studio"),
        );

        dock_panel! {
            top: title_bar,
            fill: div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .child(
                    // This child redraws independently for scroll and fade changes.
                    self.control_bar.clone(),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(self.screens.read(cx).body()),
                )
                .child(self.color_details.clone()),
        }
        .into_element()
        .luma_focus_scope(&self.focus_scope)
        .size_full()
        .bg(self.theme.page_background())
        .text_color(fg)
        .font_family("System UI")
    }
}

#[cfg(test)]
mod palette_edit_tests {
    use super::*;

    #[test]
    fn linked_accent_updates_both_modes_without_linking_other_inputs() {
        let mut edits = PaletteEdits::default();
        let color = ColorValue::srgb(1.0, 0.2, 0.4, 1.0);
        edits.set_input(ThemeMode::Dark, 0, color, true);
        assert_eq!(edits.light[0], Some(color));
        assert_eq!(edits.dark[0], Some(color));
        edits.set_input(ThemeMode::Dark, 1, color, true);
        assert_eq!(edits.light[1], None);
        let other = ColorValue::srgb(0.2, 0.4, 1.0, 1.0);
        edits.set_input(ThemeMode::Light, 0, other, false);
        assert_eq!(edits.light[0], Some(other));
        assert_eq!(edits.dark[0], Some(color));
    }

    #[test]
    fn startup_and_reset_background_are_black() {
        let theme = Look::built_in();
        theme.set_mode(STARTUP_MODE);
        let edits = PaletteEdits::startup();
        assert_eq!(edits.colors(&theme).background, ColorValue::srgb(0.0, 0.0, 0.0, 1.0));
        let draft = theme.fork();
        edits.apply(&theme, &draft);
        assert_eq!(draft.resolve_step(ScaleFamily::Gray, 1).hsla().l, 0.0);
        theme.set_mode(ThemeMode::Light);
        assert!(editor_preview(edits.colors(&theme).background).l > 0.9);
    }

    #[test]
    fn editor_inputs_regenerate_without_feedback_and_restore_per_mode() {
        let theme = Look::built_in();
        theme.set_mode(ThemeMode::Dark);
        let draft = theme.fork();
        let mut edits = PaletteEdits::default();
        let original = edits.colors(&theme);
        edits.set(ThemeMode::Dark, 0, ColorValue::srgb(148.0 / 255.0, 17.0 / 255.0, 0.0, 1.0));
        edits.apply(&theme, &draft);
        let inputs = edits.colors(&theme);
        assert_eq!(inputs.gray, original.gray);
        assert_eq!(inputs.background, original.background);
        assert_ne!(draft.resolve_step(ScaleFamily::Color, 9).hsla(), theme.resolve_step(ScaleFamily::Color, 9).hsla());
        let old_gray = draft.resolve_step(ScaleFamily::Gray, 8).hsla();
        edits.set(ThemeMode::Dark, 1, gpui_bridge::from_rgba(gpui::rgb(0x807060)));
        edits.apply(&theme, &draft);
        assert_ne!(draft.resolve_step(ScaleFamily::Gray, 8).hsla(), old_gray);
        let old_background = draft.resolve_step(ScaleFamily::Color, 1).hsla();
        edits.set(ThemeMode::Dark, 2, ColorValue::srgb(0.0, 0.0, 0.0, 1.0));
        edits.apply(&theme, &draft);
        assert_ne!(draft.resolve_step(ScaleFamily::Color, 1).hsla(), old_background);
        let dark_colors = (1..=12).map(|step| draft.resolve_step(ScaleFamily::Color, step).hsla()).collect::<Vec<_>>();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            theme.set_mode(mode);
            draft.set_mode(mode);
            draft.set_palettes(Accent::Indigo, Gray::Auto);
            edits.apply(&theme, &draft);
        }
        assert_eq!(
            (1..=12).map(|step| draft.resolve_step(ScaleFamily::Color, step).hsla()).collect::<Vec<_>>(),
            dark_colors
        );
        edits = PaletteEdits::default();
        draft.set_palettes(Accent::Indigo, Gray::Auto);
        edits.apply(&theme, &draft);
        assert_eq!(draft.resolve_step(ScaleFamily::Color, 9).hsla(), theme.resolve_step(ScaleFamily::Color, 9).hsla());
    }

    #[test]
    fn all_three_inputs_survive_mode_changes_and_reset_clears_both_modes() {
        let mut edits = PaletteEdits::default();
        let dark = [
            gpui_bridge::from_rgba(gpui::rgb(0xe85454)),
            gpui_bridge::from_rgba(gpui::rgb(0x555555)),
            gpui_bridge::from_rgba(gpui::rgb(0x000000)),
        ];
        let light = [
            gpui_bridge::from_rgba(gpui::rgb(0x3264ff)),
            gpui_bridge::from_rgba(gpui::rgb(0xaaaaaa)),
            gpui_bridge::from_rgba(gpui::rgb(0xffffff)),
        ];
        for (index, color) in dark.into_iter().enumerate() {
            edits.set(ThemeMode::Dark, index, color);
        }
        assert_eq!(*edits.for_mode(ThemeMode::Light), [None; 3]);
        for (index, color) in light.into_iter().enumerate() {
            edits.set(ThemeMode::Light, index, color);
        }
        assert_eq!(*edits.for_mode(ThemeMode::Dark), dark.map(Some));
        assert_eq!(*edits.for_mode(ThemeMode::Light), light.map(Some));
        edits = PaletteEdits::default();
        assert_eq!(*edits.for_mode(ThemeMode::Dark), [None; 3]);
        assert_eq!(*edits.for_mode(ThemeMode::Light), [None; 3]);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod screen_content_tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn linking_immediately_synchronizes_seeds_and_both_palette_modes() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.run_until_parked();
        let accent = ColorValue::srgb(0.9, 0.1, 0.3, 1.0);
        cx.update(|_, app| {
            studio.update(app, |studio, cx| {
                studio.set_mode(ThemeMode::Dark, cx);
                studio.palette_edits.set(ThemeMode::Dark, 0, ColorValue::srgb(0.1, 0.7, 0.2, 1.0));
                studio.set_mode(ThemeMode::Light, cx);
                studio.palette_edits.set(ThemeMode::Light, 0, accent);
                let field = studio.accent_field.clone();
                field.update(cx, |_, cx| cx.emit(ColorTextFieldEvent::LinkChange { linked: true }));
            })
        });
        cx.run_until_parked();
        cx.update(|_, app| {
            studio.update(app, |studio, _| {
                assert!(studio.accent_linked);
                assert_eq!(studio.palette_edits.light[0], Some(accent));
                assert_eq!(studio.palette_edits.dark[0], Some(accent));
                assert_eq!(studio.palette_edits.dark[2], Some(ColorValue::srgb(0.0, 0.0, 0.0, 1.0)));
                assert_eq!(studio.draft.mode(), ThemeMode::Light);
                assert_eq!(studio.theme.mode(), ThemeMode::Light);
                for mode in [ThemeMode::Light, ThemeMode::Dark] {
                    let defaults = studio.defaults.fork();
                    defaults.set_mode(mode);
                    let expected_draft = defaults.fork();
                    let expected_theme = defaults.fork();
                    studio.palette_edits.apply(&defaults, &expected_draft);
                    studio.palette_edits.apply_app_theme(&defaults, &expected_theme);
                    studio.draft.set_mode(mode);
                    studio.theme.set_mode(mode);
                    for step in 1..=12 {
                        assert_eq!(
                            studio.draft.resolve_step(ScaleFamily::Color, step).hsla(),
                            expected_draft.resolve_step(ScaleFamily::Color, step).hsla()
                        );
                        assert_eq!(
                            studio.theme.resolve_step(ScaleFamily::Color, step).hsla(),
                            expected_theme.resolve_step(ScaleFamily::Color, step).hsla()
                        );
                    }
                }
                studio.draft.set_mode(ThemeMode::Light);
                studio.theme.set_mode(ThemeMode::Light);
            })
        });
    }

    #[test]
    fn copy_exports_follow_edited_palette_and_mode() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.update(|_, cx| {
            studio.update(cx, |studio, cx| {
                let color = ColorValue::srgb(1.0, 0.0, 0.0, 1.0);
                studio.palette_edits.set_input(ThemeMode::Dark, 0, color, true);
                studio.palette_edits.apply(&studio.defaults, &studio.draft);
                let url = studio.palette_copy_text("copy-url").unwrap();
                assert!(url.contains("accent-light=FF0000"));
                assert!(url.contains("accent-dark=FF0000"));
                assert!(url.contains("background-dark=000000"));
                for mode in [ThemeMode::Light, ThemeMode::Dark] {
                    studio.set_mode(mode, cx);
                    let css = studio.palette_copy_text("copy-accent").unwrap();
                    let expected =
                        crate::color_hex::format_hex(studio.draft.resolve_step(ScaleFamily::Color, 9).hsla());
                    assert!(css.contains(&format!("--accent-9: #{expected};")));
                    assert_eq!(css.matches("--accent-").count(), 12);
                    let svg = studio.palette_copy_text("copy-svg").unwrap();
                    assert_eq!(svg.matches("<rect ").count(), 24);
                    assert!(svg.contains(&format!("fill=\"#{expected}\"")));
                }
                assert!(studio.palette_copy_text("copy-css").is_none());
            })
        });
    }

    #[test]
    fn seed_edits_update_app_theme_and_preserve_separate_backgrounds() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.run_until_parked();
        let accent = ColorValue::srgb(0.85, 0.2, 0.65, 1.0);
        let gray = ColorValue::srgb(0.4, 0.5, 0.45, 1.0);
        let background = ColorValue::srgb(0.04, 0.02, 0.06, 1.0);
        for (index, color) in [accent, gray, background].into_iter().enumerate() {
            cx.update(|_, cx| {
                let fields = {
                    let studio = studio.read(cx);
                    [studio.accent_field.clone(), studio.gray_field.clone(), studio.background_field.clone()]
                };
                fields[index].update(cx, |_, cx| cx.emit(ColorTextFieldEvent::Change { color }));
            });
            cx.run_until_parked();
        }
        cx.update(|_, cx| {
            let studio = studio.read(cx);
            let expected = studio.defaults.fork();
            expected
                .set_custom_colors(radix::CustomColors {
                    accent,
                    gray,
                    background: studio.defaults.resolve_role_source(SemanticRole::Background).value,
                })
                .unwrap();
            for family in [ScaleFamily::Color, ScaleFamily::Gray] {
                for step in 1..=12 {
                    assert_eq!(
                        studio.theme.resolve_step_source(family, step).value,
                        expected.resolve_step_source(family, step).value
                    );
                }
            }
            assert_eq!(studio.palette_edits.colors(&studio.defaults).background, background);
            assert_ne!(
                studio.draft.resolve_step_source(ScaleFamily::Gray, 1).value,
                studio.theme.resolve_step_source(ScaleFamily::Gray, 1).value
            );
        });
        for (_, id, _) in SCREEN_TABS {
            cx.update(|_, cx| {
                let screens = studio.read(cx).screens.clone();
                screens.update(cx, |tabs, cx| tabs.set_active(id, cx));
            });
            cx.run_until_parked();
        }
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            cx.update(|_, cx| studio.update(cx, |studio, cx| studio.set_mode(mode, cx)));
            cx.run_until_parked();
        }
        cx.update(|_, cx| studio.update(cx, |studio, cx| studio.reset_theme(cx)));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let studio = studio.read(cx);
            assert_eq!(
                studio.theme.resolve_step_source(ScaleFamily::Color, 9).value,
                studio.defaults.resolve_step_source(ScaleFamily::Color, 9).value
            );
        });
    }

    #[test]
    fn clicking_palette_swatch_opens_details_and_close_dismisses() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            RadixStudioApp::new(window, cx)
        });
        cx.simulate_resize(gpui::size(px(1400.0), px(900.0)));
        cx.run_until_parked();
        let bounds = cx.debug_bounds("swatch-color-5").expect("palette swatch");
        cx.simulate_click(bounds.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| {
            let studio = studio.read(cx);
            assert!(studio.color_details.read(cx).is_open(cx));
            let selection = studio.color_details.read(cx).selected().unwrap();
            assert_eq!(selection.step, 5);
            assert_eq!(selection.color, studio.draft.resolve_step(ScaleFamily::Color, 5).hsla());
            assert!(selection.solid.starts_with('#'));
        });
        let close = cx.debug_bounds("swatch-info-close").expect("close button");
        cx.simulate_click(close.center(), Default::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!studio.read(cx).color_details.read(cx).is_open(cx)));
    }

    #[test]
    fn alpha_catalog_swatches_have_visible_preview_geometry() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            RadixStudioApp::new(window, cx)
        });
        cx.simulate_resize(gpui::size(px(1400.0), px(3000.0)));
        cx.update(|_, cx| {
            let screens = studio.read(cx).screens.clone();
            screens.update(cx, |tabs, cx| tabs.set_active("page-colors", cx));
        });
        cx.run_until_parked();
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            cx.update(|_, cx| studio.update(cx, |studio, cx| studio.set_mode(mode, cx)));
            cx.run_until_parked();
            for (id, preview_id) in [
                ("catalog-Black-1", "catalog-Black-1-preview"),
                ("catalog-Black-12", "catalog-Black-12-preview"),
                ("catalog-White-1", "catalog-White-1-preview"),
                ("catalog-White-12", "catalog-White-12-preview"),
            ] {
                let cell = cx.debug_bounds(id).expect("alpha swatch cell");
                let preview = cx.debug_bounds(preview_id).expect("alpha preview");
                assert_eq!(preview.size.width, cell.size.width - px(2.0), "{id}");
                assert_eq!(preview.size.height, cell.size.height - px(2.0), "{id}");
                cx.simulate_click(cell.center(), Default::default());
                cx.run_until_parked();
                cx.update(|_, app| {
                    let panel = studio.read(app).color_details.read(app);
                    assert!(panel.is_open(app));
                    let selected = panel.selected().expect("selected alpha color");
                    let values = if id.contains("Black") {
                        &radix::BLACK_ALPHA_STEPS
                    } else {
                        &radix::WHITE_ALPHA_STEPS
                    };
                    let index = if id.ends_with("-1") { 0 } else { 11 };
                    assert_eq!(selected.color, radix::parse_color(values[index]));
                    assert!(selected.color.a < 1.0);
                });
                let values = cx.update(|_, app| {
                    let selected = studio.read(app).color_details.read(app).selected().unwrap();
                    [selected.solid, selected.alpha, selected.hsl, selected.hsla, selected.p3, selected.p3_alpha]
                });
                for (index, (copy_id, expected)) in [
                    "swatch-info-copy-solid",
                    "swatch-info-copy-alpha",
                    "swatch-info-copy-hsl",
                    "swatch-info-copy-hsla",
                    "swatch-info-copy-p3",
                    "swatch-info-copy-p3-alpha",
                ]
                .into_iter()
                .zip(values)
                .enumerate()
                {
                    let copy = cx.debug_bounds(copy_id).expect("copy button");
                    cx.simulate_click(copy.center(), Default::default());
                    cx.run_until_parked();
                    cx.update(|_, app| {
                        assert_eq!(app.read_from_clipboard().and_then(|item| item.text()), Some(expected));
                        assert!(studio.read(app).color_details.read(app).is_open(app));
                        assert!(studio.read(app).color_details.read(app).is_copied(index, app));
                    });
                }
                cx.executor().advance_clock(std::time::Duration::from_millis(1500));
                cx.run_until_parked();
                cx.update(|_, app| {
                    for index in 0..6 {
                        assert!(!studio.read(app).color_details.read(app).is_copied(index, app));
                    }
                });
                let popup = cx.debug_bounds("swatch-info-preview").expect("popup swatch preview");
                assert!(popup.size.width > px(0.0));
                assert_eq!(popup.size.height, px(240.0));
                let close = cx.debug_bounds("swatch-info-close").expect("close button");
                cx.simulate_click(close.center(), Default::default());
                cx.run_until_parked();
            }
        }
    }

    #[test]
    fn colors_catalog_opens_shared_details_with_current_mode_values() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            RadixStudioApp::new(window, cx)
        });
        cx.simulate_resize(gpui::size(px(1400.0), px(900.0)));
        cx.update(|_, cx| {
            let screens = studio.read(cx).screens.clone();
            screens.update(cx, |tabs, cx| tabs.set_active("page-colors", cx));
        });
        cx.run_until_parked();
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            cx.update(|_, cx| studio.update(cx, |studio, cx| studio.set_mode(mode, cx)));
            cx.run_until_parked();
            let bounds = cx.debug_bounds("catalog-gray-5").expect("catalog swatch");
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            cx.update(|_, cx| {
                let panel = studio.read(cx).color_details.read(cx);
                assert!(panel.is_open(cx));
                let selected = panel.selected().unwrap();
                assert_eq!(selected.title, "Gray 5");
                let family = radix::color_families(mode).iter().find(|family| family.family == "gray").unwrap();
                assert_eq!(selected.color, radix::parse_color(family.steps[4]));
            });
            let close = cx.debug_bounds("swatch-info-close").unwrap();
            cx.simulate_click(close.center(), Default::default());
            cx.run_until_parked();
        }
    }

    #[test]
    fn screen_scroll_positions_survive_navigation() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.simulate_resize(gpui::size(px(900.0), px(700.0)));
        cx.run_until_parked();
        let screens = cx.update(|_, cx| studio.read(cx).screens.clone());
        cx.update(|_, cx| {
            screens.update(cx, |tabs, cx| {
                tabs.set_motion_enabled(false, cx);
                tabs.set_active("page-style-guide", cx);
            })
        });
        cx.run_until_parked();
        let saved = cx.update(|window, cx| {
            let handle = studio.read(cx).page_scroll.clone();
            handle.set_offset(gpui::point(px(0.0), px(-240.0)));
            window.refresh();
            handle
        });
        cx.run_until_parked();
        let offset = saved.offset();
        assert!(offset.y < px(0.0));
        cx.update(|_, cx| screens.update(cx, |tabs, cx| tabs.set_active("page-colors", cx)));
        cx.run_until_parked();
        cx.update(|_, cx| screens.update(cx, |tabs, cx| tabs.set_active("page-style-guide", cx)));
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(studio.read(cx).page_scroll.offset(), offset));
        assert_eq!(saved.offset(), offset);
    }

    #[test]
    fn scrolling_updates_control_bar_without_notifying_app() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.simulate_resize(gpui::size(px(900.0), px(700.0)));
        cx.run_until_parked();
        let screens = cx.update(|_, cx| studio.read(cx).screens.clone());
        cx.update(|_, cx| {
            screens.update(cx, |tabs, cx| {
                tabs.set_motion_enabled(false, cx);
                tabs.set_active("page-style-guide", cx);
            })
        });
        cx.run_until_parked();
        let app_notifications = std::rc::Rc::new(std::cell::Cell::new(0));
        let bar_notifications = std::rc::Rc::new(std::cell::Cell::new(0));
        let bar = cx.update(|_, cx| studio.read(cx).control_bar.clone());
        let _app_subscription = cx.update(|_, cx| {
            cx.observe(&studio, {
                let count = app_notifications.clone();
                move |_, _| count.set(count.get() + 1)
            })
        });
        let _bar_subscription = cx.update(|_, cx| {
            cx.observe(&bar, {
                let count = bar_notifications.clone();
                move |_, _| count.set(count.get() + 1)
            })
        });
        let bounds = cx.debug_bounds("screen-scroll-StyleGuide").unwrap();
        cx.simulate_event(gpui::ScrollEvent {
            position: gpui::point(bounds.left() + px(4.0), bounds.top() + px(120.0)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-120.0))),
            modifiers: Default::default(),
            source: gpui::ScrollSource::Pointer,
            touch_phase: gpui::TouchPhase::Moved,
        });
        cx.run_until_parked();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(cx.update(|_, cx| studio.read(cx).page_scroll.offset().y) < px(0.0));
        assert!(bar_notifications.get() > 0);
        assert_eq!(app_notifications.get(), 0);
    }

    #[test]
    fn main_screen_slots_render_and_follow_programmatic_selection() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.simulate_resize(gpui::size(px(900.0), px(700.0)));
        cx.run_until_parked();
        let screens = cx.update(|_, cx| studio.read(cx).screens.clone());
        for (screen, id, _) in SCREEN_TABS {
            cx.update(|_, cx| screens.update(cx, |tabs, cx| tabs.set_active(id, cx)));
            cx.run_until_parked();
            cx.update(|_, cx| {
                let studio = studio.read(cx);
                assert_eq!(studio.active_tab(cx), screen);
                assert_eq!(studio.page_scroll.offset(), gpui::point(px(0.0), px(0.0)));
                if screen == RadixStudioTab::StyleGuide {
                    assert!(studio.style_guide.is_some());
                }
                if screen == RadixStudioTab::Developer {
                    assert!(studio.developer.is_some());
                }
            });
        }
    }
}
