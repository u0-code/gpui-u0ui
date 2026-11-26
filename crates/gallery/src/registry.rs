//! The page registry. To add a page: write `pages/<name>.rs` exposing
//! `pub fn build(window, cx) -> AnyView`, then add one [`PageDef`] line below.
use crate::pages;
use gpui_kit::{AnyView, App, Window};

/// One gallery page.
pub struct PageDef {
    /// Stable id for `--page <id>` (the reference component id, kebab-case).
    pub id: &'static str,
    /// Navigation and page title.
    pub title: &'static str,
    /// Navigation group (the reference artifact's groups).
    pub group: &'static str,
    /// The reference README (`include_str!`), whose first sentence becomes the
    /// page description; or a description of its own for non-component pages.
    pub description: Description,
    /// Build the page's live view (called once, then cached).
    pub build: fn(&mut Window, &mut App) -> AnyView,
}

/// Where a page's description comes from.
pub enum Description {
    /// The reference `components/<Name>/README.md`.
    Readme(&'static str),
    /// A literal description.
    Text(&'static str),
}

impl Description {
    /// The description shown under the title: the README's first sentence.
    pub fn text(&self) -> String {
        match self {
            Description::Text(text) => text.to_string(),
            Description::Readme(readme) => first_sentence(readme),
        }
    }
}

/// The first sentence of a README body (skipping the `# Title` line).
pub fn first_sentence(readme: &str) -> String {
    let body = readme
        .lines()
        .find(|l| !(l.starts_with('#') || l.trim().is_empty()))
        .unwrap_or("");
    let end = body
        .char_indices()
        .find(|(_, c)| matches!(c, '。' | '.' | '！' | '!'))
        .map_or(body.len(), |(i, c)| i + c.len_utf8());
    body[..end].trim().to_string()
}

/// `readme!("Button")` → the reference README for that component.
macro_rules! readme {
    ($name:literal) => {
        $crate::registry::Description::Readme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../design/reference/components/",
            $name,
            "/README.md"
        )))
    };
}

/// Every page, in navigation order. Groups appear in first-use order.
pub static PAGES: &[PageDef] = &[
    PageDef {
        id: "colors",
        title: "Colors",
        group: "Foundations",
        description: Description::Text(
            "Every color token in the active theme. Paint with roles, never stored colors.",
        ),
        build: pages::colors::build,
    },
    PageDef {
        id: "typography",
        title: "Typography",
        group: "Foundations",
        description: Description::Text(
            "IBM Plex type styles: productive, code and CJK sets from tokens.json.",
        ),
        build: pages::typography::build,
    },
    PageDef {
        id: "layout",
        title: "Spacing, size & radius",
        group: "Foundations",
        description: Description::Text(
            "The 2px/8px mini-unit spacing scale, control heights, density sizes and corner radii.",
        ),
        build: pages::layout::build,
    },
    PageDef {
        id: "motion",
        title: "Motion",
        group: "Foundations",
        description: Description::Text(
            "Carbon productive and expressive curves and duration tokens. Press Play to run them.",
        ),
        build: pages::motion::build,
    },
    PageDef {
        id: "icons",
        title: "Icons",
        group: "Foundations",
        description: Description::Text(
            "Material Symbols Sharp, weight 400, optical size 24, at 16/20/24/32px.",
        ),
        build: pages::icons::build,
    },
    PageDef {
        id: "button",
        title: "Button",
        group: "Actions",
        description: readme!("Button"),
        build: pages::button::build,
    },
    PageDef {
        id: "input",
        title: "Input",
        group: "Forms",
        description: readme!("Input"),
        build: pages::input::build,
    },
];

/// Look a page up by id.
pub fn find(id: &str) -> Option<usize> {
    PAGES.iter().position(|p| p.id == id)
}

/// The navigation groups, in first-use order.
pub fn groups() -> Vec<&'static str> {
    let mut groups = Vec::new();
    for page in PAGES {
        if !groups.contains(&page.group) {
            groups.push(page.group);
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_descriptions_present() {
        for (i, page) in PAGES.iter().enumerate() {
            assert_eq!(find(page.id), Some(i), "duplicate id {}", page.id);
            assert!(!page.description.text().is_empty(), "{}", page.id);
        }
    }

    #[test]
    fn readme_first_sentence() {
        assert_eq!(
            first_sentence("# Button\n\n触发一个操作。shadcn `Button` → Carbon"),
            "触发一个操作。"
        );
    }
}
