use anyhow::Result;
#[cfg(target_os = "macos")]
struct Options {
    control: bool,
    producers: u32,
    frames: u32,
    cycles: u32,
}
#[cfg(target_os = "macos")]
mod native {
    use super::*;
    use gpui::*;
    use anyhow::ensure;
    use gpui_apple::metal_renderer::MetalHeadlessRenderer;
    use std::{any::Any, sync::Arc};
    use xgpui_gamut::WideColor;

    #[derive(Debug)]
    struct Producer {
        index: u32,
        color: WideColor,
    }
    impl CustomContent for Producer {
        fn prepare(&self, backend: &dyn Any, extent: Size<DevicePixels>) -> Result<Box<dyn Any>> {
            eprintln!("prepare {} begin", self.index);
            let result = self.color.prepare(backend, extent);
            eprintln!("prepare {} end: {}", self.index, if result.is_ok() { "ok" } else { "error" });
            result
        }
    }

    fn scene(count: u32, green: bool) -> Result<Scene> {
        let mut scene = Scene::default();
        for index in 0..count {
            let bounds = Bounds::new(
                point(ScaledPixels(index as f32), ScaledPixels(0.)),
                size(ScaledPixels(1.), ScaledPixels(8.)),
            );
            scene.insert_primitive(CustomDraw {
                order: 0,
                bounds,
                content_mask: ContentMask { bounds },
                corner_radii: Default::default(),
                opacity: 1.,
                coverage: None,
                content: Arc::new(Producer {
                    index,
                    color: WideColor::linear_p3(if green { [0., 1., 0., 1.] } else { [1., 0., 0., 1.] })?,
                }),
            });
        }
        scene.finish();
        Ok(scene)
    }

    pub fn run(options: &Options) -> Result<()> {
        let mut renderer = MetalHeadlessRenderer::new()?;
        renderer.set_color_mode(WindowColorMode::WideGamutSdr)?;
        let count = if options.control { 1 } else { options.producers };
        for cycle in 0..options.cycles {
            // Replace dense -> sparse -> dense, like leaving and returning to a page.
            for (stage, count) in [("dense", count), ("sparse", 1), ("return", count)] {
                let green = stage == "sparse";
                let scene = scene(count, green)?;
                let extent = size(DevicePixels(count as i32), DevicePixels(8));
                for frame in 0..options.frames {
                    eprintln!("cycle={cycle} stage={stage} frame={frame} producers={count} begin");
                    // Reuse the same scene across frames; each producer still records fresh work.
                    let image = renderer.render_scene_to_linear_p3(&scene, extent)?;
                    let expected = if green {
                        [0, 0x3c00, 0, 0x3c00]
                    } else {
                        [0x3c00, 0, 0, 0x3c00]
                    };
                    ensure!(image.size == extent && image.pixels.len() == count as usize * 8, "incomplete readback");
                    ensure!(image.pixels.iter().all(|pixel| *pixel == expected), "unexpected linear-P3 pixels");
                    eprintln!("cycle={cycle} stage={stage} frame={frame} readback verified");
                }
            }
        }
        Ok(())
    }
}

pub(super) fn run(control: bool, producers: u32) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        native::run(&Options { control, producers, frames: 3, cycles: 3 })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (control, producers);
        anyhow::bail!("Dense producer regression requires macOS Metal")
    }
}
