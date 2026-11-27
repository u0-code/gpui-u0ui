//! Gallery descriptions codegen: `components/<Name>/README.md` →
//! `crates/gallery/src/generated/descriptions.rs`.
//!
//! The gallery shows the first sentence of each card's README under the page
//! title, so only that sentence is embedded. The gallery renders its
//! `` `code` `` spans; other inline Markdown (`**strong**`, `__strong__`,
//! `[text](url)`) is reduced to its text here. Every component directory gets a
//! `Component` variant, including cards without a gallery page yet.
use std::{fmt::Write as _, path::Path};

/// Generate the descriptions module from `<design>/components`.
pub fn descriptions(components: &Path) -> Result<String, String> {
    let entries =
        std::fs::read_dir(components).map_err(|e| format!("read {}: {e}", components.display()))?;
    let mut cards = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_ident(&name) {
            return Err(format!(
                "component directory {name:?} is not a Rust identifier"
            ));
        }
        let readme = entry.path().join("README.md");
        let summary = match std::fs::read_to_string(&readme) {
            Ok(text) => plain_markdown(&first_sentence(&text)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(format!("read {}: {e}", readme.display())),
        };
        cards.push((name, summary));
    }
    cards.sort();

    let mut out = String::from(crate::HEADER);
    out.push_str("// Source: $U0UI_DESIGN_DIR/components/<Name>/README.md\n\n");
    out.push_str("/// Every component card in the design reference (`components/<Name>`).\n");
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum Component {\n");
    for (name, summary) in &cards {
        if summary.is_empty() {
            writeln!(out, "    /// `{name}` (no README)").unwrap();
        } else {
            writeln!(out, "    /// `{name}`").unwrap();
        }
        writeln!(out, "    {name},").unwrap();
    }
    out.push_str("}\n\nimpl Component {\n");
    writeln!(
        out,
        "    /// Every component, by name.\n    pub const ALL: [Component; {}] = [",
        cards.len()
    )
    .unwrap();
    for (name, _) in &cards {
        writeln!(out, "        Component::{name},").unwrap();
    }
    out.push_str("    ];\n\n");
    out.push_str("    /// The component directory name, e.g. `\"Button\"`.\n    pub const fn name(self) -> &'static str {\n        match self {\n");
    for (name, _) in &cards {
        writeln!(out, "            Component::{name} => \"{name}\",").unwrap();
    }
    out.push_str("        }\n    }\n\n");
    out.push_str("    /// The first sentence of the component's README (empty when it has none).\n    pub const fn summary(self) -> &'static str {\n        match self {\n");
    for (name, summary) in &cards {
        writeln!(out, "            Component::{name} => {summary:?},").unwrap();
    }
    out.push_str("        }\n    }\n}\n");
    Ok(out)
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The first sentence of a README body (skipping the `# Title` line).
pub fn first_sentence(readme: &str) -> String {
    let body = readme
        .lines()
        .find(|l| !(l.starts_with('#') || l.trim().is_empty()))
        .unwrap_or("");
    // A Latin `.`/`!` ends the sentence only before whitespace or the end, so
    // code like `.cds--label` inside it does not.
    let end = body
        .char_indices()
        .find(|&(i, c)| match c {
            '。' | '！' => true,
            '.' | '!' => body[i + 1..].chars().next().is_none_or(char::is_whitespace),
            _ => false,
        })
        .map_or(body.len(), |(i, c)| i + c.len_utf8());
    body[..end].trim().to_string()
}

/// Inline Markdown other than code spans reduced to plain text: `**x**` and
/// `__x__` lose their markers, `[text](url)` keeps its text. Code spans are
/// left as they are (the gallery draws them as inline code).
pub fn plain_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, part) in text.split('`').enumerate() {
        if i > 0 {
            out.push('`');
        }
        // Odd parts sit between backticks: code, verbatim. An unpaired
        // trailing backtick leaves a last odd part, also verbatim.
        if i % 2 == 1 {
            out.push_str(part);
        } else {
            out.push_str(&plain_prose(part));
        }
    }
    out
}

fn plain_prose(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let tail = &rest[open + 1..];
        let link = tail.find("](").and_then(|close| {
            let url = &tail[close + 2..];
            url.find(')').map(|end| (close, close + 2 + end + 1))
        });
        match link {
            Some((close, after)) if !tail[..close].contains('[') => {
                s.push_str(&rest[..open]);
                s.push_str(&tail[..close]);
                rest = &tail[after..];
            }
            _ => {
                s.push_str(&rest[..=open]);
                rest = tail;
            }
        }
    }
    s.push_str(rest);
    for marker in ["**", "__"] {
        // Only paired markers are emphasis; an odd one out stays literal.
        if s.matches(marker).count() % 2 == 0 {
            s = s.replace(marker, "");
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_markdown_becomes_plain_text() {
        assert_eq!(
            plain_markdown("图标统一用 **Material Symbols Sharp**（Google）。"),
            "图标统一用 Material Symbols Sharp（Google）。"
        );
        assert_eq!(
            plain_markdown("See [Carbon](https://carbondesignsystem.com)."),
            "See Carbon."
        );
        assert_eq!(
            plain_markdown("__Bold__ and `**code**`"),
            "Bold and `**code**`"
        );
        assert_eq!(plain_markdown("a ** b"), "a ** b");
        assert_eq!(plain_markdown("`<select>` [x] y"), "`<select>` [x] y");
    }

    #[test]
    fn readme_first_sentence() {
        assert_eq!(
            first_sentence("# Button\n\n触发一个操作。shadcn `Button` → Carbon"),
            "触发一个操作。"
        );
        assert_eq!(
            first_sentence("# Label\n\n字段标签，样式为 `.cds--label`（`label-01`）。用"),
            "字段标签，样式为 `.cds--label`（`label-01`）。"
        );
        assert_eq!(
            first_sentence("# Kbd\n\nShows a key. More."),
            "Shows a key."
        );
    }
}
