//! T3 · Files, preview, terminal, device, settings, usage, workflows and
//! onboarding (`src/t3/workspace.jsx`). Split by card group; everything is
//! re-exported here.
//!
//! - [`files`], [`preview`], [`terminal`]: T3 · Files & preview.
//! - [`settings`], [`cards`]: T3 · Settings.
//! - [`usage`], [`workflows`]: T3 · Usage & workflows.
//! - [`onboarding`]: T3 · Onboarding & feedback.
pub mod files;
mod parts;
pub mod preview;
pub mod terminal;

pub use files::{
    FileBreadcrumbs, FileBrowserPanel, FileBrowserTexts, FileIcon, FileNode, FileSurfaceKind,
    FileSurfaceState,
};
pub use preview::{
    DeviceAction, DeviceControlsRail, LocalServer, MiniPlayer, MiniPlayerKind, MiniPlayerTexts,
    PreviewChrome, PreviewChromeTexts, PreviewEmptyState, PreviewEmptyTexts,
};
pub use terminal::{TerminalDrawer, TerminalTab, TerminalTexts};
