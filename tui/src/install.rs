//! Install engine shared by the TUI and the headless CLI.
//!
//! Installs are atomic per skill (staged into a temp directory on the same
//! filesystem, then renamed into place). Anything overwritten is first moved
//! or copied into `$CODEX_HOME/backups/dale-<timestamp>/`.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::agents;
use crate::manifest::{self, Manifest};
use crate::payload::{self, Payload};
use crate::plugin::{self, PluginInstall};

/// What to do with the user's global AGENTS.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsMdMode {
    /// Leave AGENTS.md alone.
    Skip,
    /// No existing file: write the bundled one.
    Install,
    /// Back up the existing file and replace it.
    Replace,
    /// Insert or update the managed `<!-- dale:begin/end -->` section.
    Append,
    /// Back up the file, then remove only the managed section. A no-op when
    /// the file or the marker pair is absent; the file is never deleted.
    RemoveSection,
}

/// Reminder printed by every entry point after a run that changed anything.
pub const RESTART_REMINDER: &str =
    "⟳ Restart Codex (app or CLI session) so it reloads skills and config.";

/// Per-item progress state, reported while an install runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepState {
    Pending,
    Started,
    Done(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub item: String,
    pub state: StepState,
}

/// Outcome of an install run.
#[derive(Debug, Clone, Default)]
pub struct InstallReport {
    /// (skill name, installed version)
    pub installed: Vec<(String, String)>,
    /// Skill names that were removed (loose dirs or plugin skills).
    pub removed: Vec<String>,
    /// Human-readable description of the AGENTS.md action, if any.
    pub agents_action: Option<String>,
    /// Backup directory, created lazily when something was overwritten.
    pub backup_dir: Option<PathBuf>,
    /// (item, error) pairs for items that failed.
    pub failures: Vec<(String, String)>,
    /// New plugin version directory, when the run updated a plugin install.
    pub plugin_dir: Option<PathBuf>,
    /// True when the run actually changed something on disk (a no-op run —
    /// e.g. "already present" or "nothing to remove" — stays false).
    pub changed: bool,
}

pub fn skills_dir(home: &Path) -> PathBuf {
    home.join("skills")
}

pub fn agents_md_path(home: &Path) -> PathBuf {
    home.join("AGENTS.md")
}

/// Lazily-created backup directory for one install run.
struct Backup {
    root: PathBuf,
    dir: Option<PathBuf>,
}

impl Backup {
    fn new(home: &Path) -> Self {
        Self {
            root: home.join("backups"),
            dir: None,
        }
    }

    fn ensure(&mut self) -> io::Result<PathBuf> {
        if let Some(dir) = &self.dir {
            return Ok(dir.clone());
        }
        fs::create_dir_all(&self.root)?;
        let ts = manifest::compact_timestamp();
        let mut candidate = self.root.join(format!("dale-{ts}"));
        let mut n = 1;
        while candidate.exists() {
            candidate = self.root.join(format!("dale-{ts}-{n}"));
            n += 1;
        }
        fs::create_dir_all(&candidate)?;
        self.dir = Some(candidate.clone());
        Ok(candidate)
    }
}

/// Install `skills` and remove `remove` (both by name) from/against
/// `payload` in `home`, then apply the AGENTS.md action and update the
/// manifest. Per-item failures are recorded in the report; only
/// environment-level failures abort the run.
pub fn install(
    home: &Path,
    payload: &Payload,
    skills: &[String],
    remove: &[String],
    agents: AgentsMdMode,
    on_progress: &mut dyn FnMut(InstallProgress),
) -> Result<InstallReport, String> {
    // Only valid dale skill names may be removed: a single path component
    // with the `dale-` prefix. Anything else (user directories, `..`
    // traversal) is refused before any deletion happens.
    for name in remove {
        if !payload::is_valid_skill_name(name) {
            return Err(format!("refusing to remove {name}: not a dale skill name"));
        }
    }
    let mut backup = Backup::new(home);
    let mut report = InstallReport::default();

    // When a dale plugin install exists, codex already loads the skills from
    // the plugin cache: writing loose copies into skills/ would duplicate
    // them. Instead a new version dir is written next to the old one.
    // Removals are split the same way: skills shipped by the plugin go into
    // the new version dir (minus them); loose dirs are backed up and moved.
    let plugin_target = plugin::effective(&plugin::detect(home)).cloned();
    let sdir = skills_dir(home);
    let plugin_removals: Vec<String> = match &plugin_target {
        Some(target) => remove
            .iter()
            .filter(|name| target.skills.contains(name))
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    let loose_removals: Vec<String> = remove
        .iter()
        .filter(|name| sdir.join(name.as_str()).is_dir())
        .cloned()
        .collect();

    if let Some(target) = &plugin_target {
        if !skills.is_empty() || !plugin_removals.is_empty() {
            install_into_plugin(
                target,
                payload,
                skills,
                &plugin_removals,
                &mut report,
                on_progress,
            );
        }
    } else if !skills.is_empty() {
        fs::create_dir_all(&sdir).map_err(|e| format!("cannot create {}: {e}", sdir.display()))?;
        for name in skills {
            on_progress(InstallProgress {
                item: name.clone(),
                state: StepState::Started,
            });
            match install_one(&sdir, payload, name, &mut backup) {
                Ok(version) => {
                    on_progress(InstallProgress {
                        item: name.clone(),
                        state: StepState::Done(format!("v{version}")),
                    });
                    report.installed.push((name.clone(), version));
                    report.changed = true;
                }
                Err(e) => {
                    let msg = e.to_string();
                    on_progress(InstallProgress {
                        item: name.clone(),
                        state: StepState::Failed(msg.clone()),
                    });
                    report.failures.push((name.clone(), msg));
                }
            }
        }
    }

    // Loose removals: moving the dir into the backup dir is the backup and
    // the delete in one atomic step.
    for name in &loose_removals {
        on_progress(InstallProgress {
            item: name.clone(),
            state: StepState::Started,
        });
        match remove_loose(&sdir, name, &mut backup) {
            Ok(()) => {
                on_progress(InstallProgress {
                    item: name.clone(),
                    state: StepState::Done("removed (backup kept)".to_string()),
                });
                if !report.removed.contains(name) {
                    report.removed.push(name.clone());
                }
                report.changed = true;
            }
            Err(e) => {
                let msg = e.to_string();
                on_progress(InstallProgress {
                    item: name.clone(),
                    state: StepState::Failed(msg.clone()),
                });
                report.failures.push((name.clone(), msg));
            }
        }
    }

    if agents != AgentsMdMode::Skip {
        let item = "AGENTS.md".to_string();
        on_progress(InstallProgress {
            item: item.clone(),
            state: StepState::Started,
        });
        match apply_agents_md(home, payload, agents, &mut backup) {
            Ok((action, changed)) => {
                on_progress(InstallProgress {
                    item: item.clone(),
                    state: StepState::Done(action.clone()),
                });
                report.agents_action = Some(action);
                report.changed |= changed;
            }
            Err(e) => {
                let msg = e.to_string();
                on_progress(InstallProgress {
                    item: item.clone(),
                    state: StepState::Failed(msg.clone()),
                });
                report.failures.push((item, msg));
            }
        }
    }

    // Only a run that actually landed something may claim the payload's
    // version/source in the manifest. Plugin updates live in the plugin
    // cache and are not recorded in the loose-skills manifest. Removals are
    // dropped from the manifest even when the dir was already gone (cleanup
    // for manually deleted skills), but never when their removal failed.
    let mut m = manifest::load(home);
    let mut manifest_changed = false;
    if plugin_target.is_none() && !report.installed.is_empty() {
        m.version = payload.version.clone();
        m.source = payload.source.clone();
        for (name, version) in &report.installed {
            m.skills.insert(name.clone(), version.clone());
        }
        manifest_changed = true;
    }
    for name in remove {
        let failed = report.failures.iter().any(|(item, _)| item == name);
        if !failed {
            manifest_changed |= m.skills.remove(name).is_some();
        }
    }
    if manifest_changed {
        m.installed_at = manifest::rfc3339_now();
        manifest::save(home, &m).map_err(|e| format!("cannot write manifest: {e}"))?;
    }

    // Drop backup directories that ended up empty (e.g. every item failed
    // before anything was moved aside).
    report.backup_dir = backup.dir.take().filter(|dir| !prune_empty_dirs(dir));
    if report.backup_dir.is_none() {
        let _ = fs::remove_dir(&backup.root); // Succeeds only when empty.
    }
    Ok(report)
}

/// Recursively remove empty directories; true when `dir` itself was removed.
fn prune_empty_dirs(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    let mut empty = true;
    for entry in entries.flatten() {
        let path = entry.path();
        if !(path.is_dir() && prune_empty_dirs(&path)) {
            empty = false;
        }
    }
    empty && fs::remove_dir(dir).is_ok()
}

/// Update a plugin install: write one new version dir carrying every
/// selected skill and omitting every removed one. The operation is atomic
/// as a whole, so all selected skills succeed or fail together. Existing
/// version dirs are never overwritten (the old dirs are the rollback): a
/// run that would change nothing is reported as already present, and a run
/// that would collide with an existing dir name picks a fresh suffixed name.
fn install_into_plugin(
    target: &PluginInstall,
    payload: &Payload,
    skills: &[String],
    removals: &[String],
    report: &mut InstallReport,
    on_progress: &mut dyn FnMut(InstallProgress),
) {
    for name in skills.iter().chain(removals) {
        on_progress(InstallProgress {
            item: name.clone(),
            state: StepState::Started,
        });
    }
    // No removals, every skill already shipped, and the payload's version
    // dir already exists: nothing new to write.
    let no_op = removals.is_empty()
        && skills.iter().all(|name| target.skills.contains(name))
        && target
            .path
            .parent()
            .is_some_and(|p| p.join(&payload.version).exists());
    // A removal-only run keeps the installed content, so the new dir is
    // named after the installed version, not the payload's.
    let base = if skills.is_empty() {
        target.version.clone()
    } else {
        payload.version.clone()
    };
    let outcome = if no_op {
        let dest = target
            .path
            .parent()
            .expect("checked above")
            .join(&payload.version);
        Ok((
            dest,
            format!("v{} · plugin (already present)", payload.version),
            false,
        ))
    } else {
        let version_name = match target.path.parent() {
            Some(parent) => plugin::unique_version_name(parent, &base),
            None => base.clone(), // write_new_version reports the error.
        };
        plugin::write_new_version(target, payload, skills, removals, &version_name)
            .map(|dir| (dir, format!("v{version_name} · plugin"), true))
            .map_err(|e| e.to_string())
    };
    match outcome {
        Ok((dir, msg, wrote)) => {
            for name in skills {
                on_progress(InstallProgress {
                    item: name.clone(),
                    state: StepState::Done(msg.clone()),
                });
                report
                    .installed
                    .push((name.clone(), payload.version.clone()));
            }
            for name in removals {
                on_progress(InstallProgress {
                    item: name.clone(),
                    state: StepState::Done("removed · plugin (new version dir)".to_string()),
                });
                report.removed.push(name.clone());
            }
            report.plugin_dir = Some(dir);
            report.changed |= wrote;
        }
        Err(e) => {
            for name in skills.iter().chain(removals) {
                on_progress(InstallProgress {
                    item: name.clone(),
                    state: StepState::Failed(e.clone()),
                });
                report.failures.push((name.clone(), e.clone()));
            }
        }
    }
}

/// Back up and delete a loose skill dir in one step: renaming it into the
/// backup directory removes it from skills/ while keeping every file.
fn remove_loose(sdir: &Path, name: &str, backup: &mut Backup) -> io::Result<()> {
    let bdir = backup.ensure()?.join("skills");
    fs::create_dir_all(&bdir)?;
    fs::rename(sdir.join(name), bdir.join(name))
}

fn install_one(
    sdir: &Path,
    payload: &Payload,
    name: &str,
    backup: &mut Backup,
) -> io::Result<String> {
    if payload.skill(name).is_none() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("skill {name} is not part of the payload"),
        ));
    }
    // Defense in depth: the payload sources validate too, but never let a
    // hostile name or file path escape the skills directory.
    if !payload::is_valid_skill_name(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid skill name: {name}"),
        ));
    }

    // Stage on the same filesystem so the final rename is atomic.
    let staging = sdir.join(format!(".dale-staging-{name}"));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    let result = stage_and_swap(sdir, &staging, payload, name, backup);
    if result.is_err() {
        // Leave no residue behind on failure.
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn stage_and_swap(
    sdir: &Path,
    staging: &Path,
    payload: &Payload,
    name: &str,
    backup: &mut Backup,
) -> io::Result<String> {
    let skill = payload.skill(name).expect("checked by install_one");
    for (rel, bytes) in &skill.files {
        if !payload::is_safe_rel_path(rel) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsafe file path in skill {name}: {rel}"),
            ));
        }
        let dest = staging.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, bytes)?;
    }

    let target = sdir.join(name);
    if target.exists() {
        let bdir = backup.ensure()?.join("skills");
        fs::create_dir_all(&bdir)?;
        fs::rename(&target, bdir.join(name))?;
    }
    fs::rename(staging, &target)?;
    Ok(payload.version.clone())
}

/// Apply an AGENTS.md action. Returns the human-readable action plus
/// whether the file actually changed on disk.
fn apply_agents_md(
    home: &Path,
    payload: &Payload,
    mode: AgentsMdMode,
    backup: &mut Backup,
) -> io::Result<(String, bool)> {
    let path = agents_md_path(home);
    // Existence is judged on raw bytes: a non-UTF-8 AGENTS.md still exists
    // and must still be backed up before we touch it.
    let existing = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let utf8 = |bytes: &[u8]| {
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "existing AGENTS.md is not valid UTF-8; \
                 use replace (a backup is kept) or skip",
            )
        })
    };
    let (content, action) = match mode {
        AgentsMdMode::Skip => unreachable!("Skip is handled by the caller"),
        AgentsMdMode::Install | AgentsMdMode::Replace => {
            let action = if existing.is_some() {
                "replaced (backup kept)"
            } else {
                "written"
            };
            (payload.agents_md.clone(), action)
        }
        AgentsMdMode::Append => {
            let existing_text = match &existing {
                None => String::new(),
                Some(bytes) => utf8(bytes)?,
            };
            let merged = agents::merge_managed(&existing_text, &payload.agents_md);
            if existing.is_some() && merged == existing_text {
                return Ok(("managed section already up to date".to_string(), false));
            }
            let action = if existing.is_some() {
                "managed section updated"
            } else {
                "written (managed section)"
            };
            (merged, action)
        }
        AgentsMdMode::RemoveSection => {
            let Some(bytes) = &existing else {
                return Ok(("no AGENTS.md — nothing to remove".to_string(), false));
            };
            match agents::remove_managed(&utf8(bytes)?) {
                None => {
                    return Ok((
                        "no dale-managed section — nothing to remove".to_string(),
                        false,
                    ));
                }
                Some(content) => (content, "managed section removed (backup kept)"),
            }
        }
    };
    if existing.is_some() {
        let bdir = backup.ensure()?;
        fs::copy(&path, bdir.join("AGENTS.md"))?;
    }
    write_atomic(&path, content.as_bytes())?;
    Ok((action.to_string(), true))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("dale-tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

/// Remove skills and drop them from the manifest; optionally also remove
/// the managed dale section from AGENTS.md (backup first).
///
/// Loose skill dirs are backed up (moved into the backup dir) rather than
/// deleted outright. Skills shipped by a plugin install are removed by
/// writing a new plugin version dir without them; the old dir stays as
/// rollback. Only valid dale skill names are accepted: a single path
/// component with the `dale-` prefix. Anything else (user directories,
/// `..` traversal) is refused before any deletion happens.
pub fn uninstall(
    home: &Path,
    names: &[String],
    remove_agents_section: bool,
) -> Result<InstallReport, String> {
    // The payload is only a carrier here: nothing is installed from it.
    let payload = Payload {
        version: "0.0.0".to_string(),
        source: "none".to_string(),
        agents_md: String::new(),
        skills: Vec::new(),
    };
    let agents = if remove_agents_section {
        AgentsMdMode::RemoveSection
    } else {
        AgentsMdMode::Skip
    };
    install(home, &payload, &[], names, agents, &mut |_| {})
}

/// Dale skills actually present on disk, with their manifest versions.
/// A skill counts as installed when `skills/<name>/SKILL.md` exists.
pub fn installed_skills(home: &Path, m: &Manifest) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(skills_dir(home)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("dale-") && entry.path().join("SKILL.md").is_file() {
                let version = m
                    .skills
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                out.insert(name, version);
            }
        }
    }
    out
}

/// One installed skill as seen by the picker and `dale list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledSkill {
    pub version: String,
    /// Marketplace name when the skill comes from a plugin install.
    pub marketplace: Option<String>,
}

/// Loose skills merged with the skills of the effective plugin install.
/// When a skill exists in both, the newer version wins (ties keep the
/// loose copy, which shadows nothing since codex loads both).
pub fn all_installed_skills(
    home: &Path,
    m: &Manifest,
    plugins: &[PluginInstall],
) -> BTreeMap<String, InstalledSkill> {
    let mut out: BTreeMap<String, InstalledSkill> = installed_skills(home, m)
        .into_iter()
        .map(|(name, version)| {
            (
                name,
                InstalledSkill {
                    version,
                    marketplace: None,
                },
            )
        })
        .collect();
    if let Some(pi) = plugin::effective(plugins) {
        for name in &pi.skills {
            let plugin_wins = match out.get(name) {
                None => true,
                Some(current) => payload::version_newer(&pi.version, &current.version),
            };
            if plugin_wins {
                out.insert(
                    name.clone(),
                    InstalledSkill {
                        version: pi.version.clone(),
                        marketplace: Some(pi.marketplace.clone()),
                    },
                );
            }
        }
    }
    out
}

/// Snapshot of the environment: manifest, plugin installs, the merged
/// installed-skill map, and the user's AGENTS.md. Cheap enough to run as a
/// splash-screen effect and after installs.
#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub manifest: Manifest,
    pub plugins: Vec<PluginInstall>,
    pub installed: BTreeMap<String, InstalledSkill>,
    pub agents_existing: Option<String>,
}

pub fn scan(home: &Path) -> Scan {
    let manifest = manifest::load(home);
    let plugins = plugin::detect(home);
    let installed = all_installed_skills(home, &manifest, &plugins);
    // Read bytes, not a String: a non-UTF-8 AGENTS.md still exists and must
    // be treated as such (lossy text is only used for previews).
    let agents_existing = fs::read(agents_md_path(home))
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
    Scan {
        manifest,
        plugins,
        installed,
        agents_existing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::SkillPayload;

    fn payload_with(files: Vec<(&str, &str)>) -> Payload {
        Payload {
            version: "9.9.9".to_string(),
            source: "remote".to_string(),
            agents_md: "# Dale AGENTS\n".to_string(),
            skills: vec![SkillPayload {
                name: "dale-test".to_string(),
                description: "d".to_string(),
                files: files
                    .into_iter()
                    .map(|(rel, data)| (rel.to_string(), data.as_bytes().to_vec()))
                    .collect(),
            }],
        }
    }

    fn no_progress() -> impl FnMut(InstallProgress) {
        |_| {}
    }

    #[test]
    fn unsafe_file_path_fails_without_residue_or_manifest_write() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let payload = payload_with(vec![("SKILL.md", "ok"), ("../evil.txt", "PWNED")]);
        let report = install(
            home,
            &payload,
            &["dale-test".to_string()],
            &[],
            AgentsMdMode::Skip,
            &mut no_progress(),
        )
        .unwrap();

        assert!(report.installed.is_empty());
        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].1.contains("unsafe file path"));
        // Nothing escaped and nothing was left behind.
        assert!(!home.join("evil.txt").exists());
        let leftovers: Vec<_> = fs::read_dir(skills_dir(home))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(leftovers.is_empty(), "leftovers: {leftovers:?}");
        // A fully failed run must not claim the payload version.
        assert!(!manifest::manifest_path(home).exists());
        assert!(report.backup_dir.is_none());
        assert!(!home.join("backups").exists());
    }

    #[test]
    fn invalid_skill_name_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let mut payload = payload_with(vec![("SKILL.md", "ok")]);
        payload.skills[0].name = "..".to_string();
        let report = install(
            tmp.path(),
            &payload,
            &["..".to_string()],
            &[],
            AgentsMdMode::Skip,
            &mut no_progress(),
        )
        .unwrap();
        assert!(report.installed.is_empty());
        assert_eq!(report.failures.len(), 1);
    }

    #[test]
    fn uninstall_refuses_traversal_and_non_dale_names() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(home.join("skills/my-notes")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(home.join("skills/my-notes/keep.txt"), "data").unwrap();
        fs::write(outside.join("keep.txt"), "data").unwrap();

        let err = uninstall(&home, &["my-notes".to_string()], false).unwrap_err();
        assert!(err.contains("refusing"));
        assert!(home.join("skills/my-notes/keep.txt").is_file());

        let err = uninstall(&home, &["../../outside".to_string()], false).unwrap_err();
        assert!(err.contains("refusing"));
        assert!(outside.join("keep.txt").is_file());
    }

    #[test]
    fn loose_skill_without_manifest_counts_as_installed() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        // Hand-placed skill: SKILL.md present, no manifest at all.
        fs::create_dir_all(home.join("skills/dale-handmade")).unwrap();
        fs::write(home.join("skills/dale-handmade/SKILL.md"), "x").unwrap();

        let installed = installed_skills(home, &Manifest::default());
        assert_eq!(
            installed.get("dale-handmade").map(String::as_str),
            Some("unknown")
        );

        // A manifest that mentions another skill still leaves this one
        // installed at "unknown"; a mentioned skill gets its version.
        fs::create_dir_all(home.join("skills/dale-known")).unwrap();
        fs::write(home.join("skills/dale-known/SKILL.md"), "x").unwrap();
        let mut m = Manifest::default();
        m.skills
            .insert("dale-known".to_string(), "0.4.0".to_string());
        let installed = installed_skills(home, &m);
        assert_eq!(
            installed.get("dale-handmade").map(String::as_str),
            Some("unknown")
        );
        assert_eq!(
            installed.get("dale-known").map(String::as_str),
            Some("0.4.0")
        );
        // An "unknown" version always counts as older than a real payload.
        assert!(payload::version_newer("0.4.0", "unknown"));
    }

    #[test]
    fn install_with_plugin_present_writes_new_version_dir_not_loose() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        // Fabricated plugin install at an older version.
        let old = home.join("plugins/cache/personal/dale/0.2.0+codex.1");
        fs::create_dir_all(old.join("skills/dale-test")).unwrap();
        fs::write(old.join("skills/dale-test/SKILL.md"), "OLD").unwrap();
        fs::create_dir_all(old.join("assets")).unwrap();
        fs::write(old.join("assets/a.txt"), "asset").unwrap();
        fs::write(old.join("LICENSE"), "MPL-2.0").unwrap();

        let payload = payload_with(vec![("SKILL.md", "NEW")]);
        let report = install(
            home,
            &payload,
            &["dale-test".to_string()],
            &[],
            AgentsMdMode::Skip,
            &mut no_progress(),
        )
        .unwrap();

        assert_eq!(report.failures, Vec::new());
        assert_eq!(
            report.installed,
            vec![("dale-test".to_string(), "9.9.9".to_string())]
        );
        let new_dir = home.join("plugins/cache/personal/dale/9.9.9");
        assert_eq!(report.plugin_dir.as_deref(), Some(new_dir.as_path()));
        // Full copy + overlay in the new dir; old dir untouched.
        assert_eq!(
            fs::read_to_string(new_dir.join("skills/dale-test/SKILL.md")).unwrap(),
            "NEW"
        );
        assert_eq!(
            fs::read_to_string(new_dir.join("LICENSE")).unwrap(),
            "MPL-2.0"
        );
        assert_eq!(
            fs::read_to_string(new_dir.join("assets/a.txt")).unwrap(),
            "asset"
        );
        assert_eq!(
            fs::read_to_string(old.join("skills/dale-test/SKILL.md")).unwrap(),
            "OLD"
        );
        // No loose copy and no loose manifest were written.
        assert!(!home.join("skills/dale-test").exists());
        assert!(!manifest::manifest_path(home).exists());

        // Re-running against the same payload is a no-op, not a failure.
        let report = install(
            home,
            &payload,
            &["dale-test".to_string()],
            &[],
            AgentsMdMode::Skip,
            &mut no_progress(),
        )
        .unwrap();
        assert!(report.failures.is_empty());
    }

    #[test]
    fn all_installed_skills_merges_loose_and_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        // Loose install at 0.1.0.
        fs::create_dir_all(home.join("skills/dale-loose")).unwrap();
        fs::write(home.join("skills/dale-loose/SKILL.md"), "x").unwrap();
        // Plugin install at 0.2.0 shipping dale-loose and dale-plug.
        let vdir = home.join("plugins/cache/personal/dale/0.2.0+codex.1");
        for name in ["dale-loose", "dale-plug"] {
            fs::create_dir_all(vdir.join("skills").join(name)).unwrap();
            fs::write(vdir.join("skills").join(name).join("SKILL.md"), "x").unwrap();
        }
        let mut m = Manifest::default();
        m.skills
            .insert("dale-loose".to_string(), "0.1.0".to_string());

        let plugins = plugin::detect(home);
        let all = all_installed_skills(home, &m, &plugins);
        // The newer plugin version wins for the shared skill.
        let loose = all.get("dale-loose").unwrap();
        assert_eq!(loose.version, "0.2.0+codex.1");
        assert_eq!(loose.marketplace.as_deref(), Some("personal"));
        let plug = all.get("dale-plug").unwrap();
        assert_eq!(plug.marketplace.as_deref(), Some("personal"));
        // Without a plugin, the loose entry keeps its manifest version.
        let all = all_installed_skills(home, &m, &[]);
        let loose = all.get("dale-loose").unwrap();
        assert_eq!(loose.version, "0.1.0");
        assert_eq!(loose.marketplace, None);
    }

    #[test]
    fn non_utf8_agents_md_is_backed_up_on_replace() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let original: &[u8] = b"# precious\n\xff\xfe keep me \xc0\n";
        fs::write(agents_md_path(home), original).unwrap();

        let payload = payload_with(vec![("SKILL.md", "ok")]);
        let report = install(
            home,
            &payload,
            &[],
            &[],
            AgentsMdMode::Replace,
            &mut no_progress(),
        )
        .unwrap();

        assert_eq!(
            report.agents_action.as_deref(),
            Some("replaced (backup kept)")
        );
        let backup_dir = report.backup_dir.expect("backup dir");
        assert_eq!(fs::read(backup_dir.join("AGENTS.md")).unwrap(), original);
        assert_eq!(
            fs::read(agents_md_path(home)).unwrap(),
            payload.agents_md.as_bytes()
        );
    }

    #[test]
    fn non_utf8_agents_md_append_fails_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let original: &[u8] = b"# precious\n\xff\xfe\n";
        fs::write(agents_md_path(home), original).unwrap();

        let payload = payload_with(vec![("SKILL.md", "ok")]);
        let report = install(
            home,
            &payload,
            &[],
            &[],
            AgentsMdMode::Append,
            &mut no_progress(),
        )
        .unwrap();

        assert!(report.agents_action.is_none());
        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].1.contains("not valid UTF-8"));
        assert_eq!(fs::read(agents_md_path(home)).unwrap(), original);
    }

    #[test]
    fn loose_removal_backs_up_deletes_and_updates_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let payload = payload_with(vec![("SKILL.md", "content")]);
        install(
            home,
            &payload,
            &["dale-test".to_string()],
            &[],
            AgentsMdMode::Skip,
            &mut no_progress(),
        )
        .unwrap();
        assert!(home.join("skills/dale-test/SKILL.md").is_file());

        let report = uninstall(home, &["dale-test".to_string()], false).unwrap();
        assert_eq!(report.removed, vec!["dale-test".to_string()]);
        assert!(report.changed);
        assert!(report.failures.is_empty());
        // The dir is gone from skills/ and lives on inside the backup dir.
        assert!(!home.join("skills/dale-test").exists());
        let backup = report.backup_dir.expect("backup dir");
        assert_eq!(
            fs::read_to_string(backup.join("skills/dale-test/SKILL.md")).unwrap(),
            "content"
        );
        // The manifest no longer lists the skill.
        let m = manifest::load(home);
        assert!(!m.skills.contains_key("dale-test"));
    }

    #[test]
    fn plugin_removal_writes_new_version_dir_and_keeps_old() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let old = home.join("plugins/cache/personal/dale/0.2.0+codex.1");
        for name in ["dale-test", "dale-keep"] {
            fs::create_dir_all(old.join("skills").join(name)).unwrap();
            fs::write(old.join("skills").join(name).join("SKILL.md"), "OLD").unwrap();
        }
        fs::write(old.join("LICENSE"), "MPL-2.0").unwrap();

        let report = uninstall(home, &["dale-test".to_string()], false).unwrap();
        assert_eq!(report.removed, vec!["dale-test".to_string()]);
        assert!(report.changed);
        let new_dir = report.plugin_dir.expect("new plugin version dir");
        assert_ne!(new_dir, old);
        assert!(!new_dir.join("skills/dale-test").exists());
        assert!(new_dir.join("skills/dale-keep/SKILL.md").is_file());
        assert_eq!(
            fs::read_to_string(new_dir.join("LICENSE")).unwrap(),
            "MPL-2.0"
        );
        // The old version dir is byte-for-byte untouched — it is the rollback.
        assert!(old.join("skills/dale-test/SKILL.md").is_file());
        // The new dir is the effective install and omits the skill.
        let eff = plugin::effective(&plugin::detect(home)).unwrap().clone();
        assert_eq!(eff.path, new_dir);
        assert_eq!(eff.skills, vec!["dale-keep"]);
    }

    #[test]
    fn agents_section_removal_preserves_user_text() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let user = "# My rules\n\nDo not break prod.\n";
        let payload = payload_with(vec![("SKILL.md", "ok")]);
        fs::write(agents_md_path(home), user).unwrap();
        install(
            home,
            &payload,
            &[],
            &[],
            AgentsMdMode::Append,
            &mut no_progress(),
        )
        .unwrap();
        let merged = fs::read_to_string(agents_md_path(home)).unwrap();
        assert!(agents::has_managed(&merged));

        let report = uninstall(home, &[], true).unwrap();
        assert_eq!(
            report.agents_action.as_deref(),
            Some("managed section removed (backup kept)")
        );
        assert!(report.changed);
        // All user text preserved, marked section gone, file still there.
        assert_eq!(fs::read_to_string(agents_md_path(home)).unwrap(), user);
        let backup = report.backup_dir.expect("backup dir");
        assert_eq!(
            fs::read_to_string(backup.join("AGENTS.md")).unwrap(),
            merged
        );
    }

    #[test]
    fn agents_section_removal_without_markers_is_a_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let user = "# My rules\n\nNo dale markers here.\n";
        fs::write(agents_md_path(home), user).unwrap();

        let report = uninstall(home, &[], true).unwrap();
        assert_eq!(
            report.agents_action.as_deref(),
            Some("no dale-managed section — nothing to remove")
        );
        assert!(!report.changed);
        assert!(report.backup_dir.is_none());
        assert_eq!(fs::read_to_string(agents_md_path(home)).unwrap(), user);

        // No AGENTS.md at all: still a no-op, the file is never created.
        let empty = tempfile::tempdir().unwrap();
        let report = uninstall(empty.path(), &[], true).unwrap();
        assert_eq!(
            report.agents_action.as_deref(),
            Some("no AGENTS.md — nothing to remove")
        );
        assert!(!report.changed);
        assert!(!agents_md_path(empty.path()).exists());
    }

    #[test]
    fn append_when_section_up_to_date_changes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let payload = payload_with(vec![("SKILL.md", "ok")]);
        let run = |home: &Path| {
            install(
                home,
                &payload,
                &[],
                &[],
                AgentsMdMode::Append,
                &mut no_progress(),
            )
            .unwrap()
        };
        let first = run(home);
        assert!(first.changed);
        let second = run(home);
        assert!(
            !second.changed,
            "idempotent re-append must not count as a change"
        );
        assert_eq!(
            second.agents_action.as_deref(),
            Some("managed section already up to date")
        );
        assert!(second.backup_dir.is_none());
    }
}
