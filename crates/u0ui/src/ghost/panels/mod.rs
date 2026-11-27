//! `Ghost · Right column` (`src/ghost/panels.jsx`): the right column's
//! viewers (code, image, PDF), the terminal and git history. One submodule
//! per card; everything is re-exported here, so callers write
//! `u0ui::ghost::panels::CodeViewer`.
pub mod code_viewer;
pub mod git;
pub mod head;
pub mod image_viewer;
pub mod pdf_viewer;
pub mod terminal;

pub use code_viewer::{CodeViewer, CodeViewerTexts, SyntaxKind, highlight};
pub use git::{
    ChangedFile, CommitComposer, CommitComposerTexts, CommitGraph, CommitRow, LinkedItem,
    LinkedItemsSection, LinkedItemsTexts, LinkedKind, MessageSetter, RefKind, StashRow,
    StashRowTexts,
};
pub use image_viewer::{ImageViewer, ImageViewerTexts, StageBackground, Zoom};
pub use pdf_viewer::{PdfViewer, PdfViewerTexts};
pub use terminal::{
    AnsiColor, AnsiRun, TerminalTab, TerminalTabs, TerminalTabsTexts, TerminalTexts, TerminalView,
    parse_ansi,
};
