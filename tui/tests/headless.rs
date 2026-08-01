//! Integration tests for the headless CLI. No network access: the update
//! path uses a locally built .tar.gz fixture via --update-url.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use flate2::write::GzEncoder;
use flate2::Compression;

fn dale(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dale"))
        .arg("--codex-home")
        .arg(home)
        .args(args)
        .env_remove("CODEX_HOME")
        .env_remove("DALE_UPDATE_URL")
        .output()
        .expect("failed to run dale")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn manifest(home: &Path) -> serde_json::Value {
    let raw = fs::read_to_string(home.join("skills/.dale-manifest.json")).expect("manifest");
    serde_json::from_str(&raw).expect("manifest json")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bundled_version() -> String {
    let raw = fs::read_to_string(repo_root().join(".codex-plugin/plugin.json")).unwrap();
    serde_json::from_str::<serde_json::Value>(&raw).unwrap()["version"]
        .as_str()
        .unwrap()
        .to_string()
}

fn bundled_skill_count() -> usize {
    fs::read_dir(repo_root().join("skills"))
        .unwrap()
        .filter(|e| e.as_ref().unwrap().path().join("SKILL.md").is_file())
        .count()
}

fn backup_dirs(home: &Path) -> Vec<PathBuf> {
    match fs::read_dir(home.join("backups")) {
        Ok(entries) => entries.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    }
}

#[test]
fn install_all_creates_skills_and_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "--all", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let count = bundled_skill_count();
    assert!(count >= 13);
    for name in ["dale-brainstorm", "dale-loop-goal", "dale-max"] {
        assert!(
            home.join("skills").join(name).join("SKILL.md").is_file(),
            "{name} missing"
        );
    }

    let m = manifest(home);
    assert_eq!(m["version"].as_str().unwrap(), bundled_version());
    assert_eq!(m["source"].as_str().unwrap(), "bundled");
    assert!(!m["installedAt"].as_str().unwrap().is_empty());
    assert_eq!(m["skills"].as_object().unwrap().len(), count);

    // AGENTS.md is untouched by default.
    assert!(!home.join("AGENTS.md").exists());
    // No backups on a fresh install.
    assert!(backup_dirs(home).is_empty());
}

#[test]
fn second_install_creates_backups() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let backups = backup_dirs(home);
    assert_eq!(backups.len(), 1, "expected exactly one backup dir");
    assert!(backups[0].join("skills/dale-brainstorm/SKILL.md").is_file());
    // The live skill is still in place.
    assert!(home.join("skills/dale-brainstorm/SKILL.md").is_file());
}

#[test]
fn agents_md_append_is_idempotent_and_preserves_user_text() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    fs::create_dir_all(home).unwrap();
    fs::write(home.join("AGENTS.md"), "# My own rules\n\nKeep me.\n").unwrap();

    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let first = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    assert!(first.starts_with("# My own rules\n\nKeep me."));
    assert_eq!(first.matches("<!-- dale:begin -->").count(), 1);
    assert_eq!(first.matches("<!-- dale:end -->").count(), 1);

    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let second = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    assert_eq!(first, second, "re-append must be idempotent");
}

#[test]
fn agents_md_replace_backs_up_original() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    fs::create_dir_all(home).unwrap();
    fs::write(home.join("AGENTS.md"), "original user content\n").unwrap();

    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "replace"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let replaced = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    let bundled = fs::read_to_string(repo_root().join("codex/AGENTS.md")).unwrap();
    assert_eq!(replaced, bundled);

    let backups = backup_dirs(home);
    assert_eq!(backups.len(), 1);
    let backed_up = fs::read_to_string(backups[0].join("AGENTS.md")).unwrap();
    assert_eq!(backed_up, "original user content\n");
}

#[test]
fn list_reports_installed_and_available() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success());
    let out = dale(home, &["list"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("dale-brainstorm"));
    assert!(text.contains("installed v"));
    assert!(text.contains("available"));
    assert!(text.contains("AGENTS.md"));
}

#[test]
fn uninstall_removes_skills_and_updates_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "dale-max", "--yes"]);
    assert!(out.status.success());

    let out = dale(home, &["uninstall", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(!home.join("skills/dale-brainstorm").exists());
    assert!(home.join("skills/dale-max").exists());
    let m = manifest(home);
    assert!(m["skills"].get("dale-brainstorm").is_none());
    assert!(m["skills"].get("dale-max").is_some());

    let out = dale(home, &["uninstall", "--all", "--yes"]);
    assert!(out.status.success());
    assert!(!home.join("skills/dale-max").exists());
    assert!(manifest(home)["skills"].as_object().unwrap().is_empty());
}

#[test]
fn uninstall_rejects_unknown_and_unsafe_names() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let outside = tmp.path().join("outside");

    let out = dale(&home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    // A typo must fail loudly, not report success.
    let out = dale(&home, &["uninstall", "dale-brainstrom", "--yes"]);
    assert!(!out.status.success(), "typo must not exit 0");
    assert!(
        stderr(&out).contains("not installed"),
        "stderr: {}",
        stderr(&out)
    );

    // A user's own directory under skills/ must never be deleted.
    let notes = home.join("skills/my-precious-notes");
    fs::create_dir_all(&notes).unwrap();
    fs::write(notes.join("notes.txt"), "data").unwrap();
    let out = dale(&home, &["uninstall", "my-precious-notes", "--yes"]);
    assert!(!out.status.success());
    assert!(notes.join("notes.txt").is_file(), "user dir was deleted");

    // Path traversal must never escape the skills directory.
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("important.txt"), "outside").unwrap();
    let out = dale(&home, &["uninstall", "../../outside", "--yes"]);
    assert!(!out.status.success());
    assert!(
        outside.join("important.txt").is_file(),
        "outside dir was deleted"
    );

    // The real skill is still removable.
    let out = dale(&home, &["uninstall", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(!home.join("skills/dale-brainstorm").exists());
}

#[test]
fn update_rejects_tarball_with_path_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");

    let out = dale(&home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let victim = tmp.path().join("victim.txt");
    fs::write(&victim, "ORIGINAL").unwrap();

    // Malicious tarball: one valid skill file plus a traversal entry.
    let tarball_path = tmp.path().join("mal.tar.gz");
    let file = fs::File::create(&tarball_path).unwrap();
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::fast()));
    let entries: &[(&str, &str)] = &[
        (
            "repo/skills/dale-brainstorm/SKILL.md",
            "---\nname: dale-brainstorm\ndescription: x\n---\n",
        ),
        ("repo/skills/dale-brainstorm/../../../victim.txt", "PWNED\n"),
        ("repo/codex/AGENTS.md", "# a\n"),
        ("repo/.codex-plugin/plugin.json", "{\"version\":\"99.9.9\"}"),
    ];
    for (path, content) in entries {
        // Write header name bytes directly: `append_data` refuses `..`
        // components, but a hostile archive is exactly such raw bytes.
        let mut header = tar::Header::new_gnu();
        {
            let name = &mut header.as_old_mut().name;
            assert!(path.len() <= name.len(), "fixture path too long: {path}");
            name[..path.len()].copy_from_slice(path.as_bytes());
        }
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, content.as_bytes()).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap();

    let out = dale(
        &home,
        &[
            "update",
            "--yes",
            "--update-url",
            tarball_path.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "malicious tarball must be rejected");
    assert!(
        stderr(&out).contains("unsafe path"),
        "stderr: {}",
        stderr(&out)
    );
    assert_eq!(fs::read_to_string(&victim).unwrap(), "ORIGINAL");
    // The installed skill is untouched by the failed update.
    assert!(home.join("skills/dale-brainstorm/SKILL.md").is_file());
}

#[test]
fn non_utf8_agents_md_is_backed_up_not_destroyed() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    fs::create_dir_all(home).unwrap();
    let original: &[u8] = b"# precious\n\xff\xfe keep me \xc0\n";
    fs::write(home.join("AGENTS.md"), original).unwrap();

    // Append refuses to merge undecodable content and leaves it untouched.
    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(!out.status.success(), "append on non-UTF-8 must fail");
    assert!(
        stderr(&out).contains("not valid UTF-8"),
        "stderr: {}",
        stderr(&out)
    );
    assert_eq!(fs::read(home.join("AGENTS.md")).unwrap(), original);

    // Replace works and keeps a byte-exact backup.
    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "replace"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).contains("replaced (backup kept)"));
    let backups = backup_dirs(home);
    assert!(!backups.is_empty(), "backup dir must exist");
    let backed_up = backups
        .iter()
        .map(|d| d.join("AGENTS.md"))
        .find(|p| p.is_file())
        .expect("AGENTS.md backup");
    assert_eq!(fs::read(backed_up).unwrap(), original);
}

#[test]
fn fresh_append_creates_managed_block_and_stays_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");

    let out = dale(
        &home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let first = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    assert!(first.starts_with("<!-- dale:begin -->"));
    assert_eq!(first.matches("<!-- dale:begin -->").count(), 1);
    assert_eq!(first.matches("<!-- dale:end -->").count(), 1);

    let out = dale(
        &home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let second = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    assert_eq!(first, second, "fresh append must stay idempotent");
}

#[test]
fn install_requires_confirmation_and_valid_names() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "--all"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("--yes"));

    let out = dale(home, &["install", "dale-nope", "--yes"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("unknown skill"));
}

/// Build a remote-payload fixture tarball with a higher version.
fn build_fixture(dir: &Path, version: &str) -> PathBuf {
    let root = dir.join("dale-main");
    for name in ["dale-brainstorm", "dale-lenses"] {
        let skill_dir = root.join("skills").join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Updated {name} from remote.\n---\n\n# Remote {name}\n"),
        )
        .unwrap();
    }
    fs::create_dir_all(root.join("codex")).unwrap();
    fs::write(root.join("codex/AGENTS.md"), "# Remote AGENTS.md\n").unwrap();
    fs::create_dir_all(root.join(".codex-plugin")).unwrap();
    fs::write(
        root.join(".codex-plugin/plugin.json"),
        format!("{{\"name\":\"dale\",\"version\":\"{version}\"}}"),
    )
    .unwrap();

    let tarball = dir.join("fixture.tar.gz");
    let file = fs::File::create(&tarball).unwrap();
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::fast()));
    builder.append_dir_all("dale-main", &root).unwrap();
    builder.into_inner().unwrap().finish().unwrap();
    tarball
}

#[test]
fn update_from_local_tarball_updates_installed_skills() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let fixture = build_fixture(tmp.path(), "99.0.0");

    let out = dale(
        &home,
        &["install", "dale-brainstorm", "dale-lenses", "--yes"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let out = dale(
        &home,
        &["update", "--yes", "--update-url", fixture.to_str().unwrap()],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let m = manifest(&home);
    assert_eq!(m["version"].as_str().unwrap(), "99.0.0");
    assert_eq!(m["source"].as_str().unwrap(), "remote");
    assert_eq!(m["skills"]["dale-brainstorm"].as_str().unwrap(), "99.0.0");
    assert_eq!(m["skills"]["dale-lenses"].as_str().unwrap(), "99.0.0");

    let skill_md = fs::read_to_string(home.join("skills/dale-brainstorm/SKILL.md")).unwrap();
    assert!(skill_md.contains("Remote dale-brainstorm"));

    // The bundled versions were backed up before being replaced.
    assert!(!backup_dirs(&home).is_empty());
}

#[test]
fn update_skips_skills_missing_from_remote_payload() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let fixture = build_fixture(tmp.path(), "99.0.0");

    let out = dale(&home, &["install", "dale-brainstorm", "dale-max", "--yes"]);
    assert!(out.status.success());

    let out = dale(
        &home,
        &["update", "--yes", "--update-url", fixture.to_str().unwrap()],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).contains("dale-max — skipped"));

    // dale-max keeps its bundled content and manifest entry.
    let m = manifest(&home);
    assert_eq!(m["skills"]["dale-brainstorm"].as_str().unwrap(), "99.0.0");
    assert_eq!(m["skills"]["dale-max"].as_str().unwrap(), bundled_version());
}

#[test]
fn update_fails_gracefully_without_network_or_file() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success());

    let out = dale(
        home,
        &[
            "update",
            "--yes",
            "--update-url",
            "/definitely/missing/dale.tar.gz",
        ],
    );
    assert!(!out.status.success(), "update must fail");
    assert!(
        stderr(&out).contains("not found"),
        "stderr: {}",
        stderr(&out)
    );

    // The installed skill is untouched.
    assert!(home.join("skills/dale-brainstorm/SKILL.md").is_file());
    assert_eq!(
        manifest(home)["version"].as_str().unwrap(),
        bundled_version()
    );
}

/// Fabricate a dale plugin install (Codex plugin cache layout).
fn build_plugin(home: &Path, marketplace: &str, version: &str, skills: &[&str]) -> PathBuf {
    let vdir = home
        .join("plugins/cache")
        .join(marketplace)
        .join("dale")
        .join(version);
    for name in skills {
        let sdir = vdir.join("skills").join(name);
        fs::create_dir_all(&sdir).unwrap();
        fs::write(
            sdir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Plugin {name}.\n---\n\n# Old {name}\n"),
        )
        .unwrap();
    }
    fs::create_dir_all(vdir.join("assets")).unwrap();
    fs::write(vdir.join("assets/logo.svg"), "<svg/>").unwrap();
    fs::write(vdir.join("LICENSE"), "MPL-2.0").unwrap();
    fs::write(vdir.join("README.md"), "# dale plugin").unwrap();
    vdir
}

#[test]
fn list_reports_plugin_installed_skills() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    // An older version dir must lose against the newer one.
    build_plugin(home, "personal", "0.0.1+codex.1", &["dale-brainstorm"]);
    build_plugin(
        home,
        "personal",
        "0.0.2+codex.2",
        &["dale-brainstorm", "dale-graph"],
    );

    let out = dale(home, &["list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Plugin install: dale@personal v0.0.2+codex.2"),
        "list output: {text}"
    );
    assert!(
        text.contains("installed v0.0.2+codex.2 · plugin — update available"),
        "list output: {text}"
    );
    assert!(!text.contains("v0.0.1+codex.1"), "list output: {text}");
}

#[test]
fn update_writes_new_plugin_version_dir_and_keeps_old() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let old = build_plugin(&home, "personal", "0.0.1+codex.1", &["dale-brainstorm"]);
    let old_skill = fs::read_to_string(old.join("skills/dale-brainstorm/SKILL.md")).unwrap();
    let fixture = build_fixture(tmp.path(), "99.0.0");

    let out = dale(
        &home,
        &["update", "--yes", "--update-url", fixture.to_str().unwrap()],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("v99.0.0 · plugin"),
        "stdout: {}",
        stdout(&out)
    );

    // New version dir: full copy of the old one with the skill overlaid.
    let new_dir = home.join("plugins/cache/personal/dale/99.0.0");
    let new_skill = fs::read_to_string(new_dir.join("skills/dale-brainstorm/SKILL.md")).unwrap();
    assert!(new_skill.contains("Remote dale-brainstorm"), "{new_skill}");
    assert_eq!(
        fs::read_to_string(new_dir.join("assets/logo.svg")).unwrap(),
        "<svg/>"
    );
    assert_eq!(
        fs::read_to_string(new_dir.join("LICENSE")).unwrap(),
        "MPL-2.0"
    );
    assert_eq!(
        fs::read_to_string(new_dir.join("README.md")).unwrap(),
        "# dale plugin"
    );
    // The old version dir is untouched — it is the rollback.
    assert_eq!(
        fs::read_to_string(old.join("skills/dale-brainstorm/SKILL.md")).unwrap(),
        old_skill
    );
    // No loose copies, no loose manifest.
    assert!(!home.join("skills/dale-brainstorm").exists());
    assert!(!home.join("skills/.dale-manifest.json").exists());

    // The fixture also ships dale-lenses, which was not installed: it is
    // added into the same new plugin version dir.
    assert!(
        stdout(&out).contains("dale-lenses — added"),
        "stdout: {}",
        stdout(&out)
    );
    assert!(new_dir.join("skills/dale-lenses/SKILL.md").is_file());
    assert!(
        stdout(&out).contains("Summary: updated 1, added 1, up-to-date 0."),
        "stdout: {}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Restart Codex"));

    // Re-running the same update is a no-op, not a failure.
    let out = dale(
        &home,
        &["update", "--yes", "--update-url", fixture.to_str().unwrap()],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("Summary: updated 0, added 0, up-to-date 2."),
        "stdout: {}",
        stdout(&out)
    );
    // Nothing changed: no restart reminder.
    assert!(!stdout(&out).contains("Restart Codex"));
}

#[test]
fn no_subcommand_without_tty_prints_help() {
    let tmp = tempfile::tempdir().unwrap();
    let out = dale(tmp.path(), &[]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Usage"));
}

#[test]
fn install_prints_restart_reminder_only_when_something_changed() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("Restart Codex"),
        "stdout: {}",
        stdout(&out)
    );

    // A pure no-op run must not tell the user to restart.
    let out = dale(home, &["uninstall", "--agents-md", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("no AGENTS.md — nothing to remove"),
        "stdout: {}",
        stdout(&out)
    );
    assert!(!stdout(&out).contains("Restart Codex"));
}

#[test]
fn uninstall_backs_up_loose_skill_before_deleting() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success());
    let original = fs::read_to_string(home.join("skills/dale-brainstorm/SKILL.md")).unwrap();

    let out = dale(home, &["uninstall", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).contains("Restart Codex"));
    assert!(!home.join("skills/dale-brainstorm").exists());
    // The removed dir lives on inside the backup dir, byte for byte.
    let backup = backup_dirs(home)
        .into_iter()
        .map(|d| d.join("skills/dale-brainstorm/SKILL.md"))
        .find(|p| p.is_file())
        .expect("backup of the removed skill");
    assert_eq!(fs::read_to_string(backup).unwrap(), original);
    assert!(manifest(home)["skills"].get("dale-brainstorm").is_none());
}

#[test]
fn uninstall_plugin_skill_writes_new_version_dir_without_it() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let old = build_plugin(
        &home,
        "personal",
        "0.0.1+codex.1",
        &["dale-brainstorm", "dale-graph"],
    );

    let out = dale(&home, &["uninstall", "dale-graph", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("dale-graph — removed"),
        "stdout: {}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Plugin: new version dir"));
    assert!(stdout(&out).contains("Restart Codex"));

    // Exactly one new version dir appeared, without the removed skill but
    // with everything else copied over.
    let dale_dir = home.join("plugins/cache/personal/dale");
    let dirs: Vec<_> = fs::read_dir(&dale_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(dirs.len(), 2, "dirs: {dirs:?}");
    let new_dir = dirs.iter().find(|d| **d != old).unwrap();
    assert!(!new_dir.join("skills/dale-graph").exists());
    assert!(new_dir.join("skills/dale-brainstorm/SKILL.md").is_file());
    assert_eq!(
        fs::read_to_string(new_dir.join("LICENSE")).unwrap(),
        "MPL-2.0"
    );
    // The old version dir is untouched — it is the rollback.
    assert!(old.join("skills/dale-graph/SKILL.md").is_file());
    assert!(old.join("skills/dale-brainstorm/SKILL.md").is_file());
}

#[test]
fn uninstall_agents_md_removes_only_the_managed_section() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    fs::create_dir_all(home).unwrap();
    let user_text = "# My own rules\n\nKeep me.\n";
    fs::write(home.join("AGENTS.md"), user_text).unwrap();

    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let merged = fs::read_to_string(home.join("AGENTS.md")).unwrap();
    assert!(merged.contains("<!-- dale:begin -->"));

    let out = dale(home, &["uninstall", "--agents-md", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("managed section removed (backup kept)"),
        "stdout: {}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Restart Codex"));
    // Only the managed section is gone; every byte of user text survives
    // and the file itself is never deleted.
    assert_eq!(
        fs::read_to_string(home.join("AGENTS.md")).unwrap(),
        user_text
    );
    // The pre-removal document was backed up.
    let backed_up = backup_dirs(home)
        .into_iter()
        .filter_map(|d| fs::read_to_string(d.join("AGENTS.md")).ok())
        .any(|content| content == merged);
    assert!(backed_up, "expected a backup of the merged AGENTS.md");

    // Without markers the removal is a no-op that leaves the file alone.
    let out = dale(home, &["uninstall", "--agents-md", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(
        stdout(&out).contains("no dale-managed section — nothing to remove"),
        "stdout: {}",
        stdout(&out)
    );
    assert!(!stdout(&out).contains("Restart Codex"));
    assert_eq!(
        fs::read_to_string(home.join("AGENTS.md")).unwrap(),
        user_text
    );
}

#[test]
fn update_adds_new_remote_skills_and_summarizes() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let fixture = build_fixture(tmp.path(), "99.0.0");

    // Only dale-brainstorm is installed; the fixture also ships dale-lenses.
    let out = dale(&home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(!home.join("skills/dale-lenses").exists());

    let out = dale(
        &home,
        &["update", "--yes", "--update-url", fixture.to_str().unwrap()],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("dale-lenses — added"), "stdout: {text}");
    assert!(
        text.contains("Summary: updated 1, added 1, up-to-date 0."),
        "stdout: {text}"
    );
    assert!(text.contains("Restart Codex"), "stdout: {text}");
    // The self-version check is skipped for --update-url overrides (this
    // also keeps the test offline).
    assert!(!text.contains("newer dale binary"), "stdout: {text}");

    let m = manifest(&home);
    assert_eq!(m["skills"]["dale-brainstorm"].as_str().unwrap(), "99.0.0");
    assert_eq!(m["skills"]["dale-lenses"].as_str().unwrap(), "99.0.0");
    assert!(home.join("skills/dale-lenses/SKILL.md").is_file());
}

#[test]
fn list_json_has_the_documented_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();

    let out = dale(home, &["install", "dale-brainstorm", "--yes"]);
    assert!(out.status.success());
    let out = dale(
        home,
        &["install", "dale-max", "--yes", "--agents-md", "append"],
    );
    assert!(out.status.success());

    let out = dale(home, &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");

    assert_eq!(
        doc["codexHome"].as_str().unwrap(),
        home.display().to_string()
    );
    assert_eq!(doc["bundledVersion"].as_str().unwrap(), bundled_version());
    assert!(doc["pluginInstall"].is_null());
    assert_eq!(doc["agentsMd"]["present"], serde_json::json!(true));
    assert_eq!(doc["agentsMd"]["daleManaged"], serde_json::json!(true));

    let skills = doc["skills"].as_array().unwrap();
    assert_eq!(skills.len(), bundled_skill_count());
    let find = |name: &str| {
        skills
            .iter()
            .find(|s| s["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from list --json"))
    };
    let brainstorm = find("dale-brainstorm");
    assert_eq!(brainstorm["installed"], serde_json::json!(true));
    assert_eq!(brainstorm["source"], serde_json::json!("loose"));
    assert_eq!(
        brainstorm["installedVersion"].as_str().unwrap(),
        bundled_version()
    );
    assert_eq!(
        brainstorm["availableVersion"].as_str().unwrap(),
        bundled_version()
    );
    assert_eq!(brainstorm["updateAvailable"], serde_json::json!(false));
    let graph = find("dale-graph");
    assert_eq!(graph["installed"], serde_json::json!(false));
    assert!(graph["source"].is_null());
    assert!(graph["installedVersion"].is_null());

    // With a plugin install, pluginInstall and source reflect it.
    let plug_home = tmp.path().join("plug-home");
    build_plugin(&plug_home, "personal", "0.0.2+codex.2", &["dale-graph"]);
    let out = dale(&plug_home, &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(
        doc["pluginInstall"]["marketplace"].as_str().unwrap(),
        "personal"
    );
    assert_eq!(
        doc["pluginInstall"]["version"].as_str().unwrap(),
        "0.0.2+codex.2"
    );
    assert!(doc["pluginInstall"]["path"].as_str().unwrap().len() > 1);
    let skills = doc["skills"].as_array().unwrap();
    let graph = skills.iter().find(|s| s["name"] == "dale-graph").unwrap();
    assert_eq!(graph["source"], serde_json::json!("plugin"));
    assert_eq!(graph["updateAvailable"], serde_json::json!(true));
}
