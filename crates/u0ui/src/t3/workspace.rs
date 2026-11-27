//! T3 · Files, preview, terminal, device, settings, usage, workflows and
//! onboarding (`src/t3/workspace.jsx`). Split by card group; everything is
//! re-exported here.
//!
//! - [`files`], [`preview`], [`terminal`]: T3 · Files & preview.
//! - [`settings`], [`cards`]: T3 · Settings.
//! - [`usage`], [`workflows`]: T3 · Usage & workflows.
//! - [`onboarding`]: T3 · Onboarding & feedback.
pub mod cards;
pub mod files;
pub mod onboarding;
mod parts;
pub mod preview;
pub mod settings;
pub mod terminal;
pub mod usage;
pub mod workflows;

pub use cards::{
    EnvironmentRow, EnvironmentRowTexts, ProviderCardTexts, ProviderInstanceCard, ProviderStatus,
    StatBlock, StatTone, StatsGrid, Wizard, WizardSteps, WizardTexts,
};
pub use files::{
    FileBreadcrumbs, FileBrowserPanel, FileBrowserTexts, FileIcon, FileNode, FileSurfaceKind,
    FileSurfaceState,
};
pub use onboarding::{
    ComputerSetupSteps, ExpandableToast, ExpandableToastTexts, MiddleTruncate, NoProjectsHero,
    NoProjectsTexts, Permission, PermissionChecklist, PermissionTexts, RelativeTime, SetupState,
    SetupTexts, ToastKind,
};
pub use preview::{
    DeviceAction, DeviceControlsRail, LocalServer, MiniPlayer, MiniPlayerKind, MiniPlayerTexts,
    PreviewChrome, PreviewChromeTexts, PreviewEmptyState, PreviewEmptyTexts,
};
pub use settings::{
    KeybindingPill, KeybindingRow, KeybindingTexts, RichText, SettingsBadgeTone, SettingsNav,
    SettingsNavItem, SettingsNavSection, SettingsRow, SettingsRowTexts, SettingsSection, ThemeCard,
    keybinding_min_width,
};
pub use terminal::{TerminalDrawer, TerminalTab, TerminalTexts};
pub use usage::{Pace, ShareSegment, UsageShareBar, UsageTexts, UsageWindowBar};
pub use workflows::{
    RunState, StepStatus, WorkflowFeedTexts, WorkflowRunFeed, WorkflowRunRow, WorkflowStateWord,
    WorkflowStep, run_row_look,
};
