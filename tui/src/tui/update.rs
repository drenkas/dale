//! Reducer: messages update the state and may emit side-effect commands.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::{AgentsChoice, App, Screen, AGENTS_ITEM};
use crate::install::{AgentsMdMode, InstallProgress, InstallReport, StepState};
use crate::payload::Payload;

/// Messages fed into the reducer: input events plus effect results.
pub enum Msg {
    Key(KeyEvent),
    Tick,
    Scanned(Box<crate::install::Scan>),
    /// Remote payload result plus, when checked, a newer dale binary version.
    RemoteFetched(Box<(Result<Payload, String>, Option<String>)>),
    Progress(InstallProgress),
    InstallDone(Box<Result<InstallReport, String>>),
}

/// Side effects requested by the reducer, executed off the render path.
pub enum Cmd {
    /// Scan the environment (skills dir, plugin cache, manifest, AGENTS.md).
    Scan {
        home: PathBuf,
    },
    Fetch(String),
    Install {
        home: PathBuf,
        payload: Payload,
        skills: Vec<String>,
        remove: Vec<String>,
        agents: AgentsMdMode,
    },
}

pub fn update(app: &mut App, msg: Msg) -> Option<Cmd> {
    match msg {
        Msg::Tick => {
            if app.fetching || matches!(app.screen, Screen::Installing | Screen::Splash) {
                app.spinner = app.spinner.wrapping_add(1);
            }
            app.maybe_leave_splash();
            None
        }
        Msg::Scanned(scan) => {
            app.apply_scan(*scan);
            app.scan_done = true;
            app.rebuild_items(false);
            app.maybe_leave_splash();
            None
        }
        Msg::RemoteFetched(result) => {
            app.fetching = false;
            let (payload, newer_binary) = *result;
            if newer_binary.is_some() {
                app.newer_binary = newer_binary;
            }
            match payload {
                Ok(payload) => {
                    app.remote_error = None;
                    app.adopt_payload(payload);
                }
                Err(e) => app.remote_error = Some(e),
            }
            None
        }
        Msg::Progress(p) => {
            if let Some(row) = app.progress.iter_mut().find(|(name, _)| *name == p.item) {
                row.1 = p.state;
            }
            None
        }
        Msg::InstallDone(result) => {
            app.report = Some(*result);
            app.reload_disk_state();
            app.rebuild_items(false);
            app.screen = Screen::Done;
            None
        }
        Msg::Key(key) => handle_key(app, key),
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Option<Cmd> {
    app.flash = None;
    let ctrl_c = key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');

    // While an install runs, keys are ignored — but say so, and offer a
    // double-ctrl+c escape hatch instead of silently swallowing everything.
    if app.screen == Screen::Installing {
        if ctrl_c && app.quit_armed {
            app.force_quit = true;
            app.quit = true;
        } else if ctrl_c {
            app.quit_armed = true;
            app.flash = Some("Install running — press ctrl+c again to force quit.".to_string());
        } else {
            app.flash = Some("Install running — please wait (ctrl+c twice to abort).".to_string());
        }
        return None;
    }
    app.quit_armed = false;

    if ctrl_c {
        app.quit = true;
        return None;
    }
    // The too-small fallback promises "press q to quit"; honor it directly,
    // even when the picker filter would otherwise capture the key.
    if app.too_small && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
        app.quit = true;
        return None;
    }
    match app.screen {
        Screen::Splash => {
            // Any key skips the splash as soon as the scan is done.
            app.splash_skip = true;
            app.maybe_leave_splash();
            None
        }
        Screen::Picker => key_picker(app, key),
        Screen::Confirm => key_confirm(app, key),
        Screen::Installing => None, // Handled above.
        Screen::Done => {
            app.quit = true;
            None
        }
    }
}

fn key_picker(app: &mut App, key: KeyEvent) -> Option<Cmd> {
    if app.filter_active {
        match key.code {
            KeyCode::Esc => {
                app.filter_active = false;
                app.filter.set_text("");
                app.rebuild_rows();
            }
            KeyCode::Enter => app.filter_active = false,
            KeyCode::Up => app.move_cursor(-1),
            KeyCode::Down => app.move_cursor(1),
            _ => {
                app.filter.input(key);
                app.rebuild_rows();
            }
        }
        return None;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char('/') => app.filter_active = true,
        KeyCode::Char('u') => {
            if !app.fetching {
                app.fetching = true;
                app.remote_error = None;
                return Some(Cmd::Fetch(app.update_url.clone()));
            }
        }
        KeyCode::Char(' ') => app.toggle_current(),
        KeyCode::Char('a') => app.toggle_select_all(),
        KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1),
        KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1),
        KeyCode::Enter => enter_confirm(app),
        _ => {}
    }
    None
}

fn enter_confirm(app: &mut App) {
    // Something is planned when the desired state differs from disk, when
    // the AGENTS.md item needs an action, or when the AGENTS.md choice must
    // be made (checked while a file exists).
    let agents_actionable = app.agents_dilemma()
        || matches!(
            planned_agents_mode(app),
            AgentsMdMode::Install | AgentsMdMode::RemoveSection
        );
    if app.plan().is_empty() && !agents_actionable {
        app.flash =
            Some("Nothing to do — toggle checkboxes to plan installs or removals.".to_string());
        return;
    }
    if app.agents_dilemma() {
        app.agents_choice = AgentsChoice::Append;
        app.recompute_diff();
    }
    app.screen = Screen::Confirm;
}

fn key_confirm(app: &mut App, key: KeyEvent) -> Option<Cmd> {
    let dilemma = app.agents_dilemma();
    match key.code {
        KeyCode::Enter => return Some(start_install(app)),
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Picker,
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('r') | KeyCode::Char('1') if dilemma => {
            app.agents_choice = AgentsChoice::Replace;
            app.recompute_diff();
        }
        KeyCode::Char('a') | KeyCode::Char('2') if dilemma => {
            app.agents_choice = AgentsChoice::Append;
            app.recompute_diff();
        }
        KeyCode::Char('s') | KeyCode::Char('3') if dilemma => {
            app.agents_choice = AgentsChoice::Skip;
            app.recompute_diff();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.diff_scroll = app
                .diff_scroll
                .saturating_add(1)
                .min(app.diff.len().saturating_sub(1) as u16);
        }
        KeyCode::Up | KeyCode::Char('k') => app.diff_scroll = app.diff_scroll.saturating_sub(1),
        KeyCode::PageDown => {
            app.diff_scroll = app
                .diff_scroll
                .saturating_add(10)
                .min(app.diff.len().saturating_sub(1) as u16);
        }
        KeyCode::PageUp => app.diff_scroll = app.diff_scroll.saturating_sub(10),
        _ => {}
    }
    None
}

/// Effective AGENTS.md mode for the planned run. Checked expresses desired
/// state: unchecking a present AGENTS.md plans removal of the dale-managed
/// section (a no-op handled inside the engine when no markers exist).
pub fn planned_agents_mode(app: &App) -> AgentsMdMode {
    if !app.agents_selected() {
        if app.agents_existing.is_some() && app.agents_has_managed() {
            return AgentsMdMode::RemoveSection;
        }
        return AgentsMdMode::Skip;
    }
    if app.agents_existing.is_none() {
        return AgentsMdMode::Install;
    }
    match app.agents_choice {
        AgentsChoice::Replace => AgentsMdMode::Replace,
        AgentsChoice::Append => AgentsMdMode::Append,
        AgentsChoice::Skip => AgentsMdMode::Skip,
    }
}

fn start_install(app: &mut App) -> Cmd {
    let plan = app.plan();
    let agents = planned_agents_mode(app);
    let skills: Vec<String> = plan.install.iter().chain(&plan.update).cloned().collect();
    app.progress = skills
        .iter()
        .chain(&plan.remove)
        .map(|name| (name.clone(), StepState::Pending))
        .collect();
    if agents != AgentsMdMode::Skip {
        app.progress
            .push((AGENTS_ITEM.to_string(), StepState::Pending));
    }
    app.screen = Screen::Installing;
    Cmd::Install {
        home: app.home.clone(),
        payload: app.payload.clone(),
        skills,
        remove: plan.remove,
        agents,
    }
}
