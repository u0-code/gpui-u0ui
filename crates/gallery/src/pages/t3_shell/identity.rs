//! T3 · Identity & status: ProviderIcon, ProjectIdentity, StatusIndicators.
use super::{eid, label_text, row, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::shell::{
    BadgeSize, ConnectionStatus, ConnectionStatusDot, EnvironmentBadge, EnvironmentMachineIcon,
    IdentitySize, MachineKind, ProjectFavicon, ProjectMonogram, ProviderIcon, StatusPill,
};

// ── ProviderIcon ────────────────────────────────────────────────────────

const PROVIDERS: [&str; 8] = [
    "Claude",
    "Codex",
    "Cursor",
    "Antigravity",
    "OpenCode",
    "Figma",
    "Gemini",
    "Kiro",
];
const ACCENTS: [&str; 9] = ["none", "1", "2", "3", "4", "5", "6", "7", "8"];

/// `components/ProviderIcon/preview.html`: five lg marks with their names,
/// the second one starting.
fn provider_reference() -> impl IntoElement {
    row().children(
        ["Claude", "Codex", "Cursor", "OpenCode", "Kiro"]
            .into_iter()
            .enumerate()
            .map(|(i, n)| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        ProviderIcon::new(n)
                            .id(("ref-provider", i))
                            .size(IdentitySize::Lg)
                            .starting(i == 1),
                    )
                    .child(label_text(n, TypeStyle::BodyCompact01))
            }),
    )
}

/// Overview specimen.
pub fn provider_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .children(
            ["Claude", "Codex", "Cursor", "Gemini"]
                .into_iter()
                .map(|n| {
                    ProviderIcon::new(n)
                        .id(eid("thumb-provider", n))
                        .size(IdentitySize::Lg)
                }),
        )
        .into_any_element()
}

struct ProviderPage {
    name: usize,
    size: usize,
    starting: bool,
    src: bool,
    accent: usize,
}

/// Build the ProviderIcon page.
pub fn build_provider_icon(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ProviderPage {
        name: 0,
        size: 3,
        starting: false,
        src: false,
        accent: 0,
    })
    .into()
}

impl Render for ProviderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let name = PROVIDERS[self.name];
        let mut icon = ProviderIcon::new(name)
            .id("live-provider")
            .size(IdentitySize::ALL[self.size])
            .starting(self.starting)
            .accent((self.accent > 0).then_some(self.accent as u8));
        if self.src {
            // Any image you may use; here the Gemini glyph served as an SVG asset.
            icon = icon.src("u0ui/t3/provider-Gemini.svg");
        }
        let live = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(icon)
            .child(label_text(name, TypeStyle::BodyCompact01));
        let sizes = IdentitySize::ALL.map(|s| s.name());
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "name",
                controls::choices(
                    "provider-name",
                    &PROVIDERS,
                    self.name,
                    controls::setter(&entity, |this: &mut ProviderPage, i| this.name = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "provider-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut ProviderPage, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "provider-starting",
                "starting",
                self.starting,
                controls::setter(&entity, |this: &mut ProviderPage, on| this.starting = on),
            ))
            .child(controls::toggle(
                "provider-src",
                "src (an image)",
                self.src,
                controls::setter(&entity, |this: &mut ProviderPage, on| this.src = on),
            ))
            .child(controls::field(
                "accent (the tile behind an image)",
                controls::choices(
                    "provider-accent",
                    &ACCENTS,
                    self.accent,
                    controls::setter(&entity, |this: &mut ProviderPage, i| this.accent = i),
                ),
            ));
        let sizes_example = row().children(IdentitySize::ALL.into_iter().map(|s| {
            stack()
                .items_center()
                .gap(px(8.))
                .child(
                    row()
                        .gap(px(8.))
                        .child(
                            ProviderIcon::new("Claude")
                                .id(eid("sz-c", s.name()))
                                .size(s),
                        )
                        .child(
                            ProviderIcon::new("Codex")
                                .id(eid("sz-x", s.name()))
                                .size(s)
                                .starting(true),
                        )
                        .child(ProviderIcon::new("Agent").id(eid("sz-a", s.name())).size(s)),
                )
                .child(label_text(s.name(), TypeStyle::Label01))
        }));
        Body::new()
            .playground(live)
            .hint("Hover a mark for its title")
            .config(config)
            .example(controls::block("Reference preview", provider_reference()))
            .example(controls::block("Sizes", sizes_example))
    }
}

// ── ProjectIdentity ─────────────────────────────────────────────────────

const PROJECTS: [&str; 5] = [
    "carbon-ui",
    "api-gateway",
    "Marketing Site",
    "docs",
    "infra",
];
const FAVICON: [&str; 3] = ["monogram", "icon", "src (fails → monogram)"];

/// `components/ProjectIdentity/preview.html`.
fn identity_reference() -> impl IntoElement {
    stack()
        .gap(px(16.))
        .child(
            row()
                .children(PROJECTS.map(|n| ProjectMonogram::new(n).size(IdentitySize::Lg)))
                .child(
                    ProjectFavicon::new("x")
                        .icon(CarbonIcon::Folder)
                        .size(IdentitySize::Lg),
                ),
        )
        .child(
            row()
                .child(
                    EnvironmentBadge::new("MacBook Pro")
                        .kind(MachineKind::Laptop)
                        .status(ConnectionStatus::Connected),
                )
                .child(
                    EnvironmentBadge::new("build-box")
                        .kind(MachineKind::Server)
                        .status(ConnectionStatus::Connecting),
                )
                .child(
                    EnvironmentBadge::new("Studio")
                        .kind(MachineKind::Desktop)
                        .status(ConnectionStatus::Offline),
                )
                .child(
                    EnvironmentBadge::new("cloud-1")
                        .kind(MachineKind::Cloud)
                        .status(ConnectionStatus::Error),
                ),
        )
}

/// Overview specimen.
pub fn identity_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    stack()
        .gap(px(12.))
        .child(
            row().children(
                ["carbon-ui", "api-gateway", "docs"]
                    .map(|n| ProjectMonogram::new(n).size(IdentitySize::Lg)),
            ),
        )
        .child(
            EnvironmentBadge::new("build-box")
                .id("thumb-env")
                .kind(MachineKind::Server),
        )
        .into_any_element()
}

struct IdentityPage {
    project: usize,
    size: usize,
    favicon: usize,
    machine: usize,
    status: usize,
    badge_size: usize,
}

/// Build the ProjectIdentity page.
pub fn build_project_identity(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| IdentityPage {
        project: 0,
        size: 3,
        favicon: 0,
        machine: 0,
        status: 0,
        badge_size: 1,
    })
    .into()
}

impl Render for IdentityPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let name = PROJECTS[self.project];
        let size = IdentitySize::ALL[self.size];
        let favicon = match self.favicon {
            1 => ProjectFavicon::new(name).icon(CarbonIcon::Folder),
            2 => ProjectFavicon::new(name).src("missing/favicon.png"),
            _ => ProjectFavicon::new(name),
        }
        .size(size);
        let kind = MachineKind::ALL[self.machine];
        let status = ConnectionStatus::ALL[self.status];
        let live = stack()
            .gap(px(16.))
            .child(
                row()
                    .child(favicon)
                    .child(label_text(name, TypeStyle::BodyCompact01)),
            )
            .child(
                row().child(EnvironmentMachineIcon::new(kind)).child(
                    EnvironmentBadge::new("build-box")
                        .id("live-env")
                        .kind(kind)
                        .status(status)
                        .size(BadgeSize::ALL[self.badge_size]),
                ),
            );
        let sizes = IdentitySize::ALL.map(|s| s.name());
        let kinds = MachineKind::ALL.map(|k| k.name());
        let statuses = ConnectionStatus::ALL.map(|s| s.name());
        let badge_sizes = BadgeSize::ALL.map(|s| s.name());
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::section("ProjectMonogram / ProjectFavicon"))
            .child(controls::field(
                "name",
                controls::choices(
                    "identity-name",
                    &PROJECTS,
                    self.project,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.project = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "identity-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "favicon",
                controls::choices(
                    "identity-favicon",
                    &FAVICON,
                    self.favicon,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.favicon = i),
                ),
            ))
            .child(controls::section("EnvironmentBadge"))
            .child(controls::field(
                "kind",
                controls::choices(
                    "identity-kind",
                    &kinds,
                    self.machine,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.machine = i),
                ),
            ))
            .child(controls::field(
                "status",
                controls::choices(
                    "identity-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.status = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "identity-badge-size",
                    &badge_sizes,
                    self.badge_size,
                    controls::setter(&entity, |this: &mut IdentityPage, i| this.badge_size = i),
                ),
            ));
        let machines = row().children(MachineKind::ALL.map(|k| {
            row()
                .gap(px(6.))
                .child(EnvironmentMachineIcon::new(k))
                .child(label_text(k.name(), TypeStyle::Label01))
        }));
        let monogram_sizes = row().children(IdentitySize::ALL.map(|s| {
            stack()
                .items_center()
                .gap(px(8.))
                .child(ProjectMonogram::new("carbon-ui").size(s))
                .child(label_text(s.name(), TypeStyle::Label01))
        }));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", identity_reference()))
            .example(controls::block("Monogram sizes", monogram_sizes))
            .example(controls::block("EnvironmentMachineIcon kinds", machines))
    }
}

// ── StatusIndicators ────────────────────────────────────────────────────

/// `components/StatusIndicators/preview.html`.
fn status_reference() -> impl IntoElement {
    stack()
        .gap(px(16.))
        .child(
            row().children(
                [
                    ConnectionStatus::Connected,
                    ConnectionStatus::Connecting,
                    ConnectionStatus::Offline,
                    ConnectionStatus::Error,
                ]
                .map(|s| {
                    // `display: inline-flex; gap: 6px; font-size: 12px`.
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(ConnectionStatusDot::new(s).id(eid("ref-dot", s.name())))
                        .child(label_text(s.name(), TypeStyle::Label01))
                }),
            ),
        )
        .child(
            row().children(
                [
                    Status::Working,
                    Status::NeedsYou,
                    Status::Done,
                    Status::Failed,
                    Status::Idle,
                    Status::Queued,
                ]
                .map(|s| StatusPill::new(s).id(eid("ref-pill", s.name()))),
            ),
        )
}

/// Overview specimen.
pub fn status_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    stack()
        .gap(px(12.))
        .child(
            row().children(
                [Status::Working, Status::NeedsYou, Status::Done]
                    .map(|s| StatusPill::new(s).id(eid("thumb-pill", s.name()))),
            ),
        )
        .child(
            row().children(
                ConnectionStatus::ALL[..4]
                    .iter()
                    .map(|s| ConnectionStatusDot::new(*s).id(eid("thumb-dot", s.name()))),
            ),
        )
        .into_any_element()
}

struct StatusPage {
    dot: usize,
    pill: usize,
    custom: bool,
}

/// Build the StatusIndicators page.
pub fn build_status_indicators(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StatusPage {
        dot: 1,
        pill: 0,
        custom: false,
    })
    .into()
}

impl Render for StatusPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let dot_status = ConnectionStatus::ALL[self.dot];
        let pill_status = Status::ALL[self.pill];
        let mut pill = StatusPill::new(pill_status).id("live-pill");
        let mut dot = ConnectionStatusDot::new(dot_status).id("live-dot");
        if self.custom {
            pill = pill.label("Running checks");
            dot = dot.label("Reconnecting to build-box");
        }
        let live = row()
            .gap(px(24.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(dot)
                    .child(label_text(dot_status.name(), TypeStyle::Label01)),
            )
            .child(pill);
        let dots = ConnectionStatus::ALL.map(|s| s.name());
        let pills = Status::ALL.map(|s| s.name());
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "ConnectionStatusDot status",
                controls::choices(
                    "status-dot",
                    &dots,
                    self.dot,
                    controls::setter(&entity, |this: &mut StatusPage, i| this.dot = i),
                ),
            ))
            .child(controls::field(
                "StatusPill status",
                controls::choices(
                    "status-pill",
                    &pills,
                    self.pill,
                    controls::setter(&entity, |this: &mut StatusPage, i| this.pill = i),
                ),
            ))
            .child(controls::toggle(
                "status-custom",
                "custom label / children",
                self.custom,
                controls::setter(&entity, |this: &mut StatusPage, on| this.custom = on),
            ));
        let all_pills =
            row().children(Status::ALL.map(|s| StatusPill::new(s).id(eid("all-pill", s.name()))));
        Body::new()
            .playground(live)
            .hint("Hover the dot for its label")
            .config(config)
            .example(controls::block("Reference preview", status_reference()))
            .example(controls::block("Every status", all_pills))
    }
}
