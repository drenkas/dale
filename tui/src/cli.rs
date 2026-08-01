//! Command-line interface: argument parsing plus the headless subcommands.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

use crate::install::{self, AgentsMdMode, InstallProgress, StepState, RESTART_REMINDER};
use crate::payload::{self, Payload};
use crate::remote::{self, DEFAULT_UPDATE_URL, INSTALL_ONE_LINER};
use crate::{agents, assets, manifest, plugin};

#[derive(Debug, Parser)]
#[command(
    name = "dale",
    version,
    about = "Installer for the Dale skill family for Codex",
    long_about = "Installs Dale skills into $CODEX_HOME/skills and optionally manages the \
                  global AGENTS.md. Run without a subcommand on a terminal to open the TUI."
)]
pub struct Cli {
    /// Codex home directory (overrides $CODEX_HOME; default ~/.codex).
    #[arg(long, global = true, value_name = "PATH")]
    pub codex_home: Option<PathBuf>,

    /// Update source: URL or local path to a repo .tar.gz
    /// (overrides $DALE_UPDATE_URL).
    #[arg(long, global = true, value_name = "URL_OR_PATH")]
    pub update_url: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Install skills from the bundled payload.
    Install {
        /// Install every available skill.
        #[arg(long)]
        all: bool,
        /// Skill names to install (e.g. dale-brainstorm dale-graph).
        names: Vec<String>,
        /// Confirm the installation (required; there is no prompt).
        #[arg(long)]
        yes: bool,
        /// What to do with the global AGENTS.md (default: leave untouched).
        #[arg(long, value_enum, value_name = "MODE")]
        agents_md: Option<AgentsMdArg>,
    },
    /// Fetch the remote payload, update installed skills, and add new ones.
    Update {
        /// Confirm the update (required; there is no prompt).
        #[arg(long)]
        yes: bool,
    },
    /// List available and installed skills.
    List {
        /// Emit machine-readable JSON instead of the human listing.
        #[arg(long)]
        json: bool,
    },
    /// Remove installed skills (removed dirs are backed up first).
    Uninstall {
        /// Remove every installed dale skill.
        #[arg(long)]
        all: bool,
        /// Skill names to remove.
        names: Vec<String>,
        /// Confirm the removal (required; there is no prompt).
        #[arg(long)]
        yes: bool,
        /// Also remove the dale-managed section from AGENTS.md
        /// (backup first; the file itself is never deleted).
        #[arg(long)]
        agents_md: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AgentsMdArg {
    Replace,
    Append,
    Skip,
}

/// Resolved global context: where to install and where updates come from.
#[derive(Debug, Clone)]
pub struct Ctx {
    pub home: PathBuf,
    pub update_url: String,
}

impl Ctx {
    pub fn from_cli(cli: &Cli) -> Self {
        let home = cli
            .codex_home
            .clone()
            .or_else(|| std::env::var_os("CODEX_HOME").map(PathBuf::from))
            .unwrap_or_else(default_codex_home);
        let update_url = cli
            .update_url
            .clone()
            .or_else(|| std::env::var("DALE_UPDATE_URL").ok())
            .unwrap_or_else(|| DEFAULT_UPDATE_URL.to_string());
        Self { home, update_url }
    }
}

fn default_codex_home() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".codex")
}

/// Run a headless subcommand; returns the process exit code.
pub fn run_command(ctx: &Ctx, command: Command) -> u8 {
    match command {
        Command::Install {
            all,
            names,
            yes,
            agents_md,
        } => run_install(ctx, all, &names, yes, agents_md),
        Command::Update { yes } => run_update(ctx, yes),
        Command::List { json } => run_list(ctx, json),
        Command::Uninstall {
            all,
            names,
            yes,
            agents_md,
        } => run_uninstall(ctx, all, &names, yes, agents_md),
    }
}

fn require_yes(yes: bool, action: &str) -> bool {
    if !yes {
        eprintln!("dale: refusing to {action} without --yes");
    }
    yes
}

fn resolve_names(payload: &Payload, all: bool, names: &[String]) -> Result<Vec<String>, String> {
    if all {
        return Ok(payload.skills.iter().map(|s| s.name.clone()).collect());
    }
    if names.is_empty() {
        return Err("nothing selected: pass skill names or --all".to_string());
    }
    let mut resolved = Vec::new();
    for name in names {
        if payload.skill(name).is_none() {
            let available: Vec<&str> = payload.skills.iter().map(|s| s.name.as_str()).collect();
            return Err(format!(
                "unknown skill {name}; available: {}",
                available.join(", ")
            ));
        }
        resolved.push(name.clone());
    }
    Ok(resolved)
}

fn agents_mode(home: &Path, arg: Option<AgentsMdArg>) -> AgentsMdMode {
    match arg {
        None | Some(AgentsMdArg::Skip) => AgentsMdMode::Skip,
        // Append always goes through the managed-section merge so a fresh
        // home gets a bare marked block and re-runs stay idempotent.
        Some(AgentsMdArg::Append) => AgentsMdMode::Append,
        Some(AgentsMdArg::Replace) => {
            if install::agents_md_path(home).is_file() {
                AgentsMdMode::Replace
            } else {
                AgentsMdMode::Install
            }
        }
    }
}

fn print_progress(p: InstallProgress) {
    match p.state {
        StepState::Done(msg) => println!("  {} — {msg}", p.item),
        StepState::Failed(err) => eprintln!("  {} — FAILED: {err}", p.item),
        StepState::Pending | StepState::Started => {}
    }
}

/// Run an install and print the report. Returns (exit code, changed).
fn run_install_with_payload(
    ctx: &Ctx,
    payload: &Payload,
    names: &[String],
    agents: AgentsMdMode,
) -> (u8, bool) {
    println!(
        "Installing {} skill(s) from the {} payload v{} into {}",
        names.len(),
        payload.source,
        payload.version,
        ctx.home.display()
    );
    match install::install(&ctx.home, payload, names, &[], agents, &mut print_progress) {
        Ok(report) => {
            if let Some(dir) = &report.plugin_dir {
                println!("Plugin: new version dir {}", dir.display());
            }
            if let Some(dir) = &report.backup_dir {
                println!("Backups: {}", dir.display());
            }
            if let Some(action) = &report.agents_action {
                println!("AGENTS.md: {action}");
            }
            if report.failures.is_empty() {
                println!("Done: {} skill(s) installed.", report.installed.len());
                (0, report.changed)
            } else {
                eprintln!("Completed with {} failure(s).", report.failures.len());
                (1, report.changed)
            }
        }
        Err(e) => {
            eprintln!("dale install: {e}");
            (1, false)
        }
    }
}

fn run_install(
    ctx: &Ctx,
    all: bool,
    names: &[String],
    yes: bool,
    agents_md: Option<AgentsMdArg>,
) -> u8 {
    let payload = assets::bundled_payload();
    let names = match resolve_names(&payload, all, names) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("dale install: {e}");
            return 2;
        }
    };
    if !require_yes(yes, "install") {
        return 2;
    }
    let (code, changed) =
        run_install_with_payload(ctx, &payload, &names, agents_mode(&ctx.home, agents_md));
    if changed {
        println!("{RESTART_REMINDER}");
    }
    code
}

fn run_update(ctx: &Ctx, yes: bool) -> u8 {
    if !require_yes(yes, "update") {
        return 2;
    }
    let m = manifest::load(&ctx.home);
    // Plugin-installed skills count as installed too: `update --yes` must
    // update a plugin install when one exists.
    let plugins = plugin::detect(&ctx.home);
    let installed = install::all_installed_skills(&ctx.home, &m, &plugins);
    let payload = match remote::fetch_payload(&ctx.update_url) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dale update: {e}");
            return 1;
        }
    };
    if installed.is_empty() {
        println!("No installed dale skills; nothing to update.");
        return 0;
    }
    // Partition the installed skills against the remote payload…
    let mut to_update: Vec<String> = Vec::new();
    let mut up_to_date = 0usize;
    for (name, info) in &installed {
        if payload.skill(name).is_none() {
            println!("  {name} — skipped (not in the remote payload)");
        } else if payload::version_newer(&payload.version, &info.version) {
            to_update.push(name.clone());
        } else {
            up_to_date += 1;
        }
    }
    // …and pick up skills that are new in the remote payload.
    let added: Vec<String> = payload
        .skills
        .iter()
        .filter(|s| !installed.contains_key(&s.name))
        .map(|s| s.name.clone())
        .collect();

    let names: Vec<String> = to_update.iter().chain(&added).cloned().collect();
    if !names.is_empty() {
        let (code, _) = run_install_with_payload(ctx, &payload, &names, AgentsMdMode::Skip);
        if code != 0 {
            return code;
        }
        for name in &added {
            println!("  {name} — added (new in the remote payload)");
        }
    }

    // Best-effort self-version check, only against the default source.
    if ctx.update_url == DEFAULT_UPDATE_URL {
        if let Some(version) = remote::newer_binary_version() {
            println!("A newer dale binary is available (v{version}): rerun  {INSTALL_ONE_LINER}");
        }
    }

    println!(
        "Summary: updated {}, added {}, up-to-date {}.",
        to_update.len(),
        added.len(),
        up_to_date
    );
    if !to_update.is_empty() || !added.is_empty() {
        println!("{RESTART_REMINDER}");
    }
    0
}

/// JSON entry for one skill in `dale list --json`.
fn skill_json(
    name: &str,
    info: Option<&install::InstalledSkill>,
    available: Option<&str>,
) -> serde_json::Value {
    let installed_version = info.map(|i| i.version.clone());
    let update_available = match (&installed_version, available) {
        (Some(cur), Some(avail)) => payload::version_newer(avail, cur),
        _ => false,
    };
    serde_json::json!({
        "name": name,
        "installed": info.is_some(),
        "source": info.map(|i| if i.marketplace.is_some() { "plugin" } else { "loose" }),
        "installedVersion": installed_version,
        "availableVersion": available,
        "updateAvailable": update_available,
    })
}

fn run_list(ctx: &Ctx, json: bool) -> u8 {
    let payload = assets::bundled_payload();
    let m = manifest::load(&ctx.home);
    let plugins = plugin::detect(&ctx.home);
    let installed = install::all_installed_skills(&ctx.home, &m, &plugins);
    if json {
        let mut skills: Vec<serde_json::Value> = payload
            .skills
            .iter()
            .map(|s| skill_json(&s.name, installed.get(&s.name), Some(&payload.version)))
            .collect();
        for (name, info) in &installed {
            if payload.skill(name).is_none() {
                skills.push(skill_json(name, Some(info), None));
            }
        }
        let agents_path = install::agents_md_path(&ctx.home);
        let agents_text = std::fs::read_to_string(&agents_path).ok();
        let doc = serde_json::json!({
            "codexHome": ctx.home.display().to_string(),
            "bundledVersion": payload.version,
            "pluginInstall": plugin::effective(&plugins).map(|pi| serde_json::json!({
                "marketplace": pi.marketplace,
                "version": pi.version,
                "path": pi.path.display().to_string(),
            })),
            "skills": skills,
            "agentsMd": {
                "present": agents_path.is_file(),
                "daleManaged": agents_text.as_deref().map(agents::has_managed).unwrap_or(false),
            },
        });
        match serde_json::to_string_pretty(&doc) {
            Ok(text) => {
                println!("{text}");
                return 0;
            }
            Err(e) => {
                eprintln!("dale list: cannot serialize: {e}");
                return 1;
            }
        }
    }
    println!("CODEX_HOME: {}", ctx.home.display());
    println!("Bundled payload: v{}", payload.version);
    if let Some(pi) = plugin::effective(&plugins) {
        println!(
            "Plugin install: dale@{} v{} ({})",
            pi.marketplace,
            pi.version,
            pi.path.display()
        );
    }
    println!();
    let status_of = |info: &install::InstalledSkill| {
        let tag = if info.marketplace.is_some() {
            " · plugin"
        } else {
            ""
        };
        if payload::version_newer(&payload.version, &info.version) {
            format!(
                "installed v{}{tag} — update available (v{})",
                info.version, payload.version
            )
        } else if info.version == payload.version {
            format!("installed v{}{tag}", info.version)
        } else {
            format!(
                "installed v{}{tag} (bundled: v{})",
                info.version, payload.version
            )
        }
    };
    for skill in &payload.skills {
        let status = match installed.get(&skill.name) {
            Some(info) => status_of(info),
            None => "available".to_string(),
        };
        println!("  {:<20} {status}", skill.name);
    }
    for (name, info) in &installed {
        if payload.skill(name).is_none() {
            let tag = if info.marketplace.is_some() {
                " · plugin"
            } else {
                ""
            };
            println!(
                "  {name:<20} installed v{}{tag} (not in bundled payload)",
                info.version
            );
        }
    }
    println!();
    let agents = install::agents_md_path(&ctx.home);
    if agents.is_file() {
        println!("  {:<20} present at {}", "AGENTS.md", agents.display());
    } else {
        println!("  {:<20} not present", "AGENTS.md");
    }
    0
}

fn run_uninstall(ctx: &Ctx, all: bool, names: &[String], yes: bool, agents_md: bool) -> u8 {
    let m = manifest::load(&ctx.home);
    let installed = install::installed_skills(&ctx.home, &m);
    // Skills shipped by the effective plugin install are removable too: a
    // new plugin version dir is written without them.
    let plugins = plugin::detect(&ctx.home);
    let plugin_skills: Vec<String> = plugin::effective(&plugins)
        .map(|pi| pi.skills.clone())
        .unwrap_or_default();
    // A name is known when its skill dir exists, a plugin ships it, or the
    // manifest lists it (the latter lets `uninstall` clean up manually
    // deleted skills).
    let known = |name: &String| {
        installed.contains_key(name) || plugin_skills.contains(name) || m.skills.contains_key(name)
    };
    let names: Vec<String> = if all {
        let mut all_names: Vec<String> = installed.keys().cloned().collect();
        for name in m.skills.keys().chain(&plugin_skills) {
            if !all_names.contains(name) && crate::payload::is_valid_skill_name(name) {
                all_names.push(name.clone());
            }
        }
        all_names
    } else if names.is_empty() {
        if !agents_md {
            eprintln!("dale uninstall: nothing selected: pass skill names, --all, or --agents-md");
            return 2;
        }
        Vec::new()
    } else {
        let unknown: Vec<&str> = names
            .iter()
            .filter(|n| !known(n))
            .map(String::as_str)
            .collect();
        if !unknown.is_empty() {
            eprintln!("dale uninstall: not installed: {}", unknown.join(", "));
            let mut listed: Vec<&str> = installed.keys().map(String::as_str).collect();
            for name in &plugin_skills {
                if !listed.contains(&name.as_str()) {
                    listed.push(name);
                }
            }
            if listed.is_empty() {
                eprintln!("  no dale skills are installed");
            } else {
                eprintln!("  installed: {}", listed.join(", "));
            }
            return 2;
        }
        names.to_vec()
    };
    if names.is_empty() && !agents_md {
        println!("No installed dale skills; nothing to remove.");
        return 0;
    }
    if !require_yes(yes, "uninstall") {
        return 2;
    }
    match install::uninstall(&ctx.home, &names, agents_md) {
        Ok(report) => {
            for name in &report.removed {
                println!("  {name} — removed");
            }
            if let Some(dir) = &report.plugin_dir {
                println!("Plugin: new version dir {}", dir.display());
            }
            if let Some(action) = &report.agents_action {
                println!("AGENTS.md: {action}");
            }
            if let Some(dir) = &report.backup_dir {
                println!("Backups: {}", dir.display());
            }
            for (item, err) in &report.failures {
                eprintln!("  {item} — FAILED: {err}");
            }
            if !report.failures.is_empty() {
                eprintln!("Completed with {} failure(s).", report.failures.len());
                return 1;
            }
            if !names.is_empty() {
                println!("Done: {} skill(s) removed.", report.removed.len());
            }
            if report.changed {
                println!("{RESTART_REMINDER}");
            }
            0
        }
        Err(e) => {
            eprintln!("dale uninstall: {e}");
            1
        }
    }
}
