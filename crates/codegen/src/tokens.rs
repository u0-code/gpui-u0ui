//! Token and icon codegen: `tokens.json` → `generated/tokens.rs` and
//! `src/icons/material.js` → `generated/carbon_icons.rs` (both in `crates/u0ui/src`).
//!
//! Generates the `ColorRole` enum and the four per-theme color tables, the
//! `TypeStyle` enum with its specs, and the spacing / size / radius consts that
//! `u0ui::tokens` re-exports. Never hand-type a token value: change the JSON.
use serde_json::Value;
use std::{collections::HashMap, fmt::Write as _};

/// `tokens.json` → `crates/u0ui/src/generated/tokens.rs`.
pub fn tokens(source: &str) -> String {
    let json: Value = serde_json::from_str(source).expect("tokens.json is valid JSON");
    let mut out = String::new();
    out.push_str(crate::HEADER);
    out.push_str("// Source: $U0UI_DESIGN_DIR/tokens.json\n\n");
    colors(&json, &mut out);
    type_styles(&json, &mut out);
    families(&json, &mut out);
    lengths(&json, "spacing", &mut out);
    lengths(&json, "size", &mut out);
    lengths(&json, "radius", &mut out);
    out
}

/// `CARBON_TO_MATERIAL` in the reference bundle → the `CarbonIcon` enum.
/// A trailing `+` on the Material name means FILL 1.
pub fn carbon_icons(js: &str) -> String {
    let start = js
        .find("CARBON_TO_MATERIAL = {")
        .expect("CARBON_TO_MATERIAL in material.js")
        + "CARBON_TO_MATERIAL = ".len();
    let end = start + js[start..].find('}').unwrap() + 1;
    let map: serde_json::Map<String, Value> = serde_json::from_str(&js[start..end]).unwrap();
    let mut out = String::from(crate::HEADER);
    out.push_str("// Source: $U0UI_DESIGN_DIR/src/icons/material.js\n\n");
    out.push_str("/// The reference's Carbon-named icons (`CarbonUI.Icons.<Name>`), drawn with Material Symbols Sharp.\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum CarbonIcon {\n");
    for (name, material) in &map {
        writeln!(out, "    /// `{name}` → `{}`", material.as_str().unwrap()).unwrap();
        writeln!(out, "    {},", camel(name)).unwrap();
    }
    out.push_str("}\n\nimpl CarbonIcon {\n");
    writeln!(
        out,
        "    /// Every Carbon-named icon.\n    pub const ALL: [CarbonIcon; {}] = [",
        map.len()
    )
    .unwrap();
    for name in map.keys() {
        writeln!(out, "        CarbonIcon::{},", camel(name)).unwrap();
    }
    out.push_str("    ];\n\n    /// `(carbon name, material symbol name, filled)`.\n    pub const fn spec(self) -> (&'static str, &'static str, bool) {\n        match self {\n");
    for (name, material) in &map {
        let material = material.as_str().unwrap();
        let (symbol, filled) = match material.strip_suffix('+') {
            Some(s) => (s, true),
            None => (material, false),
        };
        writeln!(
            out,
            "            CarbonIcon::{} => (\"{name}\", \"{symbol}\", {filled}),",
            camel(name)
        )
        .unwrap();
    }
    out.push_str("        }\n    }\n}\n");
    out
}

/// `border-subtle-01` → `BorderSubtle01`.
fn camel(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().unwrap().to_ascii_uppercase();
            std::iter::once(first).chain(chars).collect::<String>()
        })
        .collect()
}

/// `border-subtle-01` → `BORDER_SUBTLE_01`.
fn shouty(name: &str) -> String {
    name.replace(['-', '_'], "_").to_ascii_uppercase()
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()` or `transparent`.
pub(crate) fn parse_color(value: &str) -> [f32; 4] {
    let value = value.trim();
    if value == "transparent" {
        return [0., 0., 0., 0.];
    }
    if let Some(hex) = value.strip_prefix('#') {
        let expand = |s: &str| -> String {
            if s.len() <= 4 {
                s.chars().flat_map(|c| [c, c]).collect()
            } else {
                s.to_string()
            }
        };
        let hex = expand(hex);
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f32 / 255.;
        let a = if hex.len() == 8 { byte(6) } else { 1. };
        return [byte(0), byte(2), byte(4), a];
    }
    let inner = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unsupported color value {value:?}"));
    let parts: Vec<f32> = inner
        .split([',', ' ', '/'])
        .filter(|s| !s.is_empty())
        .map(|s| {
            if let Some(p) = s.strip_suffix('%') {
                p.parse::<f32>().unwrap() / 100.
            } else {
                s.parse::<f32>().unwrap()
            }
        })
        .collect();
    let a = parts.get(3).copied().unwrap_or(1.);
    [parts[0] / 255., parts[1] / 255., parts[2] / 255., a]
}

fn colors(json: &Value, out: &mut String) {
    let themes: Vec<String> = json["color"]["themes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(themes, ["white", "g10", "g90", "g100"], "theme order");
    let tokens = json["color"]["tokens"].as_array().unwrap();
    let names: Vec<&str> = tokens.iter().map(|t| t["name"].as_str().unwrap()).collect();
    let index: HashMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (*n, i)).collect();
    let mut seen = HashMap::new();
    for name in &names {
        if let Some(previous) = seen.insert(camel(name), *name) {
            panic!("{name} and {previous} map to the same variant");
        }
    }

    // Raw per-theme strings; a token missing a theme inherits the first theme's value.
    let raw = |token: usize, theme: &str| -> String {
        match &tokens[token]["value"] {
            Value::String(s) => s.clone(),
            Value::Object(map) => map
                .get(theme)
                .or_else(|| map.get(&themes[0]))
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{} has no value for {theme}", names[token]))
                .to_string(),
            other => panic!("{}: unexpected value {other}", names[token]),
        }
    };
    let resolve = |token: usize, theme: &str| -> [f32; 4] {
        let mut current = token;
        for _ in 0..16 {
            let value = raw(current, theme);
            match value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
                Some(alias) => {
                    current = *index
                        .get(alias)
                        .unwrap_or_else(|| panic!("{}: unknown alias {alias}", names[token]))
                }
                None => return parse_color(&value),
            }
        }
        panic!("{}: alias cycle", names[token]);
    };

    let count = names.len();
    writeln!(
        out,
        "/// One variant per Carbon color token in `tokens.json` ({count} tokens)."
    )
    .unwrap();
    out.push_str(
        "///\n/// Paint with [`crate::theme::color`]; [`ColorRole::name`] is the original token name.\n",
    );
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]\n#[repr(u16)]\npub enum ColorRole {\n");
    for (i, token) in tokens.iter().enumerate() {
        let usage = token["usage"].as_str().unwrap_or("");
        writeln!(out, "    /// `{}`: {}", names[i], usage.replace('\n', " ")).unwrap();
        writeln!(out, "    {},", camel(names[i])).unwrap();
    }
    out.push_str("}\n\nimpl ColorRole {\n");
    writeln!(
        out,
        "    /// Number of color roles.\n    pub const COUNT: usize = {count};"
    )
    .unwrap();
    out.push_str("    /// Every role, in `tokens.json` order.\n    pub const ALL: [ColorRole; Self::COUNT] = [\n");
    for name in &names {
        writeln!(out, "        ColorRole::{},", camel(name)).unwrap();
    }
    out.push_str("    ];\n");
    out.push_str("    /// The token name in `tokens.json`, e.g. `\"border-subtle-01\"`.\n    pub const fn name(self) -> &'static str {\n        COLOR_NAMES[self as usize]\n    }\n");
    out.push_str("    /// The token's usage note from `tokens.json`.\n    pub const fn usage(self) -> &'static str {\n        COLOR_USAGE[self as usize]\n    }\n");
    out.push_str("    /// Look a role up by its token name.\n    pub fn from_name(name: &str) -> Option<Self> {\n        COLOR_NAMES.iter().position(|n| *n == name).map(|i| Self::ALL[i])\n    }\n}\n\n");
    writeln!(out, "const COLOR_NAMES: [&str; {count}] = [").unwrap();
    for name in &names {
        writeln!(out, "    \"{name}\",").unwrap();
    }
    out.push_str("];\n\n");
    writeln!(out, "const COLOR_USAGE: [&str; {count}] = [").unwrap();
    for token in tokens {
        writeln!(
            out,
            "    \"{}\",",
            escape(token["usage"].as_str().unwrap_or(""))
        )
        .unwrap();
    }
    out.push_str("];\n\n");
    out.push_str("/// Resolved colors per theme (`[white, g10, g90, g100]`), aliases resolved, alpha kept.\n");
    writeln!(
        out,
        "pub(crate) static COLOR_TABLES: [[Rgba; {count}]; 4] = ["
    )
    .unwrap();
    for theme in &themes {
        writeln!(out, "    // {theme}\n    [").unwrap();
        for (i, name) in names.iter().enumerate() {
            let [r, g, b, a] = resolve(i, theme);
            writeln!(
                out,
                "        Rgba {{ r: {r:?}, g: {g:?}, b: {b:?}, a: {a:?} }}, // {name}"
            )
            .unwrap();
        }
        out.push_str("    ],\n");
    }
    out.push_str("];\n\n");
}

/// `0.875rem` → (0.875, rem); `20px` → (20, px).
fn parse_length(value: &str) -> (f32, &'static str) {
    let value = value.trim();
    if let Some(v) = value.strip_suffix("rem") {
        (v.parse().unwrap(), "Rem")
    } else if let Some(v) = value.strip_suffix("px") {
        (v.parse().unwrap(), "Px")
    } else {
        (value.parse().unwrap(), "Px")
    }
}

fn type_styles(json: &Value, out: &mut String) {
    let groups = json["type"]["groups"].as_array().unwrap();
    let mut styles = Vec::new();
    for group in groups {
        let family = group["family"].as_str().unwrap();
        for style in group["styles"].as_array().unwrap() {
            styles.push((group["name"].as_str().unwrap(), family, style));
        }
    }
    out.push_str("/// Every Carbon type style in `tokens.json`; see [`TypeStyle::spec`].\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum TypeStyle {\n");
    for (group, _, style) in &styles {
        let name = style["name"].as_str().unwrap();
        writeln!(
            out,
            "    /// `{name}` ({group}): {}",
            style["usage"].as_str().unwrap_or("")
        )
        .unwrap();
        writeln!(out, "    {},", camel(name)).unwrap();
    }
    out.push_str("}\n\nimpl TypeStyle {\n    /// Every type style, in `tokens.json` order.\n");
    writeln!(out, "    pub const ALL: [TypeStyle; {}] = [", styles.len()).unwrap();
    for (_, _, style) in &styles {
        writeln!(
            out,
            "        TypeStyle::{},",
            camel(style["name"].as_str().unwrap())
        )
        .unwrap();
    }
    out.push_str("    ];\n\n    /// The resolved spec of this style.\n    pub const fn spec(self) -> TypeSpec {\n        match self {\n");
    for (group, family, style) in &styles {
        let name = style["name"].as_str().unwrap();
        let (size, unit) = parse_length(style["fontSize"].as_str().unwrap());
        assert_eq!(unit, "Rem", "{name}: font sizes are rem");
        let line_height = match &style["lineHeight"] {
            Value::Number(n) => format!("LineHeight::Multiplier({:?})", n.as_f64().unwrap() as f32),
            Value::String(s) => {
                let (v, unit) = parse_length(s);
                if unit == "Px" {
                    format!("LineHeight::Px({v:?})")
                } else {
                    format!("LineHeight::Rem({v:?})")
                }
            }
            other => panic!("{name}: bad line height {other}"),
        };
        let weight = style["fontWeight"].as_u64().unwrap();
        let (spacing, unit) = parse_length(style["letterSpacing"].as_str().unwrap_or("0px"));
        assert_eq!(unit, "Px", "{name}: letter spacing is px");
        let family = match *family {
            "sans" => "FontFamilyKey::Sans",
            "mono" => "FontFamilyKey::Mono",
            "serif" => "FontFamilyKey::Serif",
            "sans-condensed" => "FontFamilyKey::SansCondensed",
            other => panic!("{name}: unknown family {other}"),
        };
        writeln!(
            out,
            "            TypeStyle::{} => TypeSpec {{ name: \"{name}\", group: \"{group}\", font_size_rem: {size:?}, line_height: {line_height}, weight: {weight}, letter_spacing_px: {spacing:?}, family: {family} }},",
            camel(name)
        )
        .unwrap();
    }
    out.push_str("        }\n    }\n}\n\n");
}

/// `type.families`: CSS font stacks → the quoted family names, generic keywords dropped.
fn families(json: &Value, out: &mut String) {
    let families = json["type"]["families"].as_object().unwrap();
    for (key, stack) in families {
        let names: Vec<String> = stack
            .as_str()
            .unwrap()
            .split(',')
            .map(|s| s.trim())
            .filter(|s| s.starts_with('"'))
            .map(|s| s.trim_matches('"').to_string())
            .collect();
        writeln!(
            out,
            "/// `type.families.{key}`: `{}`",
            stack.as_str().unwrap().replace('"', "")
        )
        .unwrap();
        writeln!(
            out,
            "pub const FAMILY_{}: &[&str] = &{names:?};",
            shouty(key)
        )
        .unwrap();
    }
    out.push('\n');
}

fn lengths(json: &Value, section: &str, out: &mut String) {
    let tokens = json[section]["tokens"].as_array().unwrap();
    let mut all = Vec::new();
    for token in tokens {
        let name = token["name"].as_str().unwrap();
        let (value, unit) = parse_length(token["value"].as_str().unwrap());
        let ident = shouty(name);
        writeln!(
            out,
            "/// `{name}` = `{}`: {}",
            token["value"].as_str().unwrap(),
            token["usage"].as_str().unwrap_or("")
        )
        .unwrap();
        writeln!(
            out,
            "pub const {ident}: TokenLength = TokenLength::{unit}({value:?});"
        )
        .unwrap();
        all.push((name, ident));
    }
    writeln!(
        out,
        "/// Every `{section}` token as `(name, value)`, in `tokens.json` order.\npub const {}_TOKENS: [(&str, TokenLength); {}] = [",
        section.to_ascii_uppercase(),
        all.len()
    )
    .unwrap();
    for (name, ident) in all {
        writeln!(out, "    (\"{name}\", {ident}),").unwrap();
    }
    out.push_str("];\n\n");
}
