//! Codex *plugin* installs of dale, living under
//! `$CODEX_HOME/plugins/cache/<marketplace>/dale/<version>/`.
//!
//! Detection scans the cache for dale version directories and picks the
//! highest version per marketplace (multiple version dirs may coexist; the
//! newest one is the effective install). Updates never touch an existing
//! version directory: a new sibling `<payload-version>/` is staged next to
//! the old one and renamed into place atomically, so the previous version
//! always remains as a rollback.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::payload::{self, Payload};

/// One detected dale plugin install (the effective version directory of a
/// marketplace).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstall {
    pub marketplace: String,
    /// Version directory name, e.g. `0.2.0+codex.20260725025419`.
    pub version: String,
    /// Absolute path of the version directory.
    pub path: PathBuf,
    /// Skill directories under `<path>/skills/` that contain a `SKILL.md`.
    pub skills: Vec<String>,
}

pub fn cache_dir(home: &Path) -> PathBuf {
    home.join("plugins").join("cache")
}

/// Detect dale plugin installs: one entry per marketplace that ships a dale
/// plugin, each pointing at its highest version directory. Absent or
/// partially missing trees simply yield fewer (or no) entries.
pub fn detect(home: &Path) -> Vec<PluginInstall> {
    let mut out = Vec::new();
    let Ok(markets) = fs::read_dir(cache_dir(home)) else {
        return out;
    };
    for market in markets.flatten() {
        let marketplace = market.file_name().to_string_lossy().into_owned();
        let Ok(versions) = fs::read_dir(market.path().join("dale")) else {
            continue;
        };
        let mut best: Option<(String, PathBuf)> = None;
        for entry in versions.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            // Skip hidden dirs (e.g. an interrupted `.dale-staging-*`).
            if name.starts_with('.') {
                continue;
            }
            let newer = match &best {
                None => true,
                Some((current, _)) => payload::version_newer(&name, current),
            };
            if newer {
                best = Some((name, entry.path()));
            }
        }
        if let Some((version, path)) = best {
            out.push(PluginInstall {
                marketplace,
                version,
                skills: list_skills(&path),
                path,
            });
        }
    }
    out.sort_by(|a, b| a.marketplace.cmp(&b.marketplace));
    out
}

/// The install updates apply to when several marketplaces ship dale: the
/// highest-versioned one (first marketplace alphabetically on a tie).
pub fn effective(installs: &[PluginInstall]) -> Option<&PluginInstall> {
    installs.iter().reduce(|best, candidate| {
        if payload::version_newer(&candidate.version, &best.version) {
            candidate
        } else {
            best
        }
    })
}

fn list_skills(version_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(version_dir.join("skills")) {
        for entry in entries.flatten() {
            if entry.path().join("SKILL.md").is_file() {
                out.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    out.sort();
    out
}

/// Pick a fresh version-directory name under `parent`: `base` itself when
/// free, otherwise `base` plus a sortable build-metadata suffix that
/// [`payload::version_newer`] orders after `base` (so detection picks the
/// new directory as the effective install).
pub fn unique_version_name(parent: &Path, base: &str) -> String {
    if !parent.join(base).exists() {
        return base.to_string();
    }
    let ts = crate::manifest::compact_timestamp().replace('-', "");
    let stamped = if base.contains('+') {
        format!("{base}.{ts}")
    } else {
        format!("{base}+codex.{ts}")
    };
    let mut candidate = stamped.clone();
    let mut n = 1;
    while parent.join(&candidate).exists() {
        candidate = format!("{stamped}-{n}");
        n += 1;
    }
    candidate
}

/// Write a new version directory `…/dale/<version_name>/` next to the
/// currently effective one: a full copy of the old directory (assets,
/// LICENSE, README.md, everything) with the skills in `remove` dropped and
/// the selected `skills` overlaid from the payload. Staged in a temp dir
/// under the same parent and renamed atomically. The old version directory
/// is never modified or deleted — it is the rollback. Returns the new
/// directory's path.
pub fn write_new_version(
    install: &PluginInstall,
    payload: &Payload,
    skills: &[String],
    remove: &[String],
    version_name: &str,
) -> io::Result<PathBuf> {
    if !payload::is_safe_component(version_name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsafe plugin version name: {version_name}"),
        ));
    }
    let parent = install.path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("plugin dir has no parent: {}", install.path.display()),
        )
    })?;
    let dest = parent.join(version_name);
    if dest.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("plugin version dir already exists: {}", dest.display()),
        ));
    }

    let staging = parent.join(format!(".dale-staging-{version_name}"));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    let result = stage_new_version(install, payload, skills, remove, &staging, &dest);
    if result.is_err() {
        // Leave no residue behind on failure.
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn stage_new_version(
    install: &PluginInstall,
    payload: &Payload,
    skills: &[String],
    remove: &[String],
    staging: &Path,
    dest: &Path,
) -> io::Result<PathBuf> {
    copy_dir(&install.path, staging)?;
    for name in remove {
        // Same guard as installs: never let a hostile name escape skills/.
        if !payload::is_valid_skill_name(name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid skill name: {name}"),
            ));
        }
        let skill_dir = staging.join("skills").join(name);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)?;
        }
    }
    for name in skills {
        let skill = payload.skill(name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("skill {name} is not part of the payload"),
            )
        })?;
        // Defense in depth, mirroring the loose install path.
        if !payload::is_valid_skill_name(name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid skill name: {name}"),
            ));
        }
        let skill_dir = staging.join("skills").join(name);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)?;
        }
        for (rel, bytes) in &skill.files {
            if !payload::is_safe_rel_path(rel) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unsafe file path in skill {name}: {rel}"),
                ));
            }
            let file = skill_dir.join(rel);
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(&file, bytes)?;
        }
    }
    fs::rename(staging, dest)?;
    Ok(dest.to_path_buf())
}

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::SkillPayload;

    fn add_version(
        home: &Path,
        marketplace: &str,
        version: &str,
        skills: &[(&str, &str)],
    ) -> PathBuf {
        let vdir = cache_dir(home).join(marketplace).join("dale").join(version);
        for (name, content) in skills {
            let sdir = vdir.join("skills").join(name);
            fs::create_dir_all(&sdir).unwrap();
            fs::write(sdir.join("SKILL.md"), content).unwrap();
        }
        fs::create_dir_all(vdir.join("assets")).unwrap();
        fs::write(vdir.join("assets/logo.svg"), "<svg/>").unwrap();
        fs::write(vdir.join("LICENSE"), "MPL-2.0").unwrap();
        fs::write(vdir.join("README.md"), "# dale").unwrap();
        vdir
    }

    fn payload_with(name: &str, content: &str, version: &str) -> Payload {
        Payload {
            version: version.to_string(),
            source: "bundled".to_string(),
            agents_md: String::new(),
            skills: vec![SkillPayload {
                name: name.to_string(),
                description: "d".to_string(),
                files: vec![("SKILL.md".to_string(), content.as_bytes().to_vec())],
            }],
        }
    }

    #[test]
    fn detect_absent_tree_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(detect(tmp.path()).is_empty());
        // cache exists but has no dale plugin anywhere.
        fs::create_dir_all(cache_dir(tmp.path()).join("openai-bundled/browser")).unwrap();
        assert!(detect(tmp.path()).is_empty());
        // dale dir exists but holds no version directories.
        fs::create_dir_all(cache_dir(tmp.path()).join("personal/dale")).unwrap();
        assert!(detect(tmp.path()).is_empty());
    }

    #[test]
    fn detect_picks_highest_version_per_marketplace() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        add_version(
            home,
            "personal",
            "0.1.0+codex.20260701000000",
            &[("dale-brainstorm", "old")],
        );
        add_version(
            home,
            "personal",
            "0.2.0+codex.20260725025419",
            &[("dale-brainstorm", "new"), ("dale-graph", "new")],
        );
        add_version(home, "work", "0.1.5+codex.1", &[("dale-max", "x")]);

        let installs = detect(home);
        assert_eq!(installs.len(), 2);
        assert_eq!(installs[0].marketplace, "personal");
        assert_eq!(installs[0].version, "0.2.0+codex.20260725025419");
        assert_eq!(installs[0].skills, vec!["dale-brainstorm", "dale-graph"]);
        assert!(installs[0].path.ends_with("0.2.0+codex.20260725025419"));
        assert_eq!(installs[1].marketplace, "work");

        // The effective install across marketplaces is the highest version.
        let eff = effective(&installs).unwrap();
        assert_eq!(eff.marketplace, "personal");
        assert_eq!(eff.version, "0.2.0+codex.20260725025419");
    }

    #[test]
    fn write_new_version_full_copy_plus_overlay_keeps_old_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let old = add_version(
            home,
            "personal",
            "0.2.0+codex.1",
            &[("dale-a", "OLD A"), ("dale-b", "OLD B")],
        );

        // Payload carries both skills at a newer version; only dale-a is
        // selected, so dale-b must stay at its old content.
        let mut payload = payload_with("dale-a", "NEW A", "0.3.0+codex.2");
        payload.skills.push(SkillPayload {
            name: "dale-b".to_string(),
            description: String::new(),
            files: vec![("SKILL.md".to_string(), b"NEW B".to_vec())],
        });

        let install = effective(&detect(home)).unwrap().clone();
        let new_dir = write_new_version(
            &install,
            &payload,
            &["dale-a".to_string()],
            &[],
            &payload.version,
        )
        .unwrap();
        assert_eq!(new_dir, old.parent().unwrap().join("0.3.0+codex.2"));

        // Overlay applied to the selected skill only.
        let read = |p: PathBuf| fs::read_to_string(p).unwrap();
        assert_eq!(read(new_dir.join("skills/dale-a/SKILL.md")), "NEW A");
        assert_eq!(read(new_dir.join("skills/dale-b/SKILL.md")), "OLD B");
        // Full copy: assets, LICENSE, README.md came along.
        assert_eq!(read(new_dir.join("assets/logo.svg")), "<svg/>");
        assert_eq!(read(new_dir.join("LICENSE")), "MPL-2.0");
        assert_eq!(read(new_dir.join("README.md")), "# dale");
        // The old version dir is byte-for-byte untouched.
        assert_eq!(read(old.join("skills/dale-a/SKILL.md")), "OLD A");
        assert_eq!(read(old.join("skills/dale-b/SKILL.md")), "OLD B");
        assert!(old.join("assets/logo.svg").is_file());
        // No staging residue.
        assert!(!old
            .parent()
            .unwrap()
            .join(".dale-staging-0.3.0+codex.2")
            .exists());
        // Detection now reports the new version as effective.
        let eff = effective(&detect(home)).unwrap().clone();
        assert_eq!(eff.version, "0.3.0+codex.2");
    }

    #[test]
    fn write_new_version_refuses_existing_target() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        add_version(home, "personal", "0.2.0+codex.1", &[("dale-a", "OLD")]);
        let install = effective(&detect(home)).unwrap().clone();
        // Same version as installed: the target dir already exists.
        let payload = payload_with("dale-a", "NEW", "0.2.0+codex.1");
        let err = write_new_version(
            &install,
            &payload,
            &["dale-a".to_string()],
            &[],
            &payload.version,
        )
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(install.path.join("skills/dale-a/SKILL.md")).unwrap(),
            "OLD"
        );
    }

    #[test]
    fn write_new_version_with_removals_omits_skill_and_keeps_old_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let old = add_version(
            home,
            "personal",
            "0.2.0+codex.1",
            &[("dale-a", "OLD A"), ("dale-b", "OLD B")],
        );
        let install = effective(&detect(home)).unwrap().clone();
        let payload = payload_with("dale-a", "unused", "0.2.0+codex.1");

        // Removal-only run: pick a fresh name next to the existing dir.
        let parent = old.parent().unwrap();
        let name = unique_version_name(parent, &install.version);
        assert_ne!(name, install.version);
        assert!(payload::version_newer(&name, &install.version));

        let new_dir =
            write_new_version(&install, &payload, &[], &["dale-b".to_string()], &name).unwrap();
        assert!(!new_dir.join("skills/dale-b").exists());
        assert_eq!(
            fs::read_to_string(new_dir.join("skills/dale-a/SKILL.md")).unwrap(),
            "OLD A"
        );
        assert_eq!(
            fs::read_to_string(new_dir.join("LICENSE")).unwrap(),
            "MPL-2.0"
        );
        // Old dir untouched: the removed skill is still there as rollback.
        assert_eq!(
            fs::read_to_string(old.join("skills/dale-b/SKILL.md")).unwrap(),
            "OLD B"
        );
        // The new dir is now the effective install, without the skill.
        let eff = effective(&detect(home)).unwrap().clone();
        assert_eq!(eff.version, name);
        assert_eq!(eff.skills, vec!["dale-a"]);
    }

    #[test]
    fn unique_version_name_prefers_base_and_sorts_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path();
        assert_eq!(unique_version_name(parent, "0.4.0"), "0.4.0");
        fs::create_dir_all(parent.join("0.4.0")).unwrap();
        let next = unique_version_name(parent, "0.4.0");
        assert_ne!(next, "0.4.0");
        assert!(payload::version_newer(&next, "0.4.0"));
        fs::create_dir_all(parent.join(&next)).unwrap();
        let third = unique_version_name(parent, "0.4.0");
        assert_ne!(third, next);
    }
}
