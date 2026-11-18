use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use syn::{
    visit::{self, Visit},
};
struct Scan {
    strings: BTreeSet<String>,
    formats: BTreeSet<String>,
    ids: BTreeSet<String>,
    ui: bool,
    /// UI context inherited from an enclosing boundary through nested calls
    /// (`.title(Some("..."))`) rather than set by the innermost call itself.
    nested: bool,
    ui_strings: BTreeSet<String>,
}
fn natural(s: &str) -> bool {
    !s.is_empty()
        && s.len() < 1400
        && s.chars().any(|c| c.is_ascii_alphabetic())
        && !(s.contains(" = ") && s.contains(';'))
        && !s.contains(['\n', '{', '}'])
        && !["::", "://", "assets/", "/Users/", "#!", "fn "].iter().any(|x| s.contains(x))
        && (s.trim().contains(' ')
            || s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && s.chars().nth(1).is_some_and(|c| c.is_ascii_lowercase())
            || [
                "none",
                "on",
                "off",
                "enabled",
                "disabled",
                "ready",
                "paused",
                "idle",
                "interactive",
                "refined",
                "capped",
                "clip",
                "paths",
                "scramble",
            ]
            .contains(&s))
}
impl<'a> Visit<'a> for Scan {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let (old, old_nested) = (self.ui, self.nested);
        self.ui = false;
        self.visit_expr(&node.receiver);
        let boundary = [
            "title",
            "lede",
            "caption",
            "subtitle",
            "note",
            "credits",
            "footer",
            "label",
            "placeholder",
            "help",
            "aria_label",
            "aria_description",
            "values",
        ]
        .contains(&node.method.to_string().as_str());
        // Nested calls inside a UI boundary stay UI: `.title(Some("..."))`.
        // Lookup keys (`.get("key")`) never are.
        self.ui = boundary || old && node.method != "get";
        self.nested = !boundary && self.ui;
        for arg in &node.args {
            self.visit_expr(arg);
        }
        (self.ui, self.nested) = (old, old_nested);
    }
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        self.visit_expr(&node.func);
        let name = if let syn::Expr::Path(path) = node.func.as_ref() {
            path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        } else {
            String::new()
        };
        let (old, old_nested) = (self.ui, self.nested);
        let boundary = ["text", "readout", "properties"].contains(&name.as_str())
            || [
                "l10n::ui",
                "l10n::label",
                "l10n::tooltip",
                "LabCard::new",
                "PanelShell::new",
                "ParametersPanel::new",
                "PreviewPanel::new",
                "ChainPanel::new",
                "ComparePane::new",
            ]
            .iter()
            .any(|suffix| name.ends_with(suffix));
        // `raw` text is never translated; the first argument of these is a
        // Fluent select key or a separator, not copy.
        let raw = name.ends_with("UiCopy::raw");
        let keyed = ["UiCopy::counted", "UiCopy::state", "UiCopy::Separated"].iter().any(|s| name.ends_with(s));
        for (index, arg) in node.args.iter().enumerate() {
            self.ui = boundary || old && !raw && !(keyed && index == 0);
            self.nested = !boundary && self.ui;
            self.visit_expr(arg);
        }
        (self.ui, self.nested) = (old, old_nested);
    }
    fn visit_expr_index(&mut self, node: &'a syn::ExprIndex) {
        self.visit_expr(&node.expr);
        // `result["label"]`: JSON and map keys are data, not copy.
        let (old, old_nested) = (self.ui, self.nested);
        (self.ui, self.nested) = (false, false);
        self.visit_expr(&node.index);
        (self.ui, self.nested) = (old, old_nested);
    }
    fn visit_path(&mut self, path: &'a syn::Path) {
        if let Some(segment) = path.segments.last() {
            let name = segment.ident.to_string();
            if let Some(hash) = name.strip_prefix("msg_ui_") {
                self.ids.insert(format!("ui-{hash}"));
            }
        }
        visit::visit_path(self, path);
    }
    fn visit_attribute(&mut self, _: &'a syn::Attribute) {}
    fn visit_item_mod(&mut self, n: &'a syn::ItemMod) {
        if n.attrs
            .iter()
            .any(|a| a.path().is_ident("cfg") && a.meta.to_token_stream().to_string().contains("test"))
        {
            return;
        }
        visit::visit_item_mod(self, n);
    }
    fn visit_item_fn(&mut self, n: &'a syn::ItemFn) {
        if n.attrs.iter().any(|a| a.path().is_ident("test")) {
            return;
        }
        visit::visit_item_fn(self, n);
    }
    fn visit_lit_str(&mut self, n: &'a syn::LitStr) {
        let s = n.value();
        // Inherited context needs a letter: separators and placeholders such as
        // " · " or "—" are not copy.
        let wording = !self.nested || s.chars().any(char::is_alphabetic);
        if self.ui && wording && !s.is_empty() && s.len() < 1400 && !s.contains(['\n', '{', '}']) {
            self.ui_strings.insert(s.clone());
            self.strings.insert(s);
        } else if natural(&s) {
            self.strings.insert(s);
        }
    }
    fn visit_expr_macro(&mut self, n: &'a syn::ExprMacro) {
        if ["vec", "hstack", "vstack"].iter().any(|name| n.mac.path.is_ident(name)) {
            use syn::parse::Parser;
            let mut tokens = n.mac.tokens.clone();
            if !n.mac.path.is_ident("vec") {
                let trees: Vec<_> = tokens.clone().into_iter().collect();
                if let Some(index) =
                    trees.iter().position(|tree| matches!(tree, proc_macro2::TokenTree::Punct(p) if p.as_char()==';'))
                {
                    tokens = trees.into_iter().skip(index + 1).collect();
                }
            }
            if let Ok(expressions) =
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated.parse2(tokens)
            {
                for expression in &expressions {
                    self.visit_expr(expression);
                }
            }
        }
        if n.mac.path.is_ident("format") {
            if let Some(proc_macro2::TokenTree::Literal(lit)) = n.mac.tokens.clone().into_iter().next() {
                if let Ok(lit) = syn::parse_str::<syn::LitStr>(&lit.to_string()) {
                    let s = lit.value();
                    let plain = strip_fields(&s);
                    if natural(&plain) && s.contains('{') {
                        self.formats.insert(s);
                    }
                }
            }
        }
        visit::visit_expr_macro(self, n);
    }
}
// Avoid another dependency just for cfg attribute printing.
trait Tokens {
    fn to_token_stream(&self) -> proc_macro2::TokenStream;
}
impl Tokens for syn::Meta {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            syn::Meta::List(l) => l.tokens.clone(),
            _ => Default::default(),
        }
    }
}
fn strip_fields(s: &str) -> String {
    let mut out = String::new();
    let mut field = false;
    for c in s.chars() {
        if c == '{' {
            field = true;
        } else if c == '}' {
            field = false;
        } else if !field {
            out.push(c);
        }
    }
    out
}
/// Visit in sorted order so the inventory does not depend on the filesystem.
fn walk(p: &Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(p).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, files);
        } else if p.extension().is_some_and(|s| s == "rs")
            && !p.to_string_lossy().contains("tests")
            && !p.to_string_lossy().contains("/l10n/")
            && !p.to_string_lossy().contains("/bin/")
            && p != Path::new("apps/xgpui-lab/src/chromaticity.rs")
        {
            files.push(p);
        }
    }
}
fn main() {
    let mut files = Vec::new();
    walk(Path::new("apps/xgpui-lab/src"), &mut files);
    let mut data: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    // Typed call sites retain their source/translation records after the old
    // English format literal has disappeared from Rust.
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string("apps/xgpui-lab/locales/copy.json").unwrap()).unwrap();
    for p in files {
        let text = fs::read_to_string(&p).unwrap();
        let path = p.to_str().unwrap().to_owned();
        let file = syn::parse_file(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut scan = Scan {
            strings: Default::default(),
            formats: Default::default(),
            ids: Default::default(),
            ui: false,
            nested: false,
            ui_strings: Default::default(),
        };
        scan.visit_file(&file);
        for s in &scan.strings {
            data.entry(s.clone()).or_insert_with(|| serde_json::json!({"kind":"text","files":[]}))["files"]
                .as_array_mut()
                .unwrap()
                .push(path.clone().into());
        }
        for source in &scan.ui_strings {
            let meta = data.get_mut(source).unwrap();
            let locations = meta.as_object_mut().unwrap().entry("boundaries").or_insert_with(|| serde_json::json!([]));
            locations.as_array_mut().unwrap().push(path.clone().into());
        }
        for s in &scan.formats {
            data.entry(s.clone()).or_insert_with(|| serde_json::json!({"kind":"format","files":[]}))["files"]
                .as_array_mut()
                .unwrap()
                .push(path.clone().into());
        }
        for id in &scan.ids {
            let entry = records
                .iter()
                .find(|entry| entry["id"].as_str() == Some(id.as_str()))
                .unwrap_or_else(|| panic!("{path}: unknown typed ID {id}"));
            let source = entry["source"].as_str().unwrap().to_owned();
            let kind = if entry["parts"].is_array() { "format" } else { "text" };
            let meta = data.entry(source).or_insert_with(|| serde_json::json!({"kind":kind,"files":[]}));
            meta["explicit"] = true.into();
            meta["files"].as_array_mut().unwrap().push(path.clone().into());
        }
    }

    fs::write(
        std::env::args().nth(1).expect("usage: cargo run -p lab-copy-catalog -- <inventory.json>"),
        serde_json::to_string_pretty(&data).unwrap(),
    )
    .unwrap();
    println!("{} catalog candidates", data.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_macros_keep_lowercase_ui_boundaries_and_explicit_ids() {
        let file = syn::parse_file(
            r#"
            fn screen() {
                let _ = view.caption(vec!["tiny"]);
                let _ = hstack! { gap=2; text(&look, TextRole::Caption, "waiting") };
                let _ = messages::msg_ui_012345abcdef("data");
            }
        "#,
        )
        .unwrap();
        let mut scan = Scan {
            strings: Default::default(),
            formats: Default::default(),
            ids: Default::default(),
            ui: false,
            nested: false,
            ui_strings: Default::default(),
        };
        scan.visit_file(&file);
        assert!(scan.ui_strings.contains("tiny"));
        assert!(scan.ui_strings.contains("waiting"));
        assert!(scan.ids.contains("ui-012345abcdef"));
        assert!(!scan.ui_strings.contains("data"));
    }

    fn scan(source: &str) -> Scan {
        let file = syn::parse_file(source).unwrap();
        let mut scan = Scan {
            strings: Default::default(),
            formats: Default::default(),
            ids: Default::default(),
            ui: false,
            nested: false,
            ui_strings: Default::default(),
        };
        scan.visit_file(&file);
        scan
    }

    #[test]
    fn nested_calls_inside_a_ui_boundary_keep_it() {
        let scan = scan(
            r#"
            fn screen() {
                let _ = card.title(Some("tiny title"));
                let _ = card.label(Some(String::from("wrapped")));
                let _ = crate::l10n::ui(Some("ui wrapped"));
                let _ = text(&look, TextRole::Caption, Some("caption wrapped"));
                let _ = Button::new("button-id").label(Some("button label"));
                let _ = view.caption(UiCopy::compose(f, vec![UiCopy::counted("cells", n), UiCopy::raw(Some("raw data"))]));
                let _ = view.caption(UiCopy::Separated(" · ", vec![UiCopy::from(Some("first part")), UiCopy::from("—")]));
                let _ = crate::l10n::ui(result["json-key"].as_str().unwrap_or("unknown outcome"));
                let _ = crate::l10n::ui(Some(map.get("map-key").unwrap_or("fallback text")));
            }
        "#,
        );
        for text in [
            "tiny title",
            "wrapped",
            "ui wrapped",
            "caption wrapped",
            "button label",
            "first part",
            "unknown outcome",
            "fallback text",
        ] {
            assert!(scan.ui_strings.contains(text), "{text}");
        }
        for key in ["cells", "raw data", " · ", "—", "json-key", "map-key"] {
            assert!(!scan.ui_strings.contains(key), "{key}");
        }
        // Outside a boundary, call arguments and receivers stay non-UI.
        assert!(!scan.ui_strings.contains("button-id"));
    }

    #[test]
    fn walk_order_is_sorted() {
        let directory = std::env::temp_dir().join(format!("lab-copy-walk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        for name in ["b.rs", "a/z.rs", "a/c.rs", "C.rs"] {
            let path = directory.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        let mut files = Vec::new();
        walk(&directory, &mut files);
        let names: Vec<_> =
            files.iter().map(|p| p.strip_prefix(&directory).unwrap().to_string_lossy().into_owned()).collect();
        fs::remove_dir_all(&directory).unwrap();
        assert_eq!(names, ["C.rs", "a/c.rs", "a/z.rs", "b.rs"]);
    }
}
