//! Brand marks of the agents and services an app shows (Claude, Gemini,
//! GitHub, GitLab, …): the SVGs of Simple Icons 16.37.0, vendored unmodified
//! in `assets/brands/` (CC0-1.0; see `assets/brands/NOTICE` for the two
//! exceptions and the trademark note).
//!
//! [`brand_mark`] finds the mark for a provider's id or display name
//! (`"claude-acp"`, `"Claude Code"`, `"GitHub Copilot"`); [`crate::Assets`]
//! serves it at [`BrandMark::path`], and [`BrandMark::icon`] draws it as an
//! [`Icon`] in its ink.
use super::Icon;
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{Hsla, rgb};

/// Asset path prefix [`crate::Assets`] serves the marks under.
pub(crate) const ASSET_PREFIX: &str = "u0ui/brands/";

/// One vendored mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrandMark {
    /// The Simple Icons slug, also the file name (`googlegemini`).
    pub slug: &'static str,
    /// The brand's name as Simple Icons titles it (`Google Gemini`).
    pub title: &'static str,
    /// The brand colour, or `None` for a black or white mark, which draws in
    /// `text-primary` so it reads on every theme.
    ink: Option<u32>,
    svg: &'static str,
}

macro_rules! mark {
    ($slug:literal, $title:literal, $ink:expr) => {
        BrandMark {
            slug: $slug,
            title: $title,
            ink: $ink,
            svg: include_str!(concat!("../../assets/brands/", $slug, ".svg")),
        }
    };
}

/// Every vendored mark. Inks follow the design's rule for provider marks
/// (`t3.css`: Claude `#c15f3c` and Gemini `#1a73e8` keep a brand ink, the
/// monochrome marks take `text-primary`); the other coloured brands take the
/// hex Simple Icons records for them.
pub const BRAND_MARKS: [BrandMark; 17] = [
    // `.t3-provider[title^=Claude]:not(:has(img)) { color: #c15f3c }`.
    mark!("claude", "Claude", Some(0xc15f3c)),
    mark!("anthropic", "Anthropic", None),
    mark!("opencode", "OpenCode", None),
    // `.t3-provider[title^=Gemini]:not(:has(img)) { color: #1a73e8 }`.
    mark!("googlegemini", "Google Gemini", Some(0x1a73e8)),
    mark!("cursor", "Cursor", None),
    mark!("githubcopilot", "GitHub Copilot", None),
    mark!("mistralai", "Mistral AI", Some(0xfa520f)),
    mark!("qwen", "Qwen", Some(0x6950ef)),
    mark!("figma", "Figma", Some(0xf24e1e)),
    mark!("github", "GitHub", None),
    mark!("gitlab", "GitLab", Some(0xfc6d26)),
    mark!("bitbucket", "Bitbucket", Some(0x0052cc)),
    mark!("linear", "Linear", Some(0x5e6ad2)),
    mark!("tailscale", "Tailscale", None),
    mark!("cloudflare", "Cloudflare", Some(0xf38020)),
    mark!("apple", "Apple", None),
    mark!("android", "Android", Some(0x3ddc84)),
];

/// What a name starts with, once lowercased with everything but letters and
/// digits dropped, and the mark it names. Longer prefixes come first
/// (`githubcopilot` before `github`).
const PREFIXES: [(&str, &str); 19] = [
    ("claude", "claude"),
    ("anthropic", "anthropic"),
    ("opencode", "opencode"),
    ("googlegemini", "googlegemini"),
    ("gemini", "googlegemini"),
    ("cursor", "cursor"),
    ("githubcopilot", "githubcopilot"),
    ("copilot", "githubcopilot"),
    ("mistral", "mistralai"),
    ("qwen", "qwen"),
    ("figma", "figma"),
    ("github", "github"),
    ("gitlab", "gitlab"),
    ("bitbucket", "bitbucket"),
    ("linear", "linear"),
    ("tailscale", "tailscale"),
    ("cloudflare", "cloudflare"),
    ("apple", "apple"),
    ("android", "android"),
];

/// The mark for a provider or service, by id or display name: `"claude"`,
/// `"Claude Code"`, `"gemini-cli"`, `"GitHub Copilot"`, `"mistral-vibe"`.
/// `None` for a brand with no vendored mark (Codex, Antigravity, Azure
/// DevOps, …).
pub fn brand_mark(name: &str) -> Option<&'static BrandMark> {
    let key: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let (_, slug) = PREFIXES.iter().find(|(p, _)| key.starts_with(p))?;
    BRAND_MARKS.iter().find(|m| m.slug == *slug)
}

/// The SVG of the mark with this slug, as vendored.
pub(crate) fn asset_svg(slug: &str) -> Option<&'static str> {
    BRAND_MARKS.iter().find(|m| m.slug == slug).map(|m| m.svg)
}

impl BrandMark {
    /// The asset path [`crate::Assets`] serves the SVG at
    /// (`u0ui/brands/claude.svg`), usable wherever an image source is.
    pub fn path(&self) -> String {
        format!("{ASSET_PREFIX}{}.svg", self.slug)
    }

    /// The colour the mark draws in: its brand ink, or `text-primary` for a
    /// monochrome mark.
    pub fn ink(&self) -> Hsla {
        match self.ink {
            Some(hex) => rgb(hex).into(),
            None => color(ColorRole::TextPrimary),
        }
    }

    /// The mark as an icon, in its ink.
    pub fn icon(&self) -> Icon {
        Icon::from_path(self.path()).color(self.ink())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn slug(name: &str) -> Option<&'static str> {
        brand_mark(name).map(|m| m.slug)
    }

    #[test]
    fn marks_are_found_by_id_or_display_name() {
        assert_eq!(slug("Claude"), Some("claude"));
        assert_eq!(slug("Claude Code"), Some("claude"));
        assert_eq!(slug("claude-acp"), Some("claude"));
        assert_eq!(slug("Gemini CLI"), Some("googlegemini"));
        assert_eq!(slug("Google Gemini"), Some("googlegemini"));
        assert_eq!(slug("gemini"), Some("googlegemini"));
        assert_eq!(slug("GitHub Copilot"), Some("githubcopilot"));
        assert_eq!(slug("github-copilot"), Some("githubcopilot"));
        assert_eq!(slug("Copilot"), Some("githubcopilot"));
        assert_eq!(slug("GitHub"), Some("github"));
        assert_eq!(slug("mistral-vibe"), Some("mistralai"));
        assert_eq!(slug("Qwen Code"), Some("qwen"));
        assert_eq!(slug("cursor-agent"), Some("cursor"));
        assert_eq!(slug("OpenCode"), Some("opencode"));
        assert_eq!(slug("GitLab"), Some("gitlab"));
        assert_eq!(slug("Bitbucket"), Some("bitbucket"));
    }

    #[test]
    fn brands_without_a_vendored_mark_find_none() {
        for name in [
            "Codex",
            "OpenAI",
            "Antigravity",
            "Azure DevOps",
            "Goose",
            "Kiro",
            "",
        ] {
            assert_eq!(slug(name), None, "{name}");
        }
    }

    #[test]
    fn every_mark_and_prefix_is_served() {
        for m in &BRAND_MARKS {
            assert!(m.svg.starts_with("<svg") && m.svg.contains("viewBox=\"0 0 24 24\""));
            assert_eq!(asset_svg(m.slug), Some(m.svg));
            assert_eq!(m.path(), format!("u0ui/brands/{}.svg", m.slug));
        }
        for (_, s) in PREFIXES {
            assert!(asset_svg(s).is_some(), "{s}");
        }
        assert!(asset_svg("openai").is_none());
    }

    #[test]
    fn monochrome_marks_take_text_primary_and_the_rest_their_ink() {
        crate::theme::tests_set(CarbonTheme::White);
        let hex = |m: &BrandMark| {
            let c = m.ink().to_rgb();
            let b = |v: f32| (v * 255.).round() as u8;
            format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
        };
        assert_eq!(hex(brand_mark("Claude").unwrap()), "#c15f3c");
        assert_eq!(hex(brand_mark("Gemini").unwrap()), "#1a73e8");
        assert_eq!(hex(brand_mark("GitLab").unwrap()), "#fc6d26");
        assert_eq!(hex(brand_mark("GitHub").unwrap()), "#161616");
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(hex(brand_mark("GitHub").unwrap()), "#f4f4f4");
    }
}
