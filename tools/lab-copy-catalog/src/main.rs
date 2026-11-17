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
            || ["none", "on", "off", "enabled", "disabled", "ready", "paused", "idle"].contains(&s))
}
impl<'a> Visit<'a> for Scan {
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
        if natural(&s) {
            self.strings.insert(s);
        }
    }
    fn visit_expr_macro(&mut self, n: &'a syn::ExprMacro) {
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
fn walk(p: &Path, files: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(p).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(&p, files);
        } else if p.extension().is_some_and(|s| s == "rs")
            && !p.to_string_lossy().contains("tests")
            && !p.to_string_lossy().contains("/l10n/")
        {
            files.push(p);
        }
    }
}
fn main() {
    let mut files = Vec::new();
    walk(Path::new("apps/xgpui-lab/src"), &mut files);
    let mut data: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for p in files {
        let text = fs::read_to_string(&p).unwrap();
        let path = p.to_str().unwrap().to_owned();
        let file = syn::parse_file(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut scan = Scan { strings: Default::default(), formats: Default::default() };
        scan.visit_file(&file);
        for s in &scan.strings {
            data.entry(s.clone()).or_insert_with(|| serde_json::json!({"kind":"text","files":[]}))["files"]
                .as_array_mut()
                .unwrap()
                .push(path.clone().into());
        }
        for s in &scan.formats {
            data.entry(s.clone()).or_insert_with(|| serde_json::json!({"kind":"format","files":[]}))["files"]
                .as_array_mut()
                .unwrap()
                .push(path.clone().into());
        }
    }

    fs::write(
        std::env::args().nth(1).expect("usage: cargo run -p lab-copy-catalog -- <inventory.json>"),
        serde_json::to_string_pretty(&data).unwrap(),
    )
    .unwrap();
    println!("{} catalog candidates", data.len());
}
