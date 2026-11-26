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
        id: "type-styles",
        title: "Type styles",
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
        id: "button-group",
        title: "ButtonGroup",
        group: "Actions",
        description: readme!("ButtonGroup"),
        build: pages::button_group::build,
    },
    PageDef {
        id: "toggle",
        title: "Toggle",
        group: "Actions",
        description: readme!("Toggle"),
        build: pages::toggle::build,
    },
    PageDef {
        id: "toggle-group",
        title: "ToggleGroup",
        group: "Actions",
        description: readme!("ToggleGroup"),
        build: pages::toggle_group::build,
    },
    PageDef {
        id: "input",
        title: "Input",
        group: "Forms",
        description: readme!("Input"),
        build: pages::input::build,
    },
    PageDef {
        id: "textarea",
        title: "Textarea",
        group: "Forms",
        description: readme!("Textarea"),
        build: pages::textarea::build,
    },
    PageDef {
        id: "label",
        title: "Label",
        group: "Forms",
        description: readme!("Label"),
        build: pages::label::build,
    },
    PageDef {
        id: "checkbox",
        title: "Checkbox",
        group: "Forms",
        description: readme!("Checkbox"),
        build: pages::checkbox::build,
    },
    PageDef {
        id: "radio-group",
        title: "RadioGroup",
        group: "Forms",
        description: readme!("RadioGroup"),
        build: pages::radio_group::build,
    },
    PageDef {
        id: "select",
        title: "Select",
        group: "Forms",
        description: readme!("Select"),
        build: pages::select::build,
    },
    PageDef {
        id: "native-select",
        title: "NativeSelect",
        group: "Forms",
        description: readme!("NativeSelect"),
        build: pages::native_select::build,
    },
    PageDef {
        id: "combobox",
        title: "Combobox",
        group: "Forms",
        description: readme!("Combobox"),
        build: pages::combobox::build,
    },
    PageDef {
        id: "calendar",
        title: "Calendar",
        group: "Forms",
        description: readme!("Calendar"),
        build: pages::calendar::build,
    },
    PageDef {
        id: "date-picker",
        title: "DatePicker",
        group: "Forms",
        description: readme!("DatePicker"),
        build: pages::date_picker::build,
    },
    PageDef {
        id: "input-group",
        title: "InputGroup",
        group: "Forms",
        description: readme!("InputGroup"),
        build: pages::input_group::build,
    },
    PageDef {
        id: "field",
        title: "Field",
        group: "Forms",
        description: readme!("Field"),
        build: pages::field::build,
    },
    PageDef {
        id: "input-otp",
        title: "InputOTP",
        group: "Forms",
        description: readme!("InputOTP"),
        build: pages::input_otp::build,
    },
    PageDef {
        id: "switch",
        title: "Switch",
        group: "Forms",
        description: readme!("Switch"),
        build: pages::switch::build,
    },
    PageDef {
        id: "slider",
        title: "Slider",
        group: "Forms",
        description: readme!("Slider"),
        build: pages::slider::build,
    },
    PageDef {
        id: "dialog",
        title: "Dialog",
        group: "Overlays",
        description: readme!("Dialog"),
        build: pages::dialog::build,
    },
    PageDef {
        id: "alert-dialog",
        title: "AlertDialog",
        group: "Overlays",
        description: readme!("AlertDialog"),
        build: pages::alert_dialog::build,
    },
    PageDef {
        id: "sheet",
        title: "Sheet",
        group: "Overlays",
        description: readme!("Sheet"),
        build: pages::sheet::build,
    },
    PageDef {
        id: "drawer",
        title: "Drawer",
        group: "Overlays",
        description: readme!("Drawer"),
        build: pages::drawer::build,
    },
    PageDef {
        id: "tooltip",
        title: "Tooltip",
        group: "Overlays",
        description: readme!("Tooltip"),
        build: pages::tooltip::build,
    },
    PageDef {
        id: "popover",
        title: "Popover",
        group: "Overlays",
        description: readme!("Popover"),
        build: pages::popover::build,
    },
    PageDef {
        id: "hover-card",
        title: "HoverCard",
        group: "Overlays",
        description: readme!("HoverCard"),
        build: pages::hover_card::build,
    },
    PageDef {
        id: "dropdown-menu",
        title: "DropdownMenu",
        group: "Overlays",
        description: readme!("DropdownMenu"),
        build: pages::dropdown_menu::build,
    },
    PageDef {
        id: "context-menu",
        title: "ContextMenu",
        group: "Overlays",
        description: readme!("ContextMenu"),
        build: pages::context_menu::build,
    },
    PageDef {
        id: "menubar",
        title: "Menubar",
        group: "Navigation",
        description: readme!("Menubar"),
        build: pages::menubar::build,
    },
    PageDef {
        id: "command",
        title: "Command",
        group: "Overlays",
        description: readme!("Command"),
        build: pages::command::build,
    },
    PageDef {
        id: "tabs",
        title: "Tabs",
        group: "Navigation",
        description: readme!("Tabs"),
        build: pages::tabs::build,
    },
    PageDef {
        id: "accordion",
        title: "Accordion",
        group: "Navigation",
        description: readme!("Accordion"),
        build: pages::accordion::build,
    },
    PageDef {
        id: "collapsible",
        title: "Collapsible",
        group: "Navigation",
        description: readme!("Collapsible"),
        build: pages::collapsible::build,
    },
    PageDef {
        id: "breadcrumb",
        title: "Breadcrumb",
        group: "Navigation",
        description: readme!("Breadcrumb"),
        build: pages::breadcrumb::build,
    },
    PageDef {
        id: "pagination",
        title: "Pagination",
        group: "Navigation",
        description: readme!("Pagination"),
        build: pages::pagination::build,
    },
    PageDef {
        id: "navigation-menu",
        title: "NavigationMenu",
        group: "Navigation",
        description: readme!("NavigationMenu"),
        build: pages::navigation_menu::build,
    },
    PageDef {
        id: "sidebar",
        title: "Sidebar",
        group: "Navigation",
        description: readme!("Sidebar"),
        build: pages::sidebar::build,
    },
    PageDef {
        id: "aspect-ratio",
        title: "AspectRatio",
        group: "Data display",
        description: readme!("AspectRatio"),
        build: pages::aspect_ratio::build,
    },
    PageDef {
        id: "kbd",
        title: "Kbd",
        group: "Data display",
        description: readme!("Kbd"),
        build: pages::kbd::build,
    },
    PageDef {
        id: "separator",
        title: "Separator",
        group: "Data display",
        description: readme!("Separator"),
        build: pages::separator::build,
    },
    PageDef {
        id: "badge",
        title: "Badge",
        group: "Data display",
        description: readme!("Badge"),
        build: pages::badge::build,
    },
    PageDef {
        id: "card",
        title: "Card",
        group: "Data display",
        description: readme!("Card"),
        build: pages::card::build,
    },
    PageDef {
        id: "chart",
        title: "Chart",
        group: "Data display",
        description: readme!("Chart"),
        build: pages::chart::build,
    },
    PageDef {
        id: "scroll-area",
        title: "ScrollArea",
        group: "Layout",
        description: readme!("ScrollArea"),
        build: pages::scroll_area::build,
    },
    PageDef {
        id: "resizable",
        title: "Resizable",
        group: "Layout",
        description: readme!("Resizable"),
        build: pages::resizable::build,
    },
    PageDef {
        id: "avatar",
        title: "Avatar",
        group: "Data display",
        description: readme!("Avatar"),
        build: pages::avatar::build,
    },
    PageDef {
        id: "spinner",
        title: "Spinner",
        group: "Feedback",
        description: readme!("Spinner"),
        build: pages::spinner::build,
    },
    PageDef {
        id: "direction",
        title: "Direction",
        group: "Layout",
        description: readme!("Direction"),
        build: pages::direction::build,
    },
    PageDef {
        id: "carousel",
        title: "Carousel",
        group: "Layout",
        description: readme!("Carousel"),
        build: pages::carousel::build,
    },
    PageDef {
        id: "skeleton",
        title: "Skeleton",
        group: "Feedback",
        description: readme!("Skeleton"),
        build: pages::skeleton::build,
    },
    PageDef {
        id: "progress",
        title: "Progress",
        group: "Feedback",
        description: readme!("Progress"),
        build: pages::progress::build,
    },
    PageDef {
        id: "table",
        title: "Table",
        group: "Data display",
        description: readme!("Table"),
        build: pages::table::build,
    },
    PageDef {
        id: "data-table",
        title: "DataTable",
        group: "Data display",
        description: readme!("DataTable"),
        build: pages::data_table::build,
    },
    PageDef {
        id: "bubble",
        title: "Bubble",
        group: "Chat & AI",
        description: readme!("Bubble"),
        build: pages::bubble::build,
    },
    PageDef {
        id: "message",
        title: "Message",
        group: "Chat & AI",
        description: readme!("Message"),
        build: pages::message::build,
    },
    PageDef {
        id: "marker",
        title: "Marker",
        group: "Chat & AI",
        description: readme!("Marker"),
        build: pages::marker::build,
    },
    PageDef {
        id: "code-block",
        title: "CodeBlock",
        group: "Data display",
        description: readme!("CodeBlock"),
        build: pages::code_block::build,
    },
    PageDef {
        id: "empty",
        title: "Empty",
        group: "Data display",
        description: readme!("Empty"),
        build: pages::empty::build,
    },
    PageDef {
        id: "item",
        title: "Item",
        group: "Data display",
        description: readme!("Item"),
        build: pages::item::build,
    },
    PageDef {
        id: "typography",
        title: "Typography",
        group: "Data display",
        description: readme!("Typography"),
        build: pages::typeset::build,
    },
    PageDef {
        id: "message-scroller",
        title: "MessageScroller",
        group: "Chat & AI",
        description: readme!("MessageScroller"),
        build: pages::message_scroller::build,
    },
    PageDef {
        id: "attachment",
        title: "Attachment",
        group: "Chat & AI",
        description: readme!("Attachment"),
        build: pages::attachment::build,
    },
    PageDef {
        id: "alert",
        title: "Alert",
        group: "Feedback",
        description: readme!("Alert"),
        build: pages::alert::build,
    },
    PageDef {
        id: "toast",
        title: "Toast",
        group: "Feedback",
        description: readme!("Toast"),
        build: pages::toast::build,
    },
    PageDef {
        id: "questionnaire",
        title: "Questionnaire",
        group: "Chat & AI",
        description: readme!("Questionnaire"),
        build: pages::questionnaire::build,
    },
    PageDef {
        id: "trace",
        title: "Trace",
        group: "Chat & AI",
        description: readme!("Trace"),
        build: pages::trace::build,
    },
];

/// Look a page up by id.
pub fn find(id: &str) -> Option<usize> {
    PAGES.iter().position(|p| p.id == id)
}

/// The reference artifact's group order (`components/index.d.ts` sections).
pub const GROUP_ORDER: &[&str] = &[
    "Foundations",
    "Actions",
    "Forms",
    "Data display",
    "Feedback",
    "Navigation",
    "Overlays",
    "Layout",
    "Chat & AI",
];

/// The navigation groups: the artifact's groups in its order, then any other
/// group in first-use order.
pub fn groups() -> Vec<&'static str> {
    let mut groups: Vec<&'static str> = GROUP_ORDER
        .iter()
        .copied()
        .filter(|g| PAGES.iter().any(|p| p.group == *g))
        .collect();
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
        assert_eq!(
            first_sentence("# Label\n\n字段标签，样式为 `.cds--label`（`label-01`）。用"),
            "字段标签，样式为 `.cds--label`（`label-01`）。"
        );
    }
}
