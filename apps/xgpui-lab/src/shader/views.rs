use super::*;
use xgpui_shader::{painted, Paint, WindowPaintExt, WindowShaderExt};
use crate::screen::{ComparePane, Hero, TextRole, text};

impl Render for ShaderPage {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_choices(_cx);
        if self.active_tab == "Shadertoy: Seascape" {
            self.sea_pane.update(_cx, |pane, _| pane.activate());
            return div().size_full().child(self.sea_pane.clone()).into_any_element();
        }
        if self.active_tab == "Shader Depot: Grid" {
            self.grid_pane.update(_cx, |pane, _| pane.activate());
            return div().size_full().child(self.grid_pane.clone()).into_any_element();
        }
        if self.requires_sdr() && window.color_status().effective() != gpui::WindowColorMode::Default {
            return div().p_4().child("Preparing SDR presentation for captured effects…").into_any_element();
        }
        if self.active_tab == "Brushes" {
            return div()
                .id("brush-gallery-scroll")
                .size_full()
                .overflow_y_scroll()
                .p_4()
                .child(gallery::gallery(self, _cx))
                .into_any_element();
        }
        if self.active_tab == "Layer Effects" {
            return div()
                .id("layer-gallery-scroll")
                .size_full()
                .overflow_y_scroll()
                .p_4()
                .child(gallery::layers(self, _cx))
                .into_any_element();
        }
        if self.active_tab == "Mesh Editor" {
            return div().size_full().child(self.mesh_pane.clone()).into_any_element();
        }
        if matches!(self.active_tab.as_ref(), "Shadows" | "Borders") {
            self.shadow_pane.update(_cx, |page, _| page.borders = self.active_tab == "Borders");
            return div().size_full().child(self.shadow_pane.clone()).into_any_element();
        }
        if self.active_tab == "Typography" {
            return div().size_full().child(self.typography.clone()).into_any_element();
        }
        let now = Instant::now();
        if self.animated && _cx.reduce_motion() {
            self.freeze();
            self.sync_sliders(_cx);
            self.sync_fields_except(None, _cx);
        }
        if self.animated {
            self.time += now.duration_since(self.last_frame).as_secs_f32().min(0.1);
            window.request_animation_frame();
        }
        self.last_frame = now;
        self.render_core(_cx).into_any_element()
    }
}

impl ShaderPage {
    pub(super) fn parameter_visible(&self, parameter: Parameter) -> bool {
        let effect = Self::EFFECTS[self.effect_index];
        match parameter {
            Parameter::Angle => effect != Effect::Radial,
            Parameter::CenterX | Parameter::CenterY => effect != Effect::Linear,
            Parameter::RadiusX | Parameter::RadiusY => matches!(effect, Effect::Radial | Effect::Diamond),
            _ => true,
        }
    }

    pub(super) fn gradients(&self) -> anyhow::Result<AnyElement> {
        let paint = if self.native {
            Paint::Native(white().into())
        } else {
            self.producer()?.into()
        };
        Ok(div()
            .id("analytic-preview")
            .w_full()
            .h(px(280.))
            .overflow_hidden()
            .opacity(if self.half_opacity { 0.5 } else { 1. })
            .child(
                painted()
                    .w_full()
                    .h(px(280.))
                    .rounded_2xl()
                    .bg_paint(paint)
                    .when(self.clipped, |view| view.ml(px(-120.))),
            )
            .into_any_element())
    }

    pub(super) fn coverage(&self, brushes: bool) -> anyhow::Result<AnyElement> {
        let producer = self.producer()?;
        let paint = if self.native && brushes {
            Paint::Native(white().into())
        } else {
            producer.into()
        };
        let mut cards = div().flex().flex_wrap().gap_4();
        for (index, label) in ["Rounded background", "Dashed border", "Curved path fill"].into_iter().enumerate() {
            let mut tile = painted().w(px(280.)).h(px(210.));
            match index {
                0 => {
                    tile =
                        tile.rounded_3xl().bg_paint(paint.clone()).flex().items_center().justify_center().child(text(
                            &self.look,
                            TextRole::Detail,
                            "Normal GPUI children",
                        ))
                }
                1 => tile = tile.rounded_3xl().border_8().border_dashed().border_paint(paint.clone()),
                _ => {
                    let paint = paint.clone();
                    tile = tile.child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                if let Ok(path) = shape(bounds) {
                                    window.paint_path_with_paint(path, paint.clone());
                                }
                            },
                        )
                        .size_full(),
                    );
                }
            }
            cards = cards.child(
                div().flex().flex_col().gap_2().child(text(&self.look, TextRole::Name, label)).child(
                    div()
                        .id(format!("shader-coverage-{index}"))
                        .w(px(if self.clipped { 200. } else { 280. }))
                        .h(px(210.))
                        .overflow_hidden()
                        .opacity(if self.half_opacity { 0.5 } else { 1. })
                        .child(tile),
                ),
            );
        }
        let view = div().flex().flex_col().gap_4().child(cards);
        Ok(view.into_any_element())
    }

    pub(super) fn wgsl(&self) -> anyhow::Result<Hero> {
        let programs = self.programs.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let wash = programs.wash.paint(
            xgpui_shader::shader_paint::ShaderParameters::new()
                .float("progress", self.progress + if self.animated { 0.25 * self.time.sin() } else { 0. })
                .vec3("tint", super::palette::WASH_TINT)
                .float("alpha", self.shader_alpha),
        )?;
        let program = programs.minimal.clone();
        let phase = self.time * 2.;
        let alpha = self.shader_alpha;
        let viewport =
            || div().w_full().h(px(240.)).overflow_hidden().opacity(if self.half_opacity { 0.5 } else { 1. });
        let wash = viewport().child(
            painted()
                .w_full()
                .h(px(240.))
                .rounded_2xl()
                .bg_paint(wash)
                .when(self.clipped, |view| view.ml(px(-120.))),
        );
        let typed = viewport().child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let [r, g, b] = super::palette::MINIMAL_TINT;
                    if let Err(error) = window.paint_custom_shader(
                        bounds,
                        &program,
                        EffectInstance { color: [r, g, b, alpha], effect: [12., phase, 1., 0.] },
                    ) {
                        eprintln!("Minimal shader paint failed: {error:#}");
                    }
                },
            )
            .w_full()
            .h(px(240.))
            .when(self.clipped, |view| view.ml(px(-120.))),
        );
        Ok(Hero::compare(
            ComparePane::new("Portable wash", wash).subtitle("Reflected WGSL inputs"),
            ComparePane::new("Typed fragment", typed).subtitle("Minimal shader"),
        ))
    }

    pub(super) fn instances(&self) -> anyhow::Result<AnyElement> {
        let programs = self.programs.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let program = match self.mode {
            0 => programs.glow.clone(),
            1 => programs.particles.clone(),
            _ => programs.rounded.clone(),
        };
        let mode = self.mode;
        let count = self.instance_count();
        let time = self.time;
        let clipped = self.clipped;
        Ok(div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .w_full()
                    .h(px(380.))
                    .overflow_hidden()
                    .bg(self.look.chrome().panel_background)
                    .opacity(if self.half_opacity { 0.5 } else { 1. })
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                let result = if mode == 2 {
                                    paint_composition(bounds, &program, clipped, window)
                                } else {
                                    paint_instances(bounds, &program, mode, count, time, window)
                                };
                                if let Err(error) = result {
                                    eprintln!("Instanced shader paint failed: {error:#}");
                                }
                            },
                        )
                        .size_full(),
                    ),
            )
            .into_any_element())
    }
}

fn random(seed: u32) -> f32 {
    let mut value = seed.wrapping_add(0x9e3779b9);
    value = (value ^ (value >> 16)).wrapping_mul(0x85ebca6b);
    value = (value ^ (value >> 13)).wrapping_mul(0xc2b2ae35);
    ((value ^ (value >> 16)) >> 8) as f32 / 16777216.
}

fn paint_instances(
    bounds: Bounds<Pixels>,
    program: &ShaderProgram<EffectInstance>,
    mode: usize,
    count: usize,
    t: f32,
    window: &mut Window,
) -> anyhow::Result<()> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    for i in 0..count {
        let fi = i as f32;
        let (x, y, w, h, tint) = if mode == 0 {
            let w = width / 10.;
            let h = height / 10.;
            ((i % 10) as f32 * w, (i / 10) as f32 * h, w, h, super::palette::instance_tint(fi, false))
        } else {
            let x = ((random(i as u32) + t * 0.025) % 1.) * width;
            let y = ((random((i as u32).wrapping_add(123456)) - t * 0.06).rem_euclid(1.)) * height;
            (x - 6., y - 6., 12., 12., super::palette::instance_tint(fi, true))
        };
        window.paint_custom_shader(
            Bounds::new(bounds.origin + point(px(x), px(y)), size(px(w), px(h))),
            program,
            EffectInstance { color: tint, effect: [12., t * 2. + fi * 0.1, 1., 0.] },
        )?;
    }
    Ok(())
}

fn shape(bounds: Bounds<Pixels>) -> anyhow::Result<Path<Pixels>> {
    let mut p = PathBuilder::fill();
    let w = bounds.size.width;
    let h = bounds.size.height;
    let o = bounds.origin;
    p.move_to(o + point(w * 0.5, h * 0.0));
    p.line_to(o + point(w * 0.67, h * 0.33));
    p.curve_to(o + point(w * 1.0, h * 0.5), o + point(w * 1.0, h * 0.1));
    p.line_to(o + point(w * 0.72, h * 0.68));
    p.line_to(o + point(w * 0.8, h * 1.0));
    p.line_to(o + point(w * 0.5, h * 0.8));
    p.line_to(o + point(w * 0.2, h * 1.0));
    p.line_to(o + point(w * 0.28, h * 0.68));
    p.line_to(o + point(w * 0.0, h * 0.5));
    p.line_to(o + point(w * 0.33, h * 0.33));
    p.close();
    p.build()
}

fn paint_composition(
    bounds: Bounds<Pixels>,
    program: &ShaderProgram<EffectInstance>,
    clipped: bool,
    window: &mut Window,
) -> anyhow::Result<()> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let clip = Bounds::new(
        bounds.origin + point(px(width * 0.12), px(height * 0.16)),
        size(px(width * 0.72), px(height * 0.60)),
    );
    let geometry = |x, y, w, h| {
        Bounds::new(
            clip.origin + point(clip.size.width * x, clip.size.height * y),
            size(clip.size.width * w, clip.size.height * h),
        )
    };
    window.paint_quad(fill(clip, rgb(super::palette::COMPOSITION_BACKGROUND)));
    window.with_content_mask(clipped.then_some(ContentMask { bounds: clip }), |window| -> anyhow::Result<()> {
        let effect = |color| EffectInstance { color, effect: [12., 0., 1., 0.] };
        window.paint_custom_shader(
            geometry(0.04, 0.12, 0.56, 0.76),
            program,
            effect(super::palette::COMPOSITION_SHAPES[0]),
        )?;
        window.paint_quad(fill(geometry(0.36, -0.12, 0.025, 1.40), rgb(super::palette::COMPOSITION_DIVIDER)));
        window.paint_custom_shader(
            geometry(0.30, 0.25, 0.56, 0.96),
            program,
            effect(super::palette::COMPOSITION_SHAPES[1]),
        )?;
        window.paint_custom_shader(
            geometry(0.56, -0.10, 0.64, 0.90),
            program,
            effect(super::palette::COMPOSITION_SHAPES[2]),
        )?;
        Ok(())
    })?;
    window.paint_quad(
        fill(clip, transparent_black())
            .border_widths(px(2.))
            .border_color(rgb(super::palette::COMPOSITION_CLIP)),
    );
    Ok(())
}
