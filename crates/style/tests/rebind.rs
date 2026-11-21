//! Rebind, refresh and swap: a theme never changes; every change returns a new one (API topic 4).

mod common;

use std::sync::Arc;

use common::*;
use xgpui_luma_style::{Code, Color, NoNativeTokens, PartId, Source, Stylesheet, Theme, Value};

const INDICATOR: PartId = PartId::new("indicator");

#[test]
fn mode_switch_shares_the_stylesheet() {
    let light = shadcn_theme("light");
    let dark = light.with_mode("dark").expect("dark");
    assert_eq!(dark.mode(), "dark");
    assert_ne!(light.generation(), dark.generation());
    let k = key(&light, "checkbox", &[], &["checked"]);
    assert_ne!(light.resolve(&k).part(INDICATOR).background(), dark.resolve(&k).part(INDICATOR).background());
    // The light theme is unchanged.
    assert_eq!(light.mode(), "light");
}

#[test]
fn live_token_edits_need_no_recompile() {
    let theme = sdk_theme();
    let red = Color::srgb(1.0, 0.0, 0.0, 1.0);
    let edited = theme
        .with_token_values(&[("palette.form.input.background", Value::Color(red))])
        .expect("known token");
    let k = key(&theme, "checkbox", &[], &[]);
    assert_color(edited.resolve(&k).part(INDICATOR).background(), red, "edited token");
    assert_color(theme.resolve(&k).part(INDICATOR).background(), sdk_gray(100.0), "original unchanged");
    assert_eq!(edited.token_values().len(), 1);

    let err = theme
        .with_token_values(&[("palette.form.input.backgroud", Value::Color(red))])
        .err()
        .expect("unknown");
    assert!(err.has_code(Code::UnknownName));
    assert!(err.mentions("did you mean \"palette.form.input.background\"?"), "{err}");
}

#[test]
fn a_new_native_token_source_rebinds() {
    let theme = shadcn_theme("light");
    struct Pink(ShadcnCatalog);
    impl xgpui_luma_style::TokenSource for Pink {
        fn roots(&self) -> &[&str] {
            self.0.roots()
        }
        fn token(&self, path: &str, cx: &xgpui_luma_style::TokenCx<'_>) -> Option<Value> {
            if path == "css.primary" {
                return Some(Value::Color(Color::oklch(0.7, 0.2, 350.0, 1.0)));
            }
            self.0.token(path, cx)
        }
        fn function(&self, name: &str) -> Option<xgpui_luma_style::FunctionSig> {
            self.0.function(name)
        }
        fn call(&self, name: &str, args: &[Value], cx: &xgpui_luma_style::TokenCx<'_>) -> Result<Value, String> {
            self.0.call(name, args, cx)
        }
    }
    let pink = theme.with_tokens(Arc::new(Pink(ShadcnCatalog::new()))).expect("rebinds");
    let k = key(&pink, "checkbox", &[], &["checked"]);
    assert_color(pink.resolve(&k).part(INDICATOR).background(), Color::oklch(0.7, 0.2, 350.0, 1.0), "new catalog");
}

#[test]
fn editing_a_file_recompiles_and_failures_keep_the_old_stylesheet() {
    let theme = sdk_theme();
    let sheet = theme.stylesheet().clone();
    let tweak = Source::new("app/tweak.lss", "checkbox { part indicator { background = black } }");
    let sheet2 = sheet.with_source("app", tweak).expect("compiles");
    let theme2 = theme.with_stylesheet(&sheet2).expect("binds");
    let k = key(&theme2, "checkbox", &[], &[]);
    assert_color(theme2.resolve(&k).part(INDICATOR).background(), Color::BLACK, "app layer wins");

    let broken = Source::new("app/tweak.lss", "checkbox { part indicator { background = } }");
    let err = sheet2.with_source("app", broken).err().expect("syntax error");
    assert!(err.has_code(Code::Syntax));
    // sheet2 and theme2 are untouched; the app keeps rendering with them.
    assert_color(theme2.resolve(&k).part(INDICATOR).background(), Color::BLACK, "last good theme");
}

#[test]
fn swapping_themes_is_compile_and_bind() {
    // Swapping token files (a different theme, not a mode) is compile and bind.
    let dark_sdk_tokens =
        Source::new("sdk/dark-tokens.lss", "tokens { palette { form { input { background = gray(10, 1) } } } }");
    let sheet = Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source("app", dark_sdk_tokens)
        .compile()
        .expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    let k = key(&theme, "checkbox", &[], &[]);
    assert_color(theme.resolve(&k).part(INDICATOR).background(), sdk_gray(10.0), "swapped theme");
}

#[test]
fn sources_can_be_read_from_disk() {
    let dir = std::env::temp_dir().join(format!("lss-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("tweak.lss");
    std::fs::write(&path, "checkbox { gap = 12 }").expect("write");
    let source = Source::from_path(&path).expect("reads");
    let sheet = Stylesheet::builder(luma_config())
        .layer("sdk", sdk_sources())
        .source("app", source)
        .compile()
        .expect("compiles");
    let theme = Theme::bind(&sheet, Arc::new(NoNativeTokens), "light").expect("binds");
    assert_eq!(theme.resolve(&key(&theme, "checkbox", &[], &[])).root().gap(), Some(12.0));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(Source::from_path(dir.join("missing.lss")).is_err());
}
