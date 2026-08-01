//! Elm-style application state for the installer TUI.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use xai_ratatui_textarea::TextArea;

use crate::cli::Ctx;
use crate::install::{self, InstallReport, InstalledSkill, StepState};
use crate::manifest::Manifest;
use crate::payload::{self, Payload};
use crate::plugin::PluginInstall;
use crate::{agents, assets};

/// Name of the AGENTS.md config item in the picker.
pub const AGENTS_ITEM: &str = "AGENTS.md";

/// Minimum time the splash screen stays visible.
pub const SPLASH_MIN: Duration = Duration::from_millis(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Splash,
    Picker,
    Confirm,
    Installing,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    New,
    Installed {
        version: String,
        plugin: bool,
    },
    Update {
        from: String,
        to: String,
        plugin: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub group: &'static str,
    pub description: String,
    pub status: ItemStatus,
    pub selected: bool,
    pub is_agents: bool,
}

/// One visible row of the picker list.
#[derive(Debug, Clone, Copy)]
pub enum Row {
    Header(&'static str),
    /// Index into [`App::items`].
    Item(usize),
}

/// Planned actions derived from the desired-state checkboxes: checked means
/// "should be installed", unchecked means "should not be".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanSet {
    /// Checked but not installed.
    pub install: Vec<String>,
    /// Checked, installed, and an update is available.
    pub update: Vec<String>,
    /// Installed but unchecked.
    pub remove: Vec<String>,
}

impl PlanSet {
    pub fn is_empty(&self) -> bool {
        self.install.is_empty() && self.update.is_empty() && self.remove.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsChoice {
    Replace,
    Append,
    Skip,
}

pub struct App {
    pub screen: Screen,
    pub home: PathBuf,
    pub update_url: String,

    /// Payload the next install will use (embedded until a remote fetch).
    pub payload: Payload,
    pub bundled_version: String,
    pub remote_version: Option<String>,
    pub remote_error: Option<String>,
    /// A newer dale binary was found on GitHub (best-effort self-check).
    pub newer_binary: Option<String>,
    pub fetching: bool,
    pub spinner: usize,

    pub manifest: Manifest,
    /// Detected dale plugin installs (highest version per marketplace).
    pub plugin_installs: Vec<PluginInstall>,
    /// Merged installed-skill map (loose + plugin), from the last scan.
    pub installed: BTreeMap<String, InstalledSkill>,
    /// Current content of the user's AGENTS.md, when present.
    pub agents_existing: Option<String>,

    // Splash screen.
    pub splash_started: Instant,
    /// The environment scan effect has completed.
    pub scan_done: bool,
    /// A key was pressed on the splash; skip as soon as the scan is done.
    pub splash_skip: bool,

    pub items: Vec<Item>,
    pub rows: Vec<Row>,
    /// Cursor position as an index into `rows`.
    pub cursor: usize,

    pub filter: TextArea,
    pub filter_active: bool,
    pub flash: Option<String>,
    /// Last frame was the too-small-terminal fallback; `q`/`esc` must quit
    /// directly then, regardless of filter or screen state.
    pub too_small: bool,

    // Confirm screen.
    pub agents_choice: AgentsChoice,
    pub diff: Vec<String>,
    pub diff_scroll: u16,

    // Installing / Done screens.
    pub progress: Vec<(String, StepState)>,
    pub report: Option<Result<InstallReport, String>>,

    pub quit: bool,
    /// First ctrl+c during an install arms this; the second one force-quits.
    pub quit_armed: bool,
    /// Explicitly abandon a running install (double ctrl+c).
    pub force_quit: bool,
}

impl App {
    pub fn new(ctx: Ctx) -> Self {
        let payload = assets::bundled_payload();
        let bundled_version = payload.version.clone();
        let mut app = Self {
            screen: Screen::Splash,
            home: ctx.home,
            update_url: ctx.update_url,
            payload,
            bundled_version,
            remote_version: None,
            remote_error: None,
            newer_binary: None,
            fetching: false,
            spinner: 0,
            manifest: Manifest::default(),
            plugin_installs: Vec::new(),
            installed: BTreeMap::new(),
            agents_existing: None,
            splash_started: Instant::now(),
            scan_done: false,
            splash_skip: false,
            items: Vec::new(),
            rows: Vec::new(),
            cursor: 0,
            filter: TextArea::new(),
            filter_active: false,
            flash: None,
            too_small: false,
            agents_choice: AgentsChoice::Append,
            diff: Vec::new(),
            diff_scroll: 0,
            progress: Vec::new(),
            report: None,
            quit: false,
            quit_armed: false,
            force_quit: false,
        };
        // Disk state arrives via the splash-screen scan effect; until then
        // the items carry payload data only.
        app.rebuild_items(false);
        app
    }

    /// Adopt the result of an environment scan (splash effect or re-scan
    /// after an install).
    pub fn apply_scan(&mut self, scan: install::Scan) {
        self.manifest = scan.manifest;
        self.plugin_installs = scan.plugins;
        self.installed = scan.installed;
        self.agents_existing = scan.agents_existing;
    }

    /// Synchronously re-scan the environment (used after an install ran).
    pub fn reload_disk_state(&mut self) {
        self.apply_scan(install::scan(&self.home));
    }

    /// Leave the splash for the picker once the scan finished and either the
    /// minimum display time elapsed or the user pressed a key.
    pub fn maybe_leave_splash(&mut self) {
        if self.screen == Screen::Splash
            && self.scan_done
            && (self.splash_skip || self.splash_started.elapsed() >= SPLASH_MIN)
        {
            self.screen = Screen::Picker;
        }
    }

    /// Adopt a freshly fetched remote payload and refresh statuses.
    pub fn adopt_payload(&mut self, payload: Payload) {
        self.remote_version = Some(payload.version.clone());
        self.payload = payload;
        self.rebuild_items(true);
    }

    /// Rebuild picker items from the active payload and disk state.
    ///
    /// Checkboxes express desired state: installed skills (and a present
    /// AGENTS.md) start checked, everything else unchecked. With `preserve`
    /// the user's current checkbox choices are kept on top of that.
    pub fn rebuild_items(&mut self, preserve: bool) {
        let kept: HashMap<String, bool> = if preserve {
            self.items
                .iter()
                .map(|i| (i.name.clone(), i.selected))
                .collect()
        } else {
            HashMap::new()
        };
        let mut items = Vec::new();
        for skill in &self.payload.skills {
            let status = match self.installed.get(&skill.name) {
                None => ItemStatus::New,
                Some(info) => {
                    let plugin = info.marketplace.is_some();
                    if payload::version_newer(&self.payload.version, &info.version) {
                        ItemStatus::Update {
                            from: info.version.clone(),
                            to: self.payload.version.clone(),
                            plugin,
                        }
                    } else {
                        ItemStatus::Installed {
                            version: info.version.clone(),
                            plugin,
                        }
                    }
                }
            };
            let selected = kept
                .get(&skill.name)
                .copied()
                .unwrap_or(!matches!(status, ItemStatus::New));
            items.push(Item {
                name: skill.name.clone(),
                group: payload::group_of(&skill.name),
                description: skill.description.clone(),
                status,
                selected,
                is_agents: false,
            });
        }
        items.push(Item {
            name: AGENTS_ITEM.to_string(),
            group: payload::GROUP_CONFIG,
            description: String::new(), // Rendered specially in the view.
            status: if self.agents_existing.is_some() {
                ItemStatus::Installed {
                    version: "present".to_string(),
                    plugin: false,
                }
            } else {
                ItemStatus::New
            },
            selected: kept
                .get(AGENTS_ITEM)
                .copied()
                .unwrap_or(self.agents_existing.is_some()),
            is_agents: true,
        });

        self.items = items;
        self.rebuild_rows();
    }

    /// Rebuild the visible rows applying the current filter.
    pub fn rebuild_rows(&mut self) {
        let current = self.current_item_index();
        let needle = self.filter.text().trim().to_lowercase();
        let groups: Vec<&'static str> = payload::GROUPS
            .iter()
            .map(|(g, _)| *g)
            .chain([payload::GROUP_OTHER, payload::GROUP_CONFIG])
            .collect();

        let mut rows = Vec::new();
        for group in groups {
            let members: Vec<usize> = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.group == group
                        && (needle.is_empty() || item.name.to_lowercase().contains(&needle))
                })
                .map(|(i, _)| i)
                .collect();
            if members.is_empty() {
                continue;
            }
            rows.push(Row::Header(group));
            rows.extend(members.into_iter().map(Row::Item));
        }
        self.rows = rows;

        // Keep the cursor on the same item when possible.
        self.cursor = current
            .and_then(|idx| {
                self.rows
                    .iter()
                    .position(|r| matches!(r, Row::Item(i) if *i == idx))
            })
            .or_else(|| self.first_item_row())
            .unwrap_or(0);
    }

    fn first_item_row(&self) -> Option<usize> {
        self.rows.iter().position(|r| matches!(r, Row::Item(_)))
    }

    pub fn current_item_index(&self) -> Option<usize> {
        match self.rows.get(self.cursor) {
            Some(Row::Item(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn current_item(&self) -> Option<&Item> {
        self.current_item_index().map(|i| &self.items[i])
    }

    /// Move the cursor over item rows, skipping group headers.
    pub fn move_cursor(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let mut pos = self.cursor as i32;
        loop {
            pos += delta;
            if pos < 0 || pos as usize >= self.rows.len() {
                return; // Stay put at the edges.
            }
            if matches!(self.rows[pos as usize], Row::Item(_)) {
                self.cursor = pos as usize;
                return;
            }
        }
    }

    pub fn toggle_current(&mut self) {
        if let Some(i) = self.current_item_index() {
            self.items[i].selected = !self.items[i].selected;
        }
    }

    /// Select all visible skills, or clear all if every one is selected.
    /// The AGENTS.md config item is left for explicit toggling.
    pub fn toggle_select_all(&mut self) {
        let visible: Vec<usize> = self
            .rows
            .iter()
            .filter_map(|r| match r {
                Row::Item(i) if !self.items[*i].is_agents => Some(*i),
                _ => None,
            })
            .collect();
        let all_selected = !visible.is_empty() && visible.iter().all(|i| self.items[*i].selected);
        for i in visible {
            self.items[i].selected = !all_selected;
        }
    }

    /// Derive the planned actions from the desired-state checkboxes.
    pub fn plan(&self) -> PlanSet {
        let mut plan = PlanSet::default();
        for item in &self.items {
            if item.is_agents {
                continue;
            }
            match (&item.status, item.selected) {
                (ItemStatus::New, true) => plan.install.push(item.name.clone()),
                (ItemStatus::Update { .. }, true) => plan.update.push(item.name.clone()),
                (ItemStatus::Installed { .. } | ItemStatus::Update { .. }, false) => {
                    plan.remove.push(item.name.clone())
                }
                _ => {}
            }
        }
        plan
    }

    /// The user's AGENTS.md carries a complete dale marker pair.
    pub fn agents_has_managed(&self) -> bool {
        self.agents_existing
            .as_deref()
            .is_some_and(agents::has_managed)
    }

    pub fn agents_selected(&self) -> bool {
        self.items.iter().any(|i| i.is_agents && i.selected)
    }

    /// Whether the confirm screen must offer the replace/append/skip choice.
    pub fn agents_dilemma(&self) -> bool {
        self.agents_selected() && self.agents_existing.is_some()
    }

    pub fn installed_count(&self) -> usize {
        self.installed.len()
    }

    /// Recompute the AGENTS.md diff preview for the current choice.
    pub fn recompute_diff(&mut self) {
        self.diff_scroll = 0;
        let Some(existing) = self.agents_existing.clone() else {
            self.diff = Vec::new();
            return;
        };
        let incoming = match self.agents_choice {
            AgentsChoice::Replace => self.payload.agents_md.clone(),
            AgentsChoice::Append => agents::merge_managed(&existing, &self.payload.agents_md),
            AgentsChoice::Skip => {
                self.diff = vec!["AGENTS.md will be left untouched.".to_string()];
                return;
            }
        };
        let diff = similar::TextDiff::from_lines(existing.as_str(), incoming.as_str());
        let unified = diff
            .unified_diff()
            .context_radius(2)
            .header("current AGENTS.md", "after install")
            .to_string();
        self.diff = unified.lines().map(str::to_string).collect();
        if self.diff.is_empty() {
            self.diff = vec!["No changes.".to_string()];
        }
    }
}
