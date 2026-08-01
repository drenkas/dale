//! Library for the `dale` installer: payload handling, install engine,
//! manifest, AGENTS.md merging, remote updates, headless CLI, and the TUI.

pub mod agents;
pub mod assets;
pub mod cli;
pub mod frontmatter;
pub mod install;
pub mod manifest;
pub mod payload;
pub mod plugin;
pub mod remote;
pub mod tui;
