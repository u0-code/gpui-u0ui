//! The T3 Code pattern library (`src/t3/*.jsx` and `src/t3.css` in the
//! design reference), redrawn with Carbon tokens from the u0ui components.
//!
//! - [`base`]: shared helpers (`base.jsx`): [`base::Spin`], [`base::Status`],
//!   [`base::fmt_elapsed`], [`base::use_ticker`], [`base::use_flash`].
//! - [`shell`]: shell, navigation and identity (`shell.jsx`).
//! - [`composer`]: the composer (`composer.jsx`).
//! - [`review`]: diffs and review threads (`review.jsx`).
mod action;
pub mod base;
pub mod composer;
pub(crate) mod craft;
pub mod mobile;
mod press;
pub mod review;
pub mod shell;

pub use base::{Spin, Status, Tone, fmt_elapsed};
pub use shell::{
    ConnectionStatus, ConnectionStatusDot, EnvironmentBadge, EnvironmentMachineIcon, IconMenu,
    IdentitySize, MachineKind, MonoTone, ProjectFavicon, ProjectMonogram, ProviderIcon, StatusPill,
};
