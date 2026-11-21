use std::collections::BTreeMap;

use anyhow::{Context as _, Result, anyhow, bail};
use xgpui_luma::theme::ThemeMode;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CssTokenCatalog {
    pub light: BTreeMap<String, String>,
    pub dark: BTreeMap<String, String>,
    /// Further modes from class blocks (`.hi-contrast`, `.dark.hi-contrast`), in file order.
    pub modes: Vec<CssMode>,
}

/// A mode a theme's CSS adds with a class block: `.hi-contrast { … }` is `hi_contrast` based on light
/// (`:root`), `.dark.hi-contrast { … }` is `hi_contrast_dark` based on dark (`.dark`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssMode {
    /// The style mode name: the class with `-` as `_` (and `_dark` for a `.dark` compound).
    pub name: String,
    pub base: ThemeMode,
    /// The block's own custom properties; [`CssTokenCatalog::mode_map`] layers them over the base.
    pub tokens: BTreeMap<String, String>,
}

pub fn parse_css_catalog(source: &str) -> Result<CssTokenCatalog> {
    let stripped = strip_css_comments(source);
    let mut rules = Vec::new();
    scan_rules(&stripped, &mut rules)?;

    let mut catalog = CssTokenCatalog::default();
    for (selector_list, body) in rules {
        let mut declarations = BTreeMap::new();
        parse_declarations(body, &mut declarations)?;
        if declarations.is_empty() {
            continue;
        }
        for selector in selector_list.split(',').map(str::trim) {
            match target(selector)? {
                Some(Target::Light) => catalog.light.extend(declarations.clone()),
                Some(Target::Dark) => catalog.dark.extend(declarations.clone()),
                Some(Target::Mode(name, base)) => match catalog.modes.iter_mut().find(|m| m.name == name) {
                    Some(mode) => mode.tokens.extend(declarations.clone()),
                    None => catalog.modes.push(CssMode { name, base, tokens: declarations.clone() }),
                },
                None => {}
            }
        }
    }

    if catalog.light.is_empty() {
        bail!("CSS catalog is missing `:root` custom properties");
    }
    if catalog.dark.is_empty() {
        bail!("CSS catalog is missing `.dark` custom properties");
    }
    Ok(catalog)
}

enum Target {
    Light,
    Dark,
    Mode(String, ThemeMode),
}

/// What a selector's custom properties set: `:root` (or `.light`) light, `.dark` dark, a single class a
/// mode based on light, and a class compounded with `.dark` a mode based on dark. Anything else (element
/// selectors, descendants, attributes) sets nothing.
fn target(selector: &str) -> Result<Option<Target>> {
    match selector {
        ":root" | ".light" => return Ok(Some(Target::Light)),
        ".dark" => return Ok(Some(Target::Dark)),
        _ => {}
    }
    let Some(classes) = selector.strip_prefix('.') else {
        return Ok(None);
    };
    let classes: Vec<&str> = classes.split('.').collect();
    let is_class = |c: &&str| !c.is_empty() && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if !classes.iter().all(is_class) {
        return Ok(None);
    }
    let (class, base) = match classes.as_slice() {
        [class] => (*class, ThemeMode::Light),
        ["dark", class] | [class, "dark"] => (*class, ThemeMode::Dark),
        _ => return Ok(None),
    };
    let mut name = class.replace('-', "_");
    if base == ThemeMode::Dark {
        name.push_str("_dark");
    }
    if !name.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_') {
        bail!("CSS class `.{class}` can't name a style mode (a mode name starts with a letter or _)");
    }
    Ok(Some(Target::Mode(name, base)))
}

/// Every style rule as (selector list, body), in file order. `@layer` blocks are searched; other at-rules
/// (`@theme`, `@media`, `@keyframes`, …) are skipped.
fn scan_rules<'a>(source: &'a str, rules: &mut Vec<(&'a str, &'a str)>) -> Result<()> {
    let mut start = 0;
    while let Some(open) = source[start..].find('{').map(|i| start + i) {
        // The selector is the text since the last statement (`;`) or block (`}`).
        let prelude = &source[start..open];
        let selector = prelude.rfind(';').map_or(prelude, |i| &prelude[i + 1..]).trim();
        let close = find_matching_brace(source, open).ok_or_else(|| anyhow!("unclosed block after `{selector}`"))?;
        let body = &source[open + 1..close];
        if selector.starts_with("@layer") {
            scan_rules(body, rules)?;
        } else if !selector.starts_with('@') {
            rules.push((selector, body));
        }
        start = close + 1;
    }
    Ok(())
}

fn strip_css_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start + 2..].split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

fn find_matching_brace(source: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, ch) in source[open_index..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_index + index);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_declarations(block: &str, tokens: &mut BTreeMap<String, String>) -> Result<()> {
    let mut cursor = 0;
    while cursor < block.len() {
        while cursor < block.len() && block.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= block.len() {
            break;
        }

        if block[cursor..].starts_with("--") {
            let decl = next_declaration(&block[cursor..]).context("invalid custom property declaration")?;
            let name = decl
                .name
                .strip_prefix("--")
                .ok_or_else(|| anyhow!("expected custom property name, got `{}`", decl.name))?
                .to_string();
            tokens.insert(name, decl.value.trim().to_string());
            cursor += decl.consumed;
            continue;
        }

        if let Some(skip) = block[cursor..].find(';') {
            cursor += skip + 1;
            continue;
        }
        break;
    }

    Ok(())
}

struct CssDeclaration<'a> {
    name: &'a str,
    value: &'a str,
    consumed: usize,
}

fn next_declaration(source: &str) -> Result<CssDeclaration<'_>> {
    let Some(colon) = source.find(':') else {
        bail!("missing ':' in custom property declaration");
    };
    let name = source[..colon].trim();
    if !name.starts_with("--") {
        bail!("expected custom property, got `{name}`");
    }

    let mut value_start = colon + 1;
    while value_start < source.len() && source.as_bytes()[value_start].is_ascii_whitespace() {
        value_start += 1;
    }

    let mut index = value_start;
    let bytes = source.as_bytes();
    while index < source.len() {
        let ch = bytes[index];
        if ch == b';' {
            let value = source[value_start..index].trim();
            return Ok(CssDeclaration { name, value, consumed: index + 1 });
        }
        if ch == b'"' || ch == b'\'' {
            index = skip_quoted(source, index + 1, ch as char)? + 1;
            continue;
        }
        if ch == b'(' {
            index = skip_balanced(source, index + 1, '(', ')')? + 1;
            continue;
        }
        index += 1;
    }

    bail!("unterminated custom property `{name}`");
}

fn skip_quoted(source: &str, mut index: usize, quote: char) -> Result<usize> {
    while index < source.len() {
        let ch = source[index..].chars().next().unwrap();
        if ch == quote {
            return Ok(index);
        }
        if ch == '\\' {
            index += 2;
            continue;
        }
        index += ch.len_utf8();
    }
    Err(anyhow!("unterminated string literal"))
}

fn skip_balanced(source: &str, mut index: usize, open: char, close: char) -> Result<usize> {
    let mut depth = 1;
    while index < source.len() {
        let ch = source[index..].chars().next().unwrap();
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        } else if ch == '"' || ch == '\'' {
            index = skip_quoted(source, index + 1, ch)? + 1;
            continue;
        }
        index += ch.len_utf8();
    }
    Err(anyhow!("unterminated function in custom property value"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_and_dark_blocks() {
        let css = r#"
        :root {
          --primary: oklch(0.5924 0.2025 355.8943);
          --primary-foreground: oklch(1 0 0);
          --secondary: oklch(0.6437 0.1019 187.3840);
          --secondary-foreground: oklch(1 0 0);
          --background: oklch(0.9735 0.0261 90.0953);
          --foreground: oklch(0.3092 0.0518 219.6516);
          --muted: oklch(0.6979 0.0159 196.7940);
          --muted-foreground: oklch(0.3092 0.0518 219.6516);
          --accent: oklch(0.5808 0.1732 39.5003);
          --accent-foreground: oklch(1 0 0);
          --border: oklch(0.6537 0.0197 205.2618);
          --input: oklch(0.6537 0.0197 205.2618);
          --ring: oklch(0.5924 0.2025 355.8943);
          --card: oklch(0.9306 0.0260 92.4020);
        }
        .dark {
          --primary: oklch(0.5924 0.2025 355.8943);
          --primary-foreground: oklch(1 0 0);
          --secondary: oklch(0.6437 0.1019 187.3840);
          --secondary-foreground: oklch(1 0 0);
          --background: oklch(0.2673 0.0486 219.8169);
          --foreground: oklch(0.6979 0.0159 196.7940);
          --muted: oklch(0.5230 0.0283 219.1365);
          --muted-foreground: oklch(0.6979 0.0159 196.7940);
          --accent: oklch(0.5808 0.1732 39.5003);
          --accent-foreground: oklch(1 0 0);
          --border: oklch(0.5230 0.0283 219.1365);
          --ring: oklch(0.5924 0.2025 355.8943);
          --card: oklch(0.3092 0.0518 219.6516);
        }
        "#;
        let catalog = parse_css_catalog(css).expect("fixture should parse");
        assert!(catalog.light.contains_key("primary"));
        assert!(catalog.dark.contains_key("background"));
    }

    #[test]
    fn class_blocks_become_modes_based_on_light_or_dark() {
        let css = r#"
        @import "tailwindcss";
        @custom-variant dark (&:is(.dark *));
        :root { --primary: red; --background: white; }
        .dark { --primary: pink; --background: black; }
        .hi-contrast { --primary: blue; }
        .dark.hi-contrast { --primary: cyan; }
        @theme inline { --color-primary: var(--primary); }
        @layer base { * { border-color: red; } body { color: black; } }
        .card:hover { --primary: green; }
        "#;
        let catalog = parse_css_catalog(css).expect("parses");
        assert_eq!(catalog.light["primary"], "red");
        assert_eq!(catalog.dark["primary"], "pink", "`.dark.hi-contrast` stays out of dark");
        let modes: Vec<(&str, ThemeMode)> = catalog.modes.iter().map(|m| (m.name.as_str(), m.base)).collect();
        assert_eq!(modes, [("hi_contrast", ThemeMode::Light), ("hi_contrast_dark", ThemeMode::Dark)]);
        let high = catalog.mode_map(&catalog.modes[0]);
        assert_eq!((high.get("primary"), high.get("background")), (Some("blue"), Some("white")), "layered over :root");
        let high_dark = catalog.mode_map(&catalog.modes[1]);
        assert_eq!((high_dark.get("primary"), high_dark.get("background")), (Some("cyan"), Some("black")));
    }

    #[test]
    fn blocks_inside_layer_count_and_bad_class_names_are_errors() {
        let css = "@layer base { :root { --primary: red; } .dark { --primary: pink; } }";
        let catalog = parse_css_catalog(css).expect("parses");
        assert_eq!((catalog.light["primary"].as_str(), catalog.dark["primary"].as_str()), ("red", "pink"));
        let bad = ":root { --a: 1; } .dark { --a: 2; } .2x { --a: 3; }";
        assert!(parse_css_catalog(bad).is_err());
    }
}
