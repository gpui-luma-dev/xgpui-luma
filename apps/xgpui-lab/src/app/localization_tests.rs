//! Every registered Lab destination is exercised with shipped pseudo resources.
use super::*;
use gpui::{TestAppContext, TestDispatcher, size};

#[test]
fn every_destination_survives_expansion_rtl_theme_and_viewport_changes() {
    let usage = crate::l10n::usage::CopyUsage::default();
    for parley in [false, true] {
        let mut app = if parley {
            let backend = crate::parley_lab::application_text_system().unwrap();
            let app = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend.clone());
            app.update(|cx| crate::parley_lab::paragraph_api::install_typeset(backend, cx));
            app
        } else {
            TestAppContext::single()
        };
        app.update(|cx| {
            cx.set_global(usage.clone());
            xgpui_luma::init(cx).unwrap();
            xgpui_luma_l10n::install(crate::l10n::localizer("en-XA".parse().unwrap()).unwrap(), cx).unwrap();
        });
        let (shell, cx) = app.add_window_view(XgpuiLabApp::new);
        for (width, mode, tag) in [(720., ThemeMode::Light, "en-XA"), (1280., ThemeMode::Dark, "ar-XB")] {
            cx.simulate_resize(size(px(width), px(1800.)));
            cx.update(|_, cx| {
                shell.read(cx).look.set_mode(mode);
                assert!(xgpui_luma_l10n::set_locale(tag.parse().unwrap(), cx));
            });
            for &(workspace, _, pages) in SECTIONS {
                for &(page, _) in pages {
                    shell.update(cx, |shell, cx| shell.activate(workspace, page, cx));
                    cx.run_until_parked();
                    cx.update(|_, cx| {
                        let shell = shell.read(cx);
                        assert_eq!((shell.active, shell.active_page), (workspace, page));
                    });
                    // Data/visualizations may pan or scroll. The enclosing workspace
                    // must remain constrained to the actual window at both sizes.
                    let bounds = cx.debug_bounds("shell-2026-workspace").unwrap();
                    assert!(bounds.left() >= px(0.) && bounds.right() <= px(width), "{tag} {page}: {bounds:?}");
                }
            }
        }
    }
    if let Some(path) = std::env::var_os("LAB_COPY_USAGE") {
        std::fs::write(path, serde_json::to_string_pretty(&*usage.0.lock().unwrap()).unwrap()).unwrap();
    }
}
